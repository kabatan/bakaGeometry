use crate::candidates::{
    CandidateOracle, CandidateOrigin, CandidateTrace, ModularWitnessTrace, TargetCandidate,
};
use crate::compression::{
    CertifiedSystemQ, CompressionReplayCertificate, CompressionStepCertificate,
};
use crate::finite_field::{rational_to_mod_prime, PrimeModulus};
use crate::linear_fp::{
    columns_to_matrix, nullspace_matrix_fp, solve_linear_system_fp, LinearSolveFp,
};
use crate::residual::{DenseEchelonResidualOracleFp, ResidualOracleFp};
use crate::univariate::UniPolynomialFp;
use crate::window::{
    build_membership_matrix_q, build_target_power_matrix_q, CertificateWindow, TargetPowerMatrixQ,
};
use crate::{
    GuardCertificate, NullstellensatzCertificate, PolynomialQ, Rational,
    RealInfeasibilityCertificate,
};

pub(crate) struct ResidualCyclicOracle {
    pub primes: Vec<u64>,
}

impl CandidateOracle for ResidualCyclicOracle {
    fn generate(
        &self,
        system: &CertifiedSystemQ,
        window: &CertificateWindow,
    ) -> Vec<TargetCandidate> {
        residual_cyclic_candidates(system, window, &self.primes)
    }
}

pub fn residual_cyclic_candidates(
    system: &CertifiedSystemQ,
    window: &CertificateWindow,
    primes: &[u64],
) -> Vec<TargetCandidate> {
    let membership = build_membership_matrix_q(system, window);
    let target_powers = build_target_power_matrix_q(system, window);
    let mut candidates = Vec::new();

    for prime in primes {
        let Some(modulus) = PrimeModulus::new(*prime) else {
            continue;
        };
        if !residual_prime_is_admissible(system, &target_powers, modulus) {
            continue;
        }
        let Some(membership_columns) = membership.to_modular_columns(modulus) else {
            continue;
        };
        let Some(target_power_columns) = target_powers.to_modular_columns(modulus) else {
            continue;
        };
        let Some(oracle) = DenseEchelonResidualOracleFp::from_columns_with_row_count(
            *prime,
            membership.row_count(),
            membership_columns,
        ) else {
            continue;
        };

        let membership_columns = membership
            .to_modular_columns(modulus)
            .expect("membership columns were reduced successfully above");
        let membership_matrix = columns_to_matrix(&membership_columns);
        let residuals = target_power_columns
            .iter()
            .map(|column| oracle.reduce(column))
            .collect::<Vec<_>>();
        let relation_matrix = columns_to_matrix(&residuals);

        for relation in nullspace_matrix_fp(&relation_matrix, modulus) {
            let Some(coefficients) = normalized_relation_coefficients(relation, modulus) else {
                continue;
            };
            let mut support = UniPolynomialFp {
                variable: system.target.clone(),
                modulus: *prime,
                coefficients: coefficients.clone(),
            };
            support.normalize();
            if support.is_zero() {
                continue;
            }
            let Some(target_relation) =
                target_relation_vector(&target_power_columns, &coefficients, modulus)
            else {
                continue;
            };
            let LinearSolveFp::Consistent { solution, .. } =
                solve_linear_system_fp(&membership_matrix, &target_relation, modulus)
            else {
                continue;
            };
            let Some(active_multiplier_supports) =
                active_multiplier_supports_from_solution(window, &solution)
            else {
                continue;
            };

            candidates.push(TargetCandidate::from_origin(
                vec![support],
                None,
                CandidateOrigin::ResidualCyclic,
                vec![CandidateTrace::ModularWitness(ModularWitnessTrace {
                    prime: *prime,
                    active_multiplier_supports,
                    relation_coefficients: coefficients,
                    residual_vectors: residuals.clone(),
                })],
            ));
        }
    }

    candidates
}

fn normalized_relation_coefficients(
    mut relation: Vec<u64>,
    modulus: PrimeModulus,
) -> Option<Vec<u64>> {
    while relation.last().is_some_and(|coefficient| *coefficient == 0) {
        relation.pop();
    }
    let leading = *relation.last()?;
    let leading_inverse = modulus.inv(leading)?;
    for coefficient in &mut relation {
        *coefficient = modulus.mul(*coefficient, leading_inverse);
    }
    Some(relation)
}

