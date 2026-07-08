use std::collections::{BTreeMap, BTreeSet};

use num_traits::Zero;

use crate::compression::CertifiedSystemQ;
use crate::linear_q::{dot_q, solve_linear_system_q, LinearSolveQ};
use crate::proof_learning::LeftNullObstruction;
use crate::verifier::verify_guard_certificate;
use crate::window::ProofWindow;
use crate::{
    ExactIdentity, ExactIdentityKind, GuardRecord, Monomial, PolynomialQ, Rational,
    TargetCertificate, TargetProblemQ, UniPolynomialQ, VerificationResult,
};

#[derive(Clone, Debug)]
pub(crate) struct FixedProofInput {
    pub system: CertifiedSystemQ,
    pub candidate: UniPolynomialQ,
    pub proof_window: ProofWindow,
    pub certificate_mode: CertificateMode,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum CertificateMode {
    Ideal,
    Radical {
        support_power: usize,
    },
    GuardedRadical {
        support_power: usize,
        guard_power: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProofFailure {
    InvalidInput,
    Inconsistent { obstruction: LeftNullObstruction },
    IdentityCheckFailed,
    NoCertificateFound,
}

#[derive(Clone, Debug)]
struct ProofColumn {
    equation_index: usize,
    multiplier_monomial: Monomial,
    vector: Vec<Rational>,
}

pub(crate) fn prove_fixed_target(
    input: FixedProofInput,
) -> Result<TargetCertificate, ProofFailure> {
    let problem = TargetProblemQ {
        equations: input.system.equations.clone(),
        variables: input.system.variables.clone(),
        target: input.system.target.clone(),
        semantic_guards: semantic_guards_from_system(&input.system),
    };
    prove_fixed_target_with_problem(input, &problem)
}

fn semantic_guards_from_system(system: &CertifiedSystemQ) -> Vec<GuardRecord> {
    system.semantic_guards.clone()
}

pub(crate) fn prove_fixed_target_with_problem(
    input: FixedProofInput,
    problem: &TargetProblemQ,
) -> Result<TargetCertificate, ProofFailure> {
    if input.candidate.variable != input.system.target
        || input.candidate.is_zero()
        || input.proof_window.multiplier_supports.len() != input.system.equations.len()
        || !input
            .proof_window
            .multiplier_supports
            .iter()
            .flatten()
            .all(|monomial| monomial.exponents.len() == input.system.variables.len())
    {
        return Err(ProofFailure::InvalidInput);
    }

    let support = input.candidate.primitive_integer_normalized();
    let (support_power, guard_power, guard_product) = mode_parameters(&input, problem)?;
    let support_polynomial = support
        .pow(support_power)
        .to_multivariate(&input.system.variables);
    let h = guard_product.pow(guard_power).mul(&support_polynomial);
    let row_monomials = proof_rows(&input.system, &input.proof_window, &h);
    let columns = proof_columns(&input.system, &input.proof_window, &row_monomials);
    let matrix = rows_from_columns(&columns);
    let rhs = vector_from_polynomial(&h, &row_monomials);

    let solution = match solve_linear_system_q(&matrix, &rhs) {
        LinearSolveQ::Consistent { solution, .. } => solution,
        LinearSolveQ::Inconsistent { obstruction } => {
            return Err(ProofFailure::Inconsistent {
                obstruction: LeftNullObstruction {
                    row_monomials,
                    coefficients: obstruction.coefficients,
                },
            });
        }
    };

    let multipliers = restore_multipliers(&input.system, &columns, &solution);
    if !h
        .sub(&linear_combination(&input.system, &multipliers))
        .is_zero()
    {
        return Err(ProofFailure::IdentityCheckFailed);
    }

    Ok(match input.certificate_mode {
        CertificateMode::Ideal => TargetCertificate::IdealMembership {
            support,
            multipliers,
            identity: ExactIdentity {
                kind: ExactIdentityKind::IdealMembership,
            },
        },
        CertificateMode::Radical { support_power } => TargetCertificate::RadicalMembership {
            support,
            power: support_power,
            multipliers,
            identity: ExactIdentity {
                kind: ExactIdentityKind::RadicalMembership,
            },
        },
        CertificateMode::GuardedRadical {
            support_power,
            guard_power,
        } => TargetCertificate::GuardedRadicalMembership {
            support,
            support_power,
            guard_power,
            guard_product,
            guard_certificates: input.system.guard_certificates,
            multipliers,
            identity: ExactIdentity {
                kind: ExactIdentityKind::GuardedRadicalMembership,
            },
        },
    })
}

fn mode_parameters(
    input: &FixedProofInput,
    problem: &TargetProblemQ,
) -> Result<(usize, usize, PolynomialQ), ProofFailure> {
    match input.certificate_mode {
        CertificateMode::Ideal => Ok((1, 0, PolynomialQ::one(input.system.variables.clone()))),
        CertificateMode::Radical { support_power } => {
            if support_power == 0 {
                return Err(ProofFailure::InvalidInput);
            }
            Ok((
                support_power,
                0,
                PolynomialQ::one(input.system.variables.clone()),
            ))
        }
        CertificateMode::GuardedRadical {
            support_power,
            guard_power,
        } => {
            if support_power == 0 {
                return Err(ProofFailure::InvalidInput);
            }
            let guard_product = verified_guard_product(&input.system, problem)?;
            Ok((support_power, guard_power, guard_product))
        }
    }
}

fn verified_guard_product(
    system: &CertifiedSystemQ,
    problem: &TargetProblemQ,
) -> Result<PolynomialQ, ProofFailure> {
    let mut product = PolynomialQ::one(system.variables.clone());
    for certificate in &system.guard_certificates {
        if verify_guard_certificate(problem, certificate) != VerificationResult::Verified {
            return Err(ProofFailure::InvalidInput);
        }
        product = product.mul(&guard_polynomial(certificate));
    }
    Ok(product)
}

fn guard_polynomial(certificate: &crate::GuardCertificate) -> PolynomialQ {
    match certificate {
        crate::GuardCertificate::InputSemanticNonzero { guard, .. }
        | crate::GuardCertificate::AlgebraicNonvanishing { guard, .. }
        | crate::GuardCertificate::RealAdmissibleNonvanishing { guard, .. } => guard.clone(),
        crate::GuardCertificate::DerivedProduct { product, .. } => product.clone(),
    }
}

fn proof_rows(
    system: &CertifiedSystemQ,
    proof_window: &ProofWindow,
    h: &PolynomialQ,
) -> Vec<Monomial> {
    let mut rows = h.support().into_iter().collect::<BTreeSet<_>>();
    for (equation, supports) in system
        .equations
        .iter()
        .zip(&proof_window.multiplier_supports)
    {
        for multiplier_monomial in supports {
            for equation_monomial in equation.support() {
                rows.insert(multiplier_monomial.multiply(&equation_monomial));
            }
        }
    }
    let mut row_monomials = rows.into_iter().collect::<Vec<_>>();
    row_monomials.sort_by_key(canonical_monomial_key);
    row_monomials
}

fn proof_columns(
    system: &CertifiedSystemQ,
    proof_window: &ProofWindow,
    row_monomials: &[Monomial],
) -> Vec<ProofColumn> {
    let row_index = row_index_map(row_monomials);
    let mut columns = Vec::new();
    for (equation_index, (equation, supports)) in system
        .equations
        .iter()
        .zip(&proof_window.multiplier_supports)
        .enumerate()
    {
        for multiplier_monomial in supports {
            let mut vector = vec![crate::arith::rational_zero(); row_monomials.len()];
            for (equation_monomial, coefficient) in &equation.terms {
                let product_monomial = multiplier_monomial.multiply(equation_monomial);
                if let Some(row) = row_index.get(&product_monomial) {
                    vector[*row] += coefficient.clone();
                }
            }
            columns.push(ProofColumn {
                equation_index,
                multiplier_monomial: multiplier_monomial.clone(),
                vector,
            });
        }
    }
    columns
}

fn rows_from_columns(columns: &[ProofColumn]) -> Vec<Vec<Rational>> {
    let rows = columns.first().map_or(0, |column| column.vector.len());
    (0..rows)
        .map(|row| {
            columns
                .iter()
                .map(|column| column.vector[row].clone())
                .collect()
        })
        .collect()
}

fn vector_from_polynomial(polynomial: &PolynomialQ, row_monomials: &[Monomial]) -> Vec<Rational> {
    let row_index = row_index_map(row_monomials);
    let mut vector = vec![crate::arith::rational_zero(); row_monomials.len()];
    for (monomial, coefficient) in &polynomial.terms {
        if let Some(row) = row_index.get(monomial) {
            vector[*row] = coefficient.clone();
        }
    }
    vector
}

fn restore_multipliers(
    system: &CertifiedSystemQ,
    columns: &[ProofColumn],
    solution: &[Rational],
) -> Vec<PolynomialQ> {
    let mut multipliers = vec![PolynomialQ::zero(system.variables.clone()); system.equations.len()];
    for (column, coefficient) in columns.iter().zip(solution) {
        if coefficient.is_zero() {
            continue;
        }
        let term = PolynomialQ::from_term(
            system.variables.clone(),
            coefficient.clone(),
            column.multiplier_monomial.clone(),
        );
        multipliers[column.equation_index] = multipliers[column.equation_index].add(&term);
    }
    multipliers
}

fn linear_combination(system: &CertifiedSystemQ, multipliers: &[PolynomialQ]) -> PolynomialQ {
    multipliers.iter().zip(&system.equations).fold(
        PolynomialQ::zero(system.variables.clone()),
        |sum, (multiplier, equation)| sum.add(&multiplier.mul(equation)),
    )
}

pub(crate) fn obstruction_is_valid(
    system: &CertifiedSystemQ,
    proof_window: &ProofWindow,
    h: &PolynomialQ,
    obstruction: &LeftNullObstruction,
) -> bool {
    let columns = proof_columns(system, proof_window, &obstruction.row_monomials);
    let matrix = rows_from_columns(&columns);
    let rhs = vector_from_polynomial(h, &obstruction.row_monomials);
    let left_product = (0..columns.len())
        .map(|col| {
            obstruction
                .coefficients
                .iter()
                .enumerate()
                .fold(crate::arith::rational_zero(), |sum, (row, coefficient)| {
                    sum + coefficient.clone() * matrix[row][col].clone()
                })
        })
        .collect::<Vec<_>>();
    left_product.iter().all(Zero::is_zero) && !dot_q(&obstruction.coefficients, &rhs).is_zero()
}

fn row_index_map(row_monomials: &[Monomial]) -> BTreeMap<Monomial, usize> {
    row_monomials
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, monomial)| (monomial, index))
        .collect()
}

fn canonical_monomial_key(monomial: &Monomial) -> (u32, Vec<u32>) {
    (monomial.total_degree(), monomial.exponents.clone())
}

#[cfg(test)]
mod tests {
    use num_bigint::BigInt;
    use num_rational::BigRational;

