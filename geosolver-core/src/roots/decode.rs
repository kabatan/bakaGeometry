use serde::{Deserialize, Serialize};

use crate::types::hash::Hash;
use crate::types::ids::VariableId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetCandidate {
    pub target: VariableId,
    pub support_hash: Hash,
    pub root_index: usize,
}
