use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::graph::separators::{
    algebraic_intermediate_separator_candidates, articulation_variable_candidates,
    baseline_projection_cost, bounded_min_cut_separator_candidates,
    low_degree_definitional_affine_candidates, min_fill_separator_candidates, score_separator,
    CostModel, SeparatorCandidate, SeparatorCandidateKind, SeparatorScore,
};
use crate::graph::weighted_primal::{
    components_after_removing, induced_subgraph, WeightedPrimalGraph,
};
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::VariableId;

const HIGH_COST_RETAINED_BLOCK_DIAGNOSTIC_THRESHOLD: usize = 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecompositionTree {
    pub root: DecompositionNode,
    pub diagnostics: Vec<DecompositionDiagnostic>,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecompositionDiagnostic {
    pub node_id: usize,
    pub variable_count: usize,
    pub baseline_projection_cost: usize,
    pub evaluated_candidate_count: usize,
    pub best_candidate_kind: Option<SeparatorCandidateKind>,
    pub best_candidate_vars: Vec<VariableId>,
    pub best_candidate_estimated_total_cost: Option<usize>,
    pub best_candidate_max_component_size: Option<usize>,
    pub selected: bool,
    pub reason: String,
}

#[derive(Debug, Clone)]
struct ScoredSeparator {
    candidate: SeparatorCandidate,
    score: SeparatorScore,
    max_child_size: usize,
    improves_width: bool,
    improves_estimated_cost: bool,
}

#[derive(Debug, Clone)]
struct SeparatorDecision {
    baseline_projection_cost: usize,
    evaluated: Vec<ScoredSeparator>,
    selected: Option<ScoredSeparator>,
    reason: String,
}

pub fn build_target_rooted_decomposition(
    g: &WeightedPrimalGraph,
    target: VariableId,
    cost_model: &CostModel,
) -> DecompositionTree {
    let mut next_node_id = 0;
    let mut diagnostics = Vec::new();
    let root = build_node(
        g,
        target,
        cost_model,
        &mut next_node_id,
        0,
        &mut diagnostics,
    );
    let tree_hash = hash_sequence("target-rooted-decomposition", &[root.node_hash.0.to_vec()]);
    DecompositionTree {
        root,
        diagnostics,
        tree_hash,
    }
}

fn build_node(
    g: &WeightedPrimalGraph,
    target: VariableId,
    cost_model: &CostModel,
    next_node_id: &mut usize,
    depth: usize,
    diagnostics: &mut Vec<DecompositionDiagnostic>,
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

    if g.variables.len() <= 3 {
        let baseline = baseline_projection_cost(g);
        if baseline >= HIGH_COST_RETAINED_BLOCK_DIAGNOSTIC_THRESHOLD {
            diagnostics.push(DecompositionDiagnostic {
                node_id,
                variable_count: g.variables.len(),
                baseline_projection_cost: baseline,
                evaluated_candidate_count: 0,
                best_candidate_kind: None,
                best_candidate_vars: Vec::new(),
                best_candidate_estimated_total_cost: None,
                best_candidate_max_component_size: None,
                selected: false,
                reason: "small_high_cost_block_retained_without_separator_candidate".to_owned(),
            });
        }
        node.node_hash = hash_node(&node);
        return node;
    }
    if depth >= 8 {
        diagnostics.push(DecompositionDiagnostic {
            node_id,
            variable_count: g.variables.len(),
            baseline_projection_cost: baseline_projection_cost(g),
            evaluated_candidate_count: 0,
            best_candidate_kind: None,
            best_candidate_vars: Vec::new(),
            best_candidate_estimated_total_cost: None,
            best_candidate_max_component_size: None,
            selected: false,
            reason: "depth_limit_before_cost_improving_separator".to_owned(),
        });
        node.node_hash = hash_node(&node);
        return node;
    }

    let decision = choose_useful_separator(g, target, cost_model);
    if let Some(scored) = decision.selected.clone() {
        let components = components_after_removing(g, &scored.candidate.vars)
            .into_iter()
            .filter(|component| !component.is_empty())
            .collect::<Vec<_>>();
        let mut children = Vec::new();
        for component in components {
            let mut child_vars = component;
            child_vars.extend(scored.candidate.vars.iter().copied());
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
                diagnostics,
            ));
        }
        if children.len() > 1 {
            diagnostics.push(decision_diagnostic(
                node_id,
                g.variables.len(),
                &decision,
                true,
            ));
            node.separator = scored.candidate.vars;
            node.children = children;
        } else {
            let mut diagnostic = decision_diagnostic(node_id, g.variables.len(), &decision, false);
            diagnostic.reason =
                "selected_separator_did_not_materialize_multiple_children".to_owned();
            diagnostics.push(diagnostic);
        }
    } else {
        diagnostics.push(decision_diagnostic(
            node_id,
            g.variables.len(),
            &decision,
            false,
        ));
    }

    node.node_hash = hash_node(&node);
    node
}