fn target_relation_vector(
    target_power_columns: &[Vec<u64>],
    coefficients: &[u64],
    modulus: PrimeModulus,
) -> Option<Vec<u64>> {
    if coefficients.len() > target_power_columns.len() {
        return None;
    }
    let row_count = target_power_columns.first().map_or(0, Vec::len);
    let mut target_relation = vec![0; row_count];
    for (coefficient, column) in coefficients.iter().zip(target_power_columns) {
        if column.len() != row_count {
            return None;
        }
        for (entry, column_value) in target_relation.iter_mut().zip(column) {
            *entry = modulus.add(*entry, modulus.mul(*coefficient, *column_value));
        }
    }
    Some(target_relation)
}

fn active_multiplier_supports_from_solution(
    window: &CertificateWindow,
    solution: &[u64],
) -> Option<Vec<Vec<crate::Monomial>>> {
    let total_columns = window
        .multiplier_supports
        .iter()
        .map(Vec::len)
        .sum::<usize>();
    if total_columns != solution.len() {
        return None;
    }

    let mut column_index = 0;
    let mut active_supports = Vec::with_capacity(window.multiplier_supports.len());
    for support in &window.multiplier_supports {
        let mut active = Vec::new();
        for monomial in support {
            if solution[column_index] != 0 {
                active.push(monomial.clone());
            }
            column_index += 1;
        }
        active_supports.push(active);
    }
    Some(active_supports)
}

fn residual_prime_is_admissible(
    system: &CertifiedSystemQ,
    target_powers: &TargetPowerMatrixQ,
    modulus: PrimeModulus,
) -> bool {
    system
        .equations
        .iter()
        .all(|polynomial| polynomial_is_admissible(polynomial, modulus))
        && system
            .guard_certificates
            .iter()
            .all(|certificate| guard_certificate_is_admissible(certificate, modulus))
        && replay_is_admissible(&system.replay, modulus)
        && target_powers_are_admissible(target_powers, modulus)
}

fn polynomial_is_admissible(polynomial: &PolynomialQ, modulus: PrimeModulus) -> bool {
    polynomial
        .terms
        .values()
        .all(|coefficient| rational_is_admissible(coefficient, modulus))
}

fn rational_is_admissible(coefficient: &Rational, modulus: PrimeModulus) -> bool {
    rational_to_mod_prime(coefficient, modulus).is_some()
}

fn guard_certificate_is_admissible(certificate: &GuardCertificate, modulus: PrimeModulus) -> bool {
    match certificate {
        GuardCertificate::InputSemanticNonzero { guard, record } => {
            polynomial_is_admissible(guard, modulus)
                && polynomial_is_admissible(&record.polynomial, modulus)
        }
        GuardCertificate::AlgebraicNonvanishing { guard, certificate } => {
            polynomial_is_admissible(guard, modulus)
                && nullstellensatz_certificate_is_admissible(certificate, modulus)
        }
        GuardCertificate::RealAdmissibleNonvanishing { guard, certificate } => {
            polynomial_is_admissible(guard, modulus)
                && real_infeasibility_certificate_is_admissible(certificate, modulus)
        }
        GuardCertificate::DerivedProduct {
            product, factors, ..
        } => {
            polynomial_is_admissible(product, modulus)
                && factors
                    .iter()
                    .all(|factor| guard_certificate_is_admissible(factor, modulus))
        }
    }
}

fn nullstellensatz_certificate_is_admissible(
    certificate: &NullstellensatzCertificate,
    modulus: PrimeModulus,
) -> bool {
    certificate
        .multipliers
        .iter()
        .all(|polynomial| polynomial_is_admissible(polynomial, modulus))
        && polynomial_is_admissible(&certificate.guard_multiplier, modulus)
}

