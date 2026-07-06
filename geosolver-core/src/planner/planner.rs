use crate::graph::projection_dag::{validate_projection_dag, TargetProjectionDAG};
use crate::kernels::traits::KernelKind;
use crate::planner::admission::{
    collect_kernel_admissions, KernelAdmission, KernelAdmissionStatus,
};
use crate::planner::cost_model::estimate_kernel_cost_for_admission;
use crate::planner::kernel_plan::KernelPlan;
use crate::planner::ladder::build_declared_ladder;
use crate::planner::probes::run_cost_probes;
use crate::planner::relation_schedule::{
    dense_relation_search_decline_reason, estimate_dense_relation_search_schedule,
    estimate_sparse_relation_search_schedule,
};
use crate::preprocess::compression::CompressedSystemQ;
use crate::problem::context::SolverContext;
use crate::result::diagnostics::DiagnosticRecord;
use crate::result::status::{AlgebraicReason, FailureKind, SolverError, SolverErrorKind, StageId};

pub fn plan_all_blocks(
    dag: &TargetProjectionDAG,
    system: &CompressedSystemQ,
    ctx: &mut SolverContext,
) -> Result<Vec<KernelPlan>, SolverError> {
    validate_projection_dag(dag, system)?;
    let mut blocks = dag.blocks.iter().collect::<Vec<_>>();
    blocks.sort_by_key(|block| postorder_key(dag, block.block_id));
    let mut plans = Vec::new();
    for block in blocks {
        if block.relation_ids.is_empty() {
            continue;
        }
        let probes = run_cost_probes(block, system, ctx);
        let admissions = collect_kernel_admissions(block, system, &probes, ctx);
        let cost_estimates = admissions
            .iter()
            .map(|admission| {
                estimate_kernel_cost_for_admission(
                    block,
                    system,
                    admission.kind,
                    &probes,
                    admission.execution_plan.as_ref(),
                )
            })
            .collect::<Vec<_>>();
        record_kernel_route_admission_diagnostics(block, system, &admissions, &cost_estimates, ctx);
        record_dense_relation_search_admission_diagnostics(block, system, &admissions, ctx);
        let ladder =
            build_declared_ladder(&admissions, &cost_estimates, &ctx.options.kernel_priority);
        if ladder.is_empty() {
            return Err(SolverError {
                target: Some(system.target),
                kind: SolverErrorKind::Failure(FailureKind::AlgorithmicHardCase {
                    stage: StageId("PlanProjectionMessages".to_owned()),
                    reason: AlgebraicReason(
                        "no admitted production projection kernel for block".to_owned(),
                    ),
                    minimal_block_hash: block.block_hash,
                }),
            });
        }
        let plan = KernelPlan::new(block.block_id, ladder, admissions, cost_estimates)?;
        plans.push(plan);
    }
    Ok(plans)
}

