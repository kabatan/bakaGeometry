use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::algebra::elimination::{
    eliminate_to_keep_variables, validate_local_elimination_result, EliminationStrategy,
    LocalEliminationResult,
};
use crate::algebra::groebner::GroebnerOptions;
use crate::compose::message::{MessageRepresentation, ProjectionMessage, ProjectionStrength};
use crate::graph::projection_dag::ProjectionBlock;
use crate::kernels::action_krylov::{
    execute_target_action_krylov, plan_target_action_krylov_with_messages,
};
use crate::kernels::norm_trace_projection::{
    execute_norm_trace_projection, plan_norm_trace_projection,
};
use crate::kernels::regular_chain_projection::{
    execute_regular_chain_projection, plan_regular_chain_projection,
};
use crate::kernels::sparse_resultant::{
    execute_sparse_resultant, plan_sparse_resultant_with_messages,
};
use crate::kernels::specialization_interpolation::{
    execute_specialization_interpolation, plan_specialization_interpolation_with_messages,
};
use crate::kernels::target_relation_search::{
    admit_target_relation_search, execute_target_relation_search,
};
use crate::kernels::traits::{KernelContext, KernelKind, ReplayResult, TargetProjectionKernel};
use crate::planner::admission::{KernelAdmission, KernelAdmissionStatus};
use crate::planner::algebraic_cost::{AlgebraicWorkEstimate, RouteBudget};
use crate::planner::cost_model::{classify_route_cost, estimate_kernel_cost, RouteCostClass};
use crate::planner::kernel_plan::{
    hash_kernel_execution_plan, planned_failure_behavior, rank_plan, resource_bounds_hash,
    support_plan_hash, template_plan, universal_strategy_step_with_cost, CertificateRoute,
    KernelExecutionPlan, KernelSupportPlan, LocalNonfinitePolicy, ResourceBounds,
    UniversalStrategy, UniversalStrategyPlanStep,
};
use crate::planner::probes::run_cost_probes;
use crate::preprocess::compression::CompressedSystemQ;
use crate::problem::canonicalize::CanonicalRelationQ;
use crate::problem::context::SolverContext;
use crate::result::cost_trace::ProjectionCostTrace;
use crate::result::status::{
    AlgebraicReason, FailureKind, SolverError, SolverErrorKind, SolverStatus, StageId,
};
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::{BlockId, KernelPlanId, PackageId, RelationId, VariableId};
use crate::types::matrix::{matrix_density, SparseMatrixQ};
use crate::types::polynomial::{
    max_poly_coefficient_height_bits, poly_monomial_count, poly_variables, SparsePolynomialQ,
};
use crate::verify::certificates::{
    KernelCertificate, KernelCertificatePayload, UniversalProjectionCertificate,
    UniversalStrategyTraceRecord,
};

pub struct UniversalTargetEliminationKernel;

impl TargetProjectionKernel for UniversalTargetEliminationKernel {
    fn kind(&self) -> KernelKind {
        KernelKind::UniversalTargetElimination
    }

    fn admit(&self, block: &ProjectionBlock, ctx: &KernelContext) -> KernelAdmission {
        admit_universal_elimination(block, ctx)
    }

    fn plan(
        &self,
        admission: &KernelAdmission,
        ctx: &KernelContext,
        _solver_ctx: &SolverContext,
    ) -> Result<KernelExecutionPlan, SolverError> {
        build_universal_elimination_plan(admission, ctx)
    }

    fn execute(
        &self,
        plan: &KernelExecutionPlan,
        ctx: &mut KernelContext,
        solver_ctx: &mut SolverContext,
    ) -> Result<ProjectionMessage, SolverError> {
        execute_universal_elimination_with_solver_ctx(plan, ctx, solver_ctx)
    }

    fn replay(&self, message: &ProjectionMessage, ctx: &KernelContext) -> ReplayResult {
        crate::kernels::traits::exact_replay_result(
            self.kind(),
            "universal-elimination-replay",
            message,
            ctx,
        )
    }
}

