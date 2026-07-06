use serde::{Deserialize, Serialize};

use crate::types::hash::{hash_sequence, Hash};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SaturatingCount(pub u128);

impl SaturatingCount {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1);
    pub const MAX: Self = Self(u128::MAX);

    pub fn from_usize(value: usize) -> Self {
        Self(value as u128)
    }

    pub fn from_u64(value: u64) -> Self {
        Self(value as u128)
    }

    pub fn saturating_add(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }

    pub fn saturating_mul(self, rhs: Self) -> Self {
        Self(self.0.saturating_mul(rhs.0))
    }

    pub fn exceeds_usize(self, limit: usize) -> bool {
        self.0 > limit as u128
    }

    pub fn as_usize_saturating(self) -> usize {
        self.0.min(usize::MAX as u128) as usize
    }
}

impl Default for SaturatingCount {
    fn default() -> Self {
        Self::ZERO
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlgebraicWorkEstimate {
    pub local_variable_count: usize,
    pub local_relation_count: usize,
    pub exported_variable_count: usize,
    pub input_term_count: usize,
    pub max_input_terms: usize,
    pub max_total_degree: usize,
    pub max_keep_variable_count: usize,
    pub matrix_rows: Option<usize>,
    pub matrix_cols: Option<usize>,
    pub quotient_rank_estimate: Option<usize>,
    pub predicted_output_terms: Option<SaturatingCount>,
    pub predicted_intermediate_terms: Option<SaturatingCount>,
    pub predicted_coefficient_height_bits: Option<SaturatingCount>,
    pub predicted_work_units: SaturatingCount,
    pub estimate_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteBudget {
    pub max_work_units: SaturatingCount,
    pub max_elapsed_steps: usize,
    pub max_input_terms_per_pair: usize,
    pub max_intermediate_terms: usize,
    pub max_output_terms: usize,
    pub max_keep_variables: usize,
    pub max_total_degree: usize,
    pub max_coefficient_height_bits: usize,
    pub budget_hash: Hash,
}

impl AlgebraicWorkEstimate {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        local_variable_count: usize,
        local_relation_count: usize,
        exported_variable_count: usize,
        input_term_count: usize,
        max_input_terms: usize,
        max_total_degree: usize,
        max_keep_variable_count: usize,
        matrix_rows: Option<usize>,
        matrix_cols: Option<usize>,
        quotient_rank_estimate: Option<usize>,
        predicted_output_terms: Option<SaturatingCount>,
        predicted_intermediate_terms: Option<SaturatingCount>,
        predicted_coefficient_height_bits: Option<SaturatingCount>,
        predicted_work_units: SaturatingCount,
    ) -> Self {
        let mut estimate = Self {
            local_variable_count,
            local_relation_count,
            exported_variable_count,
            input_term_count,
            max_input_terms,
            max_total_degree,
            max_keep_variable_count,
            matrix_rows,
            matrix_cols,
            quotient_rank_estimate,
            predicted_output_terms,
            predicted_intermediate_terms,
            predicted_coefficient_height_bits,
            predicted_work_units,
            estimate_hash: hash_sequence("algebraic-work-estimate", &[]),
        };
        estimate.estimate_hash = algebraic_work_estimate_hash(&estimate);
        estimate
    }

    pub fn conservative_plan_shape(
        local_variable_count: usize,
        local_relation_count: usize,
        exported_variable_count: usize,
        matrix_rows: Option<usize>,
        matrix_cols: Option<usize>,
        quotient_rank_estimate: Option<usize>,
        max_total_degree: usize,
    ) -> Self {
        let matrix_work = matrix_rows
            .unwrap_or(1)
            .saturating_mul(matrix_cols.unwrap_or(1));
        let structural_work = local_variable_count
            .saturating_mul(local_relation_count.max(1))
            .saturating_add(exported_variable_count);
        let predicted_work_units =
            SaturatingCount::from_usize(matrix_work.saturating_add(structural_work));
        Self::new(
            local_variable_count,
            local_relation_count,
            exported_variable_count,
            0,
            0,
            max_total_degree,
            exported_variable_count,
            matrix_rows,
            matrix_cols,
            quotient_rank_estimate,
            None,
            None,
            None,
            predicted_work_units,
        )
    }

    pub fn is_hash_current(&self) -> bool {
        self.estimate_hash == algebraic_work_estimate_hash(self)
    }
}

impl RouteBudget {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        max_work_units: SaturatingCount,
        max_elapsed_steps: usize,
        max_input_terms_per_pair: usize,
        max_intermediate_terms: usize,
        max_output_terms: usize,
        max_keep_variables: usize,
        max_total_degree: usize,
        max_coefficient_height_bits: usize,
    ) -> Self {
        let mut budget = Self {
            max_work_units,
            max_elapsed_steps,
            max_input_terms_per_pair,
            max_intermediate_terms,
            max_output_terms,
            max_keep_variables,
            max_total_degree,
            max_coefficient_height_bits,
            budget_hash: hash_sequence("route-budget", &[]),
        };
        budget.budget_hash = route_budget_hash(&budget);
        budget
    }

    pub fn from_estimate(estimate: &AlgebraicWorkEstimate) -> Self {
        let terms = estimate.max_input_terms.max(64);
        let intermediate = estimate
            .predicted_intermediate_terms
            .unwrap_or_else(|| SaturatingCount::from_usize(terms.saturating_mul(terms.max(1))))
            .as_usize_saturating()
            .max(terms);
        let output = estimate
            .predicted_output_terms
            .unwrap_or_else(|| SaturatingCount::from_usize(intermediate))
            .as_usize_saturating()
            .max(terms);
        let height = estimate
            .predicted_coefficient_height_bits
            .unwrap_or_else(|| SaturatingCount::from_usize(256))
            .as_usize_saturating()
            .max(256);
        let max_work_units = estimate
            .predicted_work_units
            .saturating_mul(SaturatingCount::from_usize(4))
            .saturating_add(SaturatingCount::from_usize(1024));
        Self::new(
            max_work_units,
            estimate.local_variable_count.max(1),
            terms.max(1),
            intermediate,
            output,
            estimate
                .max_keep_variable_count
                .max(estimate.exported_variable_count)
                .max(1),
            estimate.max_total_degree.max(1),
            height,
        )
    }

    pub fn is_hash_current(&self) -> bool {
        self.budget_hash == route_budget_hash(self)
    }
}

pub fn algebraic_work_estimate_hash(estimate: &AlgebraicWorkEstimate) -> Hash {
    hash_sequence(
        "algebraic-work-estimate",
        &[
            estimate.local_variable_count.to_be_bytes().to_vec(),
            estimate.local_relation_count.to_be_bytes().to_vec(),
            estimate.exported_variable_count.to_be_bytes().to_vec(),
            estimate.input_term_count.to_be_bytes().to_vec(),
            estimate.max_input_terms.to_be_bytes().to_vec(),
            estimate.max_total_degree.to_be_bytes().to_vec(),
            estimate.max_keep_variable_count.to_be_bytes().to_vec(),
            optional_usize_bytes(estimate.matrix_rows),
            optional_usize_bytes(estimate.matrix_cols),
            optional_usize_bytes(estimate.quotient_rank_estimate),
            optional_count_bytes(estimate.predicted_output_terms),
            optional_count_bytes(estimate.predicted_intermediate_terms),
            optional_count_bytes(estimate.predicted_coefficient_height_bits),
            estimate.predicted_work_units.0.to_be_bytes().to_vec(),
        ],
    )
}

pub fn route_budget_hash(budget: &RouteBudget) -> Hash {
    hash_sequence(
        "route-budget",
        &[
            budget.max_work_units.0.to_be_bytes().to_vec(),
            budget.max_elapsed_steps.to_be_bytes().to_vec(),
            budget.max_input_terms_per_pair.to_be_bytes().to_vec(),
            budget.max_intermediate_terms.to_be_bytes().to_vec(),
            budget.max_output_terms.to_be_bytes().to_vec(),
            budget.max_keep_variables.to_be_bytes().to_vec(),
            budget.max_total_degree.to_be_bytes().to_vec(),
            budget.max_coefficient_height_bits.to_be_bytes().to_vec(),
        ],
    )
}

fn optional_usize_bytes(value: Option<usize>) -> Vec<u8> {
    value
        .map(|value| value.to_be_bytes().to_vec())
        .unwrap_or_else(|| vec![0xff])
}

fn optional_count_bytes(value: Option<SaturatingCount>) -> Vec<u8> {
    value
        .map(|value| value.0.to_be_bytes().to_vec())
        .unwrap_or_else(|| vec![0xff])
}
