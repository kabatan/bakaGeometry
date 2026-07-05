use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::{RelationId, VariableId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RealConstraintKind {
    NonNegative,
    Positive,
    NonZero,
    BranchChoice,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RealConstraintEncoding {
    pub original_kind: RealConstraintKind,
    pub encoded_relation_ids: Vec<RelationId>,
    pub slack_variables: Vec<VariableId>,
    pub semantic_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid semantic reference")]
pub struct InvalidInput;

pub fn register_slack_encoding(
    kind: RealConstraintKind,
    encoded_relation_ids: Vec<RelationId>,
    slack_variables: Vec<VariableId>,
) -> RealConstraintEncoding {
    let mut chunks = Vec::new();
    chunks.push(format!("{kind:?}").into_bytes());
    chunks.push(
        encoded_relation_ids
            .iter()
            .flat_map(|id| id.0.to_be_bytes())
            .collect(),
    );
    chunks.push(
        slack_variables
            .iter()
            .flat_map(|id| id.0.to_be_bytes())
            .collect(),
    );
    RealConstraintEncoding {
        original_kind: kind,
        encoded_relation_ids,
        slack_variables,
        semantic_hash: hash_sequence("real-constraint", &chunks),
    }
}

pub fn semantic_relations(sem: &[RealConstraintEncoding]) -> BTreeSet<RelationId> {
    sem.iter()
        .flat_map(|encoding| encoding.encoded_relation_ids.iter().copied())
        .collect()
}

pub fn verify_semantic_references(
    sem: &[RealConstraintEncoding],
    relations: &[RelationId],
    variables: &[VariableId],
) -> Result<(), InvalidInput> {
    let available: BTreeSet<_> = relations.iter().copied().collect();
    let available_variables: BTreeSet<_> = variables.iter().copied().collect();
    for relation in semantic_relations(sem) {
        if !available.contains(&relation) {
            return Err(InvalidInput);
        }
    }
    for encoding in sem {
        for slack in &encoding.slack_variables {
            if !available_variables.contains(slack) {
                return Err(InvalidInput);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_relations_are_collected_without_role_dispatch() {
        let enc = register_slack_encoding(
            RealConstraintKind::Positive,
            vec![RelationId(2), RelationId(1)],
            vec![VariableId(9)],
        );
        assert_eq!(
            semantic_relations(&[enc]),
            [RelationId(1), RelationId(2)].into_iter().collect()
        );
    }

    #[test]
    fn semantic_reference_check_rejects_unknown_slack_variable() {
        let enc = register_slack_encoding(
            RealConstraintKind::Positive,
            vec![RelationId(0)],
            vec![VariableId(99)],
        );
        assert!(verify_semantic_references(&[enc], &[RelationId(0)], &[VariableId(0)]).is_err());
    }
}
