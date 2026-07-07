use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::graph::influence::TargetInfluenceGraph;
use crate::graph::tree_decomposition::{DecompositionNode, DecompositionTree};
use crate::preprocess::compression::CompressedSystemQ;
use crate::result::status::{FailureKind, SolverError, SolverErrorKind};
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::{BlockId, RelationId, VariableId};
use crate::types::polynomial::poly_variables;

pub type AuthorizationHash = Hash;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetProjectionDAG {
    pub blocks: Vec<ProjectionBlock>,
    pub root_block_id: BlockId,
    pub dag_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionBlock {
    pub block_id: BlockId,
    pub local_variables: BTreeSet<VariableId>,
    pub relation_ids: Vec<RelationId>,
    pub exported_variables: BTreeSet<VariableId>,
    pub child_block_ids: Vec<BlockId>,
    pub parent_block_id: Option<BlockId>,
    pub authorization_hash: AuthorizationHash,
    pub duplication_certificates: Vec<RelationDuplicationCertificate>,
    pub block_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationDuplicationCertificate {
    pub relation_id: RelationId,
    pub source_block_ids: Vec<BlockId>,
    pub certificate_hash: Hash,
}

pub fn build_target_projection_dag(
    system: &CompressedSystemQ,
    influence: &TargetInfluenceGraph,
    decomposition: &DecompositionTree,
) -> Result<TargetProjectionDAG, SolverError> {
    if influence.target != system.target {
        return Err(SolverError::invalid_input(
            Some(system.target),
            "target influence graph does not match compressed system target",
        ));
    }

    let mut blocks = Vec::new();
    let root_block_id = materialize_blocks(
        &decomposition.root,
        None,
        system.target,
        &BTreeSet::from([system.target]),
        &mut blocks,
    );
    if blocks.is_empty() {
        blocks.push(ProjectionBlock::new(
            BlockId(0),
            None,
            relevant_variables(system, influence),
            BTreeSet::from([system.target]),
        ));
    }

    assign_relations_once(system, &mut blocks)?;
    for block in &mut blocks {
        block.authorization_hash = authorize_block_relations(block, system);
        block.block_hash = hash_block(block);
    }
    let dag_hash = hash_dag(root_block_id, &blocks);
    let dag = TargetProjectionDAG {
        blocks,
        root_block_id,
        dag_hash,
    };
    validate_projection_dag(&dag, system)?;
    Ok(dag)
}

pub fn authorize_block_relations(
    block: &ProjectionBlock,
    system: &CompressedSystemQ,
) -> AuthorizationHash {
    let relation_hashes = system
        .relations
        .iter()
        .map(|relation| (relation.id, relation.hash))
        .collect::<BTreeMap<_, _>>();
    let mut chunks = vec![block.block_id.0.to_be_bytes().to_vec()];
    chunks.push(
        block
            .parent_block_id
            .map(|id| id.0.to_be_bytes().to_vec())
            .unwrap_or_else(|| vec![0xff]),
    );
    for var in &block.local_variables {
        chunks.push(var.0.to_be_bytes().to_vec());
    }
    chunks.push(Vec::new());
    for var in &block.exported_variables {
        chunks.push(var.0.to_be_bytes().to_vec());
    }
    chunks.push(Vec::new());
    for child in &block.child_block_ids {
        chunks.push(child.0.to_be_bytes().to_vec());
    }
    chunks.push(Vec::new());
    for relation_id in &block.relation_ids {
        chunks.push(relation_id.0.to_be_bytes().to_vec());
        if let Some(hash) = relation_hashes.get(relation_id) {
            chunks.push(hash.0.to_vec());
        }
    }
    hash_sequence("projection-block-authorization", &chunks)
}

pub fn validate_projection_dag(
    dag: &TargetProjectionDAG,
    system: &CompressedSystemQ,
) -> Result<(), SolverError> {
    let mut blocks_by_id = BTreeMap::new();
    for block in &dag.blocks {
        if blocks_by_id.insert(block.block_id, block).is_some() {
            return Err(implementation_bug("duplicate projection block id"));
        }
    }
    if !blocks_by_id.contains_key(&dag.root_block_id) {
        return Err(implementation_bug("projection DAG root block is missing"));
    }
    validate_projection_dag_topology(dag, &blocks_by_id)?;

    let relations_by_id = system
        .relations
        .iter()
        .map(|relation| (relation.id, relation))
        .collect::<BTreeMap<_, _>>();
    let mut relation_counts: BTreeMap<RelationId, usize> = BTreeMap::new();
    let mut relation_blocks: BTreeMap<RelationId, Vec<BlockId>> = BTreeMap::new();

    for block in &dag.blocks {
        if authorize_block_relations(block, system) != block.authorization_hash {
            return Err(implementation_bug(
                "projection block authorization hash mismatch",
            ));
        }
        if hash_block(block) != block.block_hash {
            return Err(implementation_bug("projection block hash mismatch"));
        }
        for child_id in &block.child_block_ids {
            let Some(child) = blocks_by_id.get(child_id) else {
                return Err(implementation_bug("projection DAG child block is missing"));
            };
            if child.parent_block_id != Some(block.block_id) {
                return Err(implementation_bug("projection DAG child parent mismatch"));
            }
        }
        for certificate in &block.duplication_certificates {
            if certificate.certificate_hash != hash_duplication_certificate(certificate) {
                return Err(implementation_bug(
                    "projection DAG duplication certificate hash mismatch",
                ));
            }
            if relation_duplication_certificate_cost(certificate) == 0 {
                return Err(implementation_bug(
                    "projection DAG duplication certificate has zero replay cost",
                ));
            }
        }
        for relation_id in &block.relation_ids {
            let Some(relation) = relations_by_id.get(relation_id) else {
                return Err(implementation_bug(
                    "projection block authorizes an unknown relation",
                ));
            };
            let relation_vars = poly_variables(&relation.polynomial);
            if !relation_vars.is_subset(&block.local_variables) {
                return Err(implementation_bug(
                    "projection block authorizes a relation outside local variables",
                ));
            }
            *relation_counts.entry(*relation_id).or_default() += 1;
            relation_blocks
                .entry(*relation_id)
                .or_default()
                .push(block.block_id);
        }
    }

    for relation in &system.relations {
        if !relation_counts.contains_key(&relation.id) {
            return Err(implementation_bug(
                "projection DAG omitted a compressed relation",
            ));
        }
    }
    for (relation_id, count) in relation_counts {
        if count <= 1 {
            continue;
        }
        let containing_blocks = relation_blocks
            .get(&relation_id)
            .cloned()
            .unwrap_or_default();
        let certified = dag.blocks.iter().any(|block| {
            block.duplication_certificates.iter().any(|certificate| {
                certificate.relation_id == relation_id
                    && certificate.source_block_ids == containing_blocks
            })
        });
        if !certified {
            return Err(implementation_bug(
                "projection DAG duplicated a relation without a certificate",
            ));
        }
    }

    if hash_dag(dag.root_block_id, &dag.blocks) != dag.dag_hash {
        return Err(implementation_bug("projection DAG hash mismatch"));
    }
    Ok(())
}

fn validate_projection_dag_topology(
    dag: &TargetProjectionDAG,
    blocks_by_id: &BTreeMap<BlockId, &ProjectionBlock>,
) -> Result<(), SolverError> {
    let root = blocks_by_id
        .get(&dag.root_block_id)
        .expect("root presence checked before topology validation");
    if root.parent_block_id.is_some() {
        return Err(implementation_bug(
            "projection DAG root block has a parent block",
        ));
    }
    for block in &dag.blocks {
        if block.block_id == dag.root_block_id {
            continue;
        }
        let Some(parent_id) = block.parent_block_id else {
            return Err(implementation_bug(
                "projection DAG non-root block has no parent",
            ));
        };
        let Some(parent) = blocks_by_id.get(&parent_id) else {
            return Err(implementation_bug(
                "projection DAG non-root parent block is missing",
            ));
        };
        if !parent.child_block_ids.contains(&block.block_id) {
            return Err(implementation_bug(
                "projection DAG parent does not list non-root child",
            ));
        }
    }

    let mut reachable = BTreeSet::new();
    let mut queue = VecDeque::from([dag.root_block_id]);
    while let Some(block_id) = queue.pop_front() {
        if !reachable.insert(block_id) {
            return Err(implementation_bug(
                "projection DAG block is reachable more than once from root",
            ));
        }
        let Some(block) = blocks_by_id.get(&block_id) else {
            return Err(implementation_bug(
                "projection DAG reachable block is missing",
            ));
        };
        for child_id in &block.child_block_ids {
            queue.push_back(*child_id);
        }
    }
    if reachable.len() != dag.blocks.len() {
        return Err(implementation_bug(
            "projection DAG contains a block unreachable from root",
        ));
    }
    Ok(())
}

impl ProjectionBlock {
    fn new(
        block_id: BlockId,
        parent_block_id: Option<BlockId>,
        local_variables: BTreeSet<VariableId>,
        exported_variables: BTreeSet<VariableId>,
    ) -> Self {
        Self {
            block_id,
            local_variables,
            relation_ids: Vec::new(),
            exported_variables,
            child_block_ids: Vec::new(),
            parent_block_id,
            authorization_hash: hash_sequence("projection-block-authorization", &[]),
            duplication_certificates: Vec::new(),
            block_hash: hash_sequence("projection-block", &[]),
        }
    }
}

fn materialize_blocks(
    node: &DecompositionNode,
    parent_block_id: Option<BlockId>,
    target: VariableId,
    export_to_parent: &BTreeSet<VariableId>,
    blocks: &mut Vec<ProjectionBlock>,
) -> BlockId {
    let block_id = BlockId(blocks.len() as u32);
    let mut local_variables = node.variables.clone();
    local_variables.insert(target);
    let mut exported_to_parent = export_to_parent.clone();
    exported_to_parent.insert(target);
    blocks.push(ProjectionBlock::new(
        block_id,
        parent_block_id,
        local_variables.clone(),
        exported_to_parent
            .intersection(&local_variables)
            .copied()
            .collect::<BTreeSet<_>>(),
    ));
    let child_ids = node
        .children
        .iter()
        .map(|child| {
            let mut child_export = node.separator.clone();
            child_export.extend(exported_to_parent.iter().copied());
            child_export.insert(target);
            materialize_blocks(child, Some(block_id), target, &child_export, blocks)
        })
        .collect::<Vec<_>>();
    blocks[block_id.0 as usize].child_block_ids = child_ids;
    block_id
}

fn assign_relations_once(
    system: &CompressedSystemQ,
    blocks: &mut [ProjectionBlock],
) -> Result<(), SolverError> {
    for relation in &system.relations {
        let relation_vars = poly_variables(&relation.polynomial);
        let Some((block_index, _)) = blocks
            .iter()
            .enumerate()
            .filter(|(_, block)| relation_vars.is_subset(&block.local_variables))
            .min_by_key(|(_, block)| {
                (
                    block.local_variables.len(),
                    block.child_block_ids.len(),
                    block.block_id,
                )
            })
        else {
            return Err(implementation_bug(
                "no projection block contains all variables of a compressed relation",
            ));
        };
        blocks[block_index].relation_ids.push(relation.id);
    }
    for block in blocks {
        block.relation_ids.sort();
    }
    Ok(())
}

fn relevant_variables(
    system: &CompressedSystemQ,
    influence: &TargetInfluenceGraph,
) -> BTreeSet<VariableId> {
    let mut variables = influence.target_component.variables.clone();
    variables.insert(system.target);
    for relation in &system.relations {
        variables.extend(poly_variables(&relation.polynomial));
    }
    variables
}

fn hash_block(block: &ProjectionBlock) -> Hash {
    let mut chunks = vec![
        block.block_id.0.to_be_bytes().to_vec(),
        block.authorization_hash.0.to_vec(),
    ];
    for relation_id in &block.relation_ids {
        chunks.push(relation_id.0.to_be_bytes().to_vec());
    }
    for child in &block.child_block_ids {
        chunks.push(child.0.to_be_bytes().to_vec());
    }
    for certificate in &block.duplication_certificates {
        chunks.push(certificate.certificate_hash.0.to_vec());
    }
    hash_sequence("projection-block", &chunks)
}

fn hash_dag(root_block_id: BlockId, blocks: &[ProjectionBlock]) -> Hash {
    let mut chunks = vec![root_block_id.0.to_be_bytes().to_vec()];
    for block in blocks {
        chunks.push(block.block_hash.0.to_vec());
    }
    hash_sequence("target-projection-dag", &chunks)
}

fn hash_duplication_certificate(certificate: &RelationDuplicationCertificate) -> Hash {
    let mut chunks = vec![certificate.relation_id.0.to_be_bytes().to_vec()];
    for block_id in &certificate.source_block_ids {
        chunks.push(block_id.0.to_be_bytes().to_vec());
    }
    chunks.push(
        relation_duplication_certificate_cost(certificate)
            .to_be_bytes()
            .to_vec(),
    );
    hash_sequence("relation-duplication-certificate", &chunks)
}

pub fn relation_duplication_certificate_cost(
    certificate: &RelationDuplicationCertificate,
) -> usize {
    certificate
        .source_block_ids
        .len()
        .saturating_mul(certificate.source_block_ids.len().max(1))
}

fn implementation_bug(message: &str) -> SolverError {
    SolverError {
        target: None,
        kind: SolverErrorKind::Failure(FailureKind::ImplementationBug {
            invariant_violated: message.to_owned(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::hypergraph::build_relation_variable_hypergraph;
    use crate::graph::influence::build_target_influence_graph;
    use crate::graph::separators::CostModel;
    use crate::graph::tree_decomposition::build_target_rooted_decomposition;
    use crate::graph::weighted_primal::build_weighted_primal_graph;
    use crate::preprocess::compression::CompressionState;
    use crate::problem::canonicalize::canonicalize_system;
    use crate::problem::input::make_problem;
    use crate::problem::validate::validate_input;
    use crate::result::status::SolverStatus;
    use crate::types::polynomial::{poly_add, poly_mul, variable_poly};

    #[test]
    fn no_separator_builds_one_large_target_block() {
        let (compressed, influence, tree) = no_separator_case();
        let dag = build_target_projection_dag(&compressed, &influence, &tree).unwrap();
        assert_eq!(dag.blocks.len(), 1);
        assert_eq!(
            dag.blocks[0].relation_ids,
            compressed
                .relations
                .iter()
                .map(|relation| relation.id)
                .collect::<Vec<_>>()
        );
        validate_projection_dag(&dag, &compressed).unwrap();
    }

    #[test]
    fn authorization_mismatch_fails_validation() {
        let (compressed, influence, tree) = no_separator_case();
        let mut dag = build_target_projection_dag(&compressed, &influence, &tree).unwrap();
        dag.blocks[0].authorization_hash = hash_sequence("tamper", &[]);
        let err = validate_projection_dag(&dag, &compressed).unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::ImplementationBug);
    }

    #[test]
    fn relation_duplication_without_certificate_fails() {
        let (compressed, influence, tree) = no_separator_case();
        let mut dag = build_target_projection_dag(&compressed, &influence, &tree).unwrap();
        let duplicated = dag.blocks[0].relation_ids[0];
        dag.blocks[0].relation_ids.push(duplicated);
        refresh_hashes(&mut dag, &compressed);
        let err = validate_projection_dag(&dag, &compressed).unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::ImplementationBug);
    }

    #[test]
    fn relation_deletion_fails_validation() {
        let (compressed, influence, tree) = no_separator_case();
        let mut dag = build_target_projection_dag(&compressed, &influence, &tree).unwrap();
        dag.blocks[0].relation_ids.clear();
        refresh_hashes(&mut dag, &compressed);
        let err = validate_projection_dag(&dag, &compressed).unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::ImplementationBug);
    }

    #[test]
    fn extra_parentless_non_root_block_fails_validation() {
        let (compressed, influence, tree) = no_separator_case();
        let mut dag = build_target_projection_dag(&compressed, &influence, &tree).unwrap();
        let root = dag.blocks[0].clone();
        dag.blocks.push(ProjectionBlock::new(
            BlockId(99),
            None,
            root.local_variables,
            BTreeSet::new(),
        ));
        refresh_hashes(&mut dag, &compressed);
        let err = validate_projection_dag(&dag, &compressed).unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::ImplementationBug);
    }

    #[test]
    fn parent_must_list_non_root_child() {
        let (compressed, influence, tree) = no_separator_case();
        let mut dag = build_target_projection_dag(&compressed, &influence, &tree).unwrap();
        let root_id = dag.root_block_id;
        let root = dag.blocks[0].clone();
        dag.blocks.push(ProjectionBlock::new(
            BlockId(100),
            Some(root_id),
            root.local_variables,
            BTreeSet::new(),
        ));
        refresh_hashes(&mut dag, &compressed);
        let err = validate_projection_dag(&dag, &compressed).unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::ImplementationBug);
    }

    #[test]
    fn duplicate_root_reachability_fails_validation() {
        let (compressed, influence, tree) = no_separator_case();
        let mut dag = build_target_projection_dag(&compressed, &influence, &tree).unwrap();
        let root_id = dag.root_block_id;
        let root = dag.blocks[0].clone();
        let child_id = BlockId(101);
        dag.blocks[0].child_block_ids = vec![child_id, child_id];
        dag.blocks.push(ProjectionBlock::new(
            child_id,
            Some(root_id),
            root.local_variables,
            BTreeSet::new(),
        ));
        refresh_hashes(&mut dag, &compressed);
        let err = validate_projection_dag(&dag, &compressed).unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::ImplementationBug);
    }

    fn no_separator_case() -> (CompressedSystemQ, TargetInfluenceGraph, DecompositionTree) {
        let t = VariableId(0);
        let x = VariableId(1);
        let y = VariableId(2);
        let relation = poly_add(
            &poly_add(
                &poly_mul(&variable_poly(t), &variable_poly(x)),
                &poly_mul(&variable_poly(x), &variable_poly(y)),
            ),
            &poly_mul(&variable_poly(y), &variable_poly(t)),
        );
        let canonical = canonicalize_system(
            validate_input(make_problem(vec![t, x, y], t, vec![relation], Vec::new())).unwrap(),
        )
        .unwrap();
        let compressed = CompressionState::from_system(canonical).to_compressed_system();
        let h = build_relation_variable_hypergraph(&compressed);
        let influence = build_target_influence_graph(&h, t);
        let g = build_weighted_primal_graph(&compressed, &influence);
        let tree = build_target_rooted_decomposition(&g, t, &CostModel::default());
        (compressed, influence, tree)
    }

    fn refresh_hashes(dag: &mut TargetProjectionDAG, system: &CompressedSystemQ) {
        for block in &mut dag.blocks {
            block.authorization_hash = authorize_block_relations(block, system);
            block.block_hash = hash_block(block);
        }
        dag.dag_hash = hash_dag(dag.root_block_id, &dag.blocks);
    }
}
