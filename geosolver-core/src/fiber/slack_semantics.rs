use serde::{Deserialize, Serialize};

use crate::problem::semantic::RealConstraintEncoding;
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::{RelationId, VariableId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FiberProblem {
    pub target: VariableId,
    pub support_hash: Hash,
    pub candidate_hash: Hash,
    pub root_index: usize,
    pub algebraic_target_condition_hash: Hash,
    pub relation_hashes: Vec<Hash>,
    pub guard_hashes: Vec<Hash>,
    pub saturation_hashes: Vec<Hash>,
    pub fiber_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppliedSemanticEncoding {
    pub original_kind: String,
    pub encoded_relation_ids: Vec<RelationId>,
    pub slack_variables: Vec<VariableId>,
    pub semantic_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FiberProblemWithSemantics {
    pub fiber: FiberProblem,
    pub applied_semantics: Vec<AppliedSemanticEncoding>,
    pub semantic_hashes: Vec<Hash>,
    pub semantics_hash: Hash,
}

pub fn make_fiber_problem(
    target: VariableId,
    support_hash: Hash,
    candidate_hash: Hash,
    root_index: usize,
    relation_hashes: Vec<Hash>,
    guard_hashes: Vec<Hash>,
    saturation_hashes: Vec<Hash>,
) -> FiberProblem {
    let algebraic_target_condition_hash = hash_sequence(
        "fiber-target-condition",
        &[
            target.0.to_be_bytes().to_vec(),
            support_hash.0.to_vec(),
            root_index.to_be_bytes().to_vec(),
            candidate_hash.0.to_vec(),
        ],
    );
    let mut fiber = FiberProblem {
        target,
        support_hash,
        candidate_hash,
        root_index,
        algebraic_target_condition_hash,
        relation_hashes,
        guard_hashes,
        saturation_hashes,
        fiber_hash: hash_sequence("fiber-problem", &[]),
    };
    fiber.fiber_hash = hash_fiber_problem(&fiber);
    fiber
}

pub fn apply_real_constraint_semantics(
    fiber: FiberProblem,
    semantics: &[RealConstraintEncoding],
) -> FiberProblemWithSemantics {
    let applied_semantics = semantics
        .iter()
        .map(|encoding| AppliedSemanticEncoding {
            original_kind: format!("{:?}", encoding.original_kind),
            encoded_relation_ids: encoding.encoded_relation_ids.clone(),
            slack_variables: encoding.slack_variables.clone(),
            semantic_hash: encoding.semantic_hash,
        })
        .collect::<Vec<_>>();
    let semantic_hashes = applied_semantics
        .iter()
        .map(|encoding| encoding.semantic_hash)
        .collect::<Vec<_>>();
    let mut with_semantics = FiberProblemWithSemantics {
        fiber,
        applied_semantics,
        semantic_hashes,
        semantics_hash: hash_sequence("fiber-semantics", &[]),
    };
    with_semantics.semantics_hash = hash_fiber_problem_with_semantics(&with_semantics);
    with_semantics
}

pub fn verify_slack_encoding_consistency(
    record: &crate::fiber::exact_image::FiberClassificationRecord,
) -> bool {
    let expected_semantic_hashes = record
        .fiber_with_semantics
        .applied_semantics
        .iter()
        .map(|encoding| encoding.semantic_hash)
        .collect::<Vec<_>>();
    record.fiber_with_semantics.semantic_hashes == expected_semantic_hashes
        && hash_fiber_problem(&record.fiber_with_semantics.fiber)
            == record.fiber_with_semantics.fiber.fiber_hash
        && hash_fiber_problem_with_semantics(&record.fiber_with_semantics)
            == record.fiber_with_semantics.semantics_hash
        && crate::fiber::exact_image::hash_fiber_classification_record(record) == record.record_hash
}

pub fn hash_fiber_problem(fiber: &FiberProblem) -> Hash {
    let mut chunks = vec![
        fiber.target.0.to_be_bytes().to_vec(),
        fiber.support_hash.0.to_vec(),
        fiber.candidate_hash.0.to_vec(),
        fiber.root_index.to_be_bytes().to_vec(),
        fiber.algebraic_target_condition_hash.0.to_vec(),
    ];
    chunks.extend(fiber.relation_hashes.iter().map(|hash| hash.0.to_vec()));
    chunks.extend(fiber.guard_hashes.iter().map(|hash| hash.0.to_vec()));
    chunks.extend(fiber.saturation_hashes.iter().map(|hash| hash.0.to_vec()));
    hash_sequence("fiber-problem", &chunks)
}

pub fn hash_fiber_problem_with_semantics(with_semantics: &FiberProblemWithSemantics) -> Hash {
    let mut chunks = vec![with_semantics.fiber.fiber_hash.0.to_vec()];
    for semantic in &with_semantics.applied_semantics {
        chunks.push(semantic.original_kind.as_bytes().to_vec());
        chunks.push(
            semantic
                .encoded_relation_ids
                .iter()
                .flat_map(|id| id.0.to_be_bytes())
                .collect(),
        );
        chunks.push(
            semantic
                .slack_variables
                .iter()
                .flat_map(|id| id.0.to_be_bytes())
                .collect(),
        );
        chunks.push(semantic.semantic_hash.0.to_vec());
    }
    hash_sequence("fiber-semantics", &chunks)
}
