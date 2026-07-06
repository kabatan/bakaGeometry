use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::graph::projection_dag::ProjectionBlock;
use crate::kernels::traits::KernelKind;
use crate::planner::algebraic_cost::{AlgebraicWorkEstimate, SaturatingCount};
use crate::planner::probes::ProbeResults;
use crate::preprocess::compression::CompressedSystemQ;
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::VariableId;
use crate::types::polynomial::{
    max_poly_coefficient_height_bits, poly_monomial_count, poly_total_degree, poly_variables,
    SparsePolynomialQ,
};
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
    pub cost_class: RouteCostClass,
    pub matrix_rows: usize,
    pub matrix_cols: usize,
    pub quotient_rank_estimate: usize,
    pub coefficient_height_bits: usize,
    pub certificate_cost_units: usize,
    pub algebraic_work_estimate: AlgebraicWorkEstimate,
    pub deterministic_score: usize,
    pub estimate_hash: Hash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RouteCostClass {
    PreferredCompact,
    Feasible,
    ExpensiveButAllowed,
    CostProhibited,
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
    system: &CompressedSystemQ,
    kernel: KernelKind,
    probes: &ProbeResults,
) -> KernelCostEstimate {
    let base_rows = probes.local_macaulay_size.template_estimate.row_count;
    let base_cols = probes.local_macaulay_size.template_estimate.column_count;
    let quotient_rank_estimate = probes.modular_rank.rank_estimate.estimated_rank;
    let coefficient_height_bits = probes.coefficient_growth.projected_height_bits;
    let certificate_cost_units = certificate_cost(kernel);
    let algebraic_work_estimate = estimate_kernel_algebraic_work(
        block,
        system,
        kernel,
        base_rows,
        base_cols,
        quotient_rank_estimate,
        coefficient_height_bits,
    );
    let penalty = kernel_order_penalty(kernel);
    let cost_class = classify_route_cost(
        kernel,
        base_rows,
        base_cols,
        quotient_rank_estimate,
        &algebraic_work_estimate,
    );
    let deterministic_score = base_rows
        .saturating_mul(base_cols)
        .saturating_add(quotient_rank_estimate)
        .saturating_add(coefficient_height_bits)
        .saturating_add(certificate_cost_units)
        .saturating_add(
            algebraic_work_estimate
                .predicted_work_units
                .as_usize_saturating(),
        )
        .saturating_add(
            algebraic_work_estimate
                .predicted_intermediate_terms
                .unwrap_or_default()
                .as_usize_saturating(),
        )
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
            algebraic_work_estimate.estimate_hash.0.to_vec(),
            format!("{cost_class:?}").into_bytes(),
            deterministic_score.to_be_bytes().to_vec(),
        ],
    );
    KernelCostEstimate {
        block_id: block.block_id,
        kernel_kind: kernel,
        cost_class,
        matrix_rows: base_rows,
        matrix_cols: base_cols,
        quotient_rank_estimate,
        coefficient_height_bits,
        certificate_cost_units,
        algebraic_work_estimate,
        deterministic_score,
        estimate_hash,
    }
}

pub fn compare_cost(a: &KernelCostEstimate, b: &KernelCostEstimate) -> Ordering {
    (
        a.cost_class,
        a.deterministic_score,
        planner_kernel_order(a.kernel_kind),
        a.estimate_hash,
    )
        .cmp(&(
            b.cost_class,
            b.deterministic_score,
            planner_kernel_order(b.kernel_kind),
            b.estimate_hash,
        ))
}