#[derive(Debug, Clone)]
struct UniversalRelationInput {
    polynomial: SparsePolynomialQ,
    source_relation_ids: Vec<RelationId>,
    source_hash: Hash,
    child_message_hash: Option<Hash>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UniversalStagePlan {
    pub parent_plan_id: KernelPlanId,
    pub parent_plan_hash: Hash,
    pub block_id: BlockId,
    pub strategy: UniversalStrategy,
    pub stage_index: usize,
    pub exported_variables: Vec<VariableId>,
    pub eliminated_variables: Vec<VariableId>,
    pub source_relation_hashes: Vec<Hash>,
    pub child_message_hashes: Vec<Hash>,
    pub resource_bounds: ResourceBounds,
    pub enabled: bool,
    pub skip_reason: Option<String>,
    pub cost_class: RouteCostClass,
    pub algebraic_work_estimate: AlgebraicWorkEstimate,
    pub route_budget: RouteBudget,
    pub stage_hash: Hash,
}

pub fn admit_universal_elimination(
    block: &ProjectionBlock,
    ctx: &KernelContext,
) -> KernelAdmission {
    let solver_ctx = SolverContext::new(Default::default());
    match plan_universal_elimination_with_messages(
        block,
        &ctx.system,
        &ctx.child_messages,
        &solver_ctx,
        KernelPlanId(KernelKind::UniversalTargetElimination as u32),
    ) {
        Ok(plan) => finish_admission(block, KernelAdmissionStatus::Admitted, Some(plan)),
        Err(_) => finish_admission(
            block,
            KernelAdmissionStatus::Declined {
                reason: "no authorized local relations or child messages for universal projection"
                    .to_owned(),
            },
            None,
        ),
    }
}

pub fn plan_universal_elimination(
    block: &ProjectionBlock,
    system: &CompressedSystemQ,
    solver_ctx: &SolverContext,
    plan_id: KernelPlanId,
) -> Result<KernelExecutionPlan, SolverError> {
    plan_universal_elimination_with_messages(block, system, &[], solver_ctx, plan_id)
}

pub fn plan_universal_elimination_with_messages(
    block: &ProjectionBlock,
    system: &CompressedSystemQ,
    child_messages: &[ProjectionMessage],
    solver_ctx: &SolverContext,
    plan_id: KernelPlanId,
) -> Result<KernelExecutionPlan, SolverError> {
    let inputs = collect_relation_inputs(block, system, child_messages);
    if inputs.is_empty() {
        return Err(algorithmic_hard_case_for_block(
            system.target,
            block,
            "universal projection has no authorized local relations or child messages",
        ));
    }
    let relation_polys = inputs
        .iter()
        .map(|input| input.polynomial.clone())
        .collect::<Vec<_>>();
    let exported_variables = sorted_set(&block.exported_variables);
    let eliminated_variables = block
        .local_variables
        .difference(&block.exported_variables)
        .copied()
        .collect::<Vec<_>>();
    let strategy_sequence = fixed_universal_strategy_sequence(
        block,
        system,
        child_messages,
        solver_ctx,
        &relation_polys,
        &exported_variables,
        system.target,
    );
    let matrix_rows = relation_polys.len().max(1);
    let matrix_cols = relation_polys
        .iter()
        .map(poly_monomial_count)
        .sum::<usize>()
        .max(1);
    let mut support_plan = KernelSupportPlan {
        dense_relation_search_schedule: None,
        sparse_relation_search_schedule: None,
        affine_elimination_order: None,
        template_plan: Some(template_plan(
            matrix_rows,
            matrix_cols,
            hash_sequence(
                "universal-local-relation-rows",
                &relation_polys
                    .iter()
                    .map(|poly| poly.hash.0.to_vec())
                    .collect::<Vec<_>>(),
            ),
            hash_sequence(
                "universal-exported-variables",
                &exported_variables
                    .iter()
                    .map(|var| var.0.to_be_bytes().to_vec())
                    .collect::<Vec<_>>(),
            ),
        )),
        rank_plan: Some(rank_plan(matrix_rows.min(matrix_cols))),
        universal_strategy_sequence: strategy_sequence,
        degree_bound: relation_polys
            .iter()
            .map(crate::types::polynomial::poly_total_degree)
            .max()
            .unwrap_or(1) as usize,
        support_hash: hash_sequence("kernel-support-plan", &[]),
    };
    support_plan.support_hash = support_plan_hash(&support_plan);
    let mut bounds = ResourceBounds {
        max_matrix_rows: solver_ctx
            .options
            .max_matrix_rows
            .or(Some(matrix_rows.max(32))),
        max_matrix_cols: solver_ctx
            .options
            .max_matrix_cols
            .or(Some(matrix_cols.max(32))),
        max_export_degree: solver_ctx
            .options
            .max_relation_search_export_degree
            .or(Some(support_plan.degree_bound.max(2))),
        max_multiplier_total_degree: Some(
            support_plan
                .degree_bound
                .saturating_add(eliminated_variables.len())
                .max(2),
        ),
        max_local_elimination_steps: Some(matrix_rows.saturating_mul(matrix_cols).max(32)),
        max_memory_bytes: solver_ctx.options.max_memory_bytes,
        bounds_hash: hash_sequence("planner-resource-bounds", &[]),
    };
    bounds.bounds_hash = resource_bounds_hash(&bounds);
    Ok(KernelExecutionPlan::new(
        plan_id,
        block.block_id,
        KernelKind::UniversalTargetElimination,
        block.authorization_hash,
        inputs
            .iter()
            .flat_map(|input| input.source_relation_ids.iter().copied())
            .collect(),
        dedup_hashes_in_order(inputs.iter().map(|input| input.source_hash).collect()),
        block.child_block_ids.clone(),
        dedup_hashes_in_order(
            inputs
                .iter()
                .filter_map(|input| input.child_message_hash)
                .collect(),
        ),
        exported_variables,
        eliminated_variables,
        support_plan,
        bounds,
        CertificateRoute::UniversalFixedLocalElimination,
        planned_failure_behavior(
            vec![
                SolverStatus::AlgorithmicHardCase,
                SolverStatus::FiniteResourceFailure,
                SolverStatus::CertificateDesignGap,
            ],
            LocalNonfinitePolicy::NoLocalCertifiedNonFinite,
        ),
    ))
}

pub fn build_universal_elimination_plan(
    admission: &KernelAdmission,
    ctx: &KernelContext,
) -> Result<KernelExecutionPlan, SolverError> {
    if admission.kind != KernelKind::UniversalTargetElimination
        || !matches!(admission.status, KernelAdmissionStatus::Admitted)
    {
        return Err(implementation_bug(
            "universal plan requested for non-admitted kernel",
        ));
    }
    if let Some(plan) = &admission.execution_plan {
        return Ok(plan.clone());
    }
    plan_universal_elimination_with_messages(
        &ctx.block,
        &ctx.system,
        &ctx.child_messages,
        &SolverContext::new(Default::default()),
        KernelPlanId(KernelKind::UniversalTargetElimination as u32),
    )
}

pub fn execute_universal_elimination(
    plan: &KernelExecutionPlan,
    ctx: &mut KernelContext,
) -> Result<ProjectionMessage, SolverError> {
    execute_universal_elimination_with_solver_ctx(
        plan,
        ctx,
        &mut SolverContext::new(Default::default()),
    )
}

fn execute_universal_elimination_with_solver_ctx(
    plan: &KernelExecutionPlan,
    ctx: &mut KernelContext,
    solver_ctx: &mut SolverContext,
) -> Result<ProjectionMessage, SolverError> {
    crate::problem::context::check_resource(
        solver_ctx,
        StageId("UniversalTargetElimination::execute_start".to_owned()),
    )?;
    validate_universal_plan_binding(plan, ctx)?;
    let inputs = planned_relation_inputs(plan, ctx)?;
    if inputs.is_empty() {
        return Err(algorithmic_hard_case(
            ctx,
            "universal projection has no planned authorized relations",
            &[],
        ));
    }
    let stages = build_stage_plans(plan)?;
    let mut stage_trace_hashes = Vec::new();
    let mut executed_failed_strategy_hashes = Vec::new();
    for (stage_index, stage) in stages.into_iter().enumerate() {
        crate::problem::context::check_resource(
            solver_ctx,
            StageId(format!(
                "UniversalTargetElimination::stage::{stage_index}::{:?}",
                stage.strategy
            )),
        )?;
        if !stage.enabled {
            stage_trace_hashes.push(stage.stage_hash);
            continue;
        }
        match execute_universal_stage_with_solver_ctx(
            &stage,
            ctx,
            solver_ctx,
            plan,
            &stage_trace_hashes,
            &executed_failed_strategy_hashes,
        ) {
            Ok(message) => return Ok(message),
            Err(err) if is_continuable_stage_failure(&err) => {
                stage_trace_hashes.push(stage.stage_hash);
                if stage.enabled && stage.cost_class != RouteCostClass::CostProhibited {
                    executed_failed_strategy_hashes.push(stage.stage_hash);
                }
            }
            Err(err) => return Err(err),
        }
    }
    Err(algorithmic_hard_case(
        ctx,
        "universal fixed local sequence produced no certified exported relation",
        &stage_trace_hashes,
    ))
}

pub fn execute_universal_stage(
    stage: &UniversalStagePlan,
    ctx: &mut KernelContext,
) -> Result<ProjectionMessage, SolverError> {
    let shadow = stage_execution_plan_shadow(stage);
    execute_universal_stage_with_solver_ctx(
        stage,
        ctx,
        &mut SolverContext::new(Default::default()),
        &shadow,
        &[],
        &[],
    )
}

fn execute_universal_stage_with_solver_ctx(
    stage: &UniversalStagePlan,
    ctx: &mut KernelContext,
    solver_ctx: &mut SolverContext,
    parent_plan: &KernelExecutionPlan,
    failed_strategy_hashes: &[Hash],
    executed_failed_strategy_hashes: &[Hash],
) -> Result<ProjectionMessage, SolverError> {
    if !stage.enabled {
        return Err(algorithmic_hard_case(
            ctx,
            stage
                .skip_reason
                .as_deref()
                .unwrap_or("universal stage disabled"),
            &[stage.stage_hash],
        ));
    }
    if stage.cost_class == RouteCostClass::CostProhibited {
        return Err(implementation_bug(
            "cost-prohibited universal internal stage reached execution",
        ));
    }
    enforce_universal_stage_route_budget(stage)?;
    match stage.strategy {
        UniversalStrategy::TargetRelationSearchEscalated => {
            let admission = admit_target_relation_search(&ctx.block, ctx, solver_ctx);
            if !admission.is_admitted() {
                return Err(algorithmic_hard_case(
                    ctx,
                    "target relation search stage not admitted",
                    &[stage.stage_hash],
                ));
            }
            let Some(mut plan) = admission.execution_plan else {
                return Err(implementation_bug(
                    "target relation search stage lacks plan",
                ));
            };
            bind_stage_cost_to_subplan(stage, &mut plan);
            let message = execute_target_relation_search(&plan, ctx, solver_ctx)?;
            wrap_stage_message(
                stage,
                ctx,
                message,
                ProjectionStrength::CandidateCoverStrong,
                parent_plan,
                failed_strategy_hashes,
                executed_failed_strategy_hashes,
            )
        }
        UniversalStrategy::SparseResultantIfSquareOrOverdetermined => {
            let mut subplan = plan_sparse_resultant_with_messages(
                &ctx.block,
                &ctx.system,
                &ctx.child_messages,
                solver_ctx,
                KernelPlanId(stage.parent_plan_id.0.saturating_add(100)),
            )
            .map_err(|_| {
                algorithmic_hard_case(
                    ctx,
                    "sparse resultant stage not applicable",
                    &[stage.stage_hash],
                )
            })?;
            bind_stage_cost_to_subplan(stage, &mut subplan);
            let message = execute_sparse_resultant(&subplan, ctx, solver_ctx)?;
            wrap_stage_message(
                stage,
                ctx,
                message,
                ProjectionStrength::CandidateCoverStrong,
                parent_plan,
                failed_strategy_hashes,
                executed_failed_strategy_hashes,
            )
        }
        UniversalStrategy::TargetActionKrylovIfQuotientCertifiable => {
            let mut subplan = plan_target_action_krylov_with_messages(
                &ctx.block,
                &ctx.system,
                &ctx.child_messages,
                solver_ctx,
                KernelPlanId(stage.parent_plan_id.0.saturating_add(150)),
            )
            .map_err(|_| {
                algorithmic_hard_case(
                    ctx,
                    "target-action Krylov stage not applicable",
                    &[stage.stage_hash],
                )
            })?;
            bind_stage_cost_to_subplan(stage, &mut subplan);
            let message = execute_target_action_krylov(&subplan, ctx, solver_ctx)?;
            wrap_stage_message(
                stage,
                ctx,
                message,
                ProjectionStrength::CandidateCoverStrong,
                parent_plan,
                failed_strategy_hashes,
                executed_failed_strategy_hashes,
            )
        }
        UniversalStrategy::SpecializeProjectInterpolateVerify => {
            let mut subplan = plan_specialization_interpolation_with_messages(
                &ctx.block,
                &ctx.system,
                &ctx.child_messages,
                solver_ctx,
                KernelPlanId(stage.parent_plan_id.0.saturating_add(200)),
            )
            .map_err(|_| {
                algorithmic_hard_case(
                    ctx,
                    "specialization-interpolation stage not applicable",
                    &[stage.stage_hash],
                )
            })?;
            bind_stage_cost_to_subplan(stage, &mut subplan);
            let message = execute_specialization_interpolation(&subplan, ctx, solver_ctx)?;
            wrap_stage_message(
                stage,
                ctx,
                message,
                ProjectionStrength::CandidateCoverStrong,
                parent_plan,
                failed_strategy_hashes,
                executed_failed_strategy_hashes,
            )
        }
        UniversalStrategy::RegularChainIfTriangular => {
            let mut subplan = plan_regular_chain_projection(
                &ctx.block,
                &ctx.system,
                solver_ctx,
                KernelPlanId(stage.parent_plan_id.0.saturating_add(250)),
            )
            .map_err(|_| {
                algorithmic_hard_case(
                    ctx,
                    "regular-chain stage not applicable",
                    &[stage.stage_hash],
                )
            })?;
            bind_stage_cost_to_subplan(stage, &mut subplan);
            let message = execute_regular_chain_projection(&subplan, ctx, solver_ctx)?;
            wrap_stage_message(
                stage,
                ctx,
                message,
                ProjectionStrength::CandidateCoverStrong,
                parent_plan,
                failed_strategy_hashes,
                executed_failed_strategy_hashes,
            )
        }
        UniversalStrategy::NormTraceIfTower => {
            let mut subplan = plan_norm_trace_projection(
                &ctx.block,
                &ctx.system,
                solver_ctx,
                KernelPlanId(stage.parent_plan_id.0.saturating_add(300)),
            )
            .map_err(|_| {
                algorithmic_hard_case(ctx, "norm-trace stage not applicable", &[stage.stage_hash])
            })?;
            bind_stage_cost_to_subplan(stage, &mut subplan);
            let message = execute_norm_trace_projection(&subplan, ctx, solver_ctx)?;
            wrap_stage_message(
                stage,
                ctx,
                message,
                ProjectionStrength::CandidateCoverStrong,
                parent_plan,
                failed_strategy_hashes,
                executed_failed_strategy_hashes,
            )
        }
        UniversalStrategy::LocalGroebnerEliminationToKeepZ => {
            let inputs = planned_stage_relation_inputs(stage, ctx)?;
            let relations = inputs
                .iter()
                .map(|input| input.polynomial.clone())
                .collect::<Vec<_>>();
            let options = GroebnerOptions {
                max_pairs: stage
                    .resource_bounds
                    .max_local_elimination_steps
                    .unwrap_or(0)
                    .max(1),
                max_basis_size: stage.resource_bounds.max_matrix_cols.unwrap_or(0).max(1),
            };
            let result = eliminate_to_keep_variables(
                &relations,
                &stage.eliminated_variables,
                &stage.exported_variables,
                EliminationStrategy::LocalGroebner(options),
                solver_ctx,
            )?;
            enforce_stage_resource_bounds(stage, ctx.system.target, &result, &relations)?;
            validate_local_elimination_result(&result, &stage.exported_variables, &relations)?;
            let output_memberships = result
                .generators
                .iter()
                .filter(|generator| !generator.generator.terms.is_empty())
                .map(|generator| generator.certificate.clone())
                .collect::<Vec<_>>();
            let generators =
                extract_verified_export_generators(&result, &stage.exported_variables)?;
            if generators.is_empty() {
                return Err(algorithmic_hard_case(
                    ctx,
                    "local Groebner stage found no exported generator",
                    &[stage.stage_hash],
                ));
            }
            let trace = ProjectionCostTrace {
                block_id: stage.block_id,
                kernel_kind: KernelKind::UniversalTargetElimination,
                local_variable_count: ctx.block.local_variables.len(),
                exported_variable_count: stage.exported_variables.len(),
                local_relation_count: relations.len(),
                local_monomial_count: relations.iter().map(poly_monomial_count).sum(),
                estimated_quotient_rank: None,
                matrix_rows: Some(result.matrix_rows),
                matrix_cols: Some(result.matrix_cols),
                matrix_density: Some(matrix_density(&SparseMatrixQ {
                    rows: result.matrix_rows.max(1),
                    cols: result.matrix_cols.max(1),
                    entries: Vec::new(),
                })),
                coefficient_height_before_bits: max_poly_coefficient_height_bits(&relations),
                coefficient_height_after_bits: max_poly_coefficient_height_bits(&generators),
                route_cost: Some(ProjectionCostTrace::route_cost_from_plan(parent_plan)),
            };
            verify_universal_no_coordinate_fallback(&stage_execution_plan_shadow(stage), &trace)?;
            let strategy_records = universal_strategy_trace_records(parent_plan)?;
            let skipped_cost_prohibited_strategy_hashes =
                skipped_cost_prohibited_strategy_hashes(&strategy_records);
            Ok(build_universal_message(
                stage,
                ctx,
                parent_plan,
                generators,
                trace,
                ProjectionStrength::CandidateCoverStrong,
                hash_sequence(
                    "universal-local-groebner-certificate",
                    &[stage.stage_hash.0.to_vec()],
                ),
                KernelCertificatePayload::Universal(UniversalProjectionCertificate {
                    stage_hash: stage.stage_hash,
                    stage_certificate_hash: hash_sequence(
                        "universal-local-groebner-certificate",
                        &[stage.stage_hash.0.to_vec()],
                    ),
                    attempted_strategies: attempted_universal_strategies(parent_plan),
                    strategy_records,
                    skipped_cost_prohibited_strategy_hashes,
                    chosen_strategy: stage.strategy,
                    failed_strategy_hashes: failed_strategy_hashes.to_vec(),
                    executed_failed_strategy_hashes: executed_failed_strategy_hashes.to_vec(),
                    output_relations: result
                        .generators
                        .iter()
                        .filter(|generator| !generator.generator.terms.is_empty())
                        .map(|generator| generator.generator.clone())
                        .collect(),
                    inner_payload: None,
                    output_memberships,
                    source_relations: relations.clone(),
                }),
            ))
        }
    }
}

fn bind_stage_cost_to_subplan(stage: &UniversalStagePlan, subplan: &mut KernelExecutionPlan) {
    subplan.algebraic_work_estimate = stage.algebraic_work_estimate.clone();
    subplan.route_budget = stage.route_budget.clone();
    subplan.plan_hash = hash_kernel_execution_plan(subplan);
}

fn enforce_universal_stage_route_budget(stage: &UniversalStagePlan) -> Result<(), SolverError> {
    if !stage.algebraic_work_estimate.is_hash_current() || !stage.route_budget.is_hash_current() {
        return Err(implementation_bug(
            "universal internal stage has stale algebraic work estimate or route budget",
        ));
    }
    if stage.algebraic_work_estimate.predicted_work_units > stage.route_budget.max_work_units
        || stage
            .algebraic_work_estimate
            .predicted_intermediate_terms
            .is_some_and(|terms| terms.exceeds_usize(stage.route_budget.max_intermediate_terms))
        || stage
            .algebraic_work_estimate
            .predicted_output_terms
            .is_some_and(|terms| terms.exceeds_usize(stage.route_budget.max_output_terms))
        || stage.algebraic_work_estimate.max_keep_variable_count
            > stage.route_budget.max_keep_variables
        || stage.algebraic_work_estimate.max_total_degree > stage.route_budget.max_total_degree
        || stage
            .algebraic_work_estimate
            .predicted_coefficient_height_bits
            .is_some_and(|bits| bits.exceeds_usize(stage.route_budget.max_coefficient_height_bits))
    {
        return Err(finite_resource_failure(
            None,
            &stage_execution_plan_shadow(stage),
            stage.algebraic_work_estimate.matrix_rows.unwrap_or(0),
            stage.algebraic_work_estimate.matrix_cols.unwrap_or(0),
            stage
                .algebraic_work_estimate
                .predicted_coefficient_height_bits
                .map(|bits| bits.as_usize_saturating())
                .unwrap_or(0),
        ));
    }
    Ok(())
}

pub fn verify_universal_no_coordinate_fallback(
    plan: &KernelExecutionPlan,
    trace: &ProjectionCostTrace,
) -> Result<(), SolverError> {
    if plan.kernel_kind != KernelKind::UniversalTargetElimination {
        return Err(implementation_bug(
            "universal fallback check received wrong kernel",
        ));
    }
    if plan.certificate_route != CertificateRoute::UniversalFixedLocalElimination {
        return Err(implementation_bug(
            "universal certificate route is not fixed local elimination",
        ));
    }
    if plan.failure_behavior.local_nonfinite_policy
        != LocalNonfinitePolicy::NoLocalCertifiedNonFinite
    {
        return Err(implementation_bug(
            "universal local nonfinite policy is not disabled",
        ));
    }
    if plan.resource_bounds.max_matrix_rows.is_none()
        || plan.resource_bounds.max_matrix_cols.is_none()
        || plan.resource_bounds.max_local_elimination_steps.is_none()
    {
        return Err(implementation_bug(
            "universal local elimination lacks explicit resource caps",
        ));
    }
    if trace.kernel_kind != KernelKind::UniversalTargetElimination {
        return Err(implementation_bug("universal trace kernel kind mismatch"));
    }
    if let (Some(rows), Some(max_rows)) = (trace.matrix_rows, plan.resource_bounds.max_matrix_rows)
    {
        if rows > max_rows {
            return Err(finite_resource_failure(
                None,
                plan,
                rows,
                trace.matrix_cols.unwrap_or(0),
                trace.coefficient_height_before_bits,
            ));
        }
    }
    if let (Some(cols), Some(max_cols)) = (trace.matrix_cols, plan.resource_bounds.max_matrix_cols)
    {
        if cols > max_cols {
            return Err(finite_resource_failure(
                None,
                plan,
                trace.matrix_rows.unwrap_or(0),
                cols,
                trace.coefficient_height_before_bits,
            ));
        }
    }
    Ok(())
}

pub fn extract_verified_export_generators(
    result: &LocalEliminationResult,
    exported: &[VariableId],
) -> Result<Vec<SparsePolynomialQ>, SolverError> {
    let exported = exported.iter().copied().collect::<BTreeSet<_>>();
    let mut out = Vec::new();
    for generator in &result.generators {
        if generator.generator.terms.is_empty() {
            continue;
        }
        if !poly_variables(&generator.generator).is_subset(&exported) {
            return Err(implementation_bug(
                "universal local elimination exported a non-keep variable",
            ));
        }
        out.push(generator.generator.clone());
    }
    Ok(out)
}

fn wrap_stage_message(
    stage: &UniversalStagePlan,
    ctx: &KernelContext,
    message: ProjectionMessage,
    strength: ProjectionStrength,
    parent_plan: &KernelExecutionPlan,
    failed_strategy_hashes: &[Hash],
    executed_failed_strategy_hashes: &[Hash],
) -> Result<ProjectionMessage, SolverError> {
    let exported = stage
        .exported_variables
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    if message
        .relation_generators
        .iter()
        .any(|relation| !poly_variables(relation).is_subset(&exported))
    {
        return Err(implementation_bug(
            "universal stage returned a relation outside exported variables",
        ));
    }
    if !message
        .relation_generators
        .iter()
        .any(|relation| !relation.terms.is_empty())
    {
        return Err(algorithmic_hard_case(
            ctx,
            "universal stage returned no nonzero exported relation",
            &[stage.stage_hash],
        ));
    }
    let trace = ProjectionCostTrace {
        block_id: stage.block_id,
        kernel_kind: KernelKind::UniversalTargetElimination,
        local_variable_count: ctx.block.local_variables.len(),
        exported_variable_count: stage.exported_variables.len(),
        local_relation_count: message.cost_trace.local_relation_count,
        local_monomial_count: message.cost_trace.local_monomial_count,
        estimated_quotient_rank: message.cost_trace.estimated_quotient_rank,
        matrix_rows: message.cost_trace.matrix_rows,
        matrix_cols: message.cost_trace.matrix_cols,
        matrix_density: message.cost_trace.matrix_density.clone(),
        coefficient_height_before_bits: message.cost_trace.coefficient_height_before_bits,
        coefficient_height_after_bits: message.cost_trace.coefficient_height_after_bits,
        route_cost: Some(ProjectionCostTrace::route_cost_from_plan(parent_plan)),
    };
    verify_universal_no_coordinate_fallback(&stage_execution_plan_shadow(stage), &trace)?;
    let relation_generators = message.relation_generators.clone();
    let stage_certificate_hash = message.certificate.certificate_hash;
    let inner_payload = message.certificate.payload.clone();
    let source_relations = planned_stage_relation_inputs(stage, ctx)?
        .into_iter()
        .map(|input| input.polynomial)
        .collect();
    let strategy_records = universal_strategy_trace_records(parent_plan)?;
    let skipped_cost_prohibited_strategy_hashes =
        skipped_cost_prohibited_strategy_hashes(&strategy_records);
    Ok(build_universal_message(
        stage,
        ctx,
        parent_plan,
        relation_generators.clone(),
        trace,
        strength,
        stage_certificate_hash,
        KernelCertificatePayload::Universal(UniversalProjectionCertificate {
            stage_hash: stage.stage_hash,
            stage_certificate_hash,
            attempted_strategies: attempted_universal_strategies(parent_plan),
            strategy_records,
            skipped_cost_prohibited_strategy_hashes,
            chosen_strategy: stage.strategy,
            failed_strategy_hashes: failed_strategy_hashes.to_vec(),
            executed_failed_strategy_hashes: executed_failed_strategy_hashes.to_vec(),
            output_relations: relation_generators,
            inner_payload: Some(Box::new(inner_payload)),
            output_memberships: Vec::new(),
            source_relations,
        }),
    ))
}

fn build_universal_message(
    stage: &UniversalStagePlan,
    ctx: &KernelContext,
    parent_plan: &KernelExecutionPlan,
    relation_generators: Vec<SparsePolynomialQ>,
    cost_trace: ProjectionCostTrace,
    projection_strength: ProjectionStrength,
    stage_certificate_hash: Hash,
    payload: KernelCertificatePayload,
) -> ProjectionMessage {
    let certificate_hash = hash_sequence(
        "universal-elimination-certificate",
        &std::iter::once(stage.parent_plan_hash.0.to_vec())
            .chain(std::iter::once(stage.stage_hash.0.to_vec()))
            .chain(std::iter::once(stage_certificate_hash.0.to_vec()))
            .chain(
                relation_generators
                    .iter()
                    .map(|relation| relation.hash.0.to_vec()),
            )
            .chain(std::iter::once(format!("{payload:?}").into_bytes()))
            .collect::<Vec<_>>(),
    );
    let certificate = KernelCertificate::from_execution_plan_with_payload(
        parent_plan,
        &relation_generators,
        certificate_hash,
        payload,
    );
    let mut message = ProjectionMessage {
        package_id: PackageId(stage.parent_plan_id.0),
        block_id: stage.block_id,
        kernel_kind: KernelKind::UniversalTargetElimination,
        source_relation_ids: ctx.block.relation_ids.clone(),
        eliminated_variables: stage.eliminated_variables.clone(),
        exported_variables: stage.exported_variables.clone(),
        relation_generators,
        representation: MessageRepresentation::GeneratorSet,
        projection_strength,
        certificate,
        compression_trace: ctx.system.compression_trace.clone(),
        cost_trace,
        package_hash: hash_sequence("projection-message-initial", &[]),
    };
    message.package_hash = projection_message_hash(&message);
    message
}

fn attempted_universal_strategies(parent_plan: &KernelExecutionPlan) -> Vec<UniversalStrategy> {
    parent_plan
        .support_plan
        .universal_strategy_sequence
        .iter()
        .map(|step| step.strategy)
        .collect()
}

fn universal_strategy_trace_records(
    parent_plan: &KernelExecutionPlan,
) -> Result<Vec<UniversalStrategyTraceRecord>, SolverError> {
    Ok(build_stage_plans(parent_plan)?
        .into_iter()
        .map(|stage| UniversalStrategyTraceRecord {
            strategy: stage.strategy,
            stage_hash: stage.stage_hash,
            enabled: stage.enabled,
            skip_reason: stage.skip_reason,
            cost_class: stage.cost_class,
            algebraic_work_estimate_hash: stage.algebraic_work_estimate.estimate_hash,
            route_budget_hash: stage.route_budget.budget_hash,
            predicted_work_units: stage.algebraic_work_estimate.predicted_work_units,
            route_budget_max_work_units: stage.route_budget.max_work_units,
            route_budget_max_elapsed_steps: stage.route_budget.max_elapsed_steps,
        })
        .collect())
}

fn skipped_cost_prohibited_strategy_hashes(records: &[UniversalStrategyTraceRecord]) -> Vec<Hash> {
    records
        .iter()
        .filter(|record| record.cost_class == RouteCostClass::CostProhibited)
        .map(|record| record.stage_hash)
        .collect()
}

fn validate_universal_plan_binding(
    plan: &KernelExecutionPlan,
    ctx: &KernelContext,
) -> Result<(), SolverError> {
    if plan.kernel_kind != KernelKind::UniversalTargetElimination {
        return Err(implementation_bug(
            "universal execution received wrong plan kind",
        ));
    }
    if hash_kernel_execution_plan(plan) != plan.plan_hash {
        return Err(implementation_bug("universal execution plan hash mismatch"));
    }
    if plan.block_id != ctx.block.block_id {
        return Err(implementation_bug("universal block id mismatch"));
    }
    if plan.input_block_authorization_hash != ctx.block.authorization_hash {
        return Err(implementation_bug(
            "universal block authorization hash mismatch",
        ));
    }
    if plan.certificate_route != CertificateRoute::UniversalFixedLocalElimination {
        return Err(implementation_bug("universal certificate route mismatch"));
    }
    if support_plan_hash(&plan.support_plan) != plan.support_plan.support_hash {
        return Err(implementation_bug("universal support plan hash mismatch"));
    }
    if plan.failure_behavior.local_nonfinite_policy
        != LocalNonfinitePolicy::NoLocalCertifiedNonFinite
    {
        return Err(implementation_bug(
            "universal local nonfinite policy mismatch",
        ));
    }
    validate_fixed_strategy_sequence(&plan.support_plan.universal_strategy_sequence)?;
    let available_child_hashes = ctx
        .child_messages
        .iter()
        .filter(|message| plan.child_block_ids.contains(&message.block_id))
        .map(|message| message.package_hash)
        .collect::<BTreeSet<_>>();
    let planned_child_hashes = plan
        .child_message_hashes
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    if available_child_hashes != planned_child_hashes {
        return Err(implementation_bug(
            "universal child message hash binding mismatch",
        ));
    }
    Ok(())
}

fn build_stage_plans(plan: &KernelExecutionPlan) -> Result<Vec<UniversalStagePlan>, SolverError> {
    validate_fixed_strategy_sequence(&plan.support_plan.universal_strategy_sequence)?;
    Ok(plan
        .support_plan
        .universal_strategy_sequence
        .iter()
        .enumerate()
        .map(|(idx, step)| {
            let mut stage = UniversalStagePlan {
                parent_plan_id: plan.plan_id,
                parent_plan_hash: plan.plan_hash,
                block_id: plan.block_id,
                strategy: step.strategy,
                stage_index: idx,
                exported_variables: plan.exported_variables.clone(),
                eliminated_variables: plan.eliminated_variables.clone(),
                source_relation_hashes: plan.source_relation_hashes.clone(),
                child_message_hashes: plan.child_message_hashes.clone(),
                resource_bounds: plan.resource_bounds.clone(),
                enabled: step.enabled,
                skip_reason: step.skip_reason.clone(),
                cost_class: step.cost_class,
                algebraic_work_estimate: step.algebraic_work_estimate.clone(),
                route_budget: step.route_budget.clone(),
                stage_hash: hash_sequence("universal-stage-plan", &[]),
            };
            stage.stage_hash = universal_stage_hash(&stage);
            stage
        })
        .collect())
}

fn validate_fixed_strategy_sequence(
    steps: &[UniversalStrategyPlanStep],
) -> Result<(), SolverError> {
    let expected = [
        UniversalStrategy::TargetRelationSearchEscalated,
        UniversalStrategy::SparseResultantIfSquareOrOverdetermined,
        UniversalStrategy::TargetActionKrylovIfQuotientCertifiable,
        UniversalStrategy::SpecializeProjectInterpolateVerify,
        UniversalStrategy::RegularChainIfTriangular,
        UniversalStrategy::NormTraceIfTower,
        UniversalStrategy::LocalGroebnerEliminationToKeepZ,
    ];
    if steps.len() != expected.len()
        || steps
            .iter()
            .zip(expected)
            .any(|(step, expected)| step.strategy != expected)
    {
        return Err(implementation_bug(
            "universal fixed strategy sequence mismatch",
        ));
    }
    Ok(())
}

fn collect_relation_inputs(
    block: &ProjectionBlock,
    system: &CompressedSystemQ,
    child_messages: &[ProjectionMessage],
) -> Vec<UniversalRelationInput> {
    let mut inputs = block_relations(block, system)
        .into_iter()
        .map(|relation| UniversalRelationInput {
            polynomial: relation.polynomial.clone(),
            source_relation_ids: vec![relation.id],
            source_hash: relation.hash,
            child_message_hash: None,
        })
        .collect::<Vec<_>>();
    for message in child_messages {
        if !block.child_block_ids.contains(&message.block_id) {
            continue;
        }
        for relation in &message.relation_generators {
            inputs.push(UniversalRelationInput {
                polynomial: relation.clone(),
                source_relation_ids: message.source_relation_ids.clone(),
                source_hash: relation.hash,
                child_message_hash: Some(message.package_hash),
            });
        }
    }
    inputs
}

fn planned_relation_inputs(
    plan: &KernelExecutionPlan,
    ctx: &KernelContext,
) -> Result<Vec<UniversalRelationInput>, SolverError> {
    let algebraic_work_estimate = plan.algebraic_work_estimate.clone();
    let route_budget = plan.route_budget.clone();
    let inputs = planned_stage_relation_inputs(
        &UniversalStagePlan {
            parent_plan_id: plan.plan_id,
            parent_plan_hash: plan.plan_hash,
            block_id: plan.block_id,
            strategy: UniversalStrategy::LocalGroebnerEliminationToKeepZ,
            stage_index: 0,
            exported_variables: plan.exported_variables.clone(),
            eliminated_variables: plan.eliminated_variables.clone(),
            source_relation_hashes: plan.source_relation_hashes.clone(),
            child_message_hashes: plan.child_message_hashes.clone(),
            resource_bounds: plan.resource_bounds.clone(),
            enabled: true,
            skip_reason: None,
            cost_class: RouteCostClass::ExpensiveButAllowed,
            algebraic_work_estimate,
            route_budget,
            stage_hash: hash_sequence("universal-stage-plan-synthetic", &[]),
        },
        ctx,
    )?;
    Ok(inputs)
}

fn planned_stage_relation_inputs(
    stage: &UniversalStagePlan,
    ctx: &KernelContext,
) -> Result<Vec<UniversalRelationInput>, SolverError> {
    let source_hashes = stage
        .source_relation_hashes
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let child_hashes = stage
        .child_message_hashes
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let mut inputs = ctx
        .system
        .relations
        .iter()
        .filter(|relation| source_hashes.contains(&relation.hash))
        .map(|relation| UniversalRelationInput {
            polynomial: relation.polynomial.clone(),
            source_relation_ids: vec![relation.id],
            source_hash: relation.hash,
            child_message_hash: None,
        })
        .collect::<Vec<_>>();
    for message in &ctx.child_messages {
        if !child_hashes.contains(&message.package_hash) {
            continue;
        }
        for relation in &message.relation_generators {
            if source_hashes.contains(&relation.hash) {
                inputs.push(UniversalRelationInput {
                    polynomial: relation.clone(),
                    source_relation_ids: message.source_relation_ids.clone(),
                    source_hash: relation.hash,
                    child_message_hash: Some(message.package_hash),
                });
            }
        }
    }
    validate_source_hash_coverage(&stage.source_relation_hashes, &inputs)?;
    Ok(inputs)
}

fn validate_source_hash_coverage(
    expected: &[Hash],
    inputs: &[UniversalRelationInput],
) -> Result<(), SolverError> {
    let mut expected = expected.to_vec();
    let mut actual = inputs
        .iter()
        .map(|input| input.source_hash)
        .collect::<Vec<_>>();
    expected.sort();
    actual.sort();
    if expected != actual {
        return Err(implementation_bug(
            "universal source relation hash coverage mismatch",
        ));
    }
    Ok(())
}

fn fixed_universal_strategy_sequence(
    block: &ProjectionBlock,
    system: &CompressedSystemQ,
    child_messages: &[ProjectionMessage],
    solver_ctx: &SolverContext,
    relations: &[SparsePolynomialQ],
    exported_variables: &[VariableId],
    target: VariableId,
) -> Vec<UniversalStrategyPlanStep> {
    let square_or_overdetermined =
        !relations.is_empty() && relations.len() >= exported_variables.len().max(1);
    let has_non_target_separator = exported_variables.iter().any(|var| *var != target);
    let mut probe_ctx = solver_ctx.clone();
    let probes = run_cost_probes(block, system, &mut probe_ctx);
    let child_message_hashes = child_messages
        .iter()
        .map(|message| message.package_hash)
        .collect::<BTreeSet<_>>();
    vec![
        universal_strategy_step_for_stage(
            block,
            system,
            child_messages,
            solver_ctx,
            &probes,
            UniversalStrategy::TargetRelationSearchEscalated,
            KernelKind::TargetRelationSearch,
            !relations.is_empty(),
            empty_skip(!relations.is_empty(), "no local relations"),
        ),
        universal_strategy_step_for_stage(
            block,
            system,
            child_messages,
            solver_ctx,
            &probes,
            UniversalStrategy::SparseResultantIfSquareOrOverdetermined,
            KernelKind::SparseResultantProjection,
            square_or_overdetermined,
            empty_skip(square_or_overdetermined, "not square or overdetermined"),
        ),
        universal_strategy_step_for_stage(
            block,
            system,
            child_messages,
            solver_ctx,
            &probes,
            UniversalStrategy::TargetActionKrylovIfQuotientCertifiable,
            KernelKind::TargetActionKrylov,
            !relations.is_empty(),
            empty_skip(
                !relations.is_empty(),
                "no relations for target action quotient",
            ),
        ),
        universal_strategy_step_for_stage(
            block,
            system,
            child_messages,
            solver_ctx,
            &probes,
            UniversalStrategy::SpecializeProjectInterpolateVerify,
            KernelKind::SpecializationInterpolation,
            has_non_target_separator,
            empty_skip(has_non_target_separator, "no non-target exported separator"),
        ),
        universal_strategy_step_for_stage(
            block,
            system,
            child_messages,
            solver_ctx,
            &probes,
            UniversalStrategy::RegularChainIfTriangular,
            KernelKind::RegularChainProjection,
            !relations.is_empty(),
            empty_skip(
                !relations.is_empty(),
                "no relations for regular-chain detection",
            ),
        ),
        universal_strategy_step_for_stage(
            block,
            system,
            child_messages,
            solver_ctx,
            &probes,
            UniversalStrategy::NormTraceIfTower,
            KernelKind::NormTraceProjection,
            !relations.is_empty(),
            empty_skip(
                !relations.is_empty(),
                "no relations for norm/trace tower detection",
            ),
        ),
        universal_strategy_step_for_stage(
            block,
            system,
            child_messages,
            solver_ctx,
            &probes,
            UniversalStrategy::LocalGroebnerEliminationToKeepZ,
            KernelKind::UniversalTargetElimination,
            !relations.is_empty(),
            empty_skip(
                !relations.is_empty(),
                &format!(
                    "no relations available for local elimination; child_message_count={}",
                    child_message_hashes.len()
                ),
            ),
        ),
    ]
}

fn universal_strategy_step_for_stage(
    block: &ProjectionBlock,
    system: &CompressedSystemQ,
    child_messages: &[ProjectionMessage],
    solver_ctx: &SolverContext,
    probes: &crate::planner::probes::ProbeResults,
    strategy: UniversalStrategy,
    kernel_kind: KernelKind,
    structurally_enabled: bool,
    structural_skip_reason: Option<String>,
) -> UniversalStrategyPlanStep {
    let fallback_cost = estimate_kernel_cost(block, system, kernel_kind, probes);
    let planned_cost = if structurally_enabled {
        universal_stage_subplan_for_cost(
            block,
            system,
            child_messages,
            solver_ctx,
            strategy,
            kernel_kind,
        )
        .ok()
    } else {
        None
    };
    let (cost_class, algebraic_work_estimate, route_budget) = if let Some(plan) = planned_cost {
        let estimate = plan.algebraic_work_estimate.clone();
        let matrix_rows = estimate
            .matrix_rows
            .or(plan.resource_bounds.max_matrix_rows)
            .unwrap_or(1);
        let matrix_cols = estimate
            .matrix_cols
            .or(plan.resource_bounds.max_matrix_cols)
            .unwrap_or(1);
        let rank = estimate
            .quotient_rank_estimate
            .or_else(|| {
                plan.support_plan
                    .rank_plan
                    .as_ref()
                    .map(|rank| rank.estimated_rank)
            })
            .unwrap_or(1);
        (
            classify_route_cost(kernel_kind, matrix_rows, matrix_cols, rank, &estimate),
            estimate,
            plan.route_budget.clone(),
        )
    } else {
        let budget = RouteBudget::from_estimate(&fallback_cost.algebraic_work_estimate);
        (
            fallback_cost.cost_class,
            fallback_cost.algebraic_work_estimate,
            budget,
        )
    };
    let cost_prohibited = cost_class == RouteCostClass::CostProhibited
        && matches!(
            strategy,
            UniversalStrategy::TargetRelationSearchEscalated
                | UniversalStrategy::SparseResultantIfSquareOrOverdetermined
        );
    let enabled = structurally_enabled && !cost_prohibited;
    let skip_reason = if !structurally_enabled {
        structural_skip_reason
    } else if cost_prohibited {
        Some(format!(
            "CostProhibitedInternalStage: strategy={strategy:?} kernel={kernel_kind:?} estimate_hash={:?} work={} budget={}",
            algebraic_work_estimate.estimate_hash,
            algebraic_work_estimate.predicted_work_units.0,
            route_budget.max_work_units.0
        ))
    } else {
        None
    };
    universal_strategy_step_with_cost(
        strategy,
        enabled,
        skip_reason,
        cost_class,
        algebraic_work_estimate,
        route_budget,
    )
}

fn universal_stage_subplan_for_cost(
    block: &ProjectionBlock,
    system: &CompressedSystemQ,
    child_messages: &[ProjectionMessage],
    solver_ctx: &SolverContext,
    strategy: UniversalStrategy,
    kernel_kind: KernelKind,
) -> Result<KernelExecutionPlan, SolverError> {
    match strategy {
        UniversalStrategy::TargetRelationSearchEscalated => {
            let kctx = KernelContext {
                block: block.clone(),
                system: system.clone(),
                child_messages: child_messages.to_vec(),
            };
            let admission = admit_target_relation_search(block, &kctx, solver_ctx);
            if !admission.is_admitted() {
                return Err(algorithmic_hard_case_for_block(
                    system.target,
                    block,
                    "target relation search stage not admitted during Universal planning",
                ));
            }
            admission.execution_plan.ok_or_else(|| {
                implementation_bug(
                    "target relation search stage admission lacked execution plan during Universal planning",
                )
            })
        }
        UniversalStrategy::SparseResultantIfSquareOrOverdetermined => {
            plan_sparse_resultant_with_messages(
                block,
                system,
                child_messages,
                solver_ctx,
                KernelPlanId(kernel_kind as u32),
            )
        }
        UniversalStrategy::TargetActionKrylovIfQuotientCertifiable => {
            plan_target_action_krylov_with_messages(
                block,
                system,
                child_messages,
                solver_ctx,
                KernelPlanId(kernel_kind as u32),
            )
        }
        UniversalStrategy::SpecializeProjectInterpolateVerify => {
            plan_specialization_interpolation_with_messages(
                block,
                system,
                child_messages,
                solver_ctx,
                KernelPlanId(kernel_kind as u32),
            )
        }
        UniversalStrategy::RegularChainIfTriangular => plan_regular_chain_projection(
            block,
            system,
            solver_ctx,
            KernelPlanId(kernel_kind as u32),
        ),
        UniversalStrategy::NormTraceIfTower => {
            plan_norm_trace_projection(block, system, solver_ctx, KernelPlanId(kernel_kind as u32))
        }
        UniversalStrategy::LocalGroebnerEliminationToKeepZ => Err(algorithmic_hard_case_for_block(
            system.target,
            block,
            "local elimination stage uses Universal parent estimate",
        )),
    }
}

fn block_relations(block: &ProjectionBlock, system: &CompressedSystemQ) -> Vec<CanonicalRelationQ> {
    let ids = block.relation_ids.iter().copied().collect::<BTreeSet<_>>();
    system
        .relations
        .iter()
        .filter(|relation| ids.contains(&relation.id))
        .cloned()
        .collect()
}

fn enforce_stage_resource_bounds(
    stage: &UniversalStagePlan,
    target: VariableId,
    result: &LocalEliminationResult,
    relations: &[SparsePolynomialQ],
) -> Result<(), SolverError> {
    if stage
        .resource_bounds
        .max_matrix_rows
        .is_some_and(|limit| result.matrix_rows > limit)
        || stage
            .resource_bounds
            .max_matrix_cols
            .is_some_and(|limit| result.matrix_cols > limit)
    {
        return Err(finite_resource_failure(
            Some(target),
            &stage_execution_plan_shadow(stage),
            result.matrix_rows,
            result.matrix_cols,
            max_poly_coefficient_height_bits(relations),
        ));
    }
    Ok(())
}

fn stage_execution_plan_shadow(stage: &UniversalStagePlan) -> KernelExecutionPlan {
    KernelExecutionPlan::new_with_algebraic_cost(
        stage.parent_plan_id,
        stage.block_id,
        KernelKind::UniversalTargetElimination,
        hash_sequence(
            "universal-stage-shadow-auth",
            &[stage.parent_plan_hash.0.to_vec()],
        ),
        Vec::new(),
        stage.source_relation_hashes.clone(),
        Vec::new(),
        stage.child_message_hashes.clone(),
        stage.exported_variables.clone(),
        stage.eliminated_variables.clone(),
        KernelSupportPlan {
            dense_relation_search_schedule: None,
            sparse_relation_search_schedule: None,
            affine_elimination_order: None,
            template_plan: None,
            rank_plan: None,
            universal_strategy_sequence: Vec::new(),
            degree_bound: 0,
            support_hash: hash_sequence("universal-stage-shadow-support", &[]),
        },
        stage.resource_bounds.clone(),
        stage.algebraic_work_estimate.clone(),
        stage.route_budget.clone(),
        CertificateRoute::UniversalFixedLocalElimination,
        planned_failure_behavior(
            vec![
                SolverStatus::AlgorithmicHardCase,
                SolverStatus::FiniteResourceFailure,
                SolverStatus::CertificateDesignGap,
            ],
            LocalNonfinitePolicy::NoLocalCertifiedNonFinite,
        ),
        crate::planner::kernel_plan::PlanWorkClassification::PurePlan,
    )
}

fn universal_stage_hash(stage: &UniversalStagePlan) -> Hash {
    hash_sequence(
        "universal-stage-plan",
        &[
            stage.parent_plan_hash.0.to_vec(),
            format!("{:?}", stage.strategy).into_bytes(),
            stage.stage_index.to_be_bytes().to_vec(),
            vec![stage.enabled as u8],
            stage
                .skip_reason
                .as_deref()
                .unwrap_or("")
                .as_bytes()
                .to_vec(),
            format!("{:?}", stage.cost_class).into_bytes(),
            stage.algebraic_work_estimate.estimate_hash.0.to_vec(),
            crate::planner::algebraic_cost::algebraic_work_estimate_hash(
                &stage.algebraic_work_estimate,
            )
            .0
            .to_vec(),
            stage.route_budget.budget_hash.0.to_vec(),
            crate::planner::algebraic_cost::route_budget_hash(&stage.route_budget)
                .0
                .to_vec(),
            stage
                .algebraic_work_estimate
                .predicted_work_units
                .0
                .to_be_bytes()
                .to_vec(),
            stage.route_budget.max_work_units.0.to_be_bytes().to_vec(),
            stage.route_budget.max_elapsed_steps.to_be_bytes().to_vec(),
        ],
    )
}

fn projection_message_hash(message: &ProjectionMessage) -> Hash {
    crate::compose::message::hash_projection_message(message)
}

fn finish_admission(
    block: &ProjectionBlock,
    status: KernelAdmissionStatus,
    execution_plan: Option<KernelExecutionPlan>,
) -> KernelAdmission {
    let mut chunks = vec![
        format!("{:?}", KernelKind::UniversalTargetElimination).into_bytes(),
        block.block_id.0.to_be_bytes().to_vec(),
        format!("{status:?}").into_bytes(),
    ];
    if let Some(plan) = &execution_plan {
        chunks.push(plan.plan_hash.0.to_vec());
    }
    KernelAdmission {
        kind: KernelKind::UniversalTargetElimination,
        block_id: block.block_id,
        status,
        exported_variables: block.exported_variables.iter().copied().collect(),
        eliminated_variables: block
            .local_variables
            .difference(&block.exported_variables)
            .copied()
            .collect(),
        execution_plan,
        admission_hash: hash_sequence("kernel-admission", &chunks),
    }
}

fn dedup_hashes_in_order(values: Vec<Hash>) -> Vec<Hash> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for value in values {
        if seen.insert(value) {
            out.push(value);
        }
    }
    out
}

