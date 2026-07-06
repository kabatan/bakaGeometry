use std::collections::BTreeMap;

use crate::compose::compose::{compose_projection_messages, ComposedProjection};
use crate::compose::final_support::{
    build_final_support_or_nonfinite_with_system, finalize_nonfinite_result,
    FinalSupportComputation,
};
use crate::compose::message::ProjectionMessage;
use crate::graph::hypergraph::{build_relation_variable_hypergraph, RelationVariableHypergraph};
use crate::graph::influence::{build_target_influence_graph, TargetInfluenceGraph};
use crate::graph::projection_dag::{validate_projection_dag, ProjectionBlock, TargetProjectionDAG};
use crate::graph::separators::CostModel;
use crate::graph::tree_decomposition::{build_target_rooted_decomposition, DecompositionTree};
use crate::graph::weighted_primal::{build_weighted_primal_graph, WeightedPrimalGraph};
use crate::kernels::traits::KernelContext;
use crate::planner::kernel_plan::{KernelExecutionPlan, KernelPlan};
use crate::planner::planner::plan_all_blocks;
use crate::preprocess::compression::{
    max_coefficient_height_bits, pre_kernel_compress, CompressedSystemQ,
};
use crate::problem::canonicalize::CanonicalSystemQ;
use crate::problem::context::SolverContext;
use crate::problem::input::RationalTargetProblem;
use crate::problem::validate::{validate_input, ValidatedProblem};
use crate::result::cost_trace::{GlobalCostTrace, ProjectionCostTrace, VerificationCostTrace};
use crate::result::output::projection_trace_from_solver_error;
use crate::result::status::{
    AlgebraicReason, FailureKind, SolverError, SolverErrorKind, SolverStatus, StageId,
};
use crate::roots::decode::{decode_candidates, TargetCandidate};
use crate::roots::isolate::{isolate_real_roots, RealRootRecord, RootIsolationOptions};
use crate::roots::squarefree::squarefree_support;
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::{BlockId, VariableId};
use crate::types::polynomial::{
    max_poly_coefficient_height_bits, poly_monomial_count, poly_total_degree, SparsePolynomialQ,
};
use crate::types::univariate::{degree_uni, UniPolynomialQ};
use crate::verify::run_certificate::{
    build_core_run_certificate, build_final_dag_replay_evidence_from_dag, CoreRunCertificate,
    CoreRunCertificateInput,
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
            if block_requires_projection(&block) {
                return Err(implementation_bug("projection block has no KernelPlan"));
            }
            continue;
        };
        let child_messages =
            collect_child_projection_messages(&block, &blocks_by_id, &messages_by_block)?;
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
    if messages_by_block.len() != messages.len() {
        return Err(implementation_bug(
            "projection message set contains duplicate block messages",
        ));
    }
    let blocks_by_id = dag
        .blocks
        .iter()
        .map(|block| (block.block_id, block))
        .collect::<BTreeMap<_, _>>();
    for block in execution_order(dag) {
        if !block_requires_projection(&block) {
            if messages_by_block.contains_key(&block.block_id) {
                return Err(implementation_bug(
                    "structural projection block unexpectedly produced a message",
                ));
            }
            continue;
        }
        let Some(message) = messages_by_block.get(&block.block_id) else {
            return Err(implementation_bug("projection block has no message"));
        };
        let child_messages =
            collect_child_projection_messages(&block, &blocks_by_id, &messages_by_block)?;
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
    compressed: &CompressedSystemQ,
    target: VariableId,
    ctx: &mut SolverContext,
) -> Result<FinalSupportComputation, SolverError> {
    build_final_support_or_nonfinite_with_system(composed.clone(), target, Some(compressed), ctx)
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
    support: Option<&UniPolynomialQ>,
    certificate: Option<&CoreRunCertificate>,
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
        final_support_degree: support.and_then(degree_uni),
        certificate_size: certificate.map(core_certificate_size_kappa),
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

pub fn step_failure_cost_trace(
    compressed: &CompressedSystemQ,
    dag: &TargetProjectionDAG,
    messages: &[ProjectionMessage],
    composed: Option<&ComposedProjection>,
    support: Option<&UniPolynomialQ>,
    certificate: Option<&CoreRunCertificate>,
    err: &SolverError,
) -> GlobalCostTrace {
    let mut trace = step_cost_trace(compressed, dag, messages, composed, support, certificate);
    if let Some(mut failure_trace) = projection_trace_from_solver_error(err) {
        enrich_failure_projection_trace(&mut failure_trace, compressed, dag);
        trace.block_traces.push(failure_trace);
    }
    trace
}

fn enrich_failure_projection_trace(
    trace: &mut ProjectionCostTrace,
    compressed: &CompressedSystemQ,
    dag: &TargetProjectionDAG,
) {
    let Some(block) = dag
        .blocks
        .iter()
        .find(|block| block.block_id == trace.block_id)
    else {
        enrich_failure_trace_from_global_system(trace, compressed);
        return;
    };

    let relation_polys = block_relation_polys(compressed, block);
    trace.local_variable_count = block.local_variables.len();
    trace.exported_variable_count = block.exported_variables.len();
    trace.local_relation_count = block.relation_ids.len();
    trace.local_monomial_count = relation_polys.iter().map(poly_monomial_count).sum();
    let coefficient_height = if relation_polys.is_empty() {
        max_coefficient_height_bits(&compressed.relations)
    } else {
        max_poly_coefficient_height_bits(&relation_polys)
    };
    if trace.coefficient_height_before_bits == 0 {
        trace.coefficient_height_before_bits = coefficient_height;
    }
    if trace.coefficient_height_after_bits == 0 {
        trace.coefficient_height_after_bits = trace.coefficient_height_before_bits;
    }
}

fn enrich_failure_trace_from_global_system(
    trace: &mut ProjectionCostTrace,
    compressed: &CompressedSystemQ,
) {
    let relation_polys = compressed
        .relations
        .iter()
        .map(|relation| relation.polynomial.clone())
        .collect::<Vec<_>>();
    trace.local_variable_count = compressed.variables.len();
    trace.exported_variable_count = if compressed.variables.contains(&compressed.target) {
        1
    } else {
        0
    };
    trace.local_relation_count = compressed.relations.len();
    trace.local_monomial_count = relation_polys.iter().map(poly_monomial_count).sum();
    let coefficient_height = max_poly_coefficient_height_bits(&relation_polys);
    if trace.coefficient_height_before_bits == 0 {
        trace.coefficient_height_before_bits = coefficient_height;
    }
    if trace.coefficient_height_after_bits == 0 {
        trace.coefficient_height_after_bits = trace.coefficient_height_before_bits;
    }
}

fn block_relation_polys(
    compressed: &CompressedSystemQ,
    block: &ProjectionBlock,
) -> Vec<SparsePolynomialQ> {
    let relation_by_id = compressed
        .relations
        .iter()
        .map(|relation| (relation.id, &relation.polynomial))
        .collect::<BTreeMap<_, _>>();
    block
        .relation_ids
        .iter()
        .filter_map(|relation_id| relation_by_id.get(relation_id).map(|poly| (*poly).clone()))
        .collect()
}

fn core_certificate_size_kappa(cert: &CoreRunCertificate) -> usize {
    let mut size = 10;
    size += cert.kernel_plan_hashes.len();
    size += cert.projection_message_hashes.len();
    size += cert.global_support_hash.is_some() as usize;
    size += cert.squarefree_support_hash.is_some() as usize;
    size += cert.root_isolation_hash.is_some() as usize;
    size += cert.decoded_candidate_hash.is_some() as usize;
    size += cert.exact_image_certificate_hash.is_some() as usize;
    size += cert.global_support_certificate_hash.is_some() as usize;
    size += cert.final_dag_replay_evidence_hash.is_some() as usize;
    if let Some(evidence) = &cert.final_dag_replay_evidence {
        size += evidence.actual_projection_dag_hash.is_some() as usize;
        size += evidence.projection_message_hashes.len();
        size += evidence.kernel_plan_hashes.len();
        size += evidence.message_block_ids.len();
        size += evidence
            .per_message_source_relation_hashes
            .iter()
            .map(Vec::len)
            .sum::<usize>();
        size += evidence
            .message_child_dependency_hashes
            .iter()
            .map(Vec::len)
            .sum::<usize>();
        size += evidence.block_authorization_hashes.len();
        size += evidence
            .block_relation_ids
            .iter()
            .map(Vec::len)
            .sum::<usize>();
        size += evidence
            .block_relation_hashes
            .iter()
            .map(Vec::len)
            .sum::<usize>();
        size += evidence.child_edges.len();
        size += evidence.edge_authorization_hashes.len();
        size += evidence.support_certificate_hash.is_some() as usize;
        size += evidence.root_isolation_hash.is_some() as usize;
        size += evidence.decoded_candidate_hash.is_some() as usize;
    }
    size
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
    exact_image_certificate: Option<&crate::fiber::exact_image::FiberClassificationResult>,
    support_certificate: Option<&GlobalSupportCertificate>,
) -> CoreRunCertificate {
    let kernel_plan_hashes = executed_plan_hashes(plans, messages);
    let root_isolation_hash = Some(crate::verify::run_certificate::hash_root_isolation(
        &roots.root_isolation,
    ));
    let decoded_candidate_hash = Some(crate::verify::run_certificate::hash_decoded_candidates(
        &roots.decoded_candidates,
    ));
    let replay_evidence = build_final_dag_replay_evidence_from_dag(
        dag,
        compressed,
        kernel_plan_hashes.clone(),
        messages,
        support_certificate.map(|cert| cert.certificate_hash),
        root_isolation_hash,
        decoded_candidate_hash,
    );
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
        exact_image_certificate,
        global_support_certificate_hash: support_certificate.map(|cert| cert.certificate_hash),
        final_dag_replay_evidence: Some(replay_evidence),
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
    let mut route_failures = Vec::new();
    for execution_plan in &plan.declared_ladder {
        if let Err(err) = enforce_route_budget_preflight(execution_plan) {
            let allowed = is_declared_route_failure_allowed(execution_plan, &err);
            record_block_projection_failure_trace(block, execution_plan, &err, allowed, ctx);
            if allowed {
                route_failures.push((execution_plan.kernel_kind, execution_plan.plan_hash, err));
                continue;
            }
            return Err(err);
        }
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
            Ok(message) => match enforce_route_budget_postflight(execution_plan, &message)
                .and_then(|_| verify_projection_message(&message, &kctx))
            {
                Ok(()) => return Ok(message),
                Err(err) => {
                    let allowed = is_declared_route_failure_allowed(execution_plan, &err);
                    record_block_projection_failure_trace(
                        block,
                        execution_plan,
                        &err,
                        allowed,
                        ctx,
                    );
                    if allowed {
                        route_failures.push((
                            execution_plan.kernel_kind,
                            execution_plan.plan_hash,
                            err,
                        ));
                    } else {
                        return Err(err);
                    }
                }
            },
            Err(err) => {
                let allowed = is_declared_route_failure_allowed(execution_plan, &err);
                record_block_projection_failure_trace(block, execution_plan, &err, allowed, ctx);
                if allowed {
                    route_failures.push((
                        execution_plan.kernel_kind,
                        execution_plan.plan_hash,
                        err,
                    ));
                } else {
                    return Err(err);
                }
            }
        }
    }
    Err(aggregate_ladder_failure(
        compressed.target,
        block.block_hash,
        &route_failures,
    ))
}

