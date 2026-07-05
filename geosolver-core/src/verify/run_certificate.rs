use serde::{Deserialize, Serialize};

use crate::types::hash::Hash;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreRunCertificate {
    pub run_hash: Hash,
}