fn sorted_set(vars: &BTreeSet<VariableId>) -> Vec<VariableId> {
    vars.iter().copied().collect()
}

fn empty_skip(enabled: bool, reason: &str) -> Option<String> {
    if enabled {
        None
    } else {
        Some(reason.to_owned())
    }
}

fn is_continuable_stage_failure(err: &SolverError) -> bool {
    matches!(
        err.kind,
        SolverErrorKind::Failure(FailureKind::AlgorithmicHardCase { .. })
            | SolverErrorKind::Failure(FailureKind::FiniteResourceFailure { .. })
            | SolverErrorKind::Failure(FailureKind::CertificateDesignGap { .. })
    )
}

fn algorithmic_hard_case(ctx: &KernelContext, reason: &str, trace_hashes: &[Hash]) -> SolverError {
    let trace_hash = hash_sequence(
        "universal-elimination-stage-trace",
        &trace_hashes
            .iter()
            .map(|hash| hash.0.to_vec())
            .chain(std::iter::once(reason.as_bytes().to_vec()))
            .collect::<Vec<_>>(),
    );
    SolverError {
        target: Some(ctx.system.target),
        kind: SolverErrorKind::Failure(FailureKind::AlgorithmicHardCase {
            stage: StageId("UniversalTargetEliminationKernel".to_owned()),
            reason: AlgebraicReason(format!("{reason}; trace={trace_hash:?}")),
            minimal_block_hash: ctx.block.block_hash,
        }),
    }
}

