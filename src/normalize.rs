use std::collections::BTreeMap;

use num_bigint::BigInt;
use num_traits::{One, Signed, Zero};

use crate::candidates::{CandidateTrace, ModularWitnessTrace, SliceWitnessTrace, TargetCandidate};
use crate::finite_field::PrimeModulus;
use crate::rational_reconstruction::{
    reconstruct_univariate_q_from_modular, RationalReconstructionBounds,
};
use crate::univariate::UniPolynomialFp;
use crate::UniPolynomialQ;

#[cfg(test)]
pub(crate) fn normalize_candidate(candidate: TargetCandidate) -> Option<TargetCandidate> {
    finish_normalized_candidate(prepare_candidate(candidate)?)
}

fn prepare_candidate(mut candidate: TargetCandidate) -> Option<TargetCandidate> {
    candidate.support_mod_primes = candidate
        .support_mod_primes
        .into_iter()
        .filter_map(normalize_modular_support)
        .collect();
    if let Some(reconstructed) = candidate.reconstructed.take() {
        let normalized = reconstructed.primitive_integer_normalized();
        if normalized.is_zero() {
            return None;
        }
        candidate.reconstructed = Some(normalized);
    }
    if candidate.reconstructed.is_none() && candidate.support_mod_primes.is_empty() {
        return None;
    }
    Some(candidate)
}

fn finish_normalized_candidate(mut candidate: TargetCandidate) -> Option<TargetCandidate> {
    if candidate.reconstructed.is_none() {
        let Some(reconstructed) = reconstruct_from_modular_support(&candidate) else {
            return Some(candidate);
        };
        candidate.reconstructed = Some(reconstructed.primitive_integer_normalized());
    }
    Some(candidate)
}

pub(crate) fn normalize_candidates(candidates: Vec<TargetCandidate>) -> Vec<TargetCandidate> {
    let prepared = candidates
        .into_iter()
        .filter_map(prepare_candidate)
        .collect::<Vec<_>>();
    merge_modular_candidates(prepared)
        .into_iter()
        .filter_map(finish_normalized_candidate)
        .collect()
}

pub(crate) fn rank_candidates(mut candidates: Vec<TargetCandidate>) -> Vec<TargetCandidate> {
    candidates.sort_by(|left, right| candidate_rank(left).cmp(&candidate_rank(right)));
    candidates
}

