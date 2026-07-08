use num_traits::Zero;

use crate::candidates::{
    CandidateOracle, CandidateOrigin, CandidateTrace, RouteWitnessTrace, TargetCandidate,
};
use crate::compression::CertifiedSystemQ;
use crate::linear_q::nullspace_matrix_q;
use crate::window::{build_membership_matrix_q, build_target_power_matrix_q, CertificateWindow};
use crate::{Rational, UniPolynomialQ};

pub(crate) struct TargetCyclicKrylovOracle;

impl CandidateOracle for TargetCyclicKrylovOracle {
    fn generate(
        &self,
        system: &CertifiedSystemQ,
        window: &CertificateWindow,
    ) -> Vec<TargetCandidate> {
        target_cyclic_krylov_candidates(system, window)
    }
}

pub(crate) fn target_cyclic_krylov_candidates(
    system: &CertifiedSystemQ,
    window: &CertificateWindow,
) -> Vec<TargetCandidate> {
    let membership = build_membership_matrix_q(system, window);
    let target_powers = build_target_power_matrix_q(system, window);
    let quotient = QuotientResidualHandleQ::from_membership_columns(
        (0..membership.column_count())
            .map(|index| membership.column(index).to_vec())
            .collect(),
    );
    if quotient.left_null_basis.is_empty() {
        return Vec::new();
    }
    let mut candidates = Vec::new();
    let residuals = (0..target_powers.column_count())
        .map(|index| quotient.residual_class(target_powers.column(index)))
        .collect::<Vec<_>>();

    for degree in 1..target_powers.column_count() {
        let relation_matrix = rows_from_columns(&residuals[..=degree]);
        let Some(coefficients) = nullspace_matrix_q(&relation_matrix)
            .into_iter()
            .find_map(|relation| normalize_relation_with_leading(relation, degree))
        else {
            continue;
        };
        let mut support = UniPolynomialQ {
            variable: system.target.clone(),
            coefficients,
        }
        .primitive_integer_normalized();
        support.normalize();
        if support.is_zero() {
            continue;
        }

        candidates.push(TargetCandidate::from_origin(
            Vec::new(),
            Some(support),
            CandidateOrigin::TargetCyclicKrylov,
            vec![CandidateTrace::RouteWitness(RouteWitnessTrace {
                origin: CandidateOrigin::TargetCyclicKrylov,
                equation_indices: (0..system.equations.len()).collect(),
                support_size: quotient.left_null_basis.len() + membership.column_count(),
            })],
        ));
    }

    candidates
}

#[derive(Clone, Debug)]
struct QuotientResidualHandleQ {
    left_null_basis: Vec<Vec<Rational>>,
}

impl QuotientResidualHandleQ {
    fn from_membership_columns(columns: Vec<Vec<Rational>>) -> Self {
        Self {
            left_null_basis: nullspace_matrix_q(&columns),
        }
    }

    fn residual_class(&self, vector: &[Rational]) -> Vec<Rational> {
        self.left_null_basis
            .iter()
            .map(|left_relation| {
                left_relation
                    .iter()
                    .zip(vector)
                    .fold(crate::arith::rational_zero(), |sum, (left, value)| {
                        sum + left.clone() * value.clone()
                    })
            })
            .collect()
    }
}

fn normalize_relation_with_leading(
    mut relation: Vec<Rational>,
    leading_degree: usize,
) -> Option<Vec<Rational>> {
    if relation.len() <= leading_degree || relation[leading_degree].is_zero() {
        return None;
    }
    relation.truncate(leading_degree + 1);
    let leading = relation[leading_degree].clone();
    for coefficient in &mut relation {
        *coefficient /= leading.clone();
    }
    Some(relation)
}

fn rows_from_columns(columns: &[Vec<Rational>]) -> Vec<Vec<Rational>> {
    let rows = columns.first().map_or(0, Vec::len);
    (0..rows)
        .map(|row| columns.iter().map(|column| column[row].clone()).collect())
        .collect()
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
        terms
            .iter()
            .fold(PolynomialQ::zero(variables.to_vec()), |sum, entry| {
                sum.add(&term(variables, entry.0, &entry.1))
            })
    }

    #[test]
    fn krylov_route_uses_target_power_recurrence() {
        let x = variable("X");
        let t = variable("T");
        let variables = vec![x.clone(), t.clone()];
        let system = CertifiedSystemQ {
            equations: vec![
                polynomial(&variables, &[(1, vec![2, 0]), (-2, vec![0, 0])]),
                polynomial(&variables, &[(-1, vec![1, 0]), (1, vec![0, 1])]),
            ],
            variables,
            target: t.clone(),
            semantic_guards: Vec::new(),
            guard_certificates: Vec::new(),
            replay: CompressionReplayCertificate { steps: Vec::new() },
        };
        let window = make_row_closed_certificate_window(
            &system,
            2,
            vec![
                vec![monomial(&[0, 0])],
                vec![monomial(&[1, 0]), monomial(&[0, 1])],
            ],
        );

        let candidates = target_cyclic_krylov_candidates(&system, &window);

        assert!(candidates.iter().any(|candidate| {
            candidate.origin == CandidateOrigin::TargetCyclicKrylov
                && candidate.reconstructed.as_ref().is_some_and(|support| {
                    support.coefficients == vec![rational(-2), rational(0), rational(1)]
                })
        }));
    }
}
