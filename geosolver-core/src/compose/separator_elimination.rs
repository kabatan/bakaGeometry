use std::collections::BTreeSet;

use crate::compose::message::ProjectionMessage;
use crate::graph::projection_dag::{authorize_block_relations, ProjectionBlock};
use crate::kernels::target_relation_search::{
    admit_target_relation_search, execute_target_relation_search,
};
use crate::kernels::traits::KernelContext;
use crate::planner::admission::KernelAdmissionStatus;
use crate::preprocess::compression::{relation_with_polynomial, CompressedSystemQ};
use crate::problem::canonicalize::RelationSource;
use crate::problem::context::SolverContext;
use crate::result::status::{AlgebraicReason, FailureKind, SolverError, SolverErrorKind, StageId};
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::{BlockId, RelationId, VariableId};
use crate::types::polynomial::{poly_variables, SparsePolynomialQ};
use crate::verify::verify_message::verify_projection_message;

pub fn eliminate_separators_from_message_relations(
    relations: Vec<SparsePolynomialQ>,
    keep_variables: BTreeSet<VariableId>,
    separator_variables: BTreeSet<VariableId>,
    target: VariableId,
    ctx: &mut SolverContext,
) -> Result<ProjectionMessage, SolverError> {
    if relations.is_empty() {
        return Err(algorithmic_hard_case(
            target,
            hash_sequence("p10-empty-message-relations", &[]),
            "separator elimination has no message relations",
        ));
    }
    if !keep_variables.contains(&target) {
        return Err(implementation_bug(
            target,
            "separator elimination keep set must contain target",
        ));
    }
    let allowed = keep_variables
        .union(&separator_variables)
        .copied()
        .collect::<BTreeSet<_>>();
    for relation in &relations {
        if !poly_variables(relation).is_subset(&allowed) {
            return Err(implementation_bug(
                target,
                "separator elimination relation uses undeclared variable",
            ));
        }
    }

    let mut system = message_only_system(relations, allowed, target);
    let mut block = message_only_block(&system, keep_variables);
    block.authorization_hash = authorize_block_relations(&block, &system);
    block.block_hash = hash_message_only_block(&block);
    system.compressed_hash = hash_message_only_system(&system);

    let mut kctx = KernelContext {
        block: block.clone(),
        system,
        child_messages: Vec::new(),
    };
    let admission = admit_target_relation_search(&block, &kctx, ctx);
    match admission.status {
        KernelAdmissionStatus::Admitted => {
            let Some(plan) = admission.execution_plan else {
                return Err(implementation_bug(
                    target,
                    "separator elimination admission lacked execution plan",
                ));
            };
            execute_target_relation_search(&plan, &mut kctx, ctx)
        }
        KernelAdmissionStatus::Declined { reason }
        | KernelAdmissionStatus::CostProhibited { reason, .. }
        | KernelAdmissionStatus::PlanProbeFailed { reason, .. } => Err(algorithmic_hard_case(
            target,
            block.block_hash,
            &format!("separator target-direct kernel declined: {reason}"),
        )),
    }
}

pub fn verify_separator_elimination_message(
    relations: &[SparsePolynomialQ],
    target: VariableId,
    message: &ProjectionMessage,
) -> Result<(), SolverError> {
    if relations.is_empty() {
        return Err(algorithmic_hard_case(
            target,
            hash_sequence("p10-empty-message-relations", &[]),
            "separator elimination has no message relations",
        ));
    }
    let all_variables = relations
        .iter()
        .flat_map(poly_variables)
        .collect::<BTreeSet<_>>();
    let keep_variables = BTreeSet::from([target]);
    let separator_variables = all_variables
        .difference(&keep_variables)
        .copied()
        .collect::<BTreeSet<_>>();
    let allowed = keep_variables
        .union(&separator_variables)
        .copied()
        .collect::<BTreeSet<_>>();
    let mut system = message_only_system(relations.to_vec(), allowed, target);
    let mut block = message_only_block(&system, keep_variables);
    block.authorization_hash = authorize_block_relations(&block, &system);
    block.block_hash = hash_message_only_block(&block);
    system.compressed_hash = hash_message_only_system(&system);
    let kctx = KernelContext {
        block,
        system,
        child_messages: Vec::new(),
    };
    verify_projection_message(message, &kctx)
}

fn message_only_system(
    relations: Vec<SparsePolynomialQ>,
    variables: BTreeSet<VariableId>,
    target: VariableId,
) -> CompressedSystemQ {
    let canonical_relations = relations
        .into_iter()
        .enumerate()
        .map(|(idx, relation)| {
            relation_with_polynomial(
                RelationId(idx as u32),
                relation,
                RelationSource::InputEquation,
            )
        })
        .collect::<Vec<_>>();
    let relation_order = canonical_relations
        .iter()
        .map(|relation| relation.id)
        .collect::<Vec<_>>();
    CompressedSystemQ {
        variables: variables.into_iter().collect(),
        target,
        relations: canonical_relations,
        relation_order,
        semantic_encodings: Vec::new(),
        substitutions: Vec::new(),
        guards: Vec::new(),
        rational_affine_transformations: Vec::new(),
        saturations: Vec::new(),
        feasibility_obligations: Vec::new(),
        diagnostics: Vec::new(),
        compression_trace: Default::default(),
        compressed_hash: hash_sequence("p10-message-only-system", &[]),
    }
}

fn message_only_block(
    system: &CompressedSystemQ,
    keep_variables: BTreeSet<VariableId>,
) -> ProjectionBlock {
    ProjectionBlock {
        block_id: BlockId(u32::MAX - 10),
        local_variables: system.variables.iter().copied().collect(),
        relation_ids: system.relation_order.clone(),
        exported_variables: keep_variables,
        child_block_ids: Vec::new(),
        parent_block_id: None,
        authorization_hash: hash_sequence("p10-message-only-authorization", &[]),
        duplication_certificates: Vec::new(),
        block_hash: hash_sequence("p10-message-only-block", &[]),
    }
}

fn hash_message_only_system(system: &CompressedSystemQ) -> Hash {
    let mut chunks = vec![system.target.0.to_be_bytes().to_vec()];
    for variable in &system.variables {
        chunks.push(variable.0.to_be_bytes().to_vec());
    }
    chunks.push(Vec::new());
    for relation in &system.relations {
        chunks.push(relation.hash.0.to_vec());
    }
    hash_sequence("p10-message-only-system", &chunks)
}

fn hash_message_only_block(block: &ProjectionBlock) -> Hash {
    let mut chunks = vec![
        block.block_id.0.to_be_bytes().to_vec(),
        block.authorization_hash.0.to_vec(),
    ];
    for relation in &block.relation_ids {
        chunks.push(relation.0.to_be_bytes().to_vec());
    }
    chunks.push(Vec::new());
    for variable in &block.exported_variables {
        chunks.push(variable.0.to_be_bytes().to_vec());
    }
    hash_sequence("p10-message-only-block", &chunks)
}

fn algorithmic_hard_case(
    target: VariableId,
    minimal_block_hash: Hash,
    reason: &str,
) -> SolverError {
    SolverError {
        target: Some(target),
        kind: SolverErrorKind::Failure(FailureKind::AlgorithmicHardCase {
            stage: StageId("P10SeparatorElimination".to_owned()),
            reason: AlgebraicReason(reason.to_owned()),
            minimal_block_hash,
        }),
    }
}

fn implementation_bug(target: VariableId, message: &str) -> SolverError {
    SolverError {
        target: Some(target),
        kind: SolverErrorKind::Failure(FailureKind::ImplementationBug {
            invariant_violated: message.to_owned(),
        }),
    }
}
