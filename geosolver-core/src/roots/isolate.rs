use serde::{Deserialize, Serialize};

use crate::result::status::SolverError;
use crate::roots::squarefree::squarefree_support;
use crate::solver::options::RootIsolationMethod;
use crate::types::hash::Hash;
use crate::types::interval::RationalInterval;
use crate::types::univariate::UniPolynomialQ;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootIsolationOptions {
    pub method: RootIsolationMethod,
}

impl Default for RootIsolationOptions {
    fn default() -> Self {
        Self {
            method: RootIsolationMethod::Sturm,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RealRootRecord {
    pub support_hash: Hash,
    pub root_index: usize,
    pub isolating_interval: RationalInterval,
}

pub fn isolate_real_roots(
    p: &UniPolynomialQ,
    options: RootIsolationOptions,
) -> Result<Vec<RealRootRecord>, SolverError> {
    let squarefree = squarefree_support(p)?;
    let mut roots = match options.method {
        RootIsolationMethod::Sturm => {
            crate::algebra::real_root::isolate_real_roots_sturm(&squarefree)
        }
        RootIsolationMethod::Descartes => {
            crate::algebra::real_root::isolate_real_roots_descartes(&squarefree)
        }
    }?;
    for (root_index, root) in roots.iter_mut().enumerate() {
        root.support_hash = squarefree.hash;
        root.root_index = root_index;
    }
    Ok(roots)
}
