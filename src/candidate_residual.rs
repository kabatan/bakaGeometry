use crate::candidates::{
    CandidateOracle, CandidateOrigin, CandidateTrace, ModularWitnessTrace, TargetCandidate,
};
use crate::compression::CertifiedSystemQ;
use crate::finite_field::PrimeModulus;
use crate::linear_fp::{columns_to_matrix, nullspace_matrix_fp};
use crate::residual::{DenseEchelonResidualOracleFp, ResidualOracleFp};
use crate::univariate::UniPolynomialFp;
use crate::window::{build_membership_matrix_q, build_target_power_matrix_q, CertificateWindow};

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

            candidates.push(TargetCandidate {
                support_mod_primes: vec![support],
                reconstructed: None,
                origin: CandidateOrigin::ResidualCyclic,
                traces: vec![CandidateTrace::ModularWitness(ModularWitnessTrace {
                    prime: *prime,
                    active_multiplier_supports: window.multiplier_supports.clone(),
                    relation_coefficients: coefficients,
                    residual_vectors: residuals.clone(),
                })],
            });
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

#[cfg(test)]
mod tests {
    use num_bigint::BigInt;
    use num_rational::BigRational;

    use super::*;
    use crate::compression::CompressionReplayCertificate;
    use crate::window::make_row_closed_certificate_window;
    use crate::{Monomial, PolynomialQ, Rational, Variable};

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
}