pub(crate) fn factor_schedule(candidate: &TargetCandidate) -> Vec<TargetCandidate> {
    let Some(reconstructed) = &candidate.reconstructed else {
        let mut scheduled = Vec::new();
        scheduled.push(candidate.clone());
        return scheduled;
    };
    let original = reconstructed.primitive_integer_normalized();
    let mut scheduled = Vec::new();
    for factor in original.factor_squarefree_over_q() {
        let factor = factor.primitive_integer_normalized();
        if factor.is_zero()
            || factor == original
            || scheduled.iter().any(|candidate: &TargetCandidate| {
                candidate.reconstructed.as_ref() == Some(&factor)
            })
        {
            continue;
        }
        let mut factor_candidate = candidate.clone();
        factor_candidate.support_mod_primes.clear();
        factor_candidate.reconstructed = Some(factor);
        scheduled.push(factor_candidate);
    }

    let mut original_candidate = candidate.clone();
    original_candidate.reconstructed = Some(original);
    scheduled.push(original_candidate);
    scheduled
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ModularMergeKey {
    origin: crate::candidates::CandidateOrigin,
    variable: crate::Variable,
    degree: usize,
    family: ModularFamilyKey,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum ModularFamilyKey {
    ActiveMultiplierSupports(Vec<Vec<crate::Monomial>>),
    SliceSpecialization {
        equation_index: usize,
        assignments: Vec<(usize, u64)>,
    },
}

fn merge_modular_candidates(candidates: Vec<TargetCandidate>) -> Vec<TargetCandidate> {
    let mut passthrough = Vec::new();
    let mut groups: BTreeMap<ModularMergeKey, Vec<TargetCandidate>> = BTreeMap::new();

    for candidate in candidates {
        if candidate.reconstructed.is_some() || candidate.support_mod_primes.len() != 1 {
            passthrough.push(candidate);
            continue;
        }
        let support = &candidate.support_mod_primes[0];
        let Some(degree) = modular_degree(support) else {
            continue;
        };
        let Some(family) = modular_family_key(&candidate) else {
            passthrough.push(candidate);
            continue;
        };
        groups
            .entry(ModularMergeKey {
                origin: candidate.origin,
                variable: support.variable.clone(),
                degree,
                family,
            })
            .or_default()
            .push(candidate);
    }

    for (_, mut group) in groups {
        if group.len() == 1 {
            passthrough.push(group.pop().unwrap());
            continue;
        }
        group.sort_by_key(|candidate| candidate.support_mod_primes[0].modulus);
        let merged = merge_distinct_prime_alternatives(&group);
        if has_duplicate_modulus(&group) {
            passthrough.extend(group);
        }
        passthrough.extend(merged);
    }

    passthrough
}

fn merge_distinct_prime_alternatives(group: &[TargetCandidate]) -> Vec<TargetCandidate> {
    let mut buckets = BTreeMap::<u64, Vec<TargetCandidate>>::new();
    for candidate in group {
        buckets
            .entry(candidate.support_mod_primes[0].modulus)
            .or_default()
            .push(candidate.clone());
    }
    if buckets.len() <= 1 {
        return Vec::new();
    }

    let bucket_values = buckets.into_values().collect::<Vec<_>>();
    let mut combinations = Vec::new();
    build_distinct_prime_combinations(&bucket_values, 0, &mut Vec::new(), &mut combinations);
    combinations
        .into_iter()
        .filter_map(merge_distinct_prime_combination)
        .collect()
}

fn build_distinct_prime_combinations(
    buckets: &[Vec<TargetCandidate>],
    index: usize,
    current: &mut Vec<TargetCandidate>,
    combinations: &mut Vec<Vec<TargetCandidate>>,
) {
    if index == buckets.len() {
        combinations.push(current.clone());
        return;
    }
    for candidate in &buckets[index] {
        current.push(candidate.clone());
        build_distinct_prime_combinations(buckets, index + 1, current, combinations);
        current.pop();
    }
}

fn merge_distinct_prime_combination(
    mut combination: Vec<TargetCandidate>,
) -> Option<TargetCandidate> {
    let mut merged = combination.first()?.clone();
    for candidate in combination.drain(1..) {
        if merged
            .support_mod_primes
            .iter()
            .any(|support| support.modulus == candidate.support_mod_primes[0].modulus)
        {
            return None;
        }
        merged
            .support_mod_primes
            .push(candidate.support_mod_primes[0].clone());
        merged.traces.extend(candidate.traces);
    }
    Some(merged)
}

fn modular_family_key(candidate: &TargetCandidate) -> Option<ModularFamilyKey> {
    if let Some(witness) = single_modular_witness(candidate) {
        return Some(ModularFamilyKey::ActiveMultiplierSupports(
            witness.active_multiplier_supports.clone(),
        ));
    }
    let witness = single_slice_witness(candidate)?;
    Some(ModularFamilyKey::SliceSpecialization {
        equation_index: witness.equation_index,
        assignments: witness
            .assignments
            .iter()
            .map(|assignment| (assignment.variable_index, assignment.value))
            .collect(),
    })
}

fn single_modular_witness(candidate: &TargetCandidate) -> Option<&ModularWitnessTrace> {
    let mut witnesses = candidate.traces.iter().filter_map(|trace| match trace {
        CandidateTrace::ModularWitness(witness) => Some(witness),
        _ => None,
    });
    let witness = witnesses.next()?;
    witnesses.next().is_none().then_some(witness)
}

fn single_slice_witness(candidate: &TargetCandidate) -> Option<&SliceWitnessTrace> {
    let mut witnesses = candidate.traces.iter().filter_map(|trace| match trace {
        CandidateTrace::SliceWitness(witness) => Some(witness),
        _ => None,
    });
    let witness = witnesses.next()?;
    witnesses.next().is_none().then_some(witness)
}

fn has_duplicate_modulus(group: &[TargetCandidate]) -> bool {
    group
        .windows(2)
        .any(|pair| pair[0].support_mod_primes[0].modulus == pair[1].support_mod_primes[0].modulus)
}

fn normalize_modular_support(mut support: UniPolynomialFp) -> Option<UniPolynomialFp> {
    support.normalize();
    if support.is_zero() {
        return None;
    }
    let modulus = PrimeModulus::new(support.modulus)?;
    for coefficient in &mut support.coefficients {
        *coefficient = modulus.normalize(*coefficient);
    }
    support.normalize();
    let degree = modular_degree(&support)?;
    let inverse = modulus.inv(support.coefficients[degree])?;
    for coefficient in &mut support.coefficients {
        *coefficient = modulus.mul(*coefficient, inverse);
    }
    support.normalize();
    Some(support)
}

fn modular_degree(support: &UniPolynomialFp) -> Option<usize> {
    support
        .coefficients
        .iter()
        .rposition(|coefficient| *coefficient != 0)
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
        return None;
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

#[cfg(test)]
mod tests {
    use num_bigint::BigInt;
    use num_rational::BigRational;

    use super::*;
    use crate::candidates::{
        CandidateOrigin, CandidateTrace, ModularWitnessTrace, TargetCandidate,
    };
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
    fn factor_schedule_trials_factors_without_replacing_original_candidate() {
        let candidate = normalize_candidate(candidate_with_coefficients(&[1, -2, 1])).unwrap();
        let scheduled = factor_schedule(&candidate);

        let supports = scheduled
            .iter()
            .map(|candidate| {
                candidate
                    .reconstructed
                    .as_ref()
                    .unwrap()
                    .coefficients
                    .clone()
            })
            .collect::<Vec<_>>();
        assert!(supports.contains(&vec![rational(-1), rational(1)]));
        assert!(supports.contains(&vec![rational(1), rational(-2), rational(1)]));
    }

    #[test]
    fn ranking_orders_candidates_without_certifying_them() {
        let lower = candidate_with_coefficients(&[-1, 1]);
        let higher = candidate_with_coefficients(&[-2, 0, 1]);

        let ranked = rank_candidates(vec![higher.clone(), lower.clone()]);

        assert_eq!(ranked[0].reconstructed, lower.reconstructed);
        assert_eq!(ranked[1].reconstructed, higher.reconstructed);
    }

    fn modular_trace(
        prime: u64,
        active_multiplier_supports: Vec<Vec<crate::Monomial>>,
    ) -> CandidateTrace {
        CandidateTrace::ModularWitness(ModularWitnessTrace {
            prime,
            active_multiplier_supports,
            relation_coefficients: Vec::new(),
            residual_vectors: Vec::new(),
        })
    }

    #[test]
    fn single_prime_modular_candidate_remains_modular_only() {
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

        assert!(normalized.reconstructed.is_none());
        assert_eq!(normalized.support_mod_primes[0].coefficients, vec![3, 0, 1]);
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

    #[test]
    fn separate_modular_candidates_merge_before_crt_reconstruction() {
        let t = variable("T");
        let active_support = vec![vec![crate::Monomial { exponents: vec![0] }]];
        let candidates = vec![
            TargetCandidate {
                support_mod_primes: vec![UniPolynomialFp {
                    variable: t.clone(),
                    modulus: 5,
                    coefficients: vec![2, 1],
                }],
                reconstructed: None,
                origin: CandidateOrigin::ResidualCyclic,
                traces: vec![modular_trace(5, active_support.clone())],
            },
            TargetCandidate {
                support_mod_primes: vec![UniPolynomialFp {
                    variable: t.clone(),
                    modulus: 11,
                    coefficients: vec![9, 1],
                }],
                reconstructed: None,
                origin: CandidateOrigin::ResidualCyclic,
                traces: vec![modular_trace(11, active_support.clone())],
            },
            TargetCandidate {
                support_mod_primes: vec![UniPolynomialFp {
                    variable: t,
                    modulus: 13,
                    coefficients: vec![3, 1],
                }],
                reconstructed: None,
                origin: CandidateOrigin::ResidualCyclic,
                traces: vec![modular_trace(13, active_support)],
            },
        ];

        let normalized = normalize_candidates(candidates);

        assert_eq!(normalized.len(), 1);
        assert_eq!(
            normalized[0].reconstructed.as_ref().unwrap().coefficients,
            vec![rational(42), rational(1)]
        );
        assert_eq!(normalized[0].support_mod_primes.len(), 3);
    }

    #[test]
    fn unrelated_same_degree_modular_candidates_are_not_merged_or_dropped() {
        let t = variable("T");
        let candidates = vec![
            TargetCandidate {
                support_mod_primes: vec![UniPolynomialFp {
                    variable: t.clone(),
                    modulus: 5,
                    coefficients: vec![1, 1],
                }],
                reconstructed: None,
                origin: CandidateOrigin::ResidualCyclic,
                traces: Vec::new(),
            },
            TargetCandidate {
                support_mod_primes: vec![UniPolynomialFp {
                    variable: t,
                    modulus: 7,
                    coefficients: vec![2, 1],
                }],
                reconstructed: None,
                origin: CandidateOrigin::ResidualCyclic,
                traces: Vec::new(),
            },
        ];

        let normalized = normalize_candidates(candidates);

        assert_eq!(normalized.len(), 2);
        assert!(normalized.iter().all(|candidate| {
            candidate.reconstructed.is_none() && candidate.support_mod_primes.len() == 1
        }));
    }

    #[test]
    fn duplicate_prime_modular_alternatives_are_preserved() {
        let t = variable("T");
        let active_support = vec![vec![crate::Monomial { exponents: vec![0] }]];
        let candidates = vec![
            TargetCandidate {
                support_mod_primes: vec![UniPolynomialFp {
                    variable: t.clone(),
                    modulus: 5,
                    coefficients: vec![1, 1],
                }],
                reconstructed: None,
                origin: CandidateOrigin::ResidualCyclic,
                traces: vec![modular_trace(5, active_support.clone())],
            },
            TargetCandidate {
                support_mod_primes: vec![UniPolynomialFp {
                    variable: t,
                    modulus: 5,
                    coefficients: vec![2, 1],
                }],
                reconstructed: None,
                origin: CandidateOrigin::ResidualCyclic,
                traces: vec![modular_trace(5, active_support)],
            },
        ];

        let normalized = normalize_candidates(candidates);

        assert_eq!(normalized.len(), 2);
        assert!(normalized.iter().all(|candidate| {
            candidate.reconstructed.is_none() && candidate.support_mod_primes.len() == 1
        }));
    }

    #[test]
    fn duplicate_prime_alternatives_still_form_distinct_prime_reconstructions() {
        let t = variable("T");
        let active_support = vec![vec![crate::Monomial { exponents: vec![0] }]];
        let candidates = vec![
            TargetCandidate {
                support_mod_primes: vec![UniPolynomialFp {
                    variable: t.clone(),
                    modulus: 5,
                    coefficients: vec![2, 1],
                }],
                reconstructed: None,
                origin: CandidateOrigin::ResidualCyclic,
                traces: vec![modular_trace(5, active_support.clone())],
            },
            TargetCandidate {
                support_mod_primes: vec![UniPolynomialFp {
                    variable: t.clone(),
                    modulus: 5,
                    coefficients: vec![3, 1],
                }],
                reconstructed: None,
                origin: CandidateOrigin::ResidualCyclic,
                traces: vec![modular_trace(5, active_support.clone())],
            },
            TargetCandidate {
                support_mod_primes: vec![UniPolynomialFp {
                    variable: t,
                    modulus: 7,
                    coefficients: vec![5, 1],
                }],
                reconstructed: None,
                origin: CandidateOrigin::ResidualCyclic,
                traces: vec![modular_trace(7, active_support)],
            },
        ];

        let normalized = normalize_candidates(candidates);

        assert_eq!(
            normalized
                .iter()
                .filter(|candidate| candidate.support_mod_primes.len() == 1)
                .count(),
            3
        );
        assert_eq!(
            normalized
                .iter()
                .filter(|candidate| {
                    candidate.reconstructed.is_some() && candidate.support_mod_primes.len() == 2
                })
                .count(),
            2
        );
    }

    #[test]
    fn modular_support_is_normalized_to_monic_representative() {
        let t = variable("T");
        let candidate = TargetCandidate {
            support_mod_primes: vec![UniPolynomialFp {
                variable: t,
                modulus: 7,
                coefficients: vec![4, 2],
            }],
            reconstructed: None,
            origin: CandidateOrigin::ResidualCyclic,
            traces: Vec::new(),
        };

        let normalized = normalize_candidate(candidate).unwrap();

        assert_eq!(normalized.support_mod_primes[0].coefficients, vec![2, 1]);
        assert!(normalized.reconstructed.is_none());
    }
}
