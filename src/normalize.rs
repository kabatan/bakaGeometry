use num_bigint::BigInt;
use num_traits::{One, Signed, Zero};

use crate::candidates::{CandidateTrace, TargetCandidate};
use crate::rational_reconstruction::{
    reconstruct_univariate_q_from_modular, RationalReconstructionBounds,
};
use crate::{Rational, UniPolynomialQ};

pub(crate) fn normalize_candidate(mut candidate: TargetCandidate) -> Option<TargetCandidate> {
    if let Some(reconstructed) = candidate.reconstructed.take() {
        let normalized = reconstructed.primitive_integer_normalized();
        if normalized.is_zero() {
            return None;
        }
        candidate.reconstructed = Some(normalized);
    } else if let Some(reconstructed) = reconstruct_from_modular_support(&candidate) {
        candidate.reconstructed = Some(reconstructed.primitive_integer_normalized());
    }
    Some(candidate)
}

pub(crate) fn normalize_candidates(candidates: Vec<TargetCandidate>) -> Vec<TargetCandidate> {
    candidates
        .into_iter()
        .filter_map(normalize_candidate)
        .collect()
}

pub(crate) fn rank_candidates(mut candidates: Vec<TargetCandidate>) -> Vec<TargetCandidate> {
    candidates.sort_by(|left, right| candidate_rank(left).cmp(&candidate_rank(right)));
    candidates
}

pub(crate) fn factor_schedule(candidate: &TargetCandidate) -> Vec<TargetCandidate> {
    vec![candidate.clone()]
}