fn record_kernel_route_admission_diagnostics(
    block: &crate::graph::projection_dag::ProjectionBlock,
    system: &CompressedSystemQ,
    admissions: &[KernelAdmission],
    cost_estimates: &[crate::planner::cost_model::KernelCostEstimate],
    ctx: &mut SolverContext,
) {
    for admission in admissions {
        let cost = cost_estimates
            .iter()
            .find(|cost| cost.kernel_kind == admission.kind);
        let mut diagnostic = DiagnosticRecord::new(
            "KernelRouteTrace",
            format!(
                "kernel={:?} block_id={} admission_status={}",
                admission.kind,
                admission.block_id.0,
                admission_status_name(&admission.status)
            ),
            Some(StageId("PlanProjectionMessages".to_owned())),
        );
        diagnostic
            .details
            .insert("kernel_kind".to_owned(), format!("{:?}", admission.kind));
        diagnostic
            .details
            .insert("block_id".to_owned(), admission.block_id.0.to_string());
        diagnostic.details.insert(
            "admission_status".to_owned(),
            admission_status_name(&admission.status).to_owned(),
        );
        diagnostic.details.insert(
            "admission_hash".to_owned(),
            format!("{:?}", admission.admission_hash),
        );
        diagnostic.details.insert(
            "exported_variables".to_owned(),
            admission.exported_variables.len().to_string(),
        );
        diagnostic.details.insert(
            "eliminated_variables".to_owned(),
            admission.eliminated_variables.len().to_string(),
        );
        if let Some(plan) = &admission.execution_plan {
            diagnostic
                .details
                .insert("plan_hash".to_owned(), format!("{:?}", plan.plan_hash));
            diagnostic.details.insert(
                "algebraic_work_estimate_hash".to_owned(),
                format!("{:?}", plan.algebraic_work_estimate.estimate_hash),
            );
            diagnostic.details.insert(
                "route_budget_hash".to_owned(),
                format!("{:?}", plan.route_budget.budget_hash),
            );
            diagnostic.details.insert(
                "predicted_work_units".to_owned(),
                plan.algebraic_work_estimate
                    .predicted_work_units
                    .0
                    .to_string(),
            );
            if let Some(terms) = plan.algebraic_work_estimate.predicted_intermediate_terms {
                diagnostic.details.insert(
                    "predicted_intermediate_terms".to_owned(),
                    terms.0.to_string(),
                );
            }
            if let Some(terms) = plan.algebraic_work_estimate.predicted_output_terms {
                diagnostic
                    .details
                    .insert("predicted_output_terms".to_owned(), terms.0.to_string());
            }
            diagnostic.details.insert(
                "route_budget_max_work_units".to_owned(),
                plan.route_budget.max_work_units.0.to_string(),
            );
            diagnostic.details.insert(
                "route_budget_max_intermediate_terms".to_owned(),
                plan.route_budget.max_intermediate_terms.to_string(),
            );
            diagnostic.details.insert(
                "route_budget_max_output_terms".to_owned(),
                plan.route_budget.max_output_terms.to_string(),
            );
        }
        if let Some(cost) = cost {
            diagnostic
                .details
                .insert("cost_class".to_owned(), format!("{:?}", cost.cost_class));
            diagnostic
                .details
                .insert("estimated_rows".to_owned(), cost.matrix_rows.to_string());
            diagnostic
                .details
                .insert("estimated_cols".to_owned(), cost.matrix_cols.to_string());
            diagnostic.details.insert(
                "estimated_rank".to_owned(),
                cost.quotient_rank_estimate.to_string(),
            );
            diagnostic.details.insert(
                "cost_estimate_hash".to_owned(),
                format!("{:?}", cost.estimate_hash),
            );
            diagnostic.details.insert(
                "dominant_work_estimate_hash".to_owned(),
                format!("{:?}", cost.algebraic_work_estimate.estimate_hash),
            );
        }
        match &admission.status {
            KernelAdmissionStatus::Declined { reason } => {
                diagnostic
                    .details
                    .insert("decline_reason".to_owned(), reason.clone());
            }
            KernelAdmissionStatus::CostProhibited {
                reason,
                estimate_hash,
            } => {
                diagnostic
                    .details
                    .insert("decline_reason".to_owned(), reason.clone());
                diagnostic.details.insert(
                    "preflight_estimate_hash".to_owned(),
                    format!("{:?}", estimate_hash),
                );
                diagnostic
                    .details
                    .insert("cost_class".to_owned(), "CostProhibited".to_owned());
            }
            KernelAdmissionStatus::PlanProbeFailed {
                reason,
                constructed_object_hash,
            } => {
                diagnostic
                    .details
                    .insert("decline_reason".to_owned(), reason.clone());
                diagnostic.details.insert(
                    "constructed_object_hash".to_owned(),
                    format!("{:?}", constructed_object_hash),
                );
            }
            KernelAdmissionStatus::Admitted => {}
        }
        if admission.kind == KernelKind::TargetRelationSearch {
            insert_dense_preflight_details(block, system, ctx, &mut diagnostic);
        }
        ctx.diagnostics.push(diagnostic);
    }
}

fn admission_status_name(status: &KernelAdmissionStatus) -> &'static str {
    match status {
        KernelAdmissionStatus::Admitted => "Admitted",
        KernelAdmissionStatus::Declined { .. } => "Declined",
        KernelAdmissionStatus::CostProhibited { .. } => "CostProhibited",
        KernelAdmissionStatus::PlanProbeFailed { .. } => "PlanProbeFailed",
    }
}

