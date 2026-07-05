use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::graph::separators::{
    articulation_variable_candidates, min_fill_separator_candidates, score_separator, CostModel,
    SeparatorCandidate,
};
use crate::graph::weighted_primal::{
    components_after_removing, induced_subgraph, WeightedPrimalGraph,
};
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::VariableId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecompositionTree {
    pub root: DecompositionNode,
    pub tree_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecompositionNode {
    pub node_id: usize,
    pub variables: BTreeSet<VariableId>,
    pub separator: BTreeSet<VariableId>,
    pub children: Vec<DecompositionNode>,
    pub node_hash: Hash,
}

pub fn build_target_rooted_decomposition(
    g: &WeightedPrimalGraph,
    target: VariableId,
    cost_model: &CostModel,
) -> DecompositionTree {
    let mut next_node_id = 0;
    let root = build_node(g, target, cost_model, &mut next_node_id, 0);
    let tree_hash = hash_sequence("target-rooted-decomposition", &[root.node_hash.0.to_vec()]);
    DecompositionTree { root, tree_hash }
}

fn build_node(
    g: &WeightedPrimalGraph,
    target: VariableId,
    cost_model: &CostModel,
    next_node_id: &mut usize,
    depth: usize,
) -> DecompositionNode {
    let node_id = *next_node_id;
    *next_node_id += 1;
    let mut node = DecompositionNode {
        node_id,
        variables: g.variables.clone(),
        separator: BTreeSet::new(),
        children: Vec::new(),
        node_hash: hash_sequence("decomposition-node", &[]),
    };

    if g.variables.len() <= 3 || depth >= 8 {
        node.node_hash = hash_node(&node);
        return node;
    }

    if let Some(candidate) = choose_useful_separator(g, target, cost_model) {
        let components = components_after_removing(g, &candidate.vars)
            .into_iter()
            .filter(|component| !component.is_empty())
            .collect::<Vec<_>>();
        let mut children = Vec::new();
        for component in components {
            let mut child_vars = component;
            child_vars.extend(candidate.vars.iter().copied());
            if child_vars == g.variables || child_vars.is_empty() {
                continue;
            }
            let child_graph = induced_subgraph(g, &child_vars);
            children.push(build_node(
                &child_graph,
                target,
                cost_model,
                next_node_id,
                depth + 1,
            ));
        }
        if children.len() > 1 {
            node.separator = candidate.vars;
            node.children = children;
        }
    }

    node.node_hash = hash_node(&node);
    node
}

fn choose_useful_separator(
    g: &WeightedPrimalGraph,
    target: VariableId,
    cost_model: &CostModel,
) -> Option<SeparatorCandidate> {
    let mut candidates = articulation_variable_candidates(g);
    candidates.extend(min_fill_separator_candidates(g, target));
    candidates.sort_by(|left, right| {
        let left_score = score_separator(left, g, cost_model);
        let right_score = score_separator(right, g, cost_model);
        (
            left_score.score.clone(),
            left_score.max_component_size,
            left_score.separator_width,
            left.vars.iter().copied().collect::<Vec<_>>(),
        )
            .cmp(&(
                right_score.score,
                right_score.max_component_size,
                right_score.separator_width,
                right.vars.iter().copied().collect::<Vec<_>>(),
            ))
    });
    candidates.into_iter().find(|candidate| {
        let components = components_after_removing(g, &candidate.vars);
        if components.len() <= 1 {
            return false;
        }
        let max_child_size = components
            .iter()
            .map(|component| component.len() + candidate.vars.len())
            .max()
            .unwrap_or(g.variables.len());
        max_child_size < g.variables.len()
    })
}

fn hash_node(node: &DecompositionNode) -> Hash {
    let mut chunks = vec![node.node_id.to_be_bytes().to_vec()];
    for var in &node.variables {
        chunks.push(var.0.to_be_bytes().to_vec());
    }
    chunks.push(Vec::new());
    for var in &node.separator {
        chunks.push(var.0.to_be_bytes().to_vec());
    }
    for child in &node.children {
        chunks.push(child.node_hash.0.to_vec());
    }
    hash_sequence("decomposition-node", &chunks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::hypergraph::build_relation_variable_hypergraph;
    use crate::graph::influence::build_target_influence_graph;
    use crate::graph::weighted_primal::build_weighted_primal_graph;
    use crate::preprocess::compression::CompressionState;
    use crate::problem::canonicalize::canonicalize_system;
    use crate::problem::input::make_problem;
    use crate::problem::validate::validate_input;
    use crate::types::polynomial::{poly_add, poly_mul, variable_poly};

    #[test]
    fn no_useful_separator_keeps_one_large_root() {
        let t = VariableId(0);
        let x = VariableId(1);
        let y = VariableId(2);
        let relation = poly_add(
            &poly_add(
                &poly_mul(&variable_poly(t), &variable_poly(x)),
                &poly_mul(&variable_poly(x), &variable_poly(y)),
            ),
            &poly_mul(&variable_poly(y), &variable_poly(t)),
        );
        let canonical = canonicalize_system(
            validate_input(make_problem(vec![t, x, y], t, vec![relation], Vec::new())).unwrap(),
        )
        .unwrap();
        let compressed = CompressionState::from_system(canonical).to_compressed_system();
        let h = build_relation_variable_hypergraph(&compressed);
        let influence = build_target_influence_graph(&h, t);
        let g = build_weighted_primal_graph(&compressed, &influence);
        let tree = build_target_rooted_decomposition(&g, t, &CostModel::default());
        assert!(tree.root.children.is_empty());
        assert_eq!(tree.root.variables, BTreeSet::from([t, x, y]));
    }
}
