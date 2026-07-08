use std::collections::VecDeque;

use crate::options::ResourceLimits;
use crate::proof_schedule::{fair_proof_trials_for_weight, proof_tuple_weight, FairProofTrial};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SolveWorkItem {
    CandidateWindow {
        window_degree: usize,
        proof_trial: FairProofTrial,
    },
    CompleteFallbackBudget {
        elimination_degree: usize,
    },
}

pub(crate) struct GlobalSolveSchedule {
    max_window_degree: Option<usize>,
    max_proof_weight: Option<usize>,
    next_weight: usize,
    queued: VecDeque<SolveWorkItem>,
    exhausted: bool,
}

impl GlobalSolveSchedule {
    pub(crate) fn from_limits(limits: &ResourceLimits) -> Self {
        Self {
            max_window_degree: limits.max_window_degree,
            max_proof_weight: limits.max_proof_weight,
            next_weight: 0,
            queued: VecDeque::new(),
            exhausted: false,
        }
    }

    fn fill_next_weight(&mut self) {
        if self.exhausted {
            return;
        }
        let weight = self.next_weight;
        let mut produced = false;

        for window_degree in 0..=weight {
            if self
                .max_window_degree
                .is_some_and(|max_degree| window_degree > max_degree)
            {
                continue;
            }
            let proof_weight = weight - window_degree;
            if self
                .max_proof_weight
                .is_some_and(|max_weight| proof_weight > max_weight)
            {
                continue;
            }
            for proof_trial in fair_proof_trials_for_weight(proof_weight) {
                debug_assert_eq!(proof_tuple_weight(&proof_trial.tuple), proof_weight);
                produced = true;
                self.queued.push_back(SolveWorkItem::CandidateWindow {
                    window_degree,
                    proof_trial,
                });
            }
        }

        if !self
            .max_window_degree
            .is_some_and(|max_degree| weight > max_degree)
        {
            produced = true;
            self.queued
                .push_back(SolveWorkItem::CompleteFallbackBudget {
                    elimination_degree: weight,
                });
        }

        self.next_weight += 1;
        if !produced && self.max_window_degree.is_some() && self.max_proof_weight.is_some() {
            self.exhausted = true;
        }
    }
}

impl Iterator for GlobalSolveSchedule {
    type Item = SolveWorkItem;

    fn next(&mut self) -> Option<Self::Item> {
        while self.queued.is_empty() && !self.exhausted {
            self.fill_next_weight();
        }
        self.queued.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proof_schedule::ProofSearchTuple;

    #[test]
    fn unbounded_global_schedule_reaches_arbitrary_tuple() {
        let limits = ResourceLimits {
            max_window_degree: None,
            max_proof_weight: None,
            max_matrix_rows: None,
            max_matrix_cols: None,
            max_candidate_count: None,
        };

        let found = GlobalSolveSchedule::from_limits(&limits)
            .take(10_000)
            .any(|item| {
                matches!(
                    item,
                    SolveWorkItem::CandidateWindow {
                        window_degree: 5,
                        proof_trial
                    } if proof_trial.tuple == ProofSearchTuple {
                        multiplier_degree: 3,
                        support_power: 4,
                        guard_power: 2,
                    }
                )
            });

        assert!(found);
    }

    #[test]
    fn bounded_global_schedule_respects_limits() {
        let limits = ResourceLimits {
            max_window_degree: Some(1),
            max_proof_weight: Some(2),
            max_matrix_rows: None,
            max_matrix_cols: None,
            max_candidate_count: None,
        };

        let items = GlobalSolveSchedule::from_limits(&limits).collect::<Vec<_>>();

        assert!(items.iter().any(|item| matches!(
            item,
            SolveWorkItem::CompleteFallbackBudget {
                elimination_degree: 1
            }
        )));
        assert!(items.iter().all(|item| match item {
            SolveWorkItem::CandidateWindow {
                window_degree,
                proof_trial,
            } => *window_degree <= 1 && proof_tuple_weight(&proof_trial.tuple) <= 2,
            SolveWorkItem::CompleteFallbackBudget { elimination_degree } =>
                *elimination_degree <= 1,
        }));
    }
}