pub fn classify_route_cost(
    kind: KernelKind,
    matrix_rows: usize,
    matrix_cols: usize,
    quotient_rank_estimate: usize,
    estimate: &AlgebraicWorkEstimate,
) -> RouteCostClass {
    match kind {
        KernelKind::TargetUnivariate | KernelKind::LinearAffine => RouteCostClass::PreferredCompact,
        KernelKind::SparseResultantProjection
        | KernelKind::TargetRelationSearch
        | KernelKind::UniversalTargetElimination
            if dominant_expression_swell_is_prohibited(estimate) =>
        {
            RouteCostClass::CostProhibited
        }
        KernelKind::UniversalTargetElimination => RouteCostClass::ExpensiveButAllowed,
        _ if matrix_rows.saturating_mul(matrix_cols) > 1_000_000
            || quotient_rank_estimate > 50_000 =>
        {
            RouteCostClass::ExpensiveButAllowed
        }
        _ => RouteCostClass::Feasible,
    }
}

pub fn dominant_expression_swell_is_prohibited(estimate: &AlgebraicWorkEstimate) -> bool {
    estimate
        .predicted_intermediate_terms
        .is_some_and(|terms| terms.exceeds_usize(250_000))
        || estimate
            .predicted_output_terms
            .is_some_and(|terms| terms.exceeds_usize(250_000))
        || estimate.predicted_work_units.exceeds_usize(20_000_000)
        || estimate.max_keep_variable_count > 24
        || estimate.max_input_terms > 8_192
}

fn estimate_kernel_algebraic_work(
    block: &ProjectionBlock,
    system: &CompressedSystemQ,
    kind: KernelKind,
    matrix_rows: usize,
    matrix_cols: usize,
    quotient_rank_estimate: usize,
    coefficient_height_bits: usize,
) -> AlgebraicWorkEstimate {
    let local_variable_count = block.local_variables.len();
    let exported_variable_count = block.exported_variables.len();
    let local_relation_count = block.relation_ids.len();
    let relation_polys = block_relation_polys(block, system);
    let total_input_terms = relation_polys
        .iter()
        .map(poly_monomial_count)
        .sum::<usize>()
        .max(local_relation_count);
    let max_relation_terms = relation_polys
        .iter()
        .map(poly_monomial_count)
        .max()
        .unwrap_or(0)
        .max(1);
    let max_input_degree = relation_polys
        .iter()
        .map(|poly| poly_total_degree(poly) as usize)
        .max()
        .unwrap_or(1)
        .max(1);
    let input_height_bits = max_poly_coefficient_height_bits(&relation_polys)
        .max(coefficient_height_bits)
        .max(1);
    let max_keep_variable_count =
        route_keep_variable_count(block, kind, &relation_polys).max(exported_variable_count);
    let structural_terms = total_input_terms
        .saturating_add(local_relation_count.saturating_mul(local_variable_count.max(1)))
        .max(1);
    let matrix_work = matrix_rows.saturating_mul(matrix_cols);
    let rank_work = if quotient_rank_estimate == usize::MAX {
        50_001
    } else {
        quotient_rank_estimate
    };
    let route_multiplier = match kind {
        KernelKind::TargetUnivariate | KernelKind::LinearAffine => 1,
        KernelKind::TargetRelationSearch => 4,
        KernelKind::SparseResultantProjection => 8,
        KernelKind::TargetActionKrylov => 3,
        KernelKind::NormTraceProjection => 6,
        KernelKind::RegularChainProjection => 6,
        KernelKind::SpecializationInterpolation => 5,
        KernelKind::UniversalTargetElimination => 10,
    };
    let predicted_intermediate_terms = match kind {
        KernelKind::SparseResultantProjection => Some(sparse_resultant_growth_estimate(
            block,
            &relation_polys,
            matrix_rows,
            matrix_cols,
        )),
        KernelKind::TargetRelationSearch => Some(SaturatingCount::from_usize(matrix_work)),
        KernelKind::UniversalTargetElimination => Some(SaturatingCount::from_usize(
            matrix_work
                .saturating_add(structural_terms)
                .saturating_mul(max_keep_variable_count.max(1)),
        )),
        _ => None,
    };
    let sparse_resultant_route_work_units = if kind == KernelKind::SparseResultantProjection {
        sparse_resultant_route_work_estimate(block, &relation_polys)
    } else {
        None
    };
    let structural_work = SaturatingCount::from_usize(
        matrix_work
            .saturating_add(rank_work)
            .saturating_add(structural_terms)
            .saturating_mul(route_multiplier),
    );
    let predicted_work_units = match (kind, predicted_intermediate_terms) {
        (KernelKind::SparseResultantProjection, Some(intermediate_terms)) => intermediate_terms
            .saturating_mul(SaturatingCount::from_usize(matrix_work.max(1)))
            .saturating_add(SaturatingCount::from_usize(
                input_height_bits
                    .saturating_mul(matrix_rows.max(matrix_cols).max(1))
                    .saturating_add(total_input_terms),
            ))
            .max(sparse_resultant_route_work_units.unwrap_or_default())
            .max(structural_work),
        _ => structural_work,
    };
    AlgebraicWorkEstimate::new(
        local_variable_count,
        local_relation_count,
        exported_variable_count,
        total_input_terms,
        max_relation_terms,
        max_input_degree
            .saturating_add(matrix_rows.max(matrix_cols))
            .max(1),
        max_keep_variable_count.max(1),
        Some(matrix_rows),
        Some(matrix_cols),
        Some(quotient_rank_estimate),
        predicted_intermediate_terms,
        predicted_intermediate_terms,
        Some(SaturatingCount::from_usize(input_height_bits)),
        predicted_work_units,
    )
}

