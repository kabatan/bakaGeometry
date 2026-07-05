use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::graph::weighted_primal::{components_after_removing, WeightedPrimalGraph};
use crate::types::ids::VariableId;
use crate::types::rational::{int_q, RationalQ};
use num_traits::ToPrimitive;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeparatorCandidate {
    pub vars: BTreeSet<VariableId>,
    pub candidate_kind: SeparatorCandidateKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeparatorCandidateKind {
    Articulation,
    MinFill,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeparatorScore {
    pub score: RationalQ,
    pub component_count: usize,
    pub max_component_size: usize,
    pub separator_width: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CostModel {
    pub block_size_weight: RationalQ,
    pub separator_width_weight: RationalQ,
    pub component_count_bonus: RationalQ,
}

impl Default for CostModel {
    fn default() -> Self {
        Self {
            block_size_weight: int_q(10),
            separator_width_weight: int_q(3),
            component_count_bonus: int_q(2),
        }
    }
}

pub fn articulation_variable_candidates(g: &WeightedPrimalGraph) -> Vec<SeparatorCandidate> {
    let base_count = components_after_removing(g, &BTreeSet::new()).len();
    let mut candidates = g
        .variables
        .iter()
        .copied()
        .filter(|var| *var != g.target)
        .filter_map(|var| {
            let removed = BTreeSet::from([var]);
            let count = components_after_removing(g, &removed).len();
            (count > base_count).then_some(SeparatorCandidate {
                vars: removed,
                candidate_kind: SeparatorCandidateKind::Articulation,
            })
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|candidate| candidate.vars.iter().copied().collect::<Vec<_>>());
    candidates
}

pub fn min_fill_separator_candidates(
    g: &WeightedPrimalGraph,
    target: VariableId,
) -> Vec<SeparatorCandidate> {
    let mut candidates = g
        .variables
        .iter()
        .copied()
        .filter(|var| *var != target)
        .map(|var| SeparatorCandidate {
            vars: BTreeSet::from([var]),
            candidate_kind: SeparatorCandidateKind::MinFill,
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|candidate| {
        let var = *candidate.vars.iter().next().unwrap();
        let degree = g.adjacency.get(&var).map_or(0, BTreeSet::len);
        (degree, var)
    });
    candidates
}

pub fn score_separator(
    candidate: &SeparatorCandidate,
    subgraph: &WeightedPrimalGraph,
    cost_model: &CostModel,
) -> SeparatorScore {
    let components = components_after_removing(subgraph, &candidate.vars);
    let max_component_size = components.iter().map(BTreeSet::len).max().unwrap_or(0);
    let mut raw = max_component_size as i64 * q_to_i64(&cost_model.block_size_weight, 10);
    raw += candidate.vars.len() as i64 * q_to_i64(&cost_model.separator_width_weight, 3);
    raw -= components.len() as i64 * q_to_i64(&cost_model.component_count_bonus, 2);
    SeparatorScore {
        score: int_q(raw),
        component_count: components.len(),
        max_component_size,
        separator_width: candidate.vars.len(),
    }
}

fn q_to_i64(q: &RationalQ, default: i64) -> i64 {
    if q.den != 1.into() {
        return default;
    }
    q.num.to_i64().unwrap_or(default)
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
    use crate::types::polynomial::{poly_mul, variable_poly};

    #[test]
    fn articulation_candidate_is_algebraic_incidence_based() {
        let t = VariableId(0);
        let x = VariableId(1);
        let y = VariableId(2);
        let problem = make_problem(
            vec![t, x, y],
            t,
            vec![
                poly_mul(&variable_poly(t), &variable_poly(x)),
                poly_mul(&variable_poly(x), &variable_poly(y)),
            ],
            Vec::new(),
        );
        let canonical = canonicalize_system(validate_input(problem).unwrap()).unwrap();
        let compressed = CompressionState::from_system(canonical).to_compressed_system();
        let h = build_relation_variable_hypergraph(&compressed);
        let influence = build_target_influence_graph(&h, t);
        let g = build_weighted_primal_graph(&compressed, &influence);
        let candidates = articulation_variable_candidates(&g);
        assert!(candidates
            .iter()
            .any(|candidate| candidate.vars.contains(&x)));
    }
}
