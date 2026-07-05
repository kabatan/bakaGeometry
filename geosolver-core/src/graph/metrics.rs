use serde::{Deserialize, Serialize};

use crate::graph::projection_dag::ProjectionBlock;
use crate::preprocess::compression::{
    max_coefficient_height_bits, max_total_degree, total_monomial_count, CompressedSystemQ,
};
use crate::types::hash::{hash_sequence, Hash};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuralMetrics {
    pub variable_count: usize,
    pub relation_count: usize,
    pub monomial_count: usize,
    pub max_total_degree: usize,
    pub separator_width: usize,
    pub coefficient_height_bits: usize,
    pub metrics_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RankEstimate {
    pub local_variable_count: usize,
    pub relation_count: usize,
    pub estimated_rank: usize,
    pub estimate_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateEstimate {
    pub row_count: usize,
    pub column_count: usize,
    pub nonzero_hint: usize,
    pub estimate_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeightEstimate {
    pub input_height_bits: usize,
    pub projected_height_bits: usize,
    pub estimate_hash: Hash,
}

pub fn structural_metrics(
    block: &ProjectionBlock,
    system: &CompressedSystemQ,
) -> StructuralMetrics {
    let relations = system
        .relations
        .iter()
        .filter(|relation| block.relation_ids.contains(&relation.id))
        .cloned()
        .collect::<Vec<_>>();
    let variable_count = block.local_variables.len();
    let relation_count = block.relation_ids.len();
    let monomial_count = total_monomial_count(&relations);
    let max_total_degree = max_total_degree(&relations);
    let separator_width = block.exported_variables.len();
    let coefficient_height_bits = max_coefficient_height_bits(&relations);
    let metrics_hash = hash_usizes(
        "structural-metrics",
        &[
            variable_count,
            relation_count,
            monomial_count,
            max_total_degree,
            separator_width,
            coefficient_height_bits,
        ],
    );
    StructuralMetrics {
        variable_count,
        relation_count,
        monomial_count,
        max_total_degree,
        separator_width,
        coefficient_height_bits,
        metrics_hash,
    }
}

pub fn estimate_local_quotient_rank(block: &ProjectionBlock) -> RankEstimate {
    let local_variable_count = block.local_variables.len();
    let relation_count = block.relation_ids.len();
    let base = local_variable_count.saturating_add(1);
    let estimated_rank = (0..relation_count.max(1))
        .fold(1_usize, |acc, _| acc.saturating_mul(base))
        .max(1);
    let estimate_hash = hash_usizes(
        "rank-estimate",
        &[local_variable_count, relation_count, estimated_rank],
    );
    RankEstimate {
        local_variable_count,
        relation_count,
        estimated_rank,
        estimate_hash,
    }
}

pub fn estimate_sparse_template_size(block: &ProjectionBlock) -> TemplateEstimate {
    let row_count = block.relation_ids.len().max(1);
    let column_count = block
        .local_variables
        .len()
        .saturating_add(block.exported_variables.len())
        .saturating_add(block.relation_ids.len())
        .max(1);
    let nonzero_hint = row_count.saturating_mul(column_count);
    let estimate_hash = hash_usizes(
        "sparse-template-estimate",
        &[row_count, column_count, nonzero_hint],
    );
    TemplateEstimate {
        row_count,
        column_count,
        nonzero_hint,
        estimate_hash,
    }
}

pub fn estimate_coefficient_growth(block: &ProjectionBlock) -> HeightEstimate {
    let input_height_bits = block.relation_ids.len().saturating_add(1);
    let projected_height_bits = input_height_bits.saturating_mul(
        block
            .local_variables
            .len()
            .saturating_add(block.exported_variables.len())
            .max(1),
    );
    let estimate_hash = hash_usizes(
        "coefficient-growth-estimate",
        &[input_height_bits, projected_height_bits],
    );
    HeightEstimate {
        input_height_bits,
        projected_height_bits,
        estimate_hash,
    }
}

fn hash_usizes(tag: &str, values: &[usize]) -> Hash {
    hash_sequence(
        tag,
        &values
            .iter()
            .map(|value| value.to_be_bytes().to_vec())
            .collect::<Vec<_>>(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::hypergraph::build_relation_variable_hypergraph;
    use crate::graph::influence::build_target_influence_graph;
    use crate::graph::projection_dag::build_target_projection_dag;
    use crate::graph::separators::CostModel;
    use crate::graph::tree_decomposition::build_target_rooted_decomposition;
    use crate::graph::weighted_primal::build_weighted_primal_graph;
    use crate::preprocess::compression::CompressionState;
    use crate::problem::canonicalize::canonicalize_system;
    use crate::problem::input::make_problem;
    use crate::problem::validate::validate_input;
    use crate::types::ids::VariableId;
    use crate::types::polynomial::{poly_add, poly_mul, variable_poly};

    #[test]
    fn structural_metrics_use_authorized_relations_only() {
        let t = VariableId(0);
        let x = VariableId(1);
        let relation = poly_add(
            &poly_mul(&variable_poly(t), &variable_poly(x)),
            &variable_poly(x),
        );
        let canonical = canonicalize_system(
            validate_input(make_problem(vec![t, x], t, vec![relation], Vec::new())).unwrap(),
        )
        .unwrap();
        let compressed = CompressionState::from_system(canonical).to_compressed_system();
        let h = build_relation_variable_hypergraph(&compressed);
        let influence = build_target_influence_graph(&h, t);
        let g = build_weighted_primal_graph(&compressed, &influence);
        let tree = build_target_rooted_decomposition(&g, t, &CostModel::default());
        let dag = build_target_projection_dag(&compressed, &influence, &tree).unwrap();
        let metrics = structural_metrics(&dag.blocks[0], &compressed);
        assert_eq!(metrics.relation_count, 1);
        assert_eq!(metrics.monomial_count, 2);
        assert_eq!(metrics.max_total_degree, 2);
        assert!(estimate_local_quotient_rank(&dag.blocks[0]).estimated_rank >= 1);
        assert!(estimate_sparse_template_size(&dag.blocks[0]).row_count >= 1);
        assert!(estimate_coefficient_growth(&dag.blocks[0]).projected_height_bits >= 1);
    }
}