fn block_relation_polys(
    block: &ProjectionBlock,
    system: &CompressedSystemQ,
) -> Vec<SparsePolynomialQ> {
    block
        .relation_ids
        .iter()
        .filter_map(|id| {
            system
                .relations
                .iter()
                .find(|relation| relation.id == *id)
                .map(|relation| relation.polynomial.clone())
        })
        .collect()
}

fn route_keep_variable_count(
    block: &ProjectionBlock,
    kind: KernelKind,
    relation_polys: &[SparsePolynomialQ],
) -> usize {
    match kind {
        KernelKind::SparseResultantProjection => {
            let eliminated = block
                .local_variables
                .difference(&block.exported_variables)
                .copied()
                .collect::<Vec<_>>();
            sparse_pair_footprints(relation_polys, &eliminated)
                .into_iter()
                .map(|footprint| footprint.keep_variable_count)
                .max()
                .unwrap_or(block.exported_variables.len())
        }
        _ => block.exported_variables.len(),
    }
}

fn sparse_resultant_growth_estimate(
    block: &ProjectionBlock,
    relation_polys: &[SparsePolynomialQ],
    matrix_rows: usize,
    matrix_cols: usize,
) -> SaturatingCount {
    let eliminated = block
        .local_variables
        .difference(&block.exported_variables)
        .copied()
        .collect::<Vec<_>>();
    let pair_growth = sparse_pair_footprints(relation_polys, &eliminated)
        .into_iter()
        .map(|footprint| footprint.predicted_intermediate_terms)
        .max()
        .unwrap_or(SaturatingCount::ZERO);
    let matrix_growth = SaturatingCount::from_usize(
        matrix_rows
            .max(1)
            .saturating_mul(matrix_cols.max(1))
            .saturating_mul(
                relation_polys
                    .iter()
                    .map(poly_monomial_count)
                    .sum::<usize>()
                    .max(1),
            ),
    );
    pair_growth.max(matrix_growth)
}

fn sparse_resultant_route_work_estimate(
    block: &ProjectionBlock,
    relation_polys: &[SparsePolynomialQ],
) -> Option<SaturatingCount> {
    let eliminated = block
        .local_variables
        .difference(&block.exported_variables)
        .copied()
        .collect::<Vec<_>>();
    sparse_pair_footprints(relation_polys, &eliminated)
        .into_iter()
        .map(|footprint| footprint.route_work_units)
        .max()
}

