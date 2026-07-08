use num_traits::Zero;

use crate::certificates::{EliminationZeroCertificate, SaturatedIdealCertificate};
use crate::compression::CertifiedSystemQ;
use crate::linear_q::{solve_linear_system_q, LinearSolveQ};
use crate::proof::{prove_fixed_target, CertificateMode, FixedProofInput};
use crate::window::{build_membership_matrix_q, make_row_closed_certificate_window, ProofWindow};
use crate::{
    EmptyAdmissibleSetCertificate, ExactIdentity, ExactIdentityKind, Monomial,
    NoTargetEliminantCertificate, PolynomialQ, Rational, ResourceLimits, TargetCertificate,
    UniPolynomialQ,
};

#[derive(Clone, Debug)]
pub(crate) enum CompleteFallbackResult {
    CertifiedSupport(TargetCertificate),
    CertifiedEmpty(EmptyAdmissibleSetCertificate),
    CertifiedNoTargetEliminant(NoTargetEliminantCertificate),
    ResourceFailure(CostTrace),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CostTrace {
    pub target_degree: usize,
    pub multiplier_degree: usize,
    pub matrix_rows: usize,
    pub matrix_cols: usize,
}

pub(crate) fn complete_target_elimination_fallback(
    system: &CertifiedSystemQ,
    limits: &ResourceLimits,
) -> CompleteFallbackResult {
    let max_degree = limits.max_window_degree.unwrap_or(2).max(1);
    let mut last_cost = CostTrace {
        target_degree: 0,
        multiplier_degree: 0,
        matrix_rows: 0,
        matrix_cols: 0,
    };

    for multiplier_degree in 0..=max_degree {
        let proof_window = proof_window_for_degree(system, multiplier_degree);
        if let Some((certificate, _cost)) = certify_empty(system, &proof_window, multiplier_degree)
        {
            return CompleteFallbackResult::CertifiedEmpty(certificate);
        } else {
            last_cost.multiplier_degree = multiplier_degree;
        }

        for target_degree in 1..=max_degree {
            match target_eliminant_candidate(system, &proof_window, target_degree) {
                RelationSearch::Found(candidate, cost) => {
                    last_cost = cost;
                    if let Ok(certificate) = prove_fixed_target(FixedProofInput {
                        system: system.clone(),
                        candidate,
                        proof_window: proof_window.clone(),
                        certificate_mode: CertificateMode::Ideal,
                    }) {
                        return CompleteFallbackResult::CertifiedSupport(certificate);
                    }
                }
                RelationSearch::NotFound(cost) => {
                    last_cost = cost;
                }
            }
        }
    }

    if let Some(certificate) = certify_no_target_eliminant_for_monomial_ideal(system) {
        return CompleteFallbackResult::CertifiedNoTargetEliminant(certificate);
    }

    CompleteFallbackResult::ResourceFailure(last_cost)
}

pub(crate) fn try_empty_admissible_set_certificate(
    system: &CertifiedSystemQ,
    limits: &ResourceLimits,
) -> Option<EmptyAdmissibleSetCertificate> {
    let max_degree = limits.max_window_degree.unwrap_or(2).max(1);
    for multiplier_degree in 0..=max_degree {
        let proof_window = proof_window_for_degree(system, multiplier_degree);
        if let Some((certificate, _cost)) = certify_empty(system, &proof_window, multiplier_degree)
        {
            return Some(certificate);
        }
    }
    None
}

enum RelationSearch {
    Found(UniPolynomialQ, CostTrace),
    NotFound(CostTrace),
}

fn proof_window_for_degree(system: &CertifiedSystemQ, multiplier_degree: usize) -> ProofWindow {
    ProofWindow {
        multiplier_supports: vec![
            monomials_up_to_degree(system.variables.len(), multiplier_degree);
            system.equations.len()
        ],
    }
}

fn certify_empty(
    system: &CertifiedSystemQ,
    proof_window: &ProofWindow,
    multiplier_degree: usize,
) -> Option<(EmptyAdmissibleSetCertificate, CostTrace)> {
    let window =
        make_row_closed_certificate_window(system, 0, proof_window.multiplier_supports.clone());
    let membership = build_membership_matrix_q(system, &window);
    let rhs = vector_for_constant_one(&membership.row_monomials);
    let matrix = rows_from_columns(
        &(0..membership.column_count())
            .map(|index| membership.column(index).to_vec())
            .collect::<Vec<_>>(),
    );
    let cost = CostTrace {
        target_degree: 0,
        multiplier_degree,
        matrix_rows: matrix.len(),
        matrix_cols: matrix.first().map_or(0, Vec::len),
    };
    let LinearSolveQ::Consistent { solution, .. } = solve_linear_system_q(&matrix, &rhs) else {
        return None;
    };
    let multipliers = restore_multipliers(system, &proof_window.multiplier_supports, &solution);
    let certificate = EmptyAdmissibleSetCertificate::AlgebraicInfeasibility {
        multipliers,
        identity: ExactIdentity {
            kind: ExactIdentityKind::AlgebraicInfeasibility,
        },
    };
    Some((certificate, cost))
}

fn target_eliminant_candidate(
    system: &CertifiedSystemQ,
    proof_window: &ProofWindow,
    target_degree: usize,
) -> RelationSearch {
    let window = make_row_closed_certificate_window(
        system,
        target_degree,
        proof_window.multiplier_supports.clone(),
    );
    let membership = build_membership_matrix_q(system, &window);
    let mut columns = (0..membership.column_count())
        .map(|index| membership.column(index).to_vec())
        .collect::<Vec<_>>();
    for degree in 0..target_degree {
        columns.push(vector_for_target_power(
            system,
            &membership.row_monomials,
            degree,
            -1,
        ));
    }
    let matrix = rows_from_columns(&columns);
    let rhs = vector_for_target_power(system, &membership.row_monomials, target_degree, 1);
    let cost = CostTrace {
        target_degree,
        multiplier_degree: proof_window
            .multiplier_supports
            .iter()
            .flatten()
            .map(|monomial| monomial.total_degree() as usize)
            .max()
            .unwrap_or(0),
        matrix_rows: matrix.len(),
        matrix_cols: matrix.first().map_or(0, Vec::len),
    };
    let LinearSolveQ::Consistent { solution, .. } = solve_linear_system_q(&matrix, &rhs) else {
        return RelationSearch::NotFound(cost);
    };

    let target_column_offset = membership.column_count();
    let mut coefficients = (0..target_degree)
        .map(|index| solution[target_column_offset + index].clone())
        .collect::<Vec<_>>();
    coefficients.push(crate::arith::rational_one());
    let mut candidate = UniPolynomialQ {
        variable: system.target.clone(),
        coefficients,
    };
    candidate = candidate.primitive_integer_normalized();
    if candidate.is_zero() {
        RelationSearch::NotFound(cost)
    } else {
        RelationSearch::Found(candidate, cost)
    }
}

fn certify_no_target_eliminant_for_monomial_ideal(
    system: &CertifiedSystemQ,
) -> Option<NoTargetEliminantCertificate> {
    let target_index = system
        .variables
        .iter()
        .position(|variable| variable == &system.target)?;
    if !system.guard_certificates.is_empty() {
        return None;
    }
    if !system.equations.iter().all(|equation| {
        if equation.terms.is_empty() {
            return true;
        }
        if equation.terms.len() != 1 {
            return false;
        }
        let monomial = equation.terms.keys().next().unwrap();
        monomial.exponents[target_index] == 0
            && monomial
                .exponents
                .iter()
                .enumerate()
                .any(|(index, exponent)| index != target_index && *exponent != 0)
    }) {
        return None;
    }

    Some(NoTargetEliminantCertificate {
        saturated_ideal_description: SaturatedIdealCertificate {
            guard_certificates: Vec::new(),
        },
        elimination_certificate: EliminationZeroCertificate {
            identity: ExactIdentity {
                kind: ExactIdentityKind::CompressionReplay,
            },
        },
        guard_certificates: Vec::new(),
    })
}

fn vector_for_constant_one(row_monomials: &[Monomial]) -> Vec<Rational> {
    let mut vector = vec![crate::arith::rational_zero(); row_monomials.len()];
    if let Some(row) = row_monomials
        .iter()
        .position(|monomial| monomial.exponents.iter().all(Zero::is_zero))
    {
        vector[row] = crate::arith::rational_one();
    }
    vector
}

fn vector_for_target_power(
    system: &CertifiedSystemQ,
    row_monomials: &[Monomial],
    degree: usize,
    sign: i32,
) -> Vec<Rational> {
    let target_index = system
        .variables
        .iter()
        .position(|variable| variable == &system.target)
        .unwrap();
    let mut exponents = vec![0; system.variables.len()];
    exponents[target_index] = degree as u32;
    let monomial = Monomial { exponents };
    let mut vector = vec![crate::arith::rational_zero(); row_monomials.len()];
    if let Some(row) = row_monomials
        .iter()
        .position(|candidate| candidate == &monomial)
    {
        vector[row] = if sign < 0 {
            -crate::arith::rational_one()
        } else {
            crate::arith::rational_one()
        };
    }
    vector
}

fn restore_multipliers(
    system: &CertifiedSystemQ,
    multiplier_supports: &[Vec<Monomial>],
    solution: &[Rational],
) -> Vec<PolynomialQ> {
    let mut multipliers = vec![PolynomialQ::zero(system.variables.clone()); system.equations.len()];
    let mut column_index = 0;
    for (equation_index, supports) in multiplier_supports.iter().enumerate() {
        for multiplier_monomial in supports {
            let coefficient = solution[column_index].clone();
            column_index += 1;
            if coefficient.is_zero() {
                continue;
            }
            let term = PolynomialQ::from_term(
                system.variables.clone(),
                coefficient,
                multiplier_monomial.clone(),
            );
            multipliers[equation_index] = multipliers[equation_index].add(&term);
        }
    }
    multipliers
}

fn rows_from_columns(columns: &[Vec<Rational>]) -> Vec<Vec<Rational>> {
    let rows = columns.first().map_or(0, Vec::len);
    (0..rows)
        .map(|row| columns.iter().map(|column| column[row].clone()).collect())
        .collect()
}

fn monomials_up_to_degree(variable_count: usize, max_degree: usize) -> Vec<Monomial> {
    let mut monomials = Vec::new();
    let mut current = vec![0; variable_count];
    enumerate_monomials(
        variable_count,
        max_degree as u32,
        0,
        &mut current,
        &mut monomials,
    );
    monomials
}

fn enumerate_monomials(
    variable_count: usize,
    remaining_degree: u32,
    index: usize,
    current: &mut [u32],
    monomials: &mut Vec<Monomial>,
) {
    if index == variable_count {
        monomials.push(Monomial {
            exponents: current.to_vec(),
        });
        return;
    }
    for exponent in 0..=remaining_degree {
        current[index] = exponent;
        enumerate_monomials(
            variable_count,
            remaining_degree - exponent,
            index + 1,
            current,
            monomials,
        );
    }
    current[index] = 0;
}

#[cfg(test)]
mod tests {
    use num_bigint::BigInt;
    use num_rational::BigRational;

