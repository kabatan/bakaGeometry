use serde::{Deserialize, Serialize};

use crate::kernels::traits::KernelKind;
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
        }
    }
}
