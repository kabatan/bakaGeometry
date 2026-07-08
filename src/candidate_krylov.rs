use crate::candidates::{
    CandidateOracle, CandidateOrigin, CandidateTrace, RouteWitnessTrace, TargetCandidate,
};
use crate::compression::CertifiedSystemQ;
use crate::linear_q::{solve_linear_system_q, LinearSolveQ};
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
    let mut candidates = Vec::new();

    for degree in 1..target_powers.column_count() {
        let mut columns = (0..membership.column_count())
            .map(|index| membership.column(index).to_vec())
            .collect::<Vec<_>>();
        for lower_degree in 0..degree {
            columns.push(target_powers.column(lower_degree).to_vec());
        }

        let matrix = rows_from_columns(&columns);
        let rhs = target_powers
            .column(degree)
            .iter()
            .map(|coefficient| -coefficient.clone())
            .collect::<Vec<_>>();
        let LinearSolveQ::Consistent { solution, .. } = solve_linear_system_q(&matrix, &rhs) else {
            continue;
        };

        let mut coefficients = solution[membership.column_count()..].to_vec();
        coefficients.push(crate::arith::rational_one());
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
                support_size: columns.len(),
            })],
        ));
    }

    candidates
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