fn enforce_route_budget_preflight(plan: &KernelExecutionPlan) -> Result<(), SolverError> {
    if !plan.algebraic_work_estimate.is_hash_current() || !plan.route_budget.is_hash_current() {
        return Err(implementation_bug(
            "route budget or algebraic work estimate hash mismatch before execution",
        ));
    }
    if plan.algebraic_work_estimate.predicted_work_units > plan.route_budget.max_work_units {
        return Err(route_budget_failure(
            plan,
            "predicted work exceeds route budget",
        ));
    }
    if plan
        .algebraic_work_estimate
        .predicted_intermediate_terms
        .is_some_and(|terms| terms.exceeds_usize(plan.route_budget.max_intermediate_terms))
    {
        return Err(route_budget_failure(
            plan,
            "predicted intermediate terms exceed route budget",
        ));
    }
    if plan
        .algebraic_work_estimate
        .predicted_output_terms
        .is_some_and(|terms| terms.exceeds_usize(plan.route_budget.max_output_terms))
    {
        return Err(route_budget_failure(
            plan,
            "predicted output terms exceed route budget",
        ));
    }
    if plan.algebraic_work_estimate.max_input_terms > plan.route_budget.max_input_terms_per_pair
        || plan.algebraic_work_estimate.max_keep_variable_count
            > plan.route_budget.max_keep_variables
        || plan.algebraic_work_estimate.max_total_degree > plan.route_budget.max_total_degree
    {
        return Err(route_budget_failure(
            plan,
            "declared algebraic footprint exceeds route budget",
        ));
    }
    if plan
        .algebraic_work_estimate
        .predicted_coefficient_height_bits
        .is_some_and(|bits| bits.exceeds_usize(plan.route_budget.max_coefficient_height_bits))
    {
        return Err(route_budget_failure(
            plan,
            "predicted coefficient height exceeds route budget",
        ));
    }
    Ok(())
}

