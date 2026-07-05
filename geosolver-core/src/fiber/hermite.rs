use serde::{Deserialize, Serialize};

use crate::result::status::{FailureKind, SolverError, SolverErrorKind};
use crate::roots::decode::TargetCandidate;
use crate::roots::isolate::RealRootRecord;
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::VariableId;
use crate::types::interval::interval_disjoint;
use crate::types::univariate::{degree_uni, UniPolynomialQ};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HermiteFiberCountMethod {
    TargetAlgebraicCondition,
    TargetOnlyRealWitness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RealFiberCountFactor {
    pub factor_hash: Hash,
    pub factor_kind: String,
    pub real_root_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HermiteFiberInput {
    pub target: VariableId,
    pub support: UniPolynomialQ,
    pub root: RealRootRecord,
    pub candidate: TargetCandidate,
    pub equality_relation_hashes: Vec<Hash>,
    pub semantic_hashes: Vec<Hash>,
    pub real_root_factors: Vec<RealFiberCountFactor>,
    pub method: HermiteFiberCountMethod,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RealFiberCountCertificate {
    pub target: VariableId,
    pub support_hash: Hash,
    pub root_index: usize,
    pub candidate_hash: Hash,
    pub equality_relation_hashes: Vec<Hash>,
    pub semantic_hashes: Vec<Hash>,
    pub real_root_count: usize,
    pub target_interval_root_count: usize,
    pub counted_interval_root_indices: Vec<usize>,
    pub real_root_factors: Vec<RealFiberCountFactor>,
    pub method: HermiteFiberCountMethod,
    pub certificate_hash: Hash,
}

pub fn hermite_real_root_count_for_fiber(
    input: HermiteFiberInput,
) -> Result<RealFiberCountCertificate, SolverError> {
    if input.support.variable != input.target
        || input.root.support_hash != input.support.hash
        || input.candidate.target != input.target
        || input.candidate.support_hash != input.support.hash
        || input.candidate.root_index != input.root.root_index
        || input.candidate.isolating_interval != input.root.isolating_interval
    {
        return Err(certificate_gap(
            input.target,
            input.support.hash,
            "HermiteFiberInput target/root/candidate binding",
        ));
    }

    let counted_interval_root_indices =
        count_real_support_roots_in_interval(&input.support, &input.root)?;
    let target_interval_root_count = counted_interval_root_indices.len();
    let semantic_factor_count = input
        .real_root_factors
        .iter()
        .map(|factor| factor.real_root_count)
        .product::<usize>()
        .max(if input.real_root_factors.is_empty() {
            1
        } else {
            0
        });
    let mut cert = RealFiberCountCertificate {
        target: input.target,
        support_hash: input.support.hash,
        root_index: input.root.root_index,
        candidate_hash: input.candidate.candidate_hash,
        equality_relation_hashes: input.equality_relation_hashes,
        semantic_hashes: input.semantic_hashes,
        real_root_count: target_interval_root_count * semantic_factor_count,
        target_interval_root_count,
        counted_interval_root_indices,
        real_root_factors: input.real_root_factors,
        method: input.method,
        certificate_hash: hash_sequence("real-fiber-count-certificate", &[]),
    };
    cert.certificate_hash = hash_real_fiber_count_certificate(&cert);
    Ok(cert)
}

pub fn hash_real_fiber_count_certificate(cert: &RealFiberCountCertificate) -> Hash {
    let mut chunks = vec![
        cert.target.0.to_be_bytes().to_vec(),
        cert.support_hash.0.to_vec(),
        cert.root_index.to_be_bytes().to_vec(),
        cert.candidate_hash.0.to_vec(),
        cert.target_interval_root_count.to_be_bytes().to_vec(),
        cert.real_root_count.to_be_bytes().to_vec(),
        format!("{:?}", cert.method).into_bytes(),
    ];
    chunks.extend(
        cert.counted_interval_root_indices
            .iter()
            .map(|index| index.to_be_bytes().to_vec()),
    );
    for factor in &cert.real_root_factors {
        chunks.push(factor.factor_hash.0.to_vec());
        chunks.push(factor.factor_kind.as_bytes().to_vec());
        chunks.push(factor.real_root_count.to_be_bytes().to_vec());
    }
    chunks.extend(
        cert.equality_relation_hashes
            .iter()
            .map(|hash| hash.0.to_vec()),
    );
    chunks.extend(cert.semantic_hashes.iter().map(|hash| hash.0.to_vec()));
    hash_sequence("real-fiber-count-certificate", &chunks)
}

fn count_real_support_roots_in_interval(
    support: &UniPolynomialQ,
    root: &RealRootRecord,
) -> Result<Vec<usize>, SolverError> {
    if degree_uni(support).is_none() {
        return Ok(Vec::new());
    }
    let roots = crate::algebra::real_root::isolate_real_roots_sturm(support)?;
    Ok(roots
        .iter()
        .filter(|support_root| {
            support_root.root_index == root.root_index
                && !interval_disjoint(&support_root.isolating_interval, &root.isolating_interval)
        })
        .map(|support_root| support_root.root_index)
        .collect())
}

fn certificate_gap(target: VariableId, object_hash: Hash, missing: &str) -> SolverError {
    SolverError {
        target: Some(target),
        kind: SolverErrorKind::Failure(FailureKind::CertificateDesignGap {
            constructed_object_hash: object_hash,
            missing_certificate_kind: missing.to_owned(),
        }),
    }
}