#[derive(Debug, Clone, Copy)]
struct SparsePairFootprint {
    keep_variable_count: usize,
    predicted_intermediate_terms: SaturatingCount,
    route_work_units: SaturatingCount,
}

fn sparse_pair_footprints(
    relation_polys: &[SparsePolynomialQ],
    eliminated: &[VariableId],
) -> Vec<SparsePairFootprint> {
    let mut out = Vec::new();
    for eliminate in eliminated {
        for i in 0..relation_polys.len() {
            for j in (i + 1)..relation_polys.len() {
                let left_degree = degree_in_variable(&relation_polys[i], *eliminate);
                let right_degree = degree_in_variable(&relation_polys[j], *eliminate);
                if left_degree == 0 || right_degree == 0 {
                    continue;
                }
                let mut keep = poly_variables(&relation_polys[i]);
                keep.extend(poly_variables(&relation_polys[j]));
                keep.remove(eliminate);
                let left_terms = poly_monomial_count(&relation_polys[i]).max(1);
                let right_terms = poly_monomial_count(&relation_polys[j]).max(1);
                let template_dim = left_degree.saturating_add(right_degree).max(1);
                let matrix_area = template_dim.saturating_mul(template_dim).max(1);
                let predicted_intermediate_terms = SaturatingCount::from_usize(left_terms)
                    .saturating_mul(SaturatingCount::from_usize(right_terms))
                    .saturating_mul(SaturatingCount::from_usize(matrix_area))
                    .saturating_mul(SaturatingCount::from_usize(keep.len().max(1)));
                let input_height_bits = max_poly_coefficient_height_bits(&[
                    relation_polys[i].clone(),
                    relation_polys[j].clone(),
                ])
                .max(1);
                let coefficient_height_growth_bits = SaturatingCount::from_usize(input_height_bits)
                    .saturating_mul(SaturatingCount::from_usize(template_dim.max(1)))
                    .saturating_add(SaturatingCount::from_usize(
                        left_terms.saturating_add(right_terms),
                    ));
                let route_work_units = predicted_intermediate_terms
                    .saturating_mul(SaturatingCount::from_usize(matrix_area))
                    .saturating_add(coefficient_height_growth_bits);
                out.push(SparsePairFootprint {
                    keep_variable_count: keep.len(),
                    predicted_intermediate_terms,
                    route_work_units,
                });
            }
        }
    }
    out
}