    use super::*;
    use crate::compression::{
        certified_system_from_problem, CertifiedSystemQ, CompressionReplayCertificate,
    };
    use crate::window::ProofWindow;
    use crate::{
        verify_certificate, GuardCertificate, GuardKind, GuardProvenance, GuardRecord, Monomial,
        PolynomialQ, Rational, SolverCertificate, TargetCertificate, TargetProblemQ,
        UniPolynomialQ, Variable, VerificationResult,
    };

    fn variable(symbol: &str) -> Variable {
        Variable {
            symbol: symbol.to_string(),
        }
    }

    fn rational(value: i64) -> Rational {
        BigRational::from_integer(BigInt::from(value))
    }

    fn monomial(exponents: &[u32]) -> Monomial {
        Monomial {
            exponents: exponents.to_vec(),
        }
    }

    fn term(variables: &[Variable], coefficient: i64, exponents: &[u32]) -> PolynomialQ {
        PolynomialQ::from_term(
            variables.to_vec(),
            rational(coefficient),
            monomial(exponents),
        )
    }

    fn polynomial(variables: &[Variable], terms: &[(i64, Vec<u32>)]) -> PolynomialQ {
        terms.iter().fold(
            PolynomialQ::zero(variables.to_vec()),
            |accumulator, entry| accumulator.add(&term(variables, entry.0, &entry.1)),
        )
    }