fn candidate_rank(candidate: &TargetCandidate) -> CandidateRank {
    CandidateRank {
        modular_only: candidate.reconstructed.is_none(),
        degree: candidate_degree(candidate),
        prime_count_order: usize::MAX - candidate.support_mod_primes.len(),
        origin: candidate.origin,
        coefficient_height: coefficient_height(candidate),
        active_support_size: active_support_size(candidate),
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct CandidateRank {
    modular_only: bool,
    degree: usize,
    prime_count_order: usize,
    origin: crate::candidates::CandidateOrigin,
    coefficient_height: BigInt,
    active_support_size: usize,
}

fn candidate_degree(candidate: &TargetCandidate) -> usize {
    if let Some(reconstructed) = &candidate.reconstructed {
        return reconstructed.degree().unwrap_or(usize::MAX);
    }
    candidate
        .support_mod_primes
        .iter()
        .filter_map(|support| {
            support
                .coefficients
                .iter()
                .rposition(|coefficient| *coefficient != 0)
        })
        .min()
        .unwrap_or(usize::MAX)
}

fn coefficient_height(candidate: &TargetCandidate) -> BigInt {
    candidate
        .reconstructed
        .as_ref()
        .map(|support| {
            support
                .coefficients
                .iter()
                .flat_map(|coefficient| {
                    [coefficient.numer().abs(), coefficient.denom().abs()].into_iter()
                })
                .max()
                .unwrap_or_else(BigInt::zero)
        })
        .unwrap_or_else(BigInt::zero)
}

fn active_support_size(candidate: &TargetCandidate) -> usize {
    candidate
        .traces
        .iter()
        .map(|trace| match trace {
            CandidateTrace::DirectEquation { .. } => 1,
            CandidateTrace::ModularWitness(witness) => witness
                .active_multiplier_supports
                .iter()
                .map(Vec::len)
                .sum(),
            CandidateTrace::RouteWitness(witness) => witness.support_size,
            CandidateTrace::SliceWitness(witness) => witness.assignments.len(),
        })
        .min()
        .unwrap_or(0)
}

fn reconstruct_from_modular_support(candidate: &TargetCandidate) -> Option<UniPolynomialQ> {
    if candidate.support_mod_primes.is_empty()
        || candidate
            .support_mod_primes
            .iter()
            .all(crate::univariate::UniPolynomialFp::is_zero)
    {
        return None;
    }
    if candidate.support_mod_primes.len() == 1 {
        return single_prime_integer_lift(&candidate.support_mod_primes[0]);
    }

    let modulus_product = candidate
        .support_mod_primes
        .iter()
        .fold(BigInt::one(), |product, support| {
            product * BigInt::from(support.modulus)
        });
    let bounds = RationalReconstructionBounds {
        numerator_abs: (&modulus_product - BigInt::one()) / 2,
        denominator_abs: BigInt::one(),
    };
    reconstruct_univariate_q_from_modular(&candidate.support_mod_primes, &bounds)
        .ok()
        .filter(|polynomial| !polynomial.is_zero())
}

fn single_prime_integer_lift(
    support: &crate::univariate::UniPolynomialFp,
) -> Option<UniPolynomialQ> {
    if support.is_zero() {
        return None;
    }
    let modulus = BigInt::from(support.modulus);
    let half_modulus = support.modulus / 2;
    let coefficients = support
        .coefficients
        .iter()
        .map(|coefficient| {
            let residue = if *coefficient > half_modulus {
                BigInt::from(*coefficient) - modulus.clone()
            } else {
                BigInt::from(*coefficient)
            };
            Rational::from_integer(residue)
        })
        .collect();
    let mut reconstructed = UniPolynomialQ {
        variable: support.variable.clone(),
        coefficients,
    };
    reconstructed.normalize();
    (!reconstructed.is_zero()).then_some(reconstructed)
}

#[cfg(test)]
mod tests {
    use num_bigint::BigInt;
    use num_rational::BigRational;

    use super::*;
    use crate::candidates::{CandidateOrigin, CandidateTrace, TargetCandidate};
    use crate::univariate::UniPolynomialFp;
    use crate::{UniPolynomialQ, Variable};

    fn variable(symbol: &str) -> Variable {
        Variable {
            symbol: symbol.to_string(),
        }
    }

    fn rational(value: i64) -> BigRational {
        BigRational::from_integer(BigInt::from(value))
    }

    fn candidate_with_coefficients(coefficients: &[i64]) -> TargetCandidate {
        let t = variable("T");
        TargetCandidate {
            support_mod_primes: Vec::new(),
            reconstructed: Some(UniPolynomialQ {
                variable: t,
                coefficients: coefficients.iter().map(|value| rational(*value)).collect(),
            }),
            origin: CandidateOrigin::DirectTargetEquation,
            traces: vec![CandidateTrace::DirectEquation { equation_index: 0 }],
        }
    }

    #[test]
    fn zero_candidate_is_discarded() {
        assert!(normalize_candidate(candidate_with_coefficients(&[0, 0])).is_none());
    }

    #[test]
    fn squarefree_support_does_not_replace_proof_candidate() {
        let candidate = normalize_candidate(candidate_with_coefficients(&[1, -2, 1])).unwrap();
        let scheduled = factor_schedule(&candidate);

        assert_eq!(scheduled.len(), 1);
        assert_eq!(
            scheduled[0].reconstructed.as_ref().unwrap().coefficients,
            vec![rational(1), rational(-2), rational(1)]
        );
        assert_eq!(
            scheduled[0]
                .reconstructed
                .as_ref()
                .unwrap()
                .squarefree_part()
                .coefficients,
            vec![rational(-1), rational(1)]
        );
    }

    #[test]
    fn ranking_orders_candidates_without_certifying_them() {
        let lower = candidate_with_coefficients(&[-1, 1]);
        let higher = candidate_with_coefficients(&[-2, 0, 1]);

        let ranked = rank_candidates(vec![higher.clone(), lower.clone()]);

        assert_eq!(ranked[0].reconstructed, lower.reconstructed);
        assert_eq!(ranked[1].reconstructed, higher.reconstructed);
    }

    #[test]
    fn modular_candidate_gets_integer_reconstruction_for_proof_search() {
        let t = variable("T");
        let candidate = TargetCandidate {
            support_mod_primes: vec![UniPolynomialFp {
                variable: t,
                modulus: 5,
                coefficients: vec![3, 0, 1],
            }],
            reconstructed: None,
            origin: CandidateOrigin::ResidualCyclic,
            traces: Vec::new(),
        };

        let normalized = normalize_candidate(candidate).unwrap();

        assert_eq!(
            normalized.reconstructed.unwrap().coefficients,
            vec![rational(-2), rational(0), rational(1)]
        );
    }

    #[test]
    fn multi_prime_modular_candidate_uses_crt_not_first_prime() {
        let t = variable("T");
        let candidate = TargetCandidate {
            support_mod_primes: vec![
                UniPolynomialFp {
                    variable: t.clone(),
                    modulus: 5,
                    coefficients: vec![2, 1],
                },
                UniPolynomialFp {
                    variable: t.clone(),
                    modulus: 11,
                    coefficients: vec![9, 1],
                },
                UniPolynomialFp {
                    variable: t,
                    modulus: 13,
                    coefficients: vec![3, 1],
                },
            ],
            reconstructed: None,
            origin: CandidateOrigin::ResidualCyclic,
            traces: Vec::new(),
        };

        let normalized = normalize_candidate(candidate).unwrap();

        assert_eq!(
            normalized.reconstructed.unwrap().coefficients,
            vec![rational(42), rational(1)]
        );
    }
}
