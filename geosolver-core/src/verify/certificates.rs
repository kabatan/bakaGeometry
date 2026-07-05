use serde::{Deserialize, Serialize};

use crate::types::hash::Hash;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KernelCertificate {
    pub certificate_hash: Hash,
}