fn choose_useful_separator(
    g: &WeightedPrimalGraph,
    target: VariableId,
    cost_model: &CostModel,
) -> SeparatorDecision {
    let mut candidates = articulation_variable_candidates(g);
    candidates.extend(min_fill_separator_candidates(g, target));
    candidates.extend(bounded_min_cut_separator_candidates(g, target));
    if g.variables.len() >= 6 {
        candidates.extend(algebraic_intermediate_separator_candidates(g, target));
        candidates.extend(low_degree_definitional_affine_candidates(g, target));
    }
    let baseline = baseline_projection_cost(g);
    let mut evaluated = candidates
        .into_iter()
        .map(|candidate| {
            let score = score_separator(&candidate, g, cost_model);
            let components = components_after_removing(g, &candidate.vars);
            let max_child_size = components
                .iter()
                .map(|component| component.len() + candidate.vars.len())
                .max()
                .unwrap_or(g.variables.len());
            ScoredSeparator {
                candidate,
                improves_width: components.len() > 1 && max_child_size < g.variables.len(),
                improves_estimated_cost: score.estimated_total_cost < baseline,
                max_child_size,
                score,
            }
        })
        .collect::<Vec<_>>();
    evaluated.sort_by(|left, right| {
        (
            left.score.estimated_total_cost,
            left.score.max_component_size,
            left.score.separator_width,
            left.candidate.vars.iter().copied().collect::<Vec<_>>(),
        )
            .cmp(&(
                right.score.estimated_total_cost,
                right.score.max_component_size,
                right.score.separator_width,
                right.candidate.vars.iter().copied().collect::<Vec<_>>(),
            ))
    });
    let selected = evaluated
        .iter()
        .find(|scored| scored.improves_width && scored.improves_estimated_cost)
        .cloned();
    let reason = if selected.is_some() {
        "selected_cost_improving_separator".to_owned()
    } else if evaluated.is_empty() {
        "no_separator_candidates".to_owned()
    } else if evaluated.iter().any(|scored| scored.improves_width) {
        "no_separator_improved_estimated_cost".to_owned()
    } else {
        "no_separator_reduced_block_width".to_owned()
    };
    SeparatorDecision {
        baseline_projection_cost: baseline,
        evaluated,
        selected,
        reason,
    }
}