fn algorithmic_hard_case_for_block(
    target: VariableId,
    block: &ProjectionBlock,
    reason: &str,
) -> SolverError {
    SolverError {
        target: Some(target),
        kind: SolverErrorKind::Failure(FailureKind::AlgorithmicHardCase {
            stage: StageId("UniversalTargetEliminationKernel".to_owned()),
            reason: AlgebraicReason(reason.to_owned()),
            minimal_block_hash: block.block_hash,
        }),
    }
}

fn finite_resource_failure(
    target: Option<VariableId>,
    plan: &KernelExecutionPlan,
    rows: usize,
    cols: usize,
    coefficient_height_bits: usize,
) -> SolverError {
    SolverError {
        target,
        kind: SolverErrorKind::Failure(FailureKind::FiniteResourceFailure {
            stage: StageId("UniversalTargetEliminationKernel".to_owned()),
            block_id: Some(plan.block_id),
            matrix_rows: Some(rows),
            matrix_cols: Some(cols),
            matrix_density: Some(matrix_density(&SparseMatrixQ {
                rows: rows.max(1),
                cols: cols.max(1),
                entries: Vec::new(),
            })),
            quotient_rank_estimate: None,
            coefficient_height_bits: Some(coefficient_height_bits),
            memory_bytes: plan.resource_bounds.max_memory_bytes,
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
    use crate::compose::message::{MessageRepresentation, ProjectionStrength};
    use crate::kernels::traits::{KernelKind, TargetProjectionKernel};
    use crate::preprocess::compression::CompressionState;
    use crate::problem::canonicalize::canonicalize_system;
    use crate::problem::context::new_context;
    use crate::problem::input::make_problem;
    use crate::problem::validate::validate_input;
    use crate::result::cost_trace::ProjectionCostTrace;
    use crate::result::status::SolverStatus;
    use crate::solver::options::SolverOptions;
    use crate::types::hash::hash_sequence;
    use crate::types::ids::{BlockId, PackageId, VariableId};
    use crate::types::polynomial::{
        constant_poly, poly_add, poly_mul, poly_scale, poly_sub, poly_variables, variable_poly,
    };
    use crate::types::rational::{div_q, int_q};

    use super::*;

    #[test]
    fn fcr_universal_one_large_block_multivariate_projection() {
        let t = VariableId(0);
        let x = VariableId(1);
        let y = VariableId(2);
        let compressed = compressed_system(
            vec![t, x, y],
            t,
            vec![
                poly_sub(&variable_poly(x), &constant_poly(int_q(1))),
                poly_sub(&variable_poly(y), &constant_poly(int_q(2))),
                poly_sub(
                    &poly_add(&variable_poly(x), &variable_poly(y)),
                    &variable_poly(t),
                ),
            ],
        );
        let (message, kctx) = execute_local_groebner_stage_for_test(compressed, [t, x, y], [t]);

        assert_eq!(message.kernel_kind, KernelKind::UniversalTargetElimination);
        assert_eq!(message.representation, MessageRepresentation::GeneratorSet);
        assert_eq!(
            message.projection_strength,
            ProjectionStrength::CandidateCoverStrong
        );
        assert!(crate::verify::verify_message::verify_projection_message(&message, &kctx).is_ok());
        let exported = [t].into_iter().collect();
        assert!(message
            .relation_generators
            .iter()
            .all(|poly| poly_variables(poly).is_subset(&exported)));
        assert!(message
            .relation_generators
            .iter()
            .any(|poly| poly_variables(poly).contains(&t)));
    }

    #[test]
    fn fcr_universal_keep_only_exports_no_coordinate_roots() {
        let t = VariableId(0);
        let s = VariableId(1);
        let x = VariableId(2);
        let compressed = compressed_system(
            vec![t, s, x],
            t,
            vec![
                poly_sub(&variable_poly(x), &variable_poly(t)),
                poly_sub(&variable_poly(s), &constant_poly(int_q(2))),
            ],
        );
        let (message, kctx) = execute_local_groebner_stage_for_test(compressed, [t, s, x], [t, s]);

        assert!(crate::verify::verify_message::verify_projection_message(&message, &kctx).is_ok());
        assert!(message
            .relation_generators
            .iter()
            .all(|poly| !poly_variables(poly).contains(&x)));
        let KernelCertificatePayload::Universal(proof) = &message.certificate.payload else {
            panic!("universal message must carry universal payload");
        };
        assert!(proof.inner_payload.is_none());
        assert!(!proof.output_memberships.is_empty());
        assert_eq!(proof.output_relations, message.relation_generators);
    }

    #[test]
    fn fcr_elimination_membership_certificates_replay() {
        let t = VariableId(0);
        let x = VariableId(1);
        let half = div_q(&int_q(1), &int_q(2)).unwrap();
        let rational_keep_relation = poly_sub(
            &poly_scale(&variable_poly(t), &half),
            &constant_poly(int_q(1)),
        );
        let raw_relations = vec![
            rational_keep_relation,
            poly_sub(&variable_poly(x), &variable_poly(t)),
        ];
        let mut direct_ctx = new_context(SolverOptions::default());
        let direct_result = eliminate_to_keep_variables(
            &raw_relations,
            &[x],
            &[t],
            EliminationStrategy::LocalGroebner(GroebnerOptions::default()),
            &mut direct_ctx,
        )
        .unwrap();
        validate_local_elimination_result(&direct_result, &[t], &raw_relations).unwrap();
        assert!(direct_result.generators.iter().all(|generator| {
            crate::algebra::normal_form::verify_membership_by_certificate(
                &generator.generator,
                &generator.certificate,
                &raw_relations,
            )
        }));

        let compressed = compressed_system(vec![t, x], t, raw_relations);
        let (message, kctx) = execute_local_groebner_stage_for_test(compressed, [t, x], [t]);

        assert!(crate::verify::verify_message::verify_projection_message(&message, &kctx).is_ok());
        let KernelCertificatePayload::Universal(proof) = &message.certificate.payload else {
            panic!("universal message must carry universal payload");
        };
        assert_eq!(proof.output_relations.len(), proof.output_memberships.len());
        assert_eq!(proof.output_relations, message.relation_generators);

        let mut tampered = message.clone();
        let KernelCertificatePayload::Universal(proof) = &mut tampered.certificate.payload else {
            panic!("universal message must carry universal payload");
        };
        proof.output_memberships.clear();
        tampered.certificate.binding_hash =
            crate::verify::certificates::kernel_certificate_binding_hash(&tampered.certificate);
        assert!(
            crate::verify::verify_message::verify_projection_message(&tampered, &kctx).is_err()
        );
    }

    #[test]
    fn fcr_nonproduction_f4_not_reachable() {
        let universal_production_source = include_str!("universal_elimination.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap();
        for fragment in [
            "NotProductionF4",
            "NonProductionGroebnerBatch",
            "groebner_backed_batch_reduce_for_tests",
            "non_production_groebner_batch_elimination_for_tests",
        ] {
            assert!(
                !universal_production_source.contains(fragment),
                "production Universal path must not reference {fragment}"
            );
        }
        let algebra_mod_source = include_str!("../algebra/mod.rs").replace("\r\n", "\n");
        assert!(algebra_mod_source.contains("#[cfg(test)]\npub mod f4;"));

        let t = VariableId(0);
        let x = VariableId(1);
        let relations = vec![poly_sub(&variable_poly(x), &variable_poly(t))];
        let mut solver_ctx = new_context(SolverOptions::default());
        let err = eliminate_to_keep_variables(
            &relations,
            &[x],
            &[t],
            EliminationStrategy::NonProductionGroebnerBatchForTests(
                crate::algebra::f4::GroebnerBackedBatchOptions::default(),
            ),
            &mut solver_ctx,
        )
        .unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::CertificateDesignGap);
    }

    #[test]
    fn p8d_universal_one_large_block_exports_target_relation() {
        let t = VariableId(0);
        let x = VariableId(1);
        let compressed = compressed_system(
            vec![t, x],
            t,
            vec![
                poly_sub(&variable_poly(x), &variable_poly(t)),
                poly_sub(
                    &poly_mul(&variable_poly(x), &variable_poly(x)),
                    &constant_poly(int_q(2)),
                ),
            ],
        );
        let block = test_block(&compressed, [t, x], [t]);
        let mut solver_ctx = new_context(SolverOptions::default());
        let mut kctx = KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };
        let kernel = UniversalTargetEliminationKernel;
        let admission = kernel.admit(&kctx.block, &kctx);
        let plan = kernel.plan(&admission, &kctx, &solver_ctx).unwrap();
        let message = kernel.execute(&plan, &mut kctx, &mut solver_ctx).unwrap();

        assert_eq!(message.kernel_kind, KernelKind::UniversalTargetElimination);
        assert_eq!(message.representation, MessageRepresentation::GeneratorSet);
        assert_eq!(
            message.projection_strength,
            ProjectionStrength::CandidateCoverStrong
        );
        let exported = [t].into_iter().collect();
        assert!(message
            .relation_generators
            .iter()
            .all(|poly| crate::types::polynomial::poly_variables(poly).is_subset(&exported)));
        assert!(kernel.replay(&message, &kctx).accepted);
    }

    #[test]
    fn p8d_universal_rejects_plan_auth_and_replay_tamper() {
        let t = VariableId(0);
        let x = VariableId(1);
        let compressed = compressed_system(
            vec![t, x],
            t,
            vec![
                poly_sub(&variable_poly(x), &variable_poly(t)),
                poly_sub(
                    &poly_mul(&variable_poly(x), &variable_poly(x)),
                    &constant_poly(int_q(2)),
                ),
            ],
        );
        let block = test_block(&compressed, [t, x], [t]);
        let mut solver_ctx = new_context(SolverOptions::default());
        let mut kctx = KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };
        let kernel = UniversalTargetEliminationKernel;
        let admission = kernel.admit(&kctx.block, &kctx);
        let plan = kernel.plan(&admission, &kctx, &solver_ctx).unwrap();
        let mut bad_plan = plan.clone();
        bad_plan.plan_hash = hash_sequence("tampered-plan", &[]);
        let err = kernel
            .execute(&bad_plan, &mut kctx.clone(), &mut solver_ctx)
            .unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::ImplementationBug);

        let mut bad_auth_ctx = kctx.clone();
        bad_auth_ctx.block.authorization_hash = hash_sequence("tampered-auth", &[]);
        let err = kernel
            .execute(&plan, &mut bad_auth_ctx, &mut solver_ctx)
            .unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::ImplementationBug);

        let mut message = kernel.execute(&plan, &mut kctx, &mut solver_ctx).unwrap();
        message.package_hash = hash_sequence("tampered-message", &[]);
        assert!(!kernel.replay(&message, &kctx).accepted);
    }

    #[test]
    fn p8d_universal_child_message_hash_binding_is_operational() {
        let t = VariableId(0);
        let x = VariableId(1);
        let compressed = compressed_system(
            vec![t, x],
            t,
            vec![poly_sub(&variable_poly(x), &variable_poly(t))],
        );
        let mut block = test_block(&compressed, [t, x], [t]);
        block.child_block_ids = vec![BlockId(7)];
        block.authorization_hash =
            crate::graph::projection_dag::authorize_block_relations(&block, &compressed);
        let child_relation = poly_sub(
            &poly_mul(&variable_poly(x), &variable_poly(x)),
            &constant_poly(int_q(2)),
        );
        let child_message = child_projection_message(&compressed, child_relation);
        let mut solver_ctx = new_context(SolverOptions::default());
        let kctx = KernelContext {
            block,
            system: compressed,
            child_messages: vec![child_message],
        };
        let kernel = UniversalTargetEliminationKernel;
        let admission = kernel.admit(&kctx.block, &kctx);
        let plan = kernel.plan(&admission, &kctx, &solver_ctx).unwrap();
        let mut tampered = kctx.clone();
        tampered.child_messages[0].package_hash = hash_sequence("tampered-child", &[]);
        let err = kernel
            .execute(&plan, &mut tampered, &mut solver_ctx)
            .unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::ImplementationBug);
    }

    #[test]
    fn p8d_universal_requires_fixed_strategy_sequence_and_resource_caps() {
        let t = VariableId(0);
        let x = VariableId(1);
        let compressed = compressed_system(
            vec![t, x],
            t,
            vec![
                poly_sub(&variable_poly(x), &variable_poly(t)),
                poly_sub(
                    &poly_mul(&variable_poly(x), &variable_poly(x)),
                    &constant_poly(int_q(2)),
                ),
            ],
        );
        let block = test_block(&compressed, [t, x], [t]);
        let solver_ctx = new_context(SolverOptions::default());
        let kctx = KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };
        let kernel = UniversalTargetEliminationKernel;
        let admission = kernel.admit(&kctx.block, &kctx);
        let mut plan = kernel.plan(&admission, &kctx, &solver_ctx).unwrap();
        let strategies = plan
            .support_plan
            .universal_strategy_sequence
            .iter()
            .map(|step| step.strategy)
            .collect::<Vec<_>>();
        assert_eq!(
            strategies,
            vec![
                UniversalStrategy::TargetRelationSearchEscalated,
                UniversalStrategy::SparseResultantIfSquareOrOverdetermined,
                UniversalStrategy::TargetActionKrylovIfQuotientCertifiable,
                UniversalStrategy::SpecializeProjectInterpolateVerify,
                UniversalStrategy::RegularChainIfTriangular,
                UniversalStrategy::NormTraceIfTower,
                UniversalStrategy::LocalGroebnerEliminationToKeepZ,
            ]
        );
        plan.support_plan.universal_strategy_sequence.swap(0, 1);
        plan.support_plan.support_hash = support_plan_hash(&plan.support_plan);
        plan.plan_hash = hash_kernel_execution_plan(&plan);
        let err = validate_universal_plan_binding(&plan, &kctx).unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::ImplementationBug);

        let trace = ProjectionCostTrace {
            kernel_kind: KernelKind::UniversalTargetElimination,
            ..ProjectionCostTrace::default()
        };
        let mut missing_caps = plan.clone();
        missing_caps
            .support_plan
            .universal_strategy_sequence
            .swap(0, 1);
        missing_caps.resource_bounds.max_local_elimination_steps = None;
        let err = verify_universal_no_coordinate_fallback(&missing_caps, &trace).unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::ImplementationBug);
    }

    #[test]
    fn p8d_universal_exhaustion_uses_only_allowed_failure_statuses() {
        let t = VariableId(0);
        let x = VariableId(1);
        let compressed = compressed_system(vec![t, x], t, vec![variable_poly(x)]);
        let block = test_block(&compressed, [t, x], [t]);
        let mut solver_ctx = new_context(SolverOptions::default());
        let mut kctx = KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };
        let kernel = UniversalTargetEliminationKernel;
        let admission = kernel.admit(&kctx.block, &kctx);
        let plan = kernel.plan(&admission, &kctx, &solver_ctx).unwrap();
        let err = kernel
            .execute(&plan, &mut kctx, &mut solver_ctx)
            .unwrap_err();

        assert!(matches!(
            err.public_status(),
            SolverStatus::AlgorithmicHardCase
                | SolverStatus::FiniteResourceFailure
                | SolverStatus::CertificateDesignGap
        ));
    }

    #[test]
    fn p8d_static_forbidden_fallback_apis_absent() {
        let source = include_str!("universal_elimination.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap();
        for fragment in [
            concat!("Certified", "NonFinite", "TargetImage"),
            concat!("coordinate", "_root"),
            concat!("coordinate", "_solution"),
            concat!("full", "_coordinate"),
            concat!("full", "_rur"),
            concat!("R", "UR"),
            concat!("quantifier", "_elimination"),
            concat!("C", "AD"),
            concat!("solve", "_all_coordinates"),
            concat!("NonProduction", "GroebnerBatch"),
        ] {
            assert!(
                !source.contains(fragment),
                "forbidden Universal fallback path found: {fragment}"
            );
        }
    }

    fn compressed_system(
        variables: Vec<VariableId>,
        target: VariableId,
        relations: Vec<crate::types::polynomial::SparsePolynomialQ>,
    ) -> crate::preprocess::compression::CompressedSystemQ {
        let canonical = canonicalize_system(
            validate_input(make_problem(variables, target, relations, Vec::new())).unwrap(),
        )
        .unwrap();
        CompressionState::from_system(canonical).to_compressed_system()
    }

    fn test_block<const N: usize, const M: usize>(
        compressed: &crate::preprocess::compression::CompressedSystemQ,
        local_variables: [VariableId; N],
        exported_variables: [VariableId; M],
    ) -> crate::graph::projection_dag::ProjectionBlock {
        let mut block = crate::graph::projection_dag::ProjectionBlock {
            block_id: BlockId(0),
            local_variables: local_variables.into_iter().collect(),
            relation_ids: compressed.relation_order.clone(),
            exported_variables: exported_variables.into_iter().collect(),
            child_block_ids: Vec::new(),
            parent_block_id: None,
            authorization_hash: hash_sequence("tmp", &[]),
            duplication_certificates: Vec::new(),
            block_hash: hash_sequence("test-block", &[]),
        };
        block.authorization_hash =
            crate::graph::projection_dag::authorize_block_relations(&block, compressed);
        block
    }

    fn child_projection_message(
        compressed: &crate::preprocess::compression::CompressedSystemQ,
        relation: crate::types::polynomial::SparsePolynomialQ,
    ) -> ProjectionMessage {
        let certificate_hash = hash_sequence("test-child-certificate", &[relation.hash.0.to_vec()]);
        let mut message = ProjectionMessage {
            package_id: PackageId(700),
            block_id: BlockId(7),
            kernel_kind: KernelKind::TargetRelationSearch,
            source_relation_ids: Vec::new(),
            eliminated_variables: Vec::new(),
            exported_variables: vec![compressed.target],
            relation_generators: vec![relation],
            representation: MessageRepresentation::GeneratorSet,
            projection_strength: ProjectionStrength::CandidateCoverStrong,
            certificate: KernelCertificate::synthetic_for_tests(certificate_hash),
            compression_trace: compressed.compression_trace.clone(),
            cost_trace: ProjectionCostTrace::default(),
            package_hash: hash_sequence("projection-message-initial", &[]),
        };
        message.package_hash = hash_sequence(
            "test-child-message",
            &[message.relation_generators[0].hash.0.to_vec()],
        );
        message
    }

    fn execute_local_groebner_stage_for_test<const N: usize, const M: usize>(
        compressed: crate::preprocess::compression::CompressedSystemQ,
        local_variables: [VariableId; N],
        exported_variables: [VariableId; M],
    ) -> (ProjectionMessage, KernelContext) {
        let block = test_block(&compressed, local_variables, exported_variables);
        let mut solver_ctx = new_context(SolverOptions::default());
        let mut kctx = KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };
        let kernel = UniversalTargetEliminationKernel;
        let admission = kernel.admit(&kctx.block, &kctx);
        let plan = kernel.plan(&admission, &kctx, &solver_ctx).unwrap();
        let stages = build_stage_plans(&plan).unwrap();
        let stage_index = stages
            .iter()
            .position(|stage| stage.strategy == UniversalStrategy::LocalGroebnerEliminationToKeepZ)
            .expect("Universal plan must contain local Groebner stage");
        let failed_strategy_hashes = stages
            .iter()
            .take(stage_index)
            .map(|stage| stage.stage_hash)
            .collect::<Vec<_>>();
        let executed_failed_strategy_hashes = stages
            .iter()
            .take(stage_index)
            .filter(|stage| stage.enabled && stage.cost_class != RouteCostClass::CostProhibited)
            .map(|stage| stage.stage_hash)
            .collect::<Vec<_>>();
        let stage = stages[stage_index].clone();
        let message = execute_universal_stage_with_solver_ctx(
            &stage,
            &mut kctx,
            &mut solver_ctx,
            &plan,
            &failed_strategy_hashes,
            &executed_failed_strategy_hashes,
        )
        .unwrap();
        (message, kctx)
    }
}
