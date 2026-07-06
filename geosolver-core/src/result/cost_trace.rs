use serde::{Deserialize, Serialize};

use crate::kernels::traits::KernelKind;
use crate::planner::algebraic_cost::SaturatingCount;
use crate::planner::kernel_plan::KernelExecutionPlan;
use crate::types::hash::Hash;
use crate::types::ids::BlockId;
use crate::types::rational::RationalQ;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GlobalCostTrace {
    pub total_variable_count: usize,
    pub total_relation_count: usize,
    pub total_monomial_count: usize,
    pub max_total_degree: usize,
    pub max_coefficient_height_bits: usize,
    pub max_block_width: usize,
    pub max_separator_width: usize,
    pub final_support_degree: Option<usize>,
    pub certificate_size: Option<usize>,
    pub block_traces: Vec<ProjectionCostTrace>,
    pub composition_trace: CompositionCostTrace,
    pub verification_trace: VerificationCostTrace,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionCostTrace {
    pub block_id: BlockId,
    pub kernel_kind: KernelKind,
    pub local_variable_count: usize,
    pub exported_variable_count: usize,
    pub local_relation_count: usize,
    pub local_monomial_count: usize,
    pub estimated_quotient_rank: Option<usize>,
    pub matrix_rows: Option<usize>,
    pub matrix_cols: Option<usize>,
    pub matrix_density: Option<RationalQ>,
    pub coefficient_height_before_bits: usize,
    pub coefficient_height_after_bits: usize,
    pub route_cost: Option<RouteCostTrace>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteCostTrace {
    pub algebraic_work_estimate_hash: Hash,
    pub route_budget_hash: Hash,
    pub predicted_work_units: SaturatingCount,
    pub predicted_intermediate_terms: Option<SaturatingCount>,
    pub predicted_output_terms: Option<SaturatingCount>,
    pub route_budget_max_work_units: SaturatingCount,
    pub route_budget_max_intermediate_terms: usize,
    pub route_budget_max_output_terms: usize,
    pub route_budget_max_keep_variables: usize,
    pub route_budget_max_total_degree: usize,
    pub route_budget_max_coefficient_height_bits: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CompositionCostTrace {
    pub relation_count_before: usize,
    pub relation_count_after: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct VerificationCostTrace {
    pub checked_relation_count: usize,
}

impl Default for ProjectionCostTrace {
    fn default() -> Self {
        ProjectionCostTrace {
            block_id: BlockId(0),
            kernel_kind: KernelKind::TargetUnivariate,
            local_variable_count: 0,
            exported_variable_count: 0,
            local_relation_count: 0,
            local_monomial_count: 0,
            estimated_quotient_rank: None,
            matrix_rows: None,
            matrix_cols: None,
            matrix_density: None,
            coefficient_height_before_bits: 0,
            coefficient_height_after_bits: 0,
            route_cost: None,
        }
    }
}

impl ProjectionCostTrace {
    pub fn route_cost_from_plan(plan: &KernelExecutionPlan) -> RouteCostTrace {
        RouteCostTrace {
            algebraic_work_estimate_hash: plan.algebraic_work_estimate.estimate_hash,
            route_budget_hash: plan.route_budget.budget_hash,
            predicted_work_units: plan.algebraic_work_estimate.predicted_work_units,
            predicted_intermediate_terms: plan.algebraic_work_estimate.predicted_intermediate_terms,
            predicted_output_terms: plan.algebraic_work_estimate.predicted_output_terms,
            route_budget_max_work_units: plan.route_budget.max_work_units,
            route_budget_max_intermediate_terms: plan.route_budget.max_intermediate_terms,
            route_budget_max_output_terms: plan.route_budget.max_output_terms,
            route_budget_max_keep_variables: plan.route_budget.max_keep_variables,
            route_budget_max_total_degree: plan.route_budget.max_total_degree,
            route_budget_max_coefficient_height_bits: plan.route_budget.max_coefficient_height_bits,
        }
    }
}