    fn uni(variable: &Variable, coefficients: &[i64]) -> UniPolynomialQ {
        let mut polynomial = UniPolynomialQ {
            variable: variable.clone(),
            coefficients: coefficients.iter().map(|value| rational(*value)).collect(),
        };
        polynomial.normalize();
        polynomial
    }

    fn system(
        equations: Vec<PolynomialQ>,
        variables: Vec<Variable>,
        target: Variable,
    ) -> CertifiedSystemQ {
        CertifiedSystemQ {
            equations,
            variables,
            target,
            semantic_guards: Vec::new(),
            guard_certificates: Vec::new(),
            replay: CompressionReplayCertificate { steps: Vec::new() },
        }
    }

    #[test]
    fn fixed_proof_builds_exact_ideal_certificate() {
        let x = variable("X");
        let t = variable("T");
        let variables = vec![x.clone(), t.clone()];
        let equations = vec![
            polynomial(&variables, &[(1, vec![2, 0]), (-2, vec![0, 0])]),
            polynomial(&variables, &[(-1, vec![1, 0]), (1, vec![0, 1])]),
        ];
        let problem = TargetProblemQ {
            equations: equations.clone(),
            variables: variables.clone(),
            target: t.clone(),
            semantic_guards: Vec::new(),
        };
        let input = FixedProofInput {
            system: system(equations, variables, t.clone()),
            candidate: uni(&t, &[-2, 0, 1]),
            proof_window: ProofWindow {
                multiplier_supports: vec![
                    vec![monomial(&[0, 0])],
                    vec![monomial(&[1, 0]), monomial(&[0, 1])],
                ],
            },
            certificate_mode: CertificateMode::Ideal,
        };

        let certificate = prove_fixed_target(input).unwrap();

        assert!(matches!(
            certificate,
            TargetCertificate::IdealMembership { .. }
        ));
        assert_eq!(
            verify_certificate(problem, SolverCertificate::TargetCover(certificate)),
            VerificationResult::Verified
        );
    }

