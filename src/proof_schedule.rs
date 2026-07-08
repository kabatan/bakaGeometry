use std::collections::VecDeque;

use crate::proof::CertificateMode;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ProofSearchTuple {
    pub multiplier_degree: usize,
    pub support_power: usize,
    pub guard_power: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProofModeKind {
    Ideal,
    Radical,
    GuardedRadical,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FairProofTrial {
    pub tuple: ProofSearchTuple,
    pub mode_kind: ProofModeKind,
    pub mode: CertificateMode,
}

#[derive(Clone, Debug)]
pub(crate) struct FairProofSchedule {
    next_weight: usize,
    queued: VecDeque<FairProofTrial>,
}

impl FairProofSchedule {
    pub(crate) fn unbounded() -> Self {
        Self {
            next_weight: 0,
            queued: VecDeque::new(),
        }
    }

    pub(crate) fn bounded(max_weight: usize) -> FairProofSchedulePrefix {
        FairProofSchedulePrefix {
            inner: Self::unbounded(),
            max_weight,
        }
    }

    fn fill_next_weight(&mut self) {
        self.queued
            .extend(fair_proof_trials_for_weight(self.next_weight));
        self.next_weight += 1;
    }
}

impl Iterator for FairProofSchedule {
    type Item = FairProofTrial;

    fn next(&mut self) -> Option<Self::Item> {
        if self.queued.is_empty() {
            self.fill_next_weight();
        }
        self.queued.pop_front()
    }
}

pub(crate) struct FairProofSchedulePrefix {
    inner: FairProofSchedule,
    max_weight: usize,
}

impl Iterator for FairProofSchedulePrefix {
    type Item = FairProofTrial;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let trial = self.inner.next()?;
            if proof_tuple_weight(&trial.tuple) > self.max_weight {
                return None;
            }
            return Some(trial);
        }
    }
}

pub(crate) fn bounded_fair_proof_prefix(max_weight: usize) -> Vec<FairProofTrial> {
    FairProofSchedule::bounded(max_weight).collect()
}

pub(crate) fn certificate_mode_for_trial(trial: &FairProofTrial) -> CertificateMode {
    trial.mode.clone()
}

pub(crate) fn bounded_certificate_mode_prefix(max_weight: usize) -> Vec<CertificateMode> {
    let mut modes = Vec::new();
    for trial in FairProofSchedule::bounded(max_weight) {
        let mode = certificate_mode_for_trial(&trial);
        if !modes.contains(&mode) {
            modes.push(mode);
        }
    }
    modes
}

pub(crate) fn fair_proof_trials_for_weight(weight: usize) -> Vec<FairProofTrial> {
    let mut trials = Vec::new();
    for multiplier_degree in 0..=weight {
        for support_power in 1..=weight + 1 {
            for guard_power in 0..=weight {
                if multiplier_degree + support_power + guard_power != weight + 1 {
                    continue;
                }
                let tuple = ProofSearchTuple {
                    multiplier_degree,
                    support_power,
                    guard_power,
                };
                if support_power == 1 && guard_power == 0 {
                    trials.push(FairProofTrial {
                        tuple: tuple.clone(),
                        mode_kind: ProofModeKind::Ideal,
                        mode: CertificateMode::Ideal,
                    });
                }
                if guard_power == 0 {
                    trials.push(FairProofTrial {
                        tuple: tuple.clone(),
                        mode_kind: ProofModeKind::Radical,
                        mode: CertificateMode::Radical { support_power },
                    });
                }
                trials.push(FairProofTrial {
                    tuple,
                    mode_kind: ProofModeKind::GuardedRadical,
                    mode: CertificateMode::GuardedRadical {
                        support_power,
                        guard_power,
                    },
                });
            }
        }
    }
    trials
}

pub(crate) fn proof_tuple_weight(tuple: &ProofSearchTuple) -> usize {
    tuple.multiplier_degree + tuple.support_power + tuple.guard_power - 1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tuple(
        multiplier_degree: usize,
        support_power: usize,
        guard_power: usize,
    ) -> ProofSearchTuple {
        ProofSearchTuple {
            multiplier_degree,
            support_power,
            guard_power,
        }
    }

    #[test]
    fn fair_schedule_reaches_every_tuple_up_to_weight_4() {
        let trials = bounded_fair_proof_prefix(4);
        for weight in 0..=4 {
            for multiplier_degree in 0..=weight {
                for support_power in 1..=weight + 1 {
                    for guard_power in 0..=weight {
                        if multiplier_degree + support_power + guard_power == weight + 1 {
                            assert!(
                                trials.iter().any(|trial| {
                                    trial.tuple
                                        == tuple(multiplier_degree, support_power, guard_power)
                                }),
                                "missing tuple ({multiplier_degree}, {support_power}, {guard_power})"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn fair_schedule_is_lazy_and_not_a_fixed_default_prefix() {
        let late_tuple = FairProofSchedule::unbounded()
            .take(500)
            .find(|trial| trial.tuple == tuple(5, 1, 2));

        assert!(late_tuple.is_some());
    }

    #[test]
    fn bounded_prefix_is_explicitly_bounded_by_argument() {
        let low = bounded_fair_proof_prefix(1);
        let high = bounded_fair_proof_prefix(4);

        assert!(low
            .iter()
            .all(|trial| proof_tuple_weight(&trial.tuple) <= 1));
        assert!(high.len() > low.len());
        assert!(high.iter().any(|trial| trial.tuple == tuple(3, 1, 1)));
    }
}
