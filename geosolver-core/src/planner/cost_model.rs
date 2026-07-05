use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::graph::projection_dag::ProjectionBlock;
use crate::kernels::traits::KernelKind;
use crate::planner::probes::ProbeResults;
use crate::types::hash::{hash_sequence, Hash};
use crate::types::rational::{int_q, RationalQ};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CostModelWeights {
    pub matrix_size_weight: RationalQ,
    pub quotient_rank_weight: RationalQ,
    pub coefficient_height_weight: RationalQ,
    pub separator_degree_weight: RationalQ,
    pub certificate_cost_weight: RationalQ,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KernelCostEstimate {
    pub block_id: crate::types::ids::BlockId,
    pub kernel_kind: KernelKind,
    pub matrix_rows: usize,
    pub matrix_cols: usize,
    pub quotient_rank_estimate: usize,
    pub coefficient_height_bits: usize,
    pub certificate_cost_units: usize,
    pub deterministic_score: usize,
    pub estimate_hash: Hash,
}

impl Default for CostModelWeights {
    fn default() -> Self {
        Self {
            matrix_size_weight: int_q(1),
            quotient_rank_weight: int_q(1),
            coefficient_height_weight: int_q(1),
            separator_degree_weight: int_q(1),
            certificate_cost_weight: int_q(1),
        }
    }
}

pub fn estimate_kernel_cost(
    block: &ProjectionBlock,
    kernel: KernelKind,
    probes: &ProbeResults,
) -> KernelCostEstimate {
    let base_rows = probes.local_macaulay_size.template_estimate.row_count;
    let base_cols = probes.local_macaulay_size.template_estimate.column_count;
    let quotient_rank_estimate = probes.modular_rank.rank_estimate.estimated_rank;
    let coefficient_height_bits = probes.coefficient_growth.projected_height_bits;
    let certificate_cost_units = certificate_cost(kernel);
    let penalty = kernel_order_penalty(kernel);
    let deterministic_score = base_rows
        .saturating_mul(base_cols)
        .saturating_add(quotient_rank_estimate)
        .saturating_add(coefficient_height_bits)
        .saturating_add(certificate_cost_units)
        .saturating_add(penalty);
    let estimate_hash = hash_sequence(
        "kernel-cost-estimate",
        &[
            block.block_id.0.to_be_bytes().to_vec(),
            format!("{kernel:?}").into_bytes(),
            base_rows.to_be_bytes().to_vec(),
            base_cols.to_be_bytes().to_vec(),
            quotient_rank_estimate.to_be_bytes().to_vec(),
            coefficient_height_bits.to_be_bytes().to_vec(),
            deterministic_score.to_be_bytes().to_vec(),
        ],
    );
    KernelCostEstimate {
        block_id: block.block_id,
        kernel_kind: kernel,
        matrix_rows: base_rows,
        matrix_cols: base_cols,
        quotient_rank_estimate,
        coefficient_height_bits,
        certificate_cost_units,
        deterministic_score,
        estimate_hash,
    }
}

pub fn compare_cost(a: &KernelCostEstimate, b: &KernelCostEstimate) -> Ordering {
    (
        a.deterministic_score,
        planner_kernel_order(a.kernel_kind),
        a.estimate_hash,
    )
        .cmp(&(
            b.deterministic_score,
            planner_kernel_order(b.kernel_kind),
            b.estimate_hash,
        ))
}

pub fn planner_kernel_order(kind: KernelKind) -> usize {
    match kind {
        KernelKind::TargetUnivariate => 0,
        KernelKind::LinearAffine => 1,
        KernelKind::TargetRelationSearch => 2,
        KernelKind::SparseResultantProjection => 3,
        KernelKind::TargetActionKrylov => 4,
        KernelKind::NormTraceProjection => 5,
        KernelKind::RegularChainProjection => 6,
        KernelKind::SpecializationInterpolation => 7,
        KernelKind::UniversalTargetElimination => 8,
    }
}

fn certificate_cost(kind: KernelKind) -> usize {
    match kind {
        KernelKind::TargetUnivariate => 1,
        KernelKind::LinearAffine => 2,
        KernelKind::TargetRelationSearch => 5,
        KernelKind::SparseResultantProjection => 8,
        KernelKind::TargetActionKrylov => 9,
        KernelKind::NormTraceProjection => 10,
        KernelKind::RegularChainProjection => 11,
        KernelKind::SpecializationInterpolation => 12,
        KernelKind::UniversalTargetElimination => 100,
    }
}

fn kernel_order_penalty(kind: KernelKind) -> usize {
    planner_kernel_order(kind).saturating_mul(10)
}