fn insert_dense_preflight_details(
    block: &crate::graph::projection_dag::ProjectionBlock,
    system: &CompressedSystemQ,
    ctx: &SolverContext,
    diagnostic: &mut DiagnosticRecord,
) {
    let relation_polys = block
        .relation_ids
        .iter()
        .filter_map(|id| system.relations.iter().find(|relation| relation.id == *id))
        .map(|relation| relation.polynomial.clone())
        .collect::<Vec<_>>();
    let eliminated_variables = block
        .local_variables
        .difference(&block.exported_variables)
        .copied()
        .collect::<Vec<_>>();
    let exported_variables = block.exported_variables.iter().copied().collect::<Vec<_>>();
    let preflight = estimate_dense_relation_search_schedule(
        &relation_polys,
        &eliminated_variables,
        &exported_variables,
        &ctx.options,
    );
    diagnostic.details.insert(
        "preflight_estimate_hash".to_owned(),
        format!("{:?}", preflight.preflight_hash),
    );
    diagnostic.details.insert(
        "planned_stage_count".to_owned(),
        preflight.planned_stage_count.display_value(),
    );
    diagnostic.details.insert(
        "materialization_allowed".to_owned(),
        preflight.materialization_allowed.to_string(),
    );
    if let Some(reason) = preflight.first_prohibition_reason() {
        diagnostic
            .details
            .insert("preflight_decline_reason".to_owned(), reason.to_owned());
    }
    let sparse_preflight = estimate_sparse_relation_search_schedule(
        &relation_polys,
        &eliminated_variables,
        &exported_variables,
        &ctx.options,
    );
    diagnostic.details.insert(
        "sparse_footprint_feasible".to_owned(),
        sparse_preflight.feasible.to_string(),
    );
    diagnostic.details.insert(
        "sparse_preflight_hash".to_owned(),
        format!("{:?}", sparse_preflight.preflight_hash),
    );
    diagnostic.details.insert(
        "sparse_matrix_rows".to_owned(),
        sparse_preflight.matrix_rows.to_string(),
    );
    diagnostic.details.insert(
        "sparse_matrix_cols".to_owned(),
        sparse_preflight.matrix_cols.to_string(),
    );
    if let Some(reason) = sparse_preflight.first_prohibition_reason() {
        diagnostic.details.insert(
            "sparse_preflight_decline_reason".to_owned(),
            reason.to_owned(),
        );
    }
}

fn record_dense_relation_search_admission_diagnostics(
    block: &crate::graph::projection_dag::ProjectionBlock,
    system: &CompressedSystemQ,
    admissions: &[KernelAdmission],
    ctx: &mut SolverContext,
) {
    let Some(reason) = admissions
        .iter()
        .find(|admission| admission.kind == KernelKind::TargetRelationSearch)
        .map(|admission| &admission.status)
        .and_then(cost_prohibited_or_declined_reason)
    else {
        return;
    };
    if !reason.contains("CostProhibitedDenseRoute") {
        return;
    }
    let relation_polys = block
        .relation_ids
        .iter()
        .filter_map(|id| system.relations.iter().find(|relation| relation.id == *id))
        .map(|relation| relation.polynomial.clone())
        .collect::<Vec<_>>();
    let eliminated_variables = block
        .local_variables
        .difference(&block.exported_variables)
        .copied()
        .collect::<Vec<_>>();
    let exported_variables = block.exported_variables.iter().copied().collect::<Vec<_>>();
    let preflight = estimate_dense_relation_search_schedule(
        &relation_polys,
        &eliminated_variables,
        &exported_variables,
        &ctx.options,
    );
    let mut diagnostic = DiagnosticRecord::new(
        "CostProhibitedDenseRoute",
        dense_relation_search_decline_reason(&preflight),
        Some(StageId("PlanProjectionMessages".to_owned())),
    );
    diagnostic
        .details
        .insert("kernel".to_owned(), "TargetRelationSearch".to_owned());
    diagnostic
        .details
        .insert("route".to_owned(), "DenseTotalDegree".to_owned());
    diagnostic
        .details
        .insert("decision".to_owned(), "CostProhibitedDenseRoute".to_owned());
    diagnostic
        .details
        .insert("block_id".to_owned(), block.block_id.0.to_string());
    diagnostic.details.insert(
        "stage_count".to_owned(),
        preflight.planned_stage_count.display_value(),
    );
    diagnostic.details.insert(
        "materialized_stage_cap".to_owned(),
        preflight.caps.max_materialized_stages.to_string(),
    );
    diagnostic.details.insert(
        "matrix_col_cap".to_owned(),
        preflight.caps.max_matrix_cols.to_string(),
    );
    diagnostic.details.insert(
        "matrix_row_cap".to_owned(),
        preflight.caps.max_matrix_rows.to_string(),
    );
    diagnostic.details.insert(
        "memory_cap_bytes".to_owned(),
        preflight.caps.max_estimated_memory_bytes.to_string(),
    );
    if let Some(stage) = preflight.stage_estimates.first() {
        diagnostic.details.insert(
            "first_export_degree".to_owned(),
            stage.export_degree.to_string(),
        );
        diagnostic.details.insert(
            "estimated_matrix_cols".to_owned(),
            stage.estimated_matrix_cols.display_value(),
        );
        diagnostic.details.insert(
            "estimated_rows".to_owned(),
            stage.estimated_rows_upper_bound.display_value(),
        );
        diagnostic.details.insert(
            "estimated_memory_bytes".to_owned(),
            stage.estimated_memory_bytes_upper_bound.display_value(),
        );
    }
    if let Some(stage) = preflight.first_prohibited_stage {
        diagnostic
            .details
            .insert("first_prohibited_stage".to_owned(), stage.to_string());
    }
    ctx.diagnostics.push(diagnostic);
}