    #[test]
    fn semantic_nonzero_guard_reaches_guarded_radical_proof_mode() {
        let t = variable("T");
        let variables = vec![t.clone()];
        let guard = term(&variables, 1, &[1]);
        let record = GuardRecord {
            polynomial: guard.clone(),
            kind: GuardKind::NonZero,
            provenance: GuardProvenance {
                description: "input nonzero guard".to_string(),
            },
        };
        let equation = polynomial(&variables, &[(1, vec![2]), (-1, vec![1])]);
        let problem = TargetProblemQ {
            equations: vec![equation],
            variables: variables.clone(),
            target: t.clone(),
            semantic_guards: vec![record],
        };
        let system = certified_system_from_problem(&problem).unwrap();
        let input = FixedProofInput {
            system,
            candidate: uni(&t, &[-1, 1]),
            proof_window: ProofWindow {
                multiplier_supports: vec![vec![monomial(&[0])]],
            },
            certificate_mode: CertificateMode::GuardedRadical {
                support_power: 1,
                guard_power: 1,
            },
        };

        let certificate = prove_fixed_target(input).unwrap();

        let TargetCertificate::GuardedRadicalMembership {
            guard_certificates,
            guard_product,
            ..
        } = &certificate
        else {
            panic!("guarded radical certificate required");
        };
        assert_eq!(guard_certificates.len(), 1);
        assert_eq!(guard_product, &guard);
        assert_eq!(
            verify_certificate(problem, SolverCertificate::TargetCover(certificate)),
            VerificationResult::Verified
        );
    }

