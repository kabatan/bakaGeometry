use std::collections::BTreeMap;
use std::time::Instant;

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

#[derive(Debug, Clone)]
struct RouteAttemptSummary {
    route_index: usize,
    kernel_kind: crate::kernels::traits::KernelKind,
    plan_hash: Hash,
    event: &'static str,
    status: String,
    allowed_to_continue: bool,
    elapsed_micros: u128,
    predicted_work_units: u128,
    route_budget_max_work_units: u128,
    route_budget_hash: Hash,
    algebraic_work_estimate_hash: Hash,
    attempt_hash: Hash,
}

#[derive(Debug, Clone)]
struct RouteFailureRecord {
    summary: RouteAttemptSummary,
    err: SolverError,
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
    let mut route_summaries = Vec::new();
    for (route_index, execution_plan) in plan.declared_ladder.iter().enumerate() {
        record_block_projection_route_start(block, route_index, execution_plan, ctx);
        let started = Instant::now();
        if let Err(err) = enforce_route_budget_preflight(execution_plan) {
            let allowed = is_declared_route_failure_allowed(execution_plan, &err);
            let summary = route_attempt_summary(
                route_index,
                execution_plan,
                route_failure_event(&err, allowed),
                format!("{:?}", err.public_status()),
                allowed,
                started.elapsed().as_micros(),
            );
            record_block_projection_failure_trace(
                block,
                execution_plan,
                &err,
                allowed,
                &summary,
                ctx,
            );
            route_summaries.push(summary.clone());
            if allowed {
                route_failures.push(RouteFailureRecord { summary, err });
                continue;
            }
            return Err(err);
        }
        let Some(kernel) = kernels
            .iter()
            .find(|kernel| kernel.kind() == execution_plan.kernel_kind)
        else {
            let summary = route_attempt_summary(
                route_index,
                execution_plan,
                "route_missing_kernel",
                "MissingKernelRegistryEntry".to_owned(),
                true,
                started.elapsed().as_micros(),
            );
            route_summaries.push(summary);
            continue;
        };
        let mut kctx = KernelContext {
            block: block.clone(),
            system: compressed.clone(),
            child_messages: child_messages.clone(),
        };
        crate::problem::context::begin_route_budget(
            ctx,
            active_route_budget_from_plan(execution_plan),
        );
        if let Err(err) =
            crate::problem::context::check_resource(ctx, StageId("RouteExecuteStart".to_owned()))
        {
            crate::problem::context::end_route_budget(ctx);
            let allowed = is_declared_route_failure_allowed(execution_plan, &err);
            let summary = route_attempt_summary(
                route_index,
                execution_plan,
                route_failure_event(&err, allowed),
                format!("{:?}", err.public_status()),
                allowed,
                started.elapsed().as_micros(),
            );
            record_block_projection_failure_trace(
                block,
                execution_plan,
                &err,
                allowed,
                &summary,
                ctx,
            );
            route_summaries.push(summary.clone());
            if allowed {
                route_failures.push(RouteFailureRecord { summary, err });
                continue;
            }
            return Err(err);
        }
        let execute_result = kernel.execute(execution_plan, &mut kctx, ctx);
        let _route_meter = crate::problem::context::end_route_budget(ctx);
        match execute_result {
            Ok(message) => match enforce_route_budget_postflight(execution_plan, &message)
                .and_then(|_| verify_projection_message(&message, &kctx))
            {
                Ok(()) => {
                    let summary = route_attempt_summary(
                        route_index,
                        execution_plan,
                        "route_success",
                        "ProjectionMessageVerified".to_owned(),
                        false,
                        started.elapsed().as_micros(),
                    );
                    record_block_projection_success_trace(
                        block,
                        execution_plan,
                        &message,
                        &summary,
                        ctx,
                    );
                    route_summaries.push(summary);
                    return Ok(message);
                }
                Err(err) => {
                    let allowed = is_declared_route_failure_allowed(execution_plan, &err);
                    let summary = route_attempt_summary(
                        route_index,
                        execution_plan,
                        route_failure_event(&err, allowed),
                        format!("{:?}", err.public_status()),
                        allowed,
                        started.elapsed().as_micros(),
                    );
                    record_block_projection_failure_trace(
                        block,
                        execution_plan,
                        &err,
                        allowed,
                        &summary,
                        ctx,
                    );
                    route_summaries.push(summary.clone());
                    if allowed {
                        route_failures.push(RouteFailureRecord { summary, err });
                    } else {
                        return Err(err);
                    }
                }
            },
            Err(err) => {
                let allowed = is_declared_route_failure_allowed(execution_plan, &err);
                let summary = route_attempt_summary(
                    route_index,
                    execution_plan,
                    route_failure_event(&err, allowed),
                    format!("{:?}", err.public_status()),
                    allowed,
                    started.elapsed().as_micros(),
                );
                record_block_projection_failure_trace(
                    block,
                    execution_plan,
                    &err,
                    allowed,
                    &summary,
                    ctx,
                );
                route_summaries.push(summary.clone());
                if allowed {
                    route_failures.push(RouteFailureRecord { summary, err });
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
        &route_summaries,
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

fn active_route_budget_from_plan(
    plan: &KernelExecutionPlan,
) -> crate::problem::context::ActiveRouteBudget {
    crate::problem::context::ActiveRouteBudget {
        block_id: plan.block_id,
        kernel_kind: format!("{:?}", plan.kernel_kind),
        plan_hash: plan.plan_hash,
        route_budget_hash: plan.route_budget.budget_hash,
        algebraic_work_estimate_hash: plan.algebraic_work_estimate.estimate_hash,
        max_elapsed_steps: plan.route_budget.max_elapsed_steps.max(1),
        max_work_units: plan.route_budget.max_work_units.0.max(1),
        consumed_steps: 0,
        consumed_work_units: 0,
    }
}

fn route_attempt_summary(
    route_index: usize,
    plan: &KernelExecutionPlan,
    event: &'static str,
    status: String,
    allowed_to_continue: bool,
    elapsed_micros: u128,
) -> RouteAttemptSummary {
    let predicted_work_units = plan.algebraic_work_estimate.predicted_work_units.0;
    let route_budget_max_work_units = plan.route_budget.max_work_units.0;
    let attempt_hash = hash_sequence(
        "declared-ladder-route-attempt",
        &[
            route_index.to_be_bytes().to_vec(),
            format!("{:?}", plan.kernel_kind).into_bytes(),
            plan.plan_hash.0.to_vec(),
            event.as_bytes().to_vec(),
            status.as_bytes().to_vec(),
            allowed_to_continue.to_string().into_bytes(),
            elapsed_micros.to_be_bytes().to_vec(),
            predicted_work_units.to_be_bytes().to_vec(),
            route_budget_max_work_units.to_be_bytes().to_vec(),
            plan.route_budget.budget_hash.0.to_vec(),
            plan.algebraic_work_estimate.estimate_hash.0.to_vec(),
        ],
    );
    RouteAttemptSummary {
        route_index,
        kernel_kind: plan.kernel_kind,
        plan_hash: plan.plan_hash,
        event,
        status,
        allowed_to_continue,
        elapsed_micros,
        predicted_work_units,
        route_budget_max_work_units,
        route_budget_hash: plan.route_budget.budget_hash,
        algebraic_work_estimate_hash: plan.algebraic_work_estimate.estimate_hash,
        attempt_hash,
    }
}

fn route_failure_event(err: &SolverError, allowed_to_continue: bool) -> &'static str {
    if err.public_status() == SolverStatus::FiniteResourceFailure {
        "route_budget_stop"
    } else if allowed_to_continue {
        "route_allowed_failure"
    } else {
        "route_blocking_failure"
    }
}

fn aggregate_ladder_failure(
    target: VariableId,
    block_hash: Hash,
    route_failures: &[RouteFailureRecord],
    route_summaries: &[RouteAttemptSummary],
) -> SolverError {
    let all_attempts_summary = route_summaries
        .iter()
        .map(format_route_attempt_summary)
        .collect::<Vec<_>>()
        .join("|");
    if route_failures.is_empty() {
        return algorithmic_hard_case(
            Some(target),
            block_hash,
            &format!(
                "declared production kernel ladder produced no projection message and no route failure; all_attempts={all_attempts_summary}"
            ),
        );
    }
    if let Some(record) = route_failures
        .iter()
        .find(|record| record.err.public_status() == SolverStatus::FiniteResourceFailure)
    {
        let mut err = record.err.clone();
        if let SolverErrorKind::Failure(FailureKind::FiniteResourceFailure { stage, .. }) =
            &mut err.kind
        {
            stage.0 = format!("{}; ladder_attempts={}", stage.0, all_attempts_summary);
        }
        return err;
    }
    let failure_hash = hash_sequence(
        "declared-ladder-aggregate-failure",
        &route_failures
            .iter()
            .flat_map(|record| {
                [
                    format!("{:?}", record.summary.kernel_kind).into_bytes(),
                    record.summary.plan_hash.0.to_vec(),
                    format!("{:?}", record.err.public_status()).into_bytes(),
                    format!("{:?}", record.err.kind).into_bytes(),
                    record.summary.attempt_hash.0.to_vec(),
                ]
            })
            .collect::<Vec<_>>(),
    );
    let summary = route_failures
        .iter()
        .map(|record| format_route_attempt_summary(&record.summary))
        .collect::<Vec<_>>()
        .join("|");
    algorithmic_hard_case(
        Some(target),
        block_hash,
        &format!(
            "declared production kernel ladder exhausted all routes; failure_hash={failure_hash:?}; failures={summary}; all_attempts={all_attempts_summary}"
        ),
    )
}

fn format_route_attempt_summary(summary: &RouteAttemptSummary) -> String {
    format!(
        "idx={} kernel={:?} event={} status={} allowed={} elapsed_us={} work={}/{} plan={:?} attempt={:?}",
        summary.route_index,
        summary.kernel_kind,
        summary.event,
        summary.status,
        summary.allowed_to_continue,
        summary.elapsed_micros,
        summary.predicted_work_units,
        summary.route_budget_max_work_units,
        summary.plan_hash,
        summary.attempt_hash
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
    summary: &RouteAttemptSummary,
    ctx: &mut SolverContext,
) {
    let mut diagnostic = crate::result::diagnostics::DiagnosticRecord::new(
        "BlockProjectionFailureTrace",
        format!(
            "block_id={} kernel={:?} event={} status={:?} allowed_to_continue={}",
            block.block_id.0,
            execution_plan.kernel_kind,
            summary.event,
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
    insert_route_attempt_summary_details(&mut diagnostic, summary);
    ctx.diagnostics.push(diagnostic);
}

fn record_block_projection_route_start(
    block: &ProjectionBlock,
    route_index: usize,
    execution_plan: &crate::planner::kernel_plan::KernelExecutionPlan,
    ctx: &mut SolverContext,
) {
    let summary = route_attempt_summary(
        route_index,
        execution_plan,
        "route_start",
        "RouteStarted".to_owned(),
        false,
        0,
    );
    let mut diagnostic = crate::result::diagnostics::DiagnosticRecord::new(
        "BlockProjectionRouteStart",
        format!(
            "block_id={} route_index={} kernel={:?}",
            block.block_id.0, route_index, execution_plan.kernel_kind
        ),
        Some(StageId("ExecuteLocalProjectionKernels".to_owned())),
    );
    diagnostic
        .details
        .insert("block_id".to_owned(), block.block_id.0.to_string());
    diagnostic
        .details
        .insert("route_index".to_owned(), route_index.to_string());
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
    diagnostic.details.insert(
        "route_budget_max_elapsed_steps".to_owned(),
        execution_plan.route_budget.max_elapsed_steps.to_string(),
    );
    insert_route_attempt_summary_details(&mut diagnostic, &summary);
    ctx.diagnostics.push(diagnostic);
}

fn record_block_projection_success_trace(
    block: &ProjectionBlock,
    execution_plan: &crate::planner::kernel_plan::KernelExecutionPlan,
    message: &ProjectionMessage,
    summary: &RouteAttemptSummary,
    ctx: &mut SolverContext,
) {
    let mut diagnostic = crate::result::diagnostics::DiagnosticRecord::new(
        "BlockProjectionRouteSuccess",
        format!(
            "block_id={} kernel={:?} event={} package={:?}",
            block.block_id.0, execution_plan.kernel_kind, summary.event, message.package_hash
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
        "package_hash".to_owned(),
        format!("{:?}", message.package_hash),
    );
    diagnostic.details.insert(
        "output_relation_count".to_owned(),
        message.relation_generators.len().to_string(),
    );
    diagnostic.details.insert(
        "output_term_count".to_owned(),
        message
            .relation_generators
            .iter()
            .map(poly_monomial_count)
            .sum::<usize>()
            .to_string(),
    );
    insert_route_attempt_summary_details(&mut diagnostic, summary);
    ctx.diagnostics.push(diagnostic);
}

fn insert_route_attempt_summary_details(
    diagnostic: &mut crate::result::diagnostics::DiagnosticRecord,
    summary: &RouteAttemptSummary,
) {
    diagnostic
        .details
        .insert("route_index".to_owned(), summary.route_index.to_string());
    diagnostic
        .details
        .insert("route_event".to_owned(), summary.event.to_owned());
    diagnostic
        .details
        .insert("route_status".to_owned(), summary.status.clone());
    diagnostic.details.insert(
        "allowed_to_continue".to_owned(),
        summary.allowed_to_continue.to_string(),
    );
    diagnostic.details.insert(
        "elapsed_micros".to_owned(),
        summary.elapsed_micros.to_string(),
    );
    diagnostic.details.insert(
        "predicted_work_units".to_owned(),
        summary.predicted_work_units.to_string(),
    );
    diagnostic.details.insert(
        "route_budget_max_work_units".to_owned(),
        summary.route_budget_max_work_units.to_string(),
    );
    diagnostic.details.insert(
        "route_budget_hash".to_owned(),
        format!("{:?}", summary.route_budget_hash),
    );
    diagnostic.details.insert(
        "algebraic_work_estimate_hash".to_owned(),
        format!("{:?}", summary.algebraic_work_estimate_hash),
    );
    diagnostic.details.insert(
        "route_attempt_hash".to_owned(),
        format!("{:?}", summary.attempt_hash),
    );
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
    use crate::graph::hypergraph::build_relation_variable_hypergraph;
    use crate::graph::influence::build_target_influence_graph;
    use crate::graph::projection_dag::build_target_projection_dag;
    use crate::graph::separators::CostModel;
    use crate::graph::tree_decomposition::build_target_rooted_decomposition;
    use crate::graph::weighted_primal::build_weighted_primal_graph;
    use crate::kernels::traits::KernelKind;
    use crate::planner::algebraic_cost::{
        algebraic_work_estimate_hash, AlgebraicWorkEstimate, RouteBudget, SaturatingCount,
    };
    use crate::planner::kernel_plan::KernelPlan;
    use crate::planner::kernel_plan::{
        hash_kernel_execution_plan, planned_failure_behavior, resource_bounds_hash,
        support_plan_hash, CertificateRoute, KernelSupportPlan, LocalNonfinitePolicy,
        PlanWorkClassification, ResourceBounds,
    };
    use crate::planner::planner::plan_all_blocks;
    use crate::preprocess::compression::CompressionState;
    use crate::preprocess::compression::CompressionTrace;
    use crate::problem::canonicalize::canonicalize_system;
    use crate::problem::context::new_context;
    use crate::problem::input::make_problem;
    use crate::problem::input::RationalTargetProblem;
    use crate::problem::validate::validate_input;
    use crate::result::cost_trace::ProjectionCostTrace;
    use crate::result::output::TargetSolveResult;
    use crate::solver::options::SolverOptions;
    use crate::types::ids::{BlockId, KernelPlanId, PackageId, RelationId, VariableId};
    use crate::types::polynomial::{constant_poly, poly_add, poly_mul, poly_sub, variable_poly};
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

    #[test]
    fn acr_p4_declared_ladder_continues_after_sparse_resultant_guard_failure() {
        let (compressed, dag) = continuation_case();
        let block = dag.blocks[0].clone();
        let mut ctx = new_context(SolverOptions::default());
        let planned = plan_all_blocks(&dag, &compressed, &mut ctx)
            .unwrap()
            .remove(0);
        let sparse = planned
            .declared_ladder
            .iter()
            .find(|entry| entry.kernel_kind == KernelKind::SparseResultantProjection)
            .cloned()
            .expect("sparse resultant route should be declared");
        let mut sparse_guard_fail = sparse;
        sparse_guard_fail
            .algebraic_work_estimate
            .predicted_output_terms = Some(SaturatingCount::ONE);
        sparse_guard_fail.algebraic_work_estimate.estimate_hash =
            algebraic_work_estimate_hash(&sparse_guard_fail.algebraic_work_estimate);
        sparse_guard_fail.route_budget.max_output_terms = 1;
        sparse_guard_fail.route_budget.budget_hash =
            crate::planner::algebraic_cost::route_budget_hash(&sparse_guard_fail.route_budget);
        sparse_guard_fail.plan_hash = hash_kernel_execution_plan(&sparse_guard_fail);
        let mut ladder = vec![sparse_guard_fail];
        ladder.extend(
            planned
                .declared_ladder
                .into_iter()
                .filter(|entry| entry.kernel_kind != KernelKind::SparseResultantProjection),
        );
        let plan = KernelPlan::new(
            planned.block_id,
            ladder,
            planned.admissions,
            planned.cost_estimates,
        )
        .unwrap();

        let message =
            execute_block_with_declared_ladder(&block, &plan, Vec::new(), &compressed, &mut ctx)
                .unwrap();

        assert_ne!(message.kernel_kind, KernelKind::SparseResultantProjection);
        assert!(ctx.diagnostics.iter().any(|record| {
            record.name == "BlockProjectionFailureTrace"
                && record.details.get("kernel_kind")
                    == Some(&"SparseResultantProjection".to_owned())
                && record.details.get("status") == Some(&"FiniteResourceFailure".to_owned())
                && record.details.get("allowed_to_continue") == Some(&"true".to_owned())
                && record
                    .details
                    .get("error_kind")
                    .map(|kind| kind.contains("route_trace_hash="))
                    .unwrap_or(false)
        }));
    }

    #[test]
    fn acr_p5_near_public_pipeline_budget_stop_yields_to_second_route_candidate_cover() {
        let result = p5_pipeline_result_after_first_plan_mutation(
            force_sparse_resultant_work_budget_stop_first,
        );

        assert_eq!(result.status, SolverStatus::CertifiedCandidateCover);
        assert!(result
            .projection_messages
            .iter()
            .any(|message| { message.kernel_kind != KernelKind::SparseResultantProjection }));
        assert_route_trace_event(
            &result,
            "BlockProjectionRouteStart",
            "SparseResultantProjection",
            None,
        );
        assert_route_trace_event(
            &result,
            "BlockProjectionFailureTrace",
            "SparseResultantProjection",
            Some("route_budget_stop"),
        );
        assert!(result.diagnostics.iter().any(|record| {
            record.name == "BlockProjectionRouteSuccess"
                && record
                    .details
                    .get("route_event")
                    .is_some_and(|event| event == "route_success")
                && record
                    .details
                    .get("route_index")
                    .and_then(|index| index.parse::<usize>().ok())
                    .is_some_and(|index| index > 0)
        }));
    }

    #[test]
    fn acr_p5_near_public_pipeline_inflight_budget_stop_yields_to_second_route_candidate_cover() {
        let result = p5_pipeline_result_after_first_plan_mutation(
            force_sparse_resultant_elapsed_budget_stop_first,
        );

        assert_eq!(result.status, SolverStatus::CertifiedCandidateCover);
        assert!(result
            .projection_messages
            .iter()
            .any(|message| { message.kernel_kind != KernelKind::SparseResultantProjection }));
        assert_route_trace_event(
            &result,
            "BlockProjectionFailureTrace",
            "SparseResultantProjection",
            Some("route_budget_stop"),
        );
        let failure = result
            .diagnostics
            .iter()
            .find(|record| {
                record.name == "BlockProjectionFailureTrace"
                    && record.details.get("kernel_kind")
                        == Some(&"SparseResultantProjection".to_owned())
                    && record.details.get("route_event") == Some(&"route_budget_stop".to_owned())
                    && record.details.get("error_kind").is_some_and(|kind| {
                        kind.contains("cooperative checkpoint exceeded route budget")
                            && kind.contains("SparseResultant::execute_start")
                    })
            })
            .expect("expected in-flight route budget failure trace");
        assert_eq!(
            failure.details.get("allowed_to_continue"),
            Some(&"true".to_owned())
        );
        assert!(result.diagnostics.iter().any(|record| {
            record.name == "BlockProjectionRouteSuccess"
                && record
                    .details
                    .get("route_index")
                    .and_then(|index| index.parse::<usize>().ok())
                    .is_some_and(|index| index > 0)
        }));
    }

    #[test]
    fn acr_p5_aggregate_no_failure_path_preserves_attempt_summaries() {
        let plan = budget_test_plan();
        let summary = route_attempt_summary(
            0,
            &plan,
            "route_missing_kernel",
            "MissingKernelRegistryEntry".to_owned(),
            true,
            17,
        );

        let err = aggregate_ladder_failure(
            VariableId(0),
            hash_sequence("acr-p5-empty-aggregate-block", &[]),
            &[],
            &[summary],
        );

        let SolverErrorKind::Failure(FailureKind::AlgorithmicHardCase { reason, .. }) = err.kind
        else {
            panic!("expected algorithmic hardcase");
        };
        assert!(reason
            .0
            .contains("no projection message and no route failure"));
        assert!(reason.0.contains("all_attempts="));
        assert!(reason.0.contains("route_missing_kernel"));
        assert!(reason.0.contains("MissingKernelRegistryEntry"));
        assert!(reason.0.contains("attempt=Hash"));
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

    fn continuation_case() -> (
        crate::preprocess::compression::CompressedSystemQ,
        crate::graph::projection_dag::TargetProjectionDAG,
    ) {
        let problem = continuation_problem();
        let t = problem.target;
        let canonical = canonicalize_system(validate_input(problem).unwrap()).unwrap();
        let compressed = CompressionState::from_system(canonical).to_compressed_system();
        let h = build_relation_variable_hypergraph(&compressed);
        let influence = build_target_influence_graph(&h, t);
        let g = build_weighted_primal_graph(&compressed, &influence);
        let tree = build_target_rooted_decomposition(&g, t, &CostModel::default());
        let dag = build_target_projection_dag(&compressed, &influence, &tree).unwrap();
        (compressed, dag)
    }

    fn continuation_problem() -> RationalTargetProblem {
        let t = VariableId(0);
        let y = VariableId(1);
        let relations = vec![
            poly_sub(&variable_poly(y), &variable_poly(t)),
            poly_sub(&variable_poly(y), &constant_poly(int_q(1))),
            poly_sub(&variable_poly(t), &constant_poly(int_q(2))),
        ];
        make_problem(vec![t, y], t, relations, Vec::new())
    }

    fn p5_budget_yield_problem() -> RationalTargetProblem {
        let t = VariableId(0);
        let x = VariableId(1);
        let y = VariableId(2);
        let relations = vec![
            poly_sub(
                &poly_mul(&variable_poly(x), &variable_poly(x)),
                &variable_poly(y),
            ),
            poly_sub(&variable_poly(y), &variable_poly(t)),
            poly_sub(
                &poly_mul(
                    &poly_mul(&variable_poly(x), &variable_poly(x)),
                    &poly_mul(&variable_poly(x), &variable_poly(x)),
                ),
                &constant_poly(int_q(2)),
            ),
        ];
        make_problem(vec![t, x, y], t, relations, Vec::new())
    }

    fn p5_pipeline_result_after_first_plan_mutation(
        mut mutate_first_plan: impl FnMut(&mut KernelPlan),
    ) -> TargetSolveResult {
        let problem = p5_budget_yield_problem();
        let target = problem.target;
        let mut ctx = new_context(SolverOptions::default());
        let validated = step_validate(problem.clone(), &mut ctx).unwrap();
        let canonical = step_canonicalize(validated, &mut ctx).unwrap();
        let compressed = step_compress(canonical.clone(), &mut ctx).unwrap();
        let graphs = step_build_graphs(&compressed, &mut ctx).unwrap();
        let dag = step_build_dag(&graphs, &compressed, &mut ctx).unwrap();
        let mut plans = step_plan(&dag, &compressed, &mut ctx).unwrap();
        mutate_first_plan(&mut plans[0]);

        let messages = step_execute(&dag, &plans, &compressed, &mut ctx).unwrap();
        step_verify_messages(&dag, &messages, &compressed).unwrap();
        let composed = step_compose(&dag, messages.clone(), target, &mut ctx).unwrap();
        let support = match step_support(&composed, &compressed, target, &mut ctx).unwrap() {
            crate::compose::final_support::FinalSupportComputation::Support(support) => support,
            crate::compose::final_support::FinalSupportComputation::CertifiedNonFinite(_) => {
                panic!("expected finite support")
            }
        };
        let support_certificate =
            crate::verify::verify_support::verify_global_support(&support, &composed).unwrap();
        let roots = step_roots(&support, target, &mut ctx).unwrap();
        let certificate = step_core_certificate(
            &problem,
            &canonical,
            &compressed,
            &graphs,
            &dag,
            &plans,
            &messages,
            Some(&support),
            &roots,
            None,
            Some(&support_certificate),
        );
        let cost_trace = step_cost_trace(
            &compressed,
            &dag,
            &messages,
            Some(&composed),
            Some(&support),
            Some(&certificate),
        );
        let mut diagnostics = compressed.diagnostics.clone();
        diagnostics.extend(ctx.diagnostics.clone());
        TargetSolveResult {
            status: SolverStatus::CertifiedCandidateCover,
            target,
            support_polynomial: Some(support),
            squarefree_support_polynomial: Some(roots.squarefree_support),
            root_isolation: roots.root_isolation,
            decoded_candidates: roots.decoded_candidates,
            projection_messages: messages,
            certificate: Some(certificate),
            exact_image_certificate: None,
            nonfinite_certificate: None,
            diagnostics,
            cost_trace,
        }
    }

    fn force_sparse_resultant_work_budget_stop_first(plan: &mut KernelPlan) {
        let sparse = plan
            .declared_ladder
            .iter()
            .find(|entry| entry.kernel_kind == KernelKind::SparseResultantProjection)
            .cloned()
            .expect("sparse resultant route should be declared");
        let mut sparse_budget_stop = sparse;
        sparse_budget_stop.route_budget.max_work_units = SaturatingCount::ONE;
        sparse_budget_stop.route_budget.budget_hash =
            crate::planner::algebraic_cost::route_budget_hash(&sparse_budget_stop.route_budget);
        sparse_budget_stop.plan_hash = hash_kernel_execution_plan(&sparse_budget_stop);
        let mut ladder = vec![sparse_budget_stop];
        ladder.extend(
            plan.declared_ladder
                .iter()
                .filter(|entry| entry.kernel_kind != KernelKind::SparseResultantProjection)
                .cloned(),
        );
        let rebuilt = KernelPlan::new(
            plan.block_id,
            ladder,
            plan.admissions.clone(),
            plan.cost_estimates.clone(),
        )
        .unwrap();
        *plan = rebuilt;
    }

    fn force_sparse_resultant_elapsed_budget_stop_first(plan: &mut KernelPlan) {
        let sparse = plan
            .declared_ladder
            .iter()
            .find(|entry| entry.kernel_kind == KernelKind::SparseResultantProjection)
            .cloned()
            .expect("sparse resultant route should be declared");
        let mut sparse_budget_stop = sparse;
        sparse_budget_stop.route_budget.max_elapsed_steps = 1;
        sparse_budget_stop.route_budget.budget_hash =
            crate::planner::algebraic_cost::route_budget_hash(&sparse_budget_stop.route_budget);
        sparse_budget_stop.plan_hash = hash_kernel_execution_plan(&sparse_budget_stop);
        let mut ladder = vec![sparse_budget_stop];
        ladder.extend(
            plan.declared_ladder
                .iter()
                .filter(|entry| entry.kernel_kind != KernelKind::SparseResultantProjection)
                .cloned(),
        );
        let rebuilt = KernelPlan::new(
            plan.block_id,
            ladder,
            plan.admissions.clone(),
            plan.cost_estimates.clone(),
        )
        .unwrap();
        *plan = rebuilt;
    }

    fn assert_route_trace_event(
        result: &TargetSolveResult,
        diagnostic_name: &str,
        kernel_kind: &str,
        route_event: Option<&str>,
    ) {
        assert!(result.diagnostics.iter().any(|record| {
            record.name == diagnostic_name
                && record.details.get("kernel_kind") == Some(&kernel_kind.to_owned())
                && match route_event {
                    Some(expected) => {
                        record.details.get("route_event") == Some(&expected.to_owned())
                    }
                    None => true,
                }
                && record.details.contains_key("elapsed_micros")
                && record.details.contains_key("predicted_work_units")
                && record.details.contains_key("route_budget_max_work_units")
                && record.details.contains_key("route_attempt_hash")
        }));
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