fn cost_prohibited_or_declined_reason(status: &KernelAdmissionStatus) -> Option<&str> {
    match status {
        KernelAdmissionStatus::Declined { reason } => Some(reason),
        KernelAdmissionStatus::CostProhibited { reason, .. } => Some(reason),
        _ => None,
    }
}

fn postorder_key(
    dag: &TargetProjectionDAG,
    block_id: crate::types::ids::BlockId,
) -> (usize, crate::types::ids::BlockId) {
    (block_depth_from_root(dag, block_id), block_id)
}

fn block_depth_from_root(dag: &TargetProjectionDAG, block_id: crate::types::ids::BlockId) -> usize {
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
    usize::MAX - depth
}

#[cfg(test)]
mod tests {
    use crate::graph::hypergraph::build_relation_variable_hypergraph;
    use crate::graph::influence::build_target_influence_graph;
    use crate::graph::projection_dag::build_target_projection_dag;
    use crate::graph::separators::CostModel;
    use crate::graph::tree_decomposition::build_target_rooted_decomposition;
    use crate::graph::weighted_primal::build_weighted_primal_graph;
    use crate::kernels::traits::KernelKind;
    use crate::planner::admission::all_planner_kernel_kinds;
    use crate::planner::kernel_plan::require_declared_kernel_plan;
    use crate::planner::relation_schedule::build_dense_relation_search_schedule;
    use crate::preprocess::compression::CompressionState;
    use crate::problem::canonicalize::canonicalize_system;
    use crate::problem::context::new_context;
    use crate::problem::input::make_problem;
    use crate::problem::validate::validate_input;
    use crate::result::status::SolverStatus;
    use crate::solver::options::SolverOptions;
    use crate::types::ids::VariableId;
    use crate::types::polynomial::{poly_add, poly_mul, poly_sub, variable_poly};

    use super::plan_all_blocks;

    #[test]
    fn p6_planning_is_deterministic_and_declares_universal_last() {
        let (compressed, dag) = one_large_block_case();
        let mut ctx_a = new_context(SolverOptions::default());
        let mut ctx_b = new_context(SolverOptions::default());

        let plans_a = plan_all_blocks(&dag, &compressed, &mut ctx_a).unwrap();
        let plans_b = plan_all_blocks(&dag, &compressed, &mut ctx_b).unwrap();

        assert_eq!(plans_a.len(), 1);
        assert_eq!(plans_a, plans_b);

        let plan = &plans_a[0];
        assert_eq!(plan.admissions.len(), all_planner_kernel_kinds().len());
        assert_eq!(
            plan.declared_ladder.last().unwrap().kernel_kind,
            KernelKind::UniversalTargetElimination
        );
        assert_eq!(plan.selected_first, plan.declared_ladder[0].kernel_kind);
        assert!(plan.admissions.iter().any(|admission| admission.kind
            == KernelKind::UniversalTargetElimination
            && admission.is_admitted()));

        let relation_search_plan =
            require_declared_kernel_plan(plan, KernelKind::TargetRelationSearch, plan.plan_hash)
                .unwrap();
        let expected_schedule = build_dense_relation_search_schedule(
            &compressed
                .relations
                .iter()
                .map(|relation| relation.polynomial.clone())
                .collect::<Vec<_>>(),
            &relation_search_plan.eliminated_variables,
            &relation_search_plan.exported_variables,
            &ctx_a.options,
        );
        assert_eq!(
            relation_search_plan
                .support_plan
                .dense_relation_search_schedule
                .as_ref()
                .unwrap()
                .schedule_hash,
            expected_schedule.schedule_hash
        );
    }