fn enforce_route_budget_postflight(
    plan: &KernelExecutionPlan,
    message: &ProjectionMessage,
) -> Result<(), SolverError> {
    let Some(route_cost) = &message.cost_trace.route_cost else {
        return Err(implementation_bug(
            "projection message is missing route budget cost trace",
        ));
    };
    if route_cost.algebraic_work_estimate_hash != plan.algebraic_work_estimate.estimate_hash
        || route_cost.route_budget_hash != plan.route_budget.budget_hash
    {
        return Err(implementation_bug(
            "projection message route cost trace is not bound to its execution plan",
        ));
    }
    let output_terms = message
        .relation_generators
        .iter()
        .map(poly_monomial_count)
        .sum::<usize>();
    if output_terms > plan.route_budget.max_output_terms {
        return Err(route_budget_failure(
            plan,
            "produced output terms exceed route budget",
        ));
    }
    let output_degree = message
        .relation_generators
        .iter()
        .map(|relation| poly_total_degree(relation) as usize)
        .max()
        .unwrap_or(0);
    if output_degree > plan.route_budget.max_total_degree {
        return Err(route_budget_failure(
            plan,
            "produced output degree exceeds route budget",
        ));
    }
    let output_height = max_poly_coefficient_height_bits(&message.relation_generators);
    if output_height > plan.route_budget.max_coefficient_height_bits {
        return Err(route_budget_failure(
            plan,
            "produced coefficient height exceeds route budget",
        ));
    }
    Ok(())
}