fn decision_diagnostic(
    node_id: usize,
    variable_count: usize,
    decision: &SeparatorDecision,
    selected: bool,
) -> DecompositionDiagnostic {
    let best = decision.evaluated.first();
    DecompositionDiagnostic {
        node_id,
        variable_count,
        baseline_projection_cost: decision.baseline_projection_cost,
        evaluated_candidate_count: decision.evaluated.len(),
        best_candidate_kind: best.map(|scored| scored.candidate.candidate_kind),
        best_candidate_vars: best
            .map(|scored| scored.candidate.vars.iter().copied().collect::<Vec<_>>())
            .unwrap_or_default(),
        best_candidate_estimated_total_cost: best.map(|scored| scored.score.estimated_total_cost),
        best_candidate_max_component_size: best.map(|scored| scored.max_child_size),
        selected,
        reason: decision.reason.clone(),
    }
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
    use crate::types::polynomial::{
        normalize_poly, poly_add, poly_mul, variable_poly, SparsePolynomialQ, TermQ,
    };
    use crate::types::rational::int_q;

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

    #[test]
    fn acr_p7_algebraic_separator_reduces_large_block_width() {
        let t = VariableId(30);
        let s = VariableId(31);
        let a = VariableId(32);
        let b = VariableId(33);
        let c = VariableId(34);
        let d = VariableId(35);
        let problem = make_problem(
            vec![t, s, a, b, c, d],
            t,
            vec![
                poly_mul(&variable_poly(t), &variable_poly(s)),
                poly_mul(&variable_poly(s), &variable_poly(a)),
                poly_mul(&variable_poly(a), &variable_poly(b)),
                poly_mul(&variable_poly(s), &variable_poly(c)),
                poly_mul(&variable_poly(c), &variable_poly(d)),
            ],
            Vec::new(),
        );
        let canonical = canonicalize_system(validate_input(problem).unwrap()).unwrap();
        let compressed = CompressionState::from_system(canonical).to_compressed_system();
        let h = build_relation_variable_hypergraph(&compressed);
        let influence = build_target_influence_graph(&h, t);
        let g = build_weighted_primal_graph(&compressed, &influence);
        let tree = build_target_rooted_decomposition(&g, t, &CostModel::default());
        let max_child_width = tree
            .root
            .children
            .iter()
            .map(|child| child.variables.len())
            .max()
            .unwrap_or(tree.root.variables.len());

        assert!(!tree.root.children.is_empty());
        assert!(max_child_width < tree.root.variables.len());
        assert!(tree.diagnostics.iter().any(|diagnostic| diagnostic.selected
            && diagnostic.reason == "selected_cost_improving_separator"));
    }

    #[test]
    fn acr_p7_large_block_records_no_improving_separator_reason() {
        let t = VariableId(40);
        let vars = (41..46).map(VariableId).collect::<Vec<_>>();
        let all_vars = std::iter::once(t).chain(vars.iter().copied()).collect();
        let relation = vars.iter().fold(variable_poly(t), |acc, var| {
            poly_mul(&acc, &variable_poly(*var))
        });
        let canonical = canonicalize_system(
            validate_input(make_problem(all_vars, t, vec![relation], Vec::new())).unwrap(),
        )
        .unwrap();
        let compressed = CompressionState::from_system(canonical).to_compressed_system();
        let h = build_relation_variable_hypergraph(&compressed);
        let influence = build_target_influence_graph(&h, t);
        let g = build_weighted_primal_graph(&compressed, &influence);
        let tree = build_target_rooted_decomposition(&g, t, &CostModel::default());

        assert!(tree.root.children.is_empty());
        assert!(tree.diagnostics.iter().any(|diagnostic| {
            !diagnostic.selected
                && diagnostic.variable_count > 3
                && diagnostic.reason == "no_separator_reduced_block_width"
        }));
    }

    #[test]
    fn acr_p7_cost_aware_split_when_variable_count_only_would_keep() {
        let t = VariableId(50);
        let a = VariableId(51);
        let s1 = VariableId(52);
        let s2 = VariableId(53);
        let b = VariableId(54);
        let c = VariableId(55);
        let problem = make_problem(
            vec![t, a, s1, s2, b, c],
            t,
            vec![
                dense_pair_relation(t, a, 96),
                dense_pair_relation(b, c, 96),
                poly_mul(&variable_poly(t), &variable_poly(s1)),
                poly_mul(&variable_poly(a), &variable_poly(s2)),
                poly_mul(&variable_poly(b), &variable_poly(s1)),
                poly_mul(&variable_poly(c), &variable_poly(s2)),
            ],
            Vec::new(),
        );
        let canonical = canonicalize_system(validate_input(problem).unwrap()).unwrap();
        let compressed = CompressionState::from_system(canonical).to_compressed_system();
        let h = build_relation_variable_hypergraph(&compressed);
        let influence = build_target_influence_graph(&h, t);
        let g = build_weighted_primal_graph(&compressed, &influence);
        let tree = build_target_rooted_decomposition(&g, t, &CostModel::default());

        assert!(variable_count_only_would_keep_for_review(&g));
        assert_eq!(tree.root.separator, BTreeSet::from([s1, s2]));
        assert_eq!(tree.root.children.len(), 2);
        assert!(tree.diagnostics.iter().any(|diagnostic| diagnostic.selected
            && diagnostic.best_candidate_vars == vec![s1, s2]
            && diagnostic.reason == "selected_cost_improving_separator"));
    }

    #[test]
    fn bounded_min_cut_is_tried_for_sub_six_variable_blocks() {
        let t = VariableId(70);
        let s1 = VariableId(71);
        let s2 = VariableId(72);
        let b = VariableId(73);
        let c = VariableId(74);
        let problem = make_problem(
            vec![t, s1, s2, b, c],
            t,
            vec![
                dense_pair_relation(t, s1, 96),
                dense_pair_relation(b, c, 96),
                poly_mul(&variable_poly(t), &variable_poly(s2)),
                poly_mul(&variable_poly(b), &variable_poly(s1)),
                poly_mul(&variable_poly(c), &variable_poly(s2)),
            ],
            Vec::new(),
        );
        let canonical = canonicalize_system(validate_input(problem).unwrap()).unwrap();
        let compressed = CompressionState::from_system(canonical).to_compressed_system();
        let h = build_relation_variable_hypergraph(&compressed);
        let influence = build_target_influence_graph(&h, t);
        let g = build_weighted_primal_graph(&compressed, &influence);
        let tree = build_target_rooted_decomposition(&g, t, &CostModel::default());

        assert_eq!(tree.root.separator.len(), 2);
        assert!(tree.diagnostics.iter().any(|diagnostic| diagnostic.selected
            && diagnostic.variable_count == 5
            && diagnostic.best_candidate_kind == Some(SeparatorCandidateKind::BoundedMinCut)));
    }

    #[test]
    fn acr_p7_small_high_cost_block_records_retention_reason() {
        let t = VariableId(60);
        let x = VariableId(61);
        let problem = make_problem(
            vec![t, x],
            t,
            vec![dense_pair_relation(t, x, 128)],
            Vec::new(),
        );
        let canonical = canonicalize_system(validate_input(problem).unwrap()).unwrap();
        let compressed = CompressionState::from_system(canonical).to_compressed_system();
        let h = build_relation_variable_hypergraph(&compressed);
        let influence = build_target_influence_graph(&h, t);
        let g = build_weighted_primal_graph(&compressed, &influence);
        let tree = build_target_rooted_decomposition(&g, t, &CostModel::default());

        assert!(tree.root.children.is_empty());
        assert!(tree.diagnostics.iter().any(|diagnostic| {
            !diagnostic.selected
                && diagnostic.variable_count == 2
                && diagnostic.reason == "small_high_cost_block_retained_without_separator_candidate"
        }));
    }

    fn variable_count_only_would_keep_for_review(g: &WeightedPrimalGraph) -> bool {
        let baseline_width = g.variables.len();
        bounded_min_cut_separator_candidates(g, g.target)
            .into_iter()
            .map(|candidate| {
                let components = components_after_removing(g, &candidate.vars);
                let max_component_width = components.iter().map(BTreeSet::len).max().unwrap_or(0);
                max_component_width.saturating_add(candidate.vars.len().saturating_mul(2))
            })
            .min()
            .is_some_and(|best_variable_count_score| best_variable_count_score >= baseline_width)
    }

    fn dense_pair_relation(u: VariableId, v: VariableId, term_count: usize) -> SparsePolynomialQ {
        normalize_poly(SparsePolynomialQ {
            terms: (0..term_count)
                .map(|idx| TermQ {
                    coeff: int_q(1),
                    monomial: crate::types::monomial::normalize_monomial(vec![
                        (u, idx as u32),
                        (v, term_count.saturating_sub(idx).saturating_sub(1) as u32),
                    ]),
                })
                .collect(),
            hash: hash_sequence("poly", &[]),
        })
    }
}