    #[test]
    fn p6_hidden_fallback_is_rejected_when_kernel_absent_from_declared_ladder() {
        let (compressed, dag) = one_large_block_case();
        let mut ctx = new_context(SolverOptions::default());
        let mut plan = plan_all_blocks(&dag, &compressed, &mut ctx)
            .unwrap()
            .remove(0);
        plan.declared_ladder
            .retain(|entry| entry.kernel_kind != KernelKind::TargetRelationSearch);

        let err =
            require_declared_kernel_plan(&plan, KernelKind::TargetRelationSearch, plan.plan_hash)
                .unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::ImplementationBug);
    }

    #[test]
    fn p6_declared_ladder_rejects_execution_plan_field_tampering() {
        let (compressed, dag) = one_large_block_case();
        let mut ctx = new_context(SolverOptions::default());
        let plan = plan_all_blocks(&dag, &compressed, &mut ctx)
            .unwrap()
            .remove(0);
        let original_hash = plan.plan_hash;
        let kernel_kind = plan.declared_ladder[0].kernel_kind;

        let mut exported_tampered = plan.clone();
        exported_tampered.declared_ladder[0]
            .exported_variables
            .push(VariableId(99));
        assert_eq!(
            require_declared_kernel_plan(&exported_tampered, kernel_kind, original_hash)
                .unwrap_err()
                .public_status(),
            SolverStatus::ImplementationBug
        );

        let mut support_tampered = plan.clone();
        support_tampered.declared_ladder[0]
            .support_plan
            .degree_bound = support_tampered.declared_ladder[0]
            .support_plan
            .degree_bound
            .saturating_add(1);
        assert_eq!(
            require_declared_kernel_plan(&support_tampered, kernel_kind, original_hash)
                .unwrap_err()
                .public_status(),
            SolverStatus::ImplementationBug
        );

        let mut resource_tampered = plan.clone();
        resource_tampered.declared_ladder[0]
            .resource_bounds
            .max_matrix_rows = Some(usize::MAX);
        assert_eq!(
            require_declared_kernel_plan(&resource_tampered, kernel_kind, original_hash)
                .unwrap_err()
                .public_status(),
            SolverStatus::ImplementationBug
        );

        let mut failure_tampered = plan;
        failure_tampered.declared_ladder[0]
            .failure_behavior
            .allowed_statuses
            .clear();
        assert_eq!(
            require_declared_kernel_plan(&failure_tampered, kernel_kind, original_hash)
                .unwrap_err()
                .public_status(),
            SolverStatus::ImplementationBug
        );
    }

    fn one_large_block_case() -> (
        crate::preprocess::compression::CompressedSystemQ,
        crate::graph::projection_dag::TargetProjectionDAG,
    ) {
        let t = VariableId(0);
        let x = VariableId(1);
        let y = VariableId(2);
        let relations = vec![
            poly_sub(
                &poly_mul(&variable_poly(x), &variable_poly(x)),
                &variable_poly(t),
            ),
            poly_add(
                &poly_mul(&variable_poly(x), &variable_poly(y)),
                &variable_poly(t),
            ),
        ];
        let canonical = canonicalize_system(
            validate_input(make_problem(vec![t, x, y], t, relations, Vec::new())).unwrap(),
        )
        .unwrap();
        let compressed = CompressionState::from_system(canonical).to_compressed_system();
        let h = build_relation_variable_hypergraph(&compressed);
        let influence = build_target_influence_graph(&h, t);
        let g = build_weighted_primal_graph(&compressed, &influence);
        let tree = build_target_rooted_decomposition(&g, t, &CostModel::default());
        let dag = build_target_projection_dag(&compressed, &influence, &tree).unwrap();
        (compressed, dag)
    }
}
