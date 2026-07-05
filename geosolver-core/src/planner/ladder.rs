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

    let universal_is_preferred = preferred_order.contains(&KernelKind::UniversalTargetElimination);
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

    if !universal_is_preferred {
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
