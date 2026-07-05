use serde::{Deserialize, Serialize};

use crate::roots::isolate::RealRootRecord;
use crate::types::hash::Hash;
use crate::types::ids::VariableId;
use crate::types::interval::RationalInterval;
use crate::types::rational::rational_to_bytes;
use crate::types::univariate::UniPolynomialQ;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetCandidate {
    pub target: VariableId,
    pub support_hash: Hash,
    pub root_index: usize,
    pub isolating_interval: RationalInterval,
    pub candidate_hash: Hash,
}

pub fn decode_candidates(
    target: VariableId,
    support: &UniPolynomialQ,
    roots: &[RealRootRecord],
) -> Vec<TargetCandidate> {
    roots
        .iter()
        .map(|root| {
            let isolating_interval = root.isolating_interval.clone();
            let candidate_hash =
                hash_target_candidate(target, support.hash, root.root_index, &isolating_interval);
            TargetCandidate {
                target,
                support_hash: support.hash,
                root_index: root.root_index,
                isolating_interval,
                candidate_hash,
            }
        })
        .collect()
}

pub fn hash_target_candidate(
    target: VariableId,
    support_hash: Hash,
    root_index: usize,
    isolating_interval: &RationalInterval,
) -> Hash {
    crate::types::hash::hash_sequence(
        "target-candidate",
        &[
            target.0.to_be_bytes().to_vec(),
            support_hash.0.to_vec(),
            root_index.to_be_bytes().to_vec(),
            rational_to_bytes(&isolating_interval.lo),
            rational_to_bytes(&isolating_interval.hi),
        ],
    )
}