fn degree_in_variable(poly: &SparsePolynomialQ, var: VariableId) -> usize {
    poly.terms
        .iter()
        .map(|term| {
            term.monomial
                .exponents
                .iter()
                .find(|(candidate, _)| *candidate == var)
                .map_or(0, |(_, exp)| *exp as usize)
        })
        .max()
        .unwrap_or(0)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::projection_dag::{authorize_block_relations, ProjectionBlock};
    use crate::planner::probes::run_cost_probes;
    use crate::preprocess::compression::CompressionState;
    use crate::problem::canonicalize::canonicalize_system;
    use crate::problem::context::new_context;
    use crate::problem::input::make_problem;
    use crate::problem::validate::validate_input;
    use crate::solver::options::SolverOptions;
    use crate::types::hash::hash_sequence;
    use crate::types::ids::BlockId;
    use crate::types::monomial::normalize_monomial;
    use crate::types::polynomial::{normalize_poly, SparsePolynomialQ, TermQ};
    use crate::types::rational::int_q;
    use std::collections::BTreeSet;

    fn synthetic_work(
        max_input_terms: usize,
        keep_variables: usize,
        predicted_intermediate_terms: usize,
    ) -> AlgebraicWorkEstimate {
        AlgebraicWorkEstimate::new(
            keep_variables.saturating_add(1),
            2,
            keep_variables,
            max_input_terms.saturating_mul(2),
            max_input_terms,
            3,
            keep_variables,
            Some(4),
            Some(4),
            Some(8),
            Some(SaturatingCount::from_usize(predicted_intermediate_terms)),
            Some(SaturatingCount::from_usize(predicted_intermediate_terms)),
            Some(SaturatingCount::from_usize(256)),
            SaturatingCount::from_usize(predicted_intermediate_terms.saturating_mul(4)),
        )
    }

    #[test]
    fn acr_p2_sparse_resultant_small_matrix_large_entries_is_cost_prohibited() {
        let work = synthetic_work(20_000, 18, 1_000_000);

        assert_eq!(
            classify_route_cost(KernelKind::SparseResultantProjection, 4, 4, 8, &work),
            RouteCostClass::CostProhibited
        );
        assert!(dominant_expression_swell_is_prohibited(&work));
    }

    #[test]
    fn acr_p2_compact_source_routes_are_not_prohibited_by_route_swell_guard() {
        let work = synthetic_work(20_000, 18, 1_000_000);

        assert_eq!(
            classify_route_cost(KernelKind::TargetUnivariate, 1, 1, 1, &work),
            RouteCostClass::PreferredCompact
        );
    }

    #[test]
    fn acr_p2_production_cost_estimate_uses_actual_polynomial_terms() {
        let target = VariableId(0);
        let eliminate = VariableId(20);
        let keep_variables = (0..12).map(VariableId).collect::<Vec<_>>();
        let mut variables = keep_variables.clone();
        variables.push(eliminate);
        let relations = vec![
            dense_linear_polynomial(eliminate, &keep_variables, 300),
            dense_linear_polynomial(eliminate, &keep_variables, 600),
        ];
        let canonical = canonicalize_system(
            validate_input(make_problem(variables, target, relations, Vec::new())).unwrap(),
        )
        .unwrap();
        let compressed = CompressionState::from_system(canonical).to_compressed_system();
        let mut block = ProjectionBlock {
            block_id: BlockId(0),
            local_variables: keep_variables
                .iter()
                .copied()
                .chain(std::iter::once(eliminate))
                .collect::<BTreeSet<_>>(),
            relation_ids: compressed.relation_order.clone(),
            exported_variables: keep_variables.iter().copied().collect::<BTreeSet<_>>(),
            child_block_ids: Vec::new(),
            parent_block_id: None,
            authorization_hash: hash_sequence("pending", &[]),
            duplication_certificates: Vec::new(),
            block_hash: hash_sequence("test-block", &[]),
        };
        block.authorization_hash = authorize_block_relations(&block, &compressed);
        let mut ctx = new_context(SolverOptions::default());
        let probes = run_cost_probes(&block, &compressed, &mut ctx);

        let cost = estimate_kernel_cost(
            &block,
            &compressed,
            KernelKind::SparseResultantProjection,
            &probes,
        );

        assert!(cost.algebraic_work_estimate.input_term_count >= 600);
        assert!(cost.algebraic_work_estimate.max_input_terms >= 300);
        assert!(cost
            .algebraic_work_estimate
            .predicted_intermediate_terms
            .unwrap()
            .exceeds_usize(250_000));
        assert_eq!(cost.cost_class, RouteCostClass::CostProhibited);
    }

    fn dense_linear_polynomial(
        eliminate: VariableId,
        keep_variables: &[VariableId],
        offset: usize,
    ) -> SparsePolynomialQ {
        let terms = (0..300)
            .map(|idx| {
                let mut exponents = vec![(eliminate, 1)];
                let code = idx + offset;
                for (bit, variable) in keep_variables.iter().enumerate() {
                    if (code >> bit) & 1 == 1 {
                        exponents.push((*variable, 1));
                    }
                }
                TermQ {
                    coeff: int_q(1),
                    monomial: normalize_monomial(exponents),
                }
            })
            .collect::<Vec<_>>();
        normalize_poly(SparsePolynomialQ {
            terms,
            hash: hash_sequence("poly", &[]),
        })
    }
}
