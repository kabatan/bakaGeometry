use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::preprocess::compression::CompressedSystemQ;
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::{RelationId, VariableId};
use crate::types::polynomial::poly_variables;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationVariableHypergraph {
    pub relations: BTreeSet<RelationId>,
    pub variables: BTreeSet<VariableId>,
    pub relation_to_variables: BTreeMap<RelationId, BTreeSet<VariableId>>,
    pub variable_to_relations: BTreeMap<VariableId, BTreeSet<RelationId>>,
    pub hypergraph_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HypergraphComponent {
    pub relation_ids: BTreeSet<RelationId>,
    pub variables: BTreeSet<VariableId>,
    pub component_hash: Hash,
}

pub fn build_relation_variable_hypergraph(
    system: &CompressedSystemQ,
) -> RelationVariableHypergraph {
    let mut h = RelationVariableHypergraph {
        relations: BTreeSet::new(),
        variables: BTreeSet::new(),
        relation_to_variables: BTreeMap::new(),
        variable_to_relations: BTreeMap::new(),
        hypergraph_hash: hash_sequence("relation-variable-hypergraph", &[]),
    };
    for variable in &system.variables {
        h.variables.insert(*variable);
    }
    for relation in &system.relations {
        h.relations.insert(relation.id);
        let vars = poly_variables(&relation.polynomial);
        for var in vars {
            h.variables.insert(var);
            h.relation_to_variables
                .entry(relation.id)
                .or_default()
                .insert(var);
            h.variable_to_relations
                .entry(var)
                .or_default()
                .insert(relation.id);
        }
        h.relation_to_variables.entry(relation.id).or_default();
    }
    h.hypergraph_hash = hash_hypergraph(&h);
    h
}

pub fn connected_components(h: &RelationVariableHypergraph) -> Vec<HypergraphComponent> {
    let mut unvisited_relations = h.relations.clone();
    let mut unvisited_variables = h.variables.clone();
    let mut components = Vec::new();

    while let Some(start_relation) = unvisited_relations.iter().next().copied() {
        let mut relations = BTreeSet::new();
        let mut variables = BTreeSet::new();
        let mut relation_queue = VecDeque::from([start_relation]);
        unvisited_relations.remove(&start_relation);
        while let Some(relation) = relation_queue.pop_front() {
            relations.insert(relation);
            for var in relation_variables(h, relation) {
                if unvisited_variables.remove(&var) {
                    variables.insert(var);
                    for neighbor in variable_relations(h, var) {
                        if unvisited_relations.remove(&neighbor) {
                            relation_queue.push_back(neighbor);
                        }
                    }
                } else {
                    variables.insert(var);
                }
            }
        }
        let component_hash = hash_component(&relations, &variables);
        components.push(HypergraphComponent {
            relation_ids: relations,
            variables,
            component_hash,
        });
    }

    for var in unvisited_variables {
        let variables = BTreeSet::from([var]);
        components.push(HypergraphComponent {
            relation_ids: BTreeSet::new(),
            variables: variables.clone(),
            component_hash: hash_component(&BTreeSet::new(), &variables),
        });
    }
    components.sort_by_key(|component| component.component_hash);
    components
}

pub fn relation_variables(h: &RelationVariableHypergraph, r: RelationId) -> BTreeSet<VariableId> {
    h.relation_to_variables.get(&r).cloned().unwrap_or_default()
}

pub fn variable_relations(h: &RelationVariableHypergraph, v: VariableId) -> BTreeSet<RelationId> {
    h.variable_to_relations.get(&v).cloned().unwrap_or_default()
}

fn hash_hypergraph(h: &RelationVariableHypergraph) -> Hash {
    let mut chunks = Vec::new();
    for relation in &h.relations {
        chunks.push(relation.0.to_be_bytes().to_vec());
        for var in relation_variables(h, *relation) {
            chunks.push(var.0.to_be_bytes().to_vec());
        }
    }
    hash_sequence("relation-variable-hypergraph", &chunks)
}

fn hash_component(relations: &BTreeSet<RelationId>, variables: &BTreeSet<VariableId>) -> Hash {
    let mut chunks = Vec::new();
    for relation in relations {
        chunks.push(relation.0.to_be_bytes().to_vec());
    }
    for var in variables {
        chunks.push(var.0.to_be_bytes().to_vec());
    }
    hash_sequence("hypergraph-component", &chunks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preprocess::compression::pre_kernel_compress;
    use crate::problem::canonicalize::canonicalize_system;
    use crate::problem::input::make_problem;
    use crate::problem::validate::validate_input;
    use crate::solver::options::SolverOptions;
    use crate::types::polynomial::{poly_sub, variable_poly};

    #[test]
    fn hypergraph_represents_every_polynomial_occurrence() {
        let t = VariableId(0);
        let x = VariableId(1);
        let y = VariableId(2);
        let problem = make_problem(
            vec![t, x, y],
            t,
            vec![
                poly_sub(&variable_poly(t), &variable_poly(x)),
                variable_poly(y),
            ],
            Vec::new(),
        );
        let canonical = canonicalize_system(validate_input(problem).unwrap()).unwrap();
        let mut ctx = crate::problem::context::new_context(SolverOptions::default());
        let compressed = pre_kernel_compress(canonical, &mut ctx).unwrap();
        let h = build_relation_variable_hypergraph(&compressed);
        assert!(compressed
            .variables
            .iter()
            .all(|variable| h.variables.contains(variable)));
        for relation in &compressed.relations {
            assert_eq!(
                relation_variables(&h, relation.id),
                poly_variables(&relation.polynomial)
            );
        }
    }
}
