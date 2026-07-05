use std::collections::BTreeMap;

use crate::compose::compose::{compose_projection_messages, ComposedProjection};
use crate::compose::final_support::{
    build_final_support_or_nonfinite, finalize_nonfinite_result, FinalSupportComputation,
};
use crate::compose::message::ProjectionMessage;
use crate::graph::hypergraph::{build_relation_variable_hypergraph, RelationVariableHypergraph};
use crate::graph::influence::{build_target_influence_graph, TargetInfluenceGraph};
use crate::graph::projection_dag::{validate_projection_dag, ProjectionBlock, TargetProjectionDAG};
use crate::graph::separators::CostModel;
use crate::graph::tree_decomposition::{build_target_rooted_decomposition, DecompositionTree};
use crate::graph::weighted_primal::{build_weighted_primal_graph, WeightedPrimalGraph};
use crate::kernels::traits::KernelContext;
use crate::planner::kernel_plan::{hash_kernel_plan, KernelPlan};
use crate::planner::planner::plan_all_blocks;
use crate::preprocess::compression::{
    max_coefficient_height_bits, pre_kernel_compress, CompressedSystemQ,
};
use crate::problem::canonicalize::CanonicalSystemQ;
use crate::problem::context::SolverContext;
use crate::problem::input::RationalTargetProblem;
use crate::problem::validate::{validate_input, ValidatedProblem};
use crate::result::cost_trace::{GlobalCostTrace, VerificationCostTrace};
use crate::result::status::{AlgebraicReason, FailureKind, SolverError, SolverErrorKind, StageId};
use crate::roots::decode::{decode_candidates, TargetCandidate};
use crate::roots::isolate::{isolate_real_roots, RealRootRecord, RootIsolationOptions};
use crate::roots::squarefree::squarefree_support;
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::{BlockId, VariableId};
use crate::types::polynomial::{poly_monomial_count, poly_total_degree};
use crate::types::univariate::UniPolynomialQ;
use crate::verify::run_certificate::{
    build_core_run_certificate, final_dag_replay_evidence, hash_projection_messages,
    CoreRunCertificate, CoreRunCertificateInput,
};
use crate::verify::verify_message::verify_projection_message;
use crate::verify::verify_support::GlobalSupportCertificate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphBundle {
    pub hypergraph: RelationVariableHypergraph,
    pub influence: TargetInfluenceGraph,
    pub weighted_primal: WeightedPrimalGraph,
    pub decomposition: DecompositionTree,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootCandidateBundle {
    pub squarefree_support: UniPolynomialQ,
    pub root_isolation: Vec<RealRootRecord>,
    pub decoded_candidates: Vec<TargetCandidate>,
}

pub fn step_validate(
    problem: RationalTargetProblem,
    _ctx: &mut SolverContext,
) -> Result<ValidatedProblem, SolverError> {
    validate_input(problem)
}

pub fn step_canonicalize(
    validated: ValidatedProblem,
    _ctx: &mut SolverContext,
) -> Result<CanonicalSystemQ, SolverError> {
    crate::problem::canonicalize::canonicalize_system(validated)
}

pub fn step_compress(
    canonical: CanonicalSystemQ,
    ctx: &mut SolverContext,
) -> Result<CompressedSystemQ, SolverError> {
    pre_kernel_compress(canonical, ctx)
}

pub fn step_build_graphs(
    compressed: &CompressedSystemQ,
    _ctx: &mut SolverContext,
) -> Result<GraphBundle, SolverError> {
    let hypergraph = build_relation_variable_hypergraph(compressed);
    let influence = build_target_influence_graph(&hypergraph, compressed.target);
    let weighted_primal = build_weighted_primal_graph(compressed, &influence);
    let decomposition = build_target_rooted_decomposition(
        &weighted_primal,
        compressed.target,
        &CostModel::default(),
    );
    Ok(GraphBundle {
        hypergraph,
        influence,
        weighted_primal,
        decomposition,
    })
}

pub fn step_build_dag(
    graphs: &GraphBundle,
    compressed: &CompressedSystemQ,
    _ctx: &mut SolverContext,
) -> Result<TargetProjectionDAG, SolverError> {
    crate::graph::projection_dag::build_target_projection_dag(
        compressed,
        &graphs.influence,
        &graphs.decomposition,
    )
}

pub fn step_plan(
    dag: &TargetProjectionDAG,
    compressed: &CompressedSystemQ,
    ctx: &mut SolverContext,
) -> Result<Vec<KernelPlan>, SolverError> {
    plan_all_blocks(dag, compressed, ctx)
}

pub fn step_execute(
    dag: &TargetProjectionDAG,
    plans: &[KernelPlan],
    compressed: &CompressedSystemQ,
    ctx: &mut SolverContext,
) -> Result<Vec<ProjectionMessage>, SolverError> {
    validate_projection_dag(dag, compressed)?;
    let plans_by_block = plans
        .iter()
        .map(|plan| (plan.block_id, plan))
        .collect::<BTreeMap<_, _>>();
    let blocks_by_id = dag
        .blocks
        .iter()
        .map(|block| (block.block_id, block))
        .collect::<BTreeMap<_, _>>();
    let mut messages_by_block = BTreeMap::<BlockId, ProjectionMessage>::new();
    let mut messages = Vec::new();

    for block in execution_order(dag) {
        let Some(plan) = plans_by_block.get(&block.block_id) else {
            return Err(implementation_bug("projection block has no KernelPlan"));
        };
        let child_messages = block
            .child_block_ids
            .iter()
            .filter_map(|child_id| messages_by_block.get(child_id).cloned())
            .collect::<Vec<_>>();
        if child_messages.len() != block.child_block_ids.len() {
            return Err(implementation_bug(
                "projection block executed before all child messages were available",
            ));
        }
        for child_id in &block.child_block_ids {
            if !blocks_by_id.contains_key(child_id) {
                return Err(implementation_bug(
                    "projection block references missing child",
                ));
            }
        }
        let message =
            execute_block_with_declared_ladder(&block, plan, child_messages, compressed, ctx)?;
        messages_by_block.insert(block.block_id, message.clone());
        messages.push(message);
    }

    Ok(messages)
}

pub fn step_verify_messages(
    dag: &TargetProjectionDAG,
    messages: &[ProjectionMessage],
    compressed: &CompressedSystemQ,
) -> Result<(), SolverError> {
    let messages_by_block = messages
        .iter()
        .map(|message| (message.block_id, message.clone()))
        .collect::<BTreeMap<_, _>>();
    if messages_by_block.len() != dag.blocks.len() {
        return Err(implementation_bug(
            "projection message set does not cover each DAG block exactly once",
        ));
    }
    for block in execution_order(dag) {
        let Some(message) = messages_by_block.get(&block.block_id) else {
            return Err(implementation_bug("projection block has no message"));
        };
        let child_messages = block
            .child_block_ids
            .iter()
            .filter_map(|child_id| messages_by_block.get(child_id).cloned())
            .collect::<Vec<_>>();
        if child_messages.len() != block.child_block_ids.len() {
            return Err(implementation_bug(
                "projection message verification lacks a child message",
            ));
        }
        let kctx = KernelContext {
            block: block.clone(),
            system: compressed.clone(),
            child_messages,
        };
        verify_projection_message(message, &kctx)?;
    }
    Ok(())
}

pub fn step_compose(
    dag: &TargetProjectionDAG,
    messages: Vec<ProjectionMessage>,
    target: VariableId,
    ctx: &mut SolverContext,
) -> Result<ComposedProjection, SolverError> {
    compose_projection_messages(dag, messages, target, ctx)
}

pub fn step_support(
    composed: &ComposedProjection,
    target: VariableId,
    ctx: &mut SolverContext,
) -> Result<FinalSupportComputation, SolverError> {
    build_final_support_or_nonfinite(composed.clone(), target, ctx)
}

pub fn step_roots(
    support: &UniPolynomialQ,
    target: VariableId,
    ctx: &mut SolverContext,
) -> Result<RootCandidateBundle, SolverError> {
    let squarefree_support = squarefree_support(support)?;
    let root_isolation = isolate_real_roots(
        &squarefree_support,
        RootIsolationOptions {
            method: ctx.options.root_isolation_method.clone(),
        },
    )?;
    let decoded_candidates = decode_candidates(target, &squarefree_support, &root_isolation);
    if decoded_candidates.len() != root_isolation.len() {
        return Err(implementation_bug(
            "decoded candidate count does not match isolated root count",
        ));
    }
    Ok(RootCandidateBundle {
        squarefree_support,
        root_isolation,
        decoded_candidates,
    })
}

pub fn step_cost_trace(
    compressed: &CompressedSystemQ,
    dag: &TargetProjectionDAG,
    messages: &[ProjectionMessage],
    composed: Option<&ComposedProjection>,
) -> GlobalCostTrace {
    let relation_polys = compressed
        .relations
        .iter()
        .map(|relation| relation.polynomial.clone())
        .collect::<Vec<_>>();
    let max_block_width = dag
        .blocks
        .iter()
        .map(|block| block.local_variables.len())
        .max()
        .unwrap_or(0);
    let max_separator_width = dag
        .blocks
        .iter()
        .map(|block| block.exported_variables.len())
        .max()
        .unwrap_or(0);
    GlobalCostTrace {
        total_variable_count: compressed.variables.len(),
        total_relation_count: compressed.relations.len(),
        total_monomial_count: relation_polys.iter().map(poly_monomial_count).sum(),
        max_total_degree: relation_polys
            .iter()
            .map(|relation| poly_total_degree(relation) as usize)
            .max()
            .unwrap_or(0),
        max_coefficient_height_bits: max_coefficient_height_bits(&compressed.relations),
        max_block_width,
        max_separator_width,
        block_traces: messages
            .iter()
            .map(|message| message.cost_trace.clone())
            .collect(),
        composition_trace: composed
            .map(|projection| projection.composition_cost.clone())
            .unwrap_or_default(),
        verification_trace: VerificationCostTrace {
            checked_relation_count: messages
                .iter()
                .map(|message| message.relation_generators.len())
                .sum(),
        },
    }
}

pub fn step_core_certificate(
    problem: &RationalTargetProblem,
    canonical: &CanonicalSystemQ,
    compressed: &CompressedSystemQ,
    graphs: &GraphBundle,
    dag: &TargetProjectionDAG,
    plans: &[KernelPlan],
    messages: &[ProjectionMessage],
    support: Option<&UniPolynomialQ>,
    roots: &RootCandidateBundle,
    support_certificate: Option<&GlobalSupportCertificate>,
) -> CoreRunCertificate {
    let kernel_plan_hashes = plans.iter().map(hash_kernel_plan).collect::<Vec<_>>();
    let replay_evidence = build_dag_replay_evidence(dag, &kernel_plan_hashes, messages);
    build_core_run_certificate(CoreRunCertificateInput {
        input_hash: problem.input_hash,
        canonical_hash: canonical.canonical_hash,
        target_variable: compressed.target,
        compression_hash: compressed.compressed_hash,
        hypergraph_hash: graphs.hypergraph.hypergraph_hash,
        dag_hash: dag.dag_hash,
        kernel_plan_hashes,
        projection_messages: messages,
        support,
        squarefree_support: Some(&roots.squarefree_support),
        root_isolation: &roots.root_isolation,
        decoded_candidates: &roots.decoded_candidates,
        global_support_certificate_hash: support_certificate.map(|cert| cert.certificate_hash),
        final_dag_replay_evidence_hash: Some(replay_evidence.evidence_hash),
    })
}

pub fn finalize_nonfinite_pipeline_result(
    target: VariableId,
    cert: crate::compose::final_support::NonFiniteCertificate,
    composed: &ComposedProjection,
    messages: Vec<ProjectionMessage>,
    cost_trace: GlobalCostTrace,
) -> Result<crate::result::output::TargetSolveResult, SolverError> {
    finalize_nonfinite_result(target, cert, composed, messages, cost_trace)
}

fn execute_block_with_declared_ladder(
    block: &ProjectionBlock,
    plan: &KernelPlan,
    child_messages: Vec<ProjectionMessage>,
    compressed: &CompressedSystemQ,
    ctx: &mut SolverContext,
) -> Result<ProjectionMessage, SolverError> {
    let kernels = crate::kernels::all_kernels();
    let mut last_allowed_error = None;
    for execution_plan in &plan.declared_ladder {
        let Some(kernel) = kernels
            .iter()
            .find(|kernel| kernel.kind() == execution_plan.kernel_kind)
        else {
            continue;
        };
        let mut kctx = KernelContext {
            block: block.clone(),
            system: compressed.clone(),
            child_messages: child_messages.clone(),
        };
        match kernel.execute(execution_plan, &mut kctx, ctx) {
            Ok(message) => {
                verify_projection_message(&message, &kctx)?;
                return Ok(message);
            }
            Err(err)
                if execution_plan
                    .failure_behavior
                    .allowed_statuses
                    .contains(&err.public_status()) =>
            {
                last_allowed_error = Some(err);
            }
            Err(err) => return Err(err),
        }
    }
    Err(last_allowed_error.unwrap_or_else(|| {
        algorithmic_hard_case(
            Some(compressed.target),
            block.block_hash,
            "declared production kernel ladder produced no projection message",
        )
    }))
}

fn execution_order(dag: &TargetProjectionDAG) -> Vec<ProjectionBlock> {
    let mut blocks = dag.blocks.clone();
    blocks.sort_by_key(|block| {
        (
            usize::MAX - block_depth_from_root(dag, block.block_id),
            block.block_id,
        )
    });
    blocks
}

fn block_depth_from_root(dag: &TargetProjectionDAG, block_id: BlockId) -> usize {
    let mut depth = 0;
    let mut current = Some(block_id);
    while let Some(id) = current {
        let Some(block) = dag.blocks.iter().find(|block| block.block_id == id) else {
            break;
        };
        current = block.parent_block_id;
        if current.is_some() {
            depth += 1;
        }
    }
    depth
}

fn build_dag_replay_evidence(
    dag: &TargetProjectionDAG,
    kernel_plan_hashes: &[Hash],
    messages: &[ProjectionMessage],
) -> crate::verify::run_certificate::FinalDagReplayEvidence {
    let message_hashes = hash_projection_messages(messages);
    let message_block_ids = messages
        .iter()
        .map(|message| message.block_id)
        .collect::<Vec<_>>();
    let per_message_source_relation_hashes = messages
        .iter()
        .map(|message| message.certificate.source_relation_hashes.clone())
        .collect::<Vec<_>>();
    let message_child_dependency_hashes = messages
        .iter()
        .map(|message| {
            dag.blocks
                .iter()
                .find(|block| block.block_id == message.block_id)
                .map(|block| {
                    block
                        .child_block_ids
                        .iter()
                        .filter_map(|child_id| {
                            messages
                                .iter()
                                .find(|candidate| candidate.block_id == *child_id)
                                .map(|candidate| candidate.package_hash)
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        })
        .collect::<Vec<_>>();
    let block_authorization_hashes = messages
        .iter()
        .filter_map(|message| {
            dag.blocks
                .iter()
                .find(|block| block.block_id == message.block_id)
                .map(|block| block.authorization_hash)
        })
        .collect::<Vec<_>>();
    let edge_authorization_hashes = dag
        .blocks
        .iter()
        .flat_map(|block| {
            block.child_block_ids.iter().map(|child_id| {
                hash_sequence(
                    "projection-dag-edge-authorization",
                    &[
                        block.block_id.0.to_be_bytes().to_vec(),
                        child_id.0.to_be_bytes().to_vec(),
                        block.authorization_hash.0.to_vec(),
                    ],
                )
            })
        })
        .collect::<Vec<_>>();
    final_dag_replay_evidence(
        Some(dag.dag_hash),
        message_hashes,
        kernel_plan_hashes.to_vec(),
        message_block_ids,
        per_message_source_relation_hashes,
        message_child_dependency_hashes,
        block_authorization_hashes,
        edge_authorization_hashes,
        true,
    )
}

fn algorithmic_hard_case(target: Option<VariableId>, hash: Hash, reason: &str) -> SolverError {
    SolverError {
        target,
        kind: SolverErrorKind::Failure(FailureKind::AlgorithmicHardCase {
            stage: StageId("ExecuteLocalProjectionKernels".to_owned()),
            reason: AlgebraicReason(reason.to_owned()),
            minimal_block_hash: hash,
        }),
    }
}

fn implementation_bug(message: &str) -> SolverError {
    SolverError {
        target: None,
        kind: SolverErrorKind::Failure(FailureKind::ImplementationBug {
            invariant_violated: message.to_owned(),
        }),
    }
}
