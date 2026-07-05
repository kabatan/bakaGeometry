use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::graph::influence::TargetInfluenceGraph;
use crate::preprocess::compression::{max_coefficient_height_bits, CompressedSystemQ};
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::{RelationId, VariableId};
use crate::types::polynomial::{poly_total_degree, poly_variables};
use crate::types::rational::{int_q, RationalQ};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeightedPrimalGraph {
    pub target: VariableId,
    pub variables: BTreeSet<VariableId>,
    pub adjacency: BTreeMap<VariableId, BTreeSet<VariableId>>,
    pub edge_relations: BTreeMap<(VariableId, VariableId), BTreeSet<RelationId>>,
    pub variable_weights: BTreeMap<VariableId, AlgebraicWeight>,
    pub edge_weights: BTreeMap<(VariableId, VariableId), AlgebraicWeight>,
    pub graph_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlgebraicWeight {
    pub score: RationalQ,
    pub degree_participation: usize,
    pub monomial_contribution: usize,
    pub coefficient_height_bits: usize,
    pub target_distance: Option<usize>,
    pub occurrence_count: usize,
}

pub fn build_weighted_primal_graph(
    system: &CompressedSystemQ,
    influence: &TargetInfluenceGraph,
) -> WeightedPrimalGraph {
    let mut variables = influence.target_component.variables.clone();
    variables.insert(system.target);
    let mut adjacency: BTreeMap<VariableId, BTreeSet<VariableId>> = BTreeMap::new();
    let mut edge_relations: BTreeMap<(VariableId, VariableId), BTreeSet<RelationId>> =
        BTreeMap::new();
    for variable in &variables {
        adjacency.entry(*variable).or_default();
    }
    for relation in &system.relations {
        let relation_vars = poly_variables(&relation.polynomial)
            .into_iter()
            .filter(|var| variables.contains(var))
            .collect::<Vec<_>>();
        for i in 0..relation_vars.len() {
            for j in (i + 1)..relation_vars.len() {
                let key = edge_key(relation_vars[i], relation_vars[j]);
                adjacency.entry(key.0).or_default().insert(key.1);
                adjacency.entry(key.1).or_default().insert(key.0);
                edge_relations.entry(key).or_default().insert(relation.id);
            }
        }
    }
    let distances = target_distances(&adjacency, system.target);
    let variable_weights = variables
        .iter()
        .map(|var| {
            (
                *var,
                variable_weight_with_distance(*var, system, distances.get(var).copied()),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let edge_weights = edge_relations
        .iter()
        .map(|(edge, relations)| {
            (
                *edge,
                edge_weight(
                    edge.0,
                    edge.1,
                    &relations.iter().copied().collect::<Vec<_>>(),
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let graph_hash = hash_primal(system.target, &variables, &edge_relations);
    WeightedPrimalGraph {
        target: system.target,
        variables,
        adjacency,
        edge_relations,
        variable_weights,
        edge_weights,
        graph_hash,
    }
}

pub fn variable_weight(v: VariableId, system: &CompressedSystemQ) -> AlgebraicWeight {
    variable_weight_with_distance(v, system, None)
}

pub fn edge_weight(_u: VariableId, _v: VariableId, relations: &[RelationId]) -> AlgebraicWeight {
    AlgebraicWeight {
        score: int_q(relations.len() as i64),
        degree_participation: relations.len(),
        monomial_contribution: relations.len(),
        coefficient_height_bits: 0,
        target_distance: None,
        occurrence_count: relations.len(),
    }
}

pub fn edge_key(a: VariableId, b: VariableId) -> (VariableId, VariableId) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

pub fn induced_subgraph(
    g: &WeightedPrimalGraph,
    vars: &BTreeSet<VariableId>,
) -> WeightedPrimalGraph {
    let mut adjacency = BTreeMap::new();
    let mut edge_relations = BTreeMap::new();
    for var in vars {
        let neighbors = g
            .adjacency
            .get(var)
            .cloned()
            .unwrap_or_default()
            .intersection(vars)
            .copied()
            .collect::<BTreeSet<_>>();
        adjacency.insert(*var, neighbors);
    }
    for (edge, relations) in &g.edge_relations {
        if vars.contains(&edge.0) && vars.contains(&edge.1) {
            edge_relations.insert(*edge, relations.clone());
        }
    }
    let variable_weights = g
        .variable_weights
        .iter()
        .filter(|(var, _)| vars.contains(var))
        .map(|(var, weight)| (*var, weight.clone()))
        .collect();
    let edge_weights = g
        .edge_weights
        .iter()
        .filter(|(edge, _)| vars.contains(&edge.0) && vars.contains(&edge.1))
        .map(|(edge, weight)| (*edge, weight.clone()))
        .collect();
    WeightedPrimalGraph {
        target: g.target,
        variables: vars.clone(),
        adjacency,
        graph_hash: hash_primal(g.target, vars, &edge_relations),
        edge_relations,
        variable_weights,
        edge_weights,
    }
}

pub fn components_after_removing(
    g: &WeightedPrimalGraph,
    remove: &BTreeSet<VariableId>,
) -> Vec<BTreeSet<VariableId>> {
    let mut unvisited = g
        .variables
        .difference(remove)
        .copied()
        .collect::<BTreeSet<_>>();
    let mut components = Vec::new();
    while let Some(start) = unvisited.iter().next().copied() {
        let mut component = BTreeSet::new();
        let mut queue = VecDeque::from([start]);
        unvisited.remove(&start);
        while let Some(var) = queue.pop_front() {
            component.insert(var);
            for neighbor in g.adjacency.get(&var).cloned().unwrap_or_default() {
                if remove.contains(&neighbor) {
                    continue;
                }
                if unvisited.remove(&neighbor) {
                    queue.push_back(neighbor);
                }
            }
        }
        components.push(component);
    }
    components
}

fn variable_weight_with_distance(
    v: VariableId,
    system: &CompressedSystemQ,
    target_distance: Option<usize>,
) -> AlgebraicWeight {
    let mut degree_participation = 0;
    let mut monomial_contribution = 0;
    let mut occurrence_count = 0;
    let mut related = Vec::new();
    for relation in &system.relations {
        let vars = poly_variables(&relation.polynomial);
        if vars.contains(&v) {
            degree_participation += poly_total_degree(&relation.polynomial) as usize;
            monomial_contribution += relation.polynomial.terms.len();
            occurrence_count += 1;
            related.push(relation.clone());
        }
    }
    let coefficient_height_bits = max_coefficient_height_bits(&related);
    let distance_penalty = target_distance.unwrap_or(system.variables.len().saturating_add(1));
    let score = degree_participation
        + monomial_contribution
        + coefficient_height_bits
        + occurrence_count
        + distance_penalty;
    AlgebraicWeight {
        score: int_q(score as i64),
        degree_participation,
        monomial_contribution,
        coefficient_height_bits,
        target_distance,
        occurrence_count,
    }
}

fn target_distances(
    adjacency: &BTreeMap<VariableId, BTreeSet<VariableId>>,
    target: VariableId,
) -> BTreeMap<VariableId, usize> {
    let mut distances = BTreeMap::new();
    let mut queue = VecDeque::from([(target, 0_usize)]);
    distances.insert(target, 0);
    while let Some((var, distance)) = queue.pop_front() {
        for neighbor in adjacency.get(&var).cloned().unwrap_or_default() {
            if let std::collections::btree_map::Entry::Vacant(entry) = distances.entry(neighbor) {
                entry.insert(distance + 1);
                queue.push_back((neighbor, distance + 1));
            }
        }
    }
    distances
}

fn hash_primal(
    target: VariableId,
    variables: &BTreeSet<VariableId>,
    edges: &BTreeMap<(VariableId, VariableId), BTreeSet<RelationId>>,
) -> Hash {
    let mut chunks = vec![target.0.to_be_bytes().to_vec()];
    for var in variables {
        chunks.push(var.0.to_be_bytes().to_vec());
    }
    for (edge, relations) in edges {
        chunks.push(edge.0 .0.to_be_bytes().to_vec());
        chunks.push(edge.1 .0.to_be_bytes().to_vec());
        for relation in relations {
            chunks.push(relation.0.to_be_bytes().to_vec());
        }
    }
    hash_sequence("weighted-primal", &chunks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::hypergraph::build_relation_variable_hypergraph;
    use crate::graph::influence::build_target_influence_graph;
    use crate::preprocess::compression::CompressionState;
    use crate::problem::canonicalize::canonicalize_system;
    use crate::problem::input::make_problem;
    use crate::problem::validate::validate_input;
    use crate::types::polynomial::{poly_add, poly_mul, variable_poly};

    #[test]
    fn weighted_primal_weights_are_algebraic_only() {
        let t = VariableId(0);
        let x = VariableId(1);
        let y = VariableId(2);
        let problem = make_problem(
            vec![t, x, y],
            t,
            vec![poly_add(
                &poly_mul(&variable_poly(t), &variable_poly(x)),
                &poly_mul(&variable_poly(x), &variable_poly(y)),
            )],
            Vec::new(),
        );
        let canonical = canonicalize_system(validate_input(problem).unwrap()).unwrap();
        let compressed = CompressionState::from_system(canonical).to_compressed_system();
        let h = build_relation_variable_hypergraph(&compressed);
        let influence = build_target_influence_graph(&h, t);
        let g = build_weighted_primal_graph(&compressed, &influence);
        let x_weight = g.variable_weights.get(&x).unwrap();
        assert_eq!(x_weight.degree_participation, 2);
        assert_eq!(x_weight.monomial_contribution, 2);
        assert_eq!(x_weight.occurrence_count, 1);
        assert!(g.edge_weights.contains_key(&edge_key(t, x)));
    }
}