    use super::*;
    use crate::compression::CompressionReplayCertificate;
    use crate::{
        verify_certificate, GuardRecord, SolverCertificate, TargetProblemQ, Variable,
        VerificationResult,
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
        terms
            .iter()
            .fold(PolynomialQ::zero(variables.to_vec()), |sum, entry| {
                sum.add(&term(variables, entry.0, &entry.1))
            })
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
            guard_certificates: Vec::new(),
            replay: CompressionReplayCertificate { steps: Vec::new() },
        }
    }

    fn limits() -> ResourceLimits {
        ResourceLimits {
            max_window_degree: Some(2),
            max_proof_weight: None,
            max_matrix_rows: None,
            max_matrix_cols: None,
            max_candidate_count: None,
        }
    }

    #[test]
    fn fallback_certifies_simple_target_eliminant() {
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
            semantic_guards: Vec::<GuardRecord>::new(),
        };

        let result =
            complete_target_elimination_fallback(&system(equations, variables, t), &limits());

        let CompleteFallbackResult::CertifiedSupport(certificate) = result else {
            panic!("fallback should certify target support");
        };
        assert_eq!(
            verify_certificate(problem, SolverCertificate::TargetCover(certificate)),
            VerificationResult::Verified
        );
    }

    #[test]
    fn fallback_certifies_empty_admissible_set() {
        let t = variable("T");
        let variables = vec![t.clone()];
        let equations = vec![PolynomialQ::one(variables.clone())];
        let problem = TargetProblemQ {
            equations: equations.clone(),
            variables: variables.clone(),
            target: t.clone(),
            semantic_guards: Vec::<GuardRecord>::new(),
        };

        let result =
            complete_target_elimination_fallback(&system(equations, variables, t), &limits());

        let CompleteFallbackResult::CertifiedEmpty(certificate) = result else {
            panic!("fallback should certify empty admissible set");
        };
        assert_eq!(
            verify_certificate(problem, SolverCertificate::EmptyAdmissibleSet(certificate)),
            VerificationResult::Verified
        );
    }

    #[test]
    fn no_target_eliminant_is_algebraic_certificate_only() {
        let x = variable("X");
        let t = variable("T");
        let variables = vec![x.clone(), t.clone()];
        let equations = vec![polynomial(&variables, &[(1, vec![1, 0])])];
        let problem = TargetProblemQ {
            equations: equations.clone(),
            variables: variables.clone(),
            target: t.clone(),
            semantic_guards: Vec::<GuardRecord>::new(),
        };

        let result =
            complete_target_elimination_fallback(&system(equations, variables, t), &limits());

        let CompleteFallbackResult::CertifiedNoTargetEliminant(certificate) = result else {
            panic!("fallback should certify no target eliminant");
        };
        assert!(matches!(
            verify_certificate(
                problem,
                SolverCertificate::NoNonzeroTargetEliminant(certificate)
            ),
            VerificationResult::CertificateDesignGap { .. }
        ));
    }
}
