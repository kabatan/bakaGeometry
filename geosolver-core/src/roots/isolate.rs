use serde::{Deserialize, Serialize};

use crate::types::hash::Hash;
use crate::types::interval::RationalInterval;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RealRootRecord {
    pub support_hash: Hash,
    pub root_index: usize,
    pub isolating_interval: RationalInterval,
}
