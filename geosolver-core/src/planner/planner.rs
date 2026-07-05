use crate::graph::projection_dag::{validate_projection_dag, TargetProjectionDAG};
use crate::planner::admission::collect_kernel_admissions;
use crate::planner::cost_model::estimate_kernel_cost;
use crate::planner::kernel_plan::KernelPlan;
use crate::planner::ladder::build_declared_ladder;
use crate::planner::probes::run_cost_probes;
use crate::preprocess::compression::CompressedSystemQ;
use crate::problem::context::SolverContext;
use crate::result::status::SolverError;

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
        let probes = run_cost_probes(block, system, ctx);
        let admissions = collect_kernel_admissions(block, system, &probes, ctx);
        let cost_estimates = admissions
            .iter()
            .map(|admission| estimate_kernel_cost(block, admission.kind, &probes))
            .collect::<Vec<_>>();
        let ladder = build_declared_ladder(&admissions, &cost_estimates);
        let plan = KernelPlan::new(block.block_id, ladder, admissions, cost_estimates)?;
        plans.push(plan);
    }
    Ok(plans)
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
    use crate::kernels::target_relation_search::build_dense_relation_search_schedule;
    use crate::kernels::traits::KernelKind;
    use crate::planner::admission::all_planner_kernel_kinds;
    use crate::planner::kernel_plan::require_declared_kernel_plan;
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