fn real_infeasibility_certificate_is_admissible(
    certificate: &RealInfeasibilityCertificate,
    modulus: PrimeModulus,
) -> bool {
    match certificate {
        RealInfeasibilityCertificate::VerifiedByExactAlgebraicCertificate(certificate) => {
            nullstellensatz_certificate_is_admissible(certificate, modulus)
        }
        RealInfeasibilityCertificate::VerifiedByExternalReplay { .. } => true,
    }
}

fn replay_is_admissible(replay: &CompressionReplayCertificate, modulus: PrimeModulus) -> bool {
    replay.steps.iter().all(|step| match step {
        CompressionStepCertificate::IdentityInput => true,
        CompressionStepCertificate::DefinitionSubstitution { expression, .. } => {
            polynomial_is_admissible(expression, modulus)
        }
        CompressionStepCertificate::AffineElimination {
            pivot, pivot_guard, ..
        } => {
            polynomial_is_admissible(pivot, modulus)
                && guard_certificate_is_admissible(pivot_guard, modulus)
        }
        CompressionStepCertificate::ExplicitGuardSaturation { guard, .. } => {
            guard_certificate_is_admissible(guard, modulus)
        }
        CompressionStepCertificate::PrimitiveNormalization {
            before,
            after,
            multiplier,
        } => {
            polynomial_is_admissible(before, modulus)
                && polynomial_is_admissible(after, modulus)
                && rational_is_admissible(multiplier, modulus)
        }
        CompressionStepCertificate::ZeroEquationRemoval { removed } => {
            polynomial_is_admissible(removed, modulus)
        }
    })
}

fn target_powers_are_admissible(target_powers: &TargetPowerMatrixQ, modulus: PrimeModulus) -> bool {
    (0..target_powers.column_count()).all(|index| {
        target_powers
            .column(index)
            .iter()
            .all(|coefficient| rational_is_admissible(coefficient, modulus))
    })
}

#[cfg(test)]
mod tests {
    use num_bigint::BigInt;
    use num_rational::BigRational;

    use super::*;
    use crate::compression::{CompressionReplayCertificate, CompressionStepCertificate};
    use crate::window::make_row_closed_certificate_window;
    use crate::{
        ExactIdentity, ExactIdentityKind, GuardCertificate, GuardKind, GuardProvenance,
        GuardRecord, Monomial, PolynomialQ, Rational, Variable,
    };

    fn variable(symbol: &str) -> Variable {
        Variable {
            symbol: symbol.to_string(),
        }
    }

    fn rational(value: i64) -> Rational {
        BigRational::from_integer(BigInt::from(value))
    }