    #[test]
    fn radical_proof_rejects_zero_support_power() {
        let t = variable("T");
        let variables = vec![t.clone()];
        let input = FixedProofInput {
            system: system(
                vec![polynomial(&variables, &[(1, vec![2])])],
                variables,
                t.clone(),
            ),
            candidate: uni(&t, &[0, 1]),
            proof_window: ProofWindow {
                multiplier_supports: vec![vec![monomial(&[0])]],
            },
            certificate_mode: CertificateMode::Radical { support_power: 0 },
        };

        assert!(matches!(
            prove_fixed_target(input),
            Err(ProofFailure::InvalidInput)
        ));
    }

    #[test]
    fn inconsistent_window_emits_left_null_obstruction() {
        let x = variable("X");
        let t = variable("T");
        let variables = vec![x.clone(), t.clone()];
        let certified = system(
            vec![
                polynomial(&variables, &[(1, vec![2, 0]), (-2, vec![0, 0])]),
                polynomial(&variables, &[(-1, vec![1, 0]), (1, vec![0, 1])]),
            ],
            variables.clone(),
            t.clone(),
        );
        let input = FixedProofInput {
            system: certified.clone(),
            candidate: uni(&t, &[-2, 0, 1]),
            proof_window: ProofWindow {
                multiplier_supports: vec![vec![monomial(&[0, 0])], Vec::new()],
            },
            certificate_mode: CertificateMode::Ideal,
        };
        let h = uni(&t, &[-2, 0, 1]).to_multivariate(&variables);

        let failure = prove_fixed_target(input).unwrap_err();

        match failure {
            ProofFailure::Inconsistent { obstruction } => {
                let proof_window = ProofWindow {
                    multiplier_supports: vec![vec![monomial(&[0, 0])], Vec::new()],
                };
                assert!(obstruction_is_valid(
                    &certified,
                    &proof_window,
                    &h,
                    &obstruction
                ));
            }
            _ => panic!("inconsistent proof window should emit obstruction"),
        }
    }

    #[test]
    fn guarded_radical_refuses_unverified_guards() {
        let t = variable("T");
        let variables = vec![t.clone()];
        let guard = polynomial(&variables, &[(1, vec![1])]);
        let record = GuardRecord {
            polynomial: guard.clone(),
            kind: GuardKind::NonZero,
            provenance: GuardProvenance {
                description: "input nonzero".to_string(),
            },
        };
        let bad_record = GuardRecord {
            polynomial: PolynomialQ::one(variables.clone()),
            kind: GuardKind::NonZero,
            provenance: GuardProvenance {
                description: "other nonzero".to_string(),
            },
        };
        let mut certified = system(
            vec![polynomial(&variables, &[(1, vec![2])])],
            variables.clone(),
            t.clone(),
        );
        certified
            .guard_certificates
            .push(GuardCertificate::InputSemanticNonzero {
                guard,
                record: bad_record,
            });

        let input = FixedProofInput {
            system: certified,
            candidate: uni(&t, &[0, 1]),
            proof_window: ProofWindow {
                multiplier_supports: vec![vec![monomial(&[0])]],
            },
            certificate_mode: CertificateMode::GuardedRadical {
                support_power: 1,
                guard_power: 1,
            },
        };
        let problem = TargetProblemQ {
            equations: Vec::new(),
            variables,
            target: t,
            semantic_guards: vec![record],
        };

        assert!(matches!(
            prove_fixed_target_with_problem(input, &problem),
            Err(ProofFailure::InvalidInput)
        ));
    }
}
