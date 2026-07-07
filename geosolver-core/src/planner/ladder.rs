use crate::kernels::traits::KernelKind;
use crate::planner::admission::KernelAdmission;
use crate::planner::cost_model::{compare_cost, KernelCostEstimate};
use crate::planner::kernel_plan::KernelExecutionPlan;

pub fn build_declared_ladder(
    admissions: &[KernelAdmission],
    costs: &[KernelCostEstimate],
    preferred_order: &[KernelKind],
) -> Vec<KernelExecutionPlan> {
    let mut plans = admissions
        .iter()
        .filter_map(|admission| admission.execution_plan.clone())
        .filter(has_enforceable_route_budget)
        .collect::<Vec<_>>();
    plans.sort_by(|a, b| {
        let a_cost = costs
            .iter()
            .find(|cost| cost.kernel_kind == a.kernel_kind)
            .expect("cost estimate missing for admitted kernel");
        let b_cost = costs
            .iter()
            .find(|cost| cost.kernel_kind == b.kernel_kind)
            .expect("cost estimate missing for admitted kernel");
        compare_cost(a_cost, b_cost)
    });

    if !preferred_order.is_empty() {
        let mut prioritized = Vec::new();
        for preferred in preferred_order {
            if let Some(index) = plans.iter().position(|plan| plan.kernel_kind == *preferred) {
                prioritized.push(plans.remove(index));
            }
        }
        prioritized.extend(plans);
        plans = prioritized;
    }

    if !preferred_order.contains(&KernelKind::UniversalTargetElimination) {
        if let Some(index) = plans
            .iter()
            .position(|plan| plan.kernel_kind == KernelKind::UniversalTargetElimination)
        {
            let universal = plans.remove(index);
            plans.push(universal);
        }
    }
    plans
}

fn has_enforceable_route_budget(plan: &KernelExecutionPlan) -> bool {
    plan.algebraic_work_estimate.is_hash_current()
        && plan.route_budget.is_hash_current()
        && plan.route_budget.max_work_units.0 > 0
        && plan.route_budget.max_elapsed_steps > 0
        && plan.route_budget.max_input_terms_per_pair > 0
        && plan.route_budget.max_intermediate_terms > 0
        && plan.route_budget.max_output_terms > 0
        && plan.route_budget.max_keep_variables > 0
        && plan.route_budget.max_total_degree > 0
        && plan.route_budget.max_coefficient_height_bits > 0
}