    fn fraction(numerator: i64, denominator: i64) -> Rational {
        BigRational::new(BigInt::from(numerator), BigInt::from(denominator))
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

    #[test]
    fn residual_cyclic_route_returns_modular_target_candidate_only() {
        let x = variable("X");
        let t = variable("T");
        let variables = vec![x.clone(), t.clone()];
        let equations = vec![
            polynomial(&variables, &[(1, vec![2, 0]), (-2, vec![0, 0])]),
            polynomial(&variables, &[(-1, vec![1, 0]), (1, vec![0, 1])]),
        ];
        let sys = system(equations, variables, t.clone());
        let x_support = monomial(&[1, 0]);
        let t_support = monomial(&[0, 1]);
        let window = make_row_closed_certificate_window(
            &sys,
            2,
            vec![vec![monomial(&[0, 0])], vec![t_support, x_support]],
        );

        let candidates = residual_cyclic_candidates(&sys, &window, &[5]);

        assert!(candidates.iter().any(|candidate| {
            candidate.origin == CandidateOrigin::ResidualCyclic
                && candidate.reconstructed.is_none()
                && candidate.support_mod_primes.iter().any(|support| {
                    support.modulus == 5
                        && support.variable == t
                        && support.coefficients == vec![3, 0, 1]
                })
        }));
        assert!(candidates
            .iter()
            .all(|candidate| !candidate.traces.is_empty()));
    }

    #[test]
    fn residual_witness_active_support_is_solved_not_full_window_copy() {
        let x = variable("X");
        let t = variable("T");
        let variables = vec![x.clone(), t.clone()];
        let equations = vec![
            polynomial(&variables, &[(1, vec![2, 0]), (-2, vec![0, 0])]),
            polynomial(&variables, &[(-1, vec![1, 0]), (1, vec![0, 1])]),
        ];
        let sys = system(equations, variables, t.clone());
        let one = monomial(&[0, 0]);
        let x_support = monomial(&[1, 0]);
        let t_support = monomial(&[0, 1]);
        let unused_support = monomial(&[2, 0]);
        let window = make_row_closed_certificate_window(
            &sys,
            2,
            vec![
                vec![one.clone(), unused_support],
                vec![one, t_support, x_support],
            ],
        );

        let candidates = residual_cyclic_candidates(&sys, &window, &[5]);
        let witness = candidates
            .iter()
            .filter_map(|candidate| candidate.traces.first())
            .find_map(|trace| match trace {
                CandidateTrace::ModularWitness(witness)
                    if witness.relation_coefficients == vec![3, 0, 1] =>
                {
                    Some(witness)
                }
                _ => None,
            })
            .expect("residual relation should produce a modular witness");

        assert_ne!(
            witness.active_multiplier_supports,
            window.multiplier_supports
        );
        assert!(witness.active_multiplier_supports[0].contains(&monomial(&[0, 0])));
        assert!(!witness.active_multiplier_supports[0].contains(&monomial(&[2, 0])));
    }

    #[test]
    fn residual_prime_filter_reads_guard_rationals() {
        let x = variable("X");
        let t = variable("T");
        let variables = vec![x.clone(), t.clone()];
        let equations = vec![
            polynomial(&variables, &[(1, vec![2, 0]), (-2, vec![0, 0])]),
            polynomial(&variables, &[(-1, vec![1, 0]), (1, vec![0, 1])]),
        ];
        let mut sys = system(equations, variables.clone(), t.clone());
        let guard = PolynomialQ::from_term(variables.clone(), fraction(1, 5), monomial(&[0, 0]));
        let record = GuardRecord {
            polynomial: guard.clone(),
            kind: GuardKind::NonZero,
            provenance: GuardProvenance {
                description: "guard prime filter test".to_string(),
            },
        };
        sys.guard_certificates
            .push(GuardCertificate::InputSemanticNonzero { guard, record });
        let window = make_row_closed_certificate_window(
            &sys,
            2,
            vec![
                vec![monomial(&[0, 0])],
                vec![monomial(&[0, 1]), monomial(&[1, 0])],
            ],
        );

        assert!(residual_cyclic_candidates(&sys, &window, &[5]).is_empty());
        assert!(!residual_cyclic_candidates(&sys, &window, &[7]).is_empty());
    }

    #[test]
    fn residual_prime_filter_reads_replay_rationals() {
        let x = variable("X");
        let t = variable("T");
        let variables = vec![x.clone(), t.clone()];
        let equations = vec![
            polynomial(&variables, &[(1, vec![2, 0]), (-2, vec![0, 0])]),
            polynomial(&variables, &[(-1, vec![1, 0]), (1, vec![0, 1])]),
        ];
        let mut sys = system(equations, variables.clone(), t.clone());
        sys.replay
            .steps
            .push(CompressionStepCertificate::PrimitiveNormalization {
                before: PolynomialQ::from_term(
                    variables.clone(),
                    fraction(1, 5),
                    monomial(&[0, 0]),
                ),
                after: PolynomialQ::from_term(variables.clone(), rational(1), monomial(&[0, 0])),
                multiplier: fraction(1, 5),
            });
        sys.replay
            .steps
            .push(CompressionStepCertificate::DefinitionSubstitution {
                variable: x,
                expression: PolynomialQ::from_term(variables, rational(1), monomial(&[0, 0])),
                identity: ExactIdentity {
                    kind: ExactIdentityKind::CompressionReplay,
                },
            });
        let window = make_row_closed_certificate_window(
            &sys,
            2,
            vec![
                vec![monomial(&[0, 0])],
                vec![monomial(&[0, 1]), monomial(&[1, 0])],
            ],
        );

        assert!(residual_cyclic_candidates(&sys, &window, &[5]).is_empty());
        assert!(!residual_cyclic_candidates(&sys, &window, &[7]).is_empty());
    }
}
