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
    BoundedMinCut,
    AlgebraicIntermediate,
    LowDegreeDefinitionalAffine,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeparatorScore {
    pub score: RationalQ,
    pub component_count: usize,
    pub max_component_size: usize,
    pub separator_width: usize,
    pub relation_heavy_penalty: usize,
    pub dense_trs_penalty: usize,
    pub quotient_rank_penalty: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CostModel {
    pub block_size_weight: RationalQ,
    pub separator_width_weight: RationalQ,
    pub component_count_bonus: RationalQ,
    pub relation_heavy_penalty_weight: RationalQ,
    pub dense_trs_penalty_weight: RationalQ,
    pub quotient_rank_penalty_weight: RationalQ,
}

impl Default for CostModel {
    fn default() -> Self {
        Self {
            block_size_weight: int_q(10),
            separator_width_weight: int_q(3),
            component_count_bonus: int_q(2),
            relation_heavy_penalty_weight: int_q(1),
            dense_trs_penalty_weight: int_q(4),
            quotient_rank_penalty_weight: int_q(1),
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

pub fn bounded_min_cut_separator_candidates(
    g: &WeightedPrimalGraph,
    target: VariableId,
) -> Vec<SeparatorCandidate> {
    let vars = g
        .variables
        .iter()
        .copied()
        .filter(|var| *var != target)
        .collect::<Vec<_>>();
    let mut candidates = Vec::new();
    for i in 0..vars.len().min(12) {
        for j in (i + 1)..vars.len().min(12) {
            let removed = BTreeSet::from([vars[i], vars[j]]);
            if components_after_removing(g, &removed).len() > 1 {
                candidates.push(SeparatorCandidate {
                    vars: removed,
                    candidate_kind: SeparatorCandidateKind::BoundedMinCut,
                });
            }
        }
    }
    candidates.sort_by_key(|candidate| candidate.vars.iter().copied().collect::<Vec<_>>());
    candidates
}

pub fn algebraic_intermediate_separator_candidates(
    g: &WeightedPrimalGraph,
    target: VariableId,
) -> Vec<SeparatorCandidate> {
    let mut candidates = g
        .variables
        .iter()
        .copied()
        .filter(|var| *var != target)
        .filter(|var| {
            g.variable_weights.get(var).is_some_and(|weight| {
                weight.target_distance.unwrap_or(usize::MAX) > 0
                    && weight.occurrence_count >= 2
                    && weight.monomial_contribution <= 8
            })
        })
        .map(|var| SeparatorCandidate {
            vars: BTreeSet::from([var]),
            candidate_kind: SeparatorCandidateKind::AlgebraicIntermediate,
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|candidate| candidate.vars.iter().copied().collect::<Vec<_>>());
    candidates
}

pub fn low_degree_definitional_affine_candidates(
    g: &WeightedPrimalGraph,
    target: VariableId,
) -> Vec<SeparatorCandidate> {
    let mut candidates = g
        .variables
        .iter()
        .copied()
        .filter(|var| *var != target)
        .filter(|var| {
            g.variable_weights.get(var).is_some_and(|weight| {
                weight.degree_participation <= 2
                    && weight.monomial_contribution <= 4
                    && weight.occurrence_count <= 3
            })
        })
        .map(|var| SeparatorCandidate {
            vars: BTreeSet::from([var]),
            candidate_kind: SeparatorCandidateKind::LowDegreeDefinitionalAffine,
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|candidate| candidate.vars.iter().copied().collect::<Vec<_>>());
    candidates
}

pub fn score_separator(
    candidate: &SeparatorCandidate,
    subgraph: &WeightedPrimalGraph,
    cost_model: &CostModel,
) -> SeparatorScore {
    let components = components_after_removing(subgraph, &candidate.vars);
    let max_component_size = components.iter().map(BTreeSet::len).max().unwrap_or(0);
    let relation_heavy_penalty = candidate
        .vars
        .iter()
        .filter_map(|var| subgraph.variable_weights.get(var))
        .map(|weight| {
            weight
                .occurrence_count
                .saturating_add(weight.degree_participation)
                .saturating_add(weight.monomial_contribution)
        })
        .sum::<usize>();
    let quotient_rank_penalty = max_component_size.saturating_mul(max_component_size);
    let dense_trs_penalty = components
        .iter()
        .map(|component| {
            component
                .iter()
                .filter_map(|var| subgraph.variable_weights.get(var))
                .map(|weight| weight.monomial_contribution)
                .sum::<usize>()
        })
        .max()
        .unwrap_or(0);
    let mut raw = max_component_size as i64 * q_to_i64(&cost_model.block_size_weight, 10);
    raw += candidate.vars.len() as i64 * q_to_i64(&cost_model.separator_width_weight, 3);
    raw += relation_heavy_penalty as i64 * q_to_i64(&cost_model.relation_heavy_penalty_weight, 1);
    raw += dense_trs_penalty as i64 * q_to_i64(&cost_model.dense_trs_penalty_weight, 4);
    raw += quotient_rank_penalty as i64 * q_to_i64(&cost_model.quotient_rank_penalty_weight, 1);
    raw -= components.len() as i64 * q_to_i64(&cost_model.component_count_bonus, 2);
    SeparatorScore {
        score: int_q(raw),
        component_count: components.len(),
        max_component_size,
        separator_width: candidate.vars.len(),
        relation_heavy_penalty,
        dense_trs_penalty,
        quotient_rank_penalty,
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

    #[test]
    fn generic_algebraic_separator_candidate_classes_are_available() {
        let t = VariableId(10);
        let a = VariableId(11);
        let b = VariableId(12);
        let c = VariableId(13);
        let d = VariableId(14);
        let problem = make_problem(
            vec![t, a, b, c, d],
            t,
            vec![
                poly_mul(&variable_poly(t), &variable_poly(a)),
                poly_mul(&variable_poly(a), &variable_poly(b)),
                poly_mul(&variable_poly(b), &variable_poly(c)),
                poly_mul(&variable_poly(c), &variable_poly(d)),
            ],
            Vec::new(),
        );
        let canonical = canonicalize_system(validate_input(problem).unwrap()).unwrap();
        let compressed = CompressionState::from_system(canonical).to_compressed_system();
        let h = build_relation_variable_hypergraph(&compressed);
        let influence = build_target_influence_graph(&h, t);
        let g = build_weighted_primal_graph(&compressed, &influence);

        assert!(!bounded_min_cut_separator_candidates(&g, t).is_empty());
        assert!(!algebraic_intermediate_separator_candidates(&g, t).is_empty());
        assert!(!low_degree_definitional_affine_candidates(&g, t).is_empty());
    }
}
