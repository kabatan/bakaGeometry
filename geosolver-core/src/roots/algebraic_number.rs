use serde::{Deserialize, Serialize};

use crate::result::status::SolverError;
use crate::roots::isolate::{isolate_real_roots, RealRootRecord, RootIsolationOptions};
use crate::types::hash::{hash_sequence, Hash};
use crate::types::interval::RationalInterval;
use crate::types::rational::{rational_to_bytes, RationalQ};
use crate::types::univariate::UniPolynomialQ;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlgebraicRootOrdering {
    Less,
    Equal,
    Greater,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlgebraicRootRecord {
    pub support: UniPolynomialQ,
    pub support_hash: Hash,
    pub root_index: usize,
    pub isolating_interval: RationalInterval,
    pub root_hash: Hash,
}

pub fn algebraic_roots_from_support(
    support: &UniPolynomialQ,
    options: RootIsolationOptions,
) -> Result<Vec<AlgebraicRootRecord>, SolverError> {
    let roots = isolate_real_roots(support, options)?;
    roots
        .iter()
        .map(|root| algebraic_root_from_record(support, root))
        .collect()
}

pub fn algebraic_root_from_record(
    support: &UniPolynomialQ,
    root: &RealRootRecord,
) -> Result<AlgebraicRootRecord, SolverError> {
    if root.support_hash != support.hash {
        return Err(SolverError::invalid_input(
            Some(support.variable),
            "root record support hash does not match support polynomial",
        ));
    }
    let root_hash = hash_algebraic_root(support.hash, root.root_index, &root.isolating_interval);
    Ok(AlgebraicRootRecord {
        support: support.clone(),
        support_hash: support.hash,
        root_index: root.root_index,
        isolating_interval: root.isolating_interval.clone(),
        root_hash,
    })
}

pub fn refine_algebraic_root(
    root: &AlgebraicRootRecord,
    options: RootIsolationOptions,
) -> Result<AlgebraicRootRecord, SolverError> {
    let roots = algebraic_roots_from_support(&root.support, options)?;
    roots
        .into_iter()
        .find(|candidate| candidate.root_index == root.root_index)
        .ok_or_else(|| {
            SolverError::invalid_input(
                Some(root.support.variable),
                "algebraic root index is not present in refined isolation",
            )
        })
}

pub fn refine_algebraic_root_to_width(
    root: &AlgebraicRootRecord,
    max_width: RationalQ,
) -> Result<AlgebraicRootRecord, SolverError> {
    let roots = crate::algebra::real_root::isolate_real_roots_sturm_with_max_width(
        &root.support,
        max_width,
    )?;
    roots
        .into_iter()
        .find(|candidate| candidate.root_index == root.root_index)
        .map(|candidate| algebraic_root_from_record(&root.support, &candidate))
        .transpose()?
        .ok_or_else(|| {
            SolverError::invalid_input(
                Some(root.support.variable),
                "algebraic root index is not present in refined isolation",
            )
        })
}

pub fn compare_algebraic_roots(
    a: &AlgebraicRootRecord,
    b: &AlgebraicRootRecord,
) -> Option<AlgebraicRootOrdering> {
    if a.support_hash == b.support_hash && a.root_index == b.root_index {
        return Some(AlgebraicRootOrdering::Equal);
    }
    if crate::algebra::real_root::cmp_q(&a.isolating_interval.hi, &b.isolating_interval.lo)
        != std::cmp::Ordering::Greater
    {
        return Some(AlgebraicRootOrdering::Less);
    }
    if crate::algebra::real_root::cmp_q(&b.isolating_interval.hi, &a.isolating_interval.lo)
        != std::cmp::Ordering::Greater
    {
        return Some(AlgebraicRootOrdering::Greater);
    }
    None
}

pub fn hash_algebraic_root(
    support_hash: Hash,
    root_index: usize,
    isolating_interval: &RationalInterval,
) -> Hash {
    hash_sequence(
        "algebraic-root-record",
        &[
            support_hash.0.to_vec(),
            root_index.to_be_bytes().to_vec(),
            rational_to_bytes(&isolating_interval.lo),
            rational_to_bytes(&isolating_interval.hi),
        ],
    )
}