fn route_budget_failure(plan: &KernelExecutionPlan, reason: &str) -> SolverError {
    SolverError {
        target: plan.exported_variables.first().copied(),
        kind: SolverErrorKind::Failure(FailureKind::FiniteResourceFailure {
            stage: StageId(format!("RouteBudget::{:?}::{reason}", plan.kernel_kind)),
            block_id: Some(plan.block_id),
            matrix_rows: plan
                .resource_bounds
                .max_matrix_rows
                .or(plan.algebraic_work_estimate.matrix_rows),
            matrix_cols: plan
                .resource_bounds
                .max_matrix_cols
                .or(plan.algebraic_work_estimate.matrix_cols),
            matrix_density: None,
            quotient_rank_estimate: plan.algebraic_work_estimate.quotient_rank_estimate,
            coefficient_height_bits: plan
                .algebraic_work_estimate
                .predicted_coefficient_height_bits
                .map(|bits| bits.as_usize_saturating()),
            memory_bytes: plan
                .resource_bounds
                .max_memory_bytes
                .or_else(|| Some(plan.route_budget.max_work_units.0.min(u64::MAX as u128) as u64)),
        }),
    }
}

fn aggregate_ladder_failure(
    target: VariableId,
    block_hash: Hash,
    route_failures: &[(crate::kernels::traits::KernelKind, Hash, SolverError)],
) -> SolverError {
    if route_failures.is_empty() {
        return algorithmic_hard_case(
            Some(target),
            block_hash,
            "declared production kernel ladder produced no projection message and no route failure",
        );
    }
    if let Some((_, _, err)) = route_failures.iter().find(|(_, _, err)| {
        err.public_status() == crate::result::status::SolverStatus::FiniteResourceFailure
    }) {
        return err.clone();
    }
    let failure_hash = hash_sequence(
        "declared-ladder-aggregate-failure",
        &route_failures
            .iter()
            .flat_map(|(kind, plan_hash, err)| {
                [
                    format!("{kind:?}").into_bytes(),
                    plan_hash.0.to_vec(),
                    format!("{:?}", err.public_status()).into_bytes(),
                    format!("{:?}", err.kind).into_bytes(),
                ]
            })
            .collect::<Vec<_>>(),
    );
    let summary = route_failures
        .iter()
        .map(|(kind, plan_hash, err)| {
            format!("{kind:?}:{:?}:plan={plan_hash:?}", err.public_status())
        })
        .collect::<Vec<_>>()
        .join("|");
    algorithmic_hard_case(
        Some(target),
        block_hash,
        &format!(
            "declared production kernel ladder exhausted all routes; failure_hash={failure_hash:?}; failures={summary}"
        ),
    )
}

fn is_declared_route_failure_allowed(
    execution_plan: &KernelExecutionPlan,
    err: &SolverError,
) -> bool {
    let status = err.public_status();
    status != SolverStatus::ImplementationBug
        && execution_plan
            .failure_behavior
            .allowed_statuses
            .contains(&status)
}

fn record_block_projection_failure_trace(
    block: &ProjectionBlock,
    execution_plan: &crate::planner::kernel_plan::KernelExecutionPlan,
    err: &SolverError,
    allowed_to_continue: bool,
    ctx: &mut SolverContext,
) {
    let mut diagnostic = crate::result::diagnostics::DiagnosticRecord::new(
        "BlockProjectionFailureTrace",
        format!(
            "block_id={} kernel={:?} status={:?} allowed_to_continue={}",
            block.block_id.0,
            execution_plan.kernel_kind,
            err.public_status(),
            allowed_to_continue
        ),
        Some(StageId("ExecuteLocalProjectionKernels".to_owned())),
    );
    diagnostic
        .details
        .insert("block_id".to_owned(), block.block_id.0.to_string());
    diagnostic.details.insert(
        "kernel_kind".to_owned(),
        format!("{:?}", execution_plan.kernel_kind),
    );
    diagnostic.details.insert(
        "plan_hash".to_owned(),
        format!("{:?}", execution_plan.plan_hash),
    );
    diagnostic.details.insert(
        "route_budget_hash".to_owned(),
        format!("{:?}", execution_plan.route_budget.budget_hash),
    );
    diagnostic.details.insert(
        "algebraic_work_estimate_hash".to_owned(),
        format!("{:?}", execution_plan.algebraic_work_estimate.estimate_hash),
    );
    diagnostic
        .details
        .insert("status".to_owned(), format!("{:?}", err.public_status()));
    diagnostic.details.insert(
        "allowed_to_continue".to_owned(),
        allowed_to_continue.to_string(),
    );
    diagnostic
        .details
        .insert("error_kind".to_owned(), format!("{:?}", err.kind));
    ctx.diagnostics.push(diagnostic);
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

fn block_requires_projection(block: &ProjectionBlock) -> bool {
    !block.relation_ids.is_empty()
}

fn collect_child_projection_messages(
    block: &ProjectionBlock,
    blocks_by_id: &BTreeMap<BlockId, &ProjectionBlock>,
    messages_by_block: &BTreeMap<BlockId, ProjectionMessage>,
) -> Result<Vec<ProjectionMessage>, SolverError> {
    let mut out = Vec::new();
    for child_id in &block.child_block_ids {
        collect_projection_messages_from_subtree(
            *child_id,
            blocks_by_id,
            messages_by_block,
            &mut out,
        )?;
    }
    Ok(out)
}

fn collect_projection_messages_from_subtree(
    block_id: BlockId,
    blocks_by_id: &BTreeMap<BlockId, &ProjectionBlock>,
    messages_by_block: &BTreeMap<BlockId, ProjectionMessage>,
    out: &mut Vec<ProjectionMessage>,
) -> Result<(), SolverError> {
    let Some(block) = blocks_by_id.get(&block_id) else {
        return Err(implementation_bug(
            "projection block references missing child",
        ));
    };
    if let Some(message) = messages_by_block.get(&block_id) {
        out.push(message.clone());
        return Ok(());
    }
    if block_requires_projection(block) {
        return Err(implementation_bug(
            "projection block executed before all descendant messages were available",
        ));
    }
    for child_id in &block.child_block_ids {
        collect_projection_messages_from_subtree(*child_id, blocks_by_id, messages_by_block, out)?;
    }
    Ok(())
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

fn executed_plan_hashes(plans: &[KernelPlan], messages: &[ProjectionMessage]) -> Vec<Hash> {
    messages
        .iter()
        .map(|message| {
            plans
                .iter()
                .find(|plan| {
                    plan.block_id == message.block_id
                        && plan
                            .declared_ladder
                            .iter()
                            .any(|entry| entry.plan_hash == message.certificate.plan_hash)
                })
                .map(|_| message.certificate.plan_hash)
                .unwrap_or(message.certificate.plan_hash)
        })
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compose::message::{MessageRepresentation, ProjectionMessage, ProjectionStrength};
    use crate::kernels::traits::KernelKind;
    use crate::planner::algebraic_cost::{AlgebraicWorkEstimate, RouteBudget, SaturatingCount};
    use crate::planner::kernel_plan::{
        hash_kernel_execution_plan, planned_failure_behavior, resource_bounds_hash,
        support_plan_hash, CertificateRoute, KernelSupportPlan, LocalNonfinitePolicy,
        PlanWorkClassification, ResourceBounds,
    };
    use crate::preprocess::compression::CompressionTrace;
    use crate::result::cost_trace::ProjectionCostTrace;
    use crate::types::ids::{BlockId, KernelPlanId, PackageId, RelationId};
    use crate::types::polynomial::{constant_poly, poly_add, variable_poly};
    use crate::types::rational::int_q;
    use crate::verify::certificates::KernelCertificate;

    #[test]
    fn acr_p2_route_budget_preflight_stops_over_budget_estimate() {
        let mut plan = budget_test_plan();
        plan.route_budget.max_intermediate_terms = 10;
        plan.route_budget.budget_hash =
            crate::planner::algebraic_cost::route_budget_hash(&plan.route_budget);
        plan.plan_hash = hash_kernel_execution_plan(&plan);

        let err = enforce_route_budget_preflight(&plan).unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::FiniteResourceFailure);
    }

    #[test]
    fn acr_p2_route_budget_postflight_stops_over_budget_output() {
        let mut plan = budget_test_plan();
        plan.route_budget.max_output_terms = 1;
        plan.route_budget.budget_hash =
            crate::planner::algebraic_cost::route_budget_hash(&plan.route_budget);
        plan.plan_hash = hash_kernel_execution_plan(&plan);
        let message = budget_test_message(&plan);

        let err = enforce_route_budget_postflight(&plan, &message).unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::FiniteResourceFailure);
    }

    fn budget_test_plan() -> KernelExecutionPlan {
        let estimate = AlgebraicWorkEstimate::new(
            2,
            1,
            1,
            2,
            2,
            4,
            1,
            Some(2),
            Some(2),
            Some(2),
            Some(SaturatingCount::from_usize(128)),
            Some(SaturatingCount::from_usize(128)),
            Some(SaturatingCount::from_usize(32)),
            SaturatingCount::from_usize(128),
        );
        let mut budget = RouteBudget::from_estimate(&estimate);
        budget.max_output_terms = 8;
        budget.budget_hash = crate::planner::algebraic_cost::route_budget_hash(&budget);
        let mut support_plan = KernelSupportPlan {
            dense_relation_search_schedule: None,
            affine_elimination_order: None,
            template_plan: None,
            rank_plan: None,
            universal_strategy_sequence: Vec::new(),
            degree_bound: 4,
            support_hash: hash_sequence("kernel-support-plan", &[]),
        };
        support_plan.support_hash = support_plan_hash(&support_plan);
        let mut resource_bounds = ResourceBounds {
            max_matrix_rows: Some(2),
            max_matrix_cols: Some(2),
            max_export_degree: Some(4),
            max_multiplier_total_degree: Some(4),
            max_local_elimination_steps: Some(1),
            max_memory_bytes: Some(1024),
            bounds_hash: hash_sequence("planner-resource-bounds", &[]),
        };
        resource_bounds.bounds_hash = resource_bounds_hash(&resource_bounds);
        KernelExecutionPlan::new_with_algebraic_cost(
            KernelPlanId(99),
            BlockId(7),
            KernelKind::SparseResultantProjection,
            hash_sequence("authorization", &[]),
            vec![RelationId(1)],
            vec![hash_sequence("source", &[])],
            Vec::new(),
            Vec::new(),
            vec![VariableId(0)],
            vec![VariableId(1)],
            support_plan,
            resource_bounds,
            estimate,
            budget,
            CertificateRoute::SparseResultantExactVerification,
            planned_failure_behavior(
                vec![SolverStatus::FiniteResourceFailure],
                LocalNonfinitePolicy::NotApplicable,
            ),
            PlanWorkClassification::PurePlan,
        )
    }

    fn budget_test_message(plan: &KernelExecutionPlan) -> ProjectionMessage {
        let relation = poly_add(&variable_poly(VariableId(0)), &constant_poly(int_q(1)));
        let certificate = KernelCertificate::from_execution_plan(
            plan,
            std::slice::from_ref(&relation),
            hash_sequence("cert", &[]),
        );
        let mut message = ProjectionMessage {
            package_id: PackageId(99),
            block_id: plan.block_id,
            kernel_kind: plan.kernel_kind,
            source_relation_ids: plan.source_relation_ids.clone(),
            eliminated_variables: plan.eliminated_variables.clone(),
            exported_variables: plan.exported_variables.clone(),
            relation_generators: vec![relation],
            representation: MessageRepresentation::GeneratorSet,
            projection_strength: ProjectionStrength::CandidateCoverStrong,
            certificate,
            compression_trace: CompressionTrace::default(),
            cost_trace: ProjectionCostTrace {
                block_id: plan.block_id,
                kernel_kind: plan.kernel_kind,
                route_cost: Some(ProjectionCostTrace::route_cost_from_plan(plan)),
                ..ProjectionCostTrace::default()
            },
            package_hash: hash_sequence("projection-message-initial", &[]),
        };
        message.package_hash = crate::compose::message::hash_projection_message(&message);
        message
    }
}
