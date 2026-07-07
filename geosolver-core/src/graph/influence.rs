use std::collections::{BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::graph::hypergraph::{
    connected_components, relation_variables, variable_relations, HypergraphComponent,
    RelationVariableHypergraph,
};
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::VariableId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetInfluenceGraph {
    pub target: VariableId,
    pub target_component: HypergraphComponent,
    pub independent_components: Vec<HypergraphComponent>,
    pub influence_hash: Hash,
}

pub fn build_target_influence_graph(
    h: &RelationVariableHypergraph,
    target: VariableId,
) -> TargetInfluenceGraph {
    let target_component = bfs_target_component(h, target);
    let components = connected_components(h);
    let independent_components = components
        .into_iter()
        .filter(|component| {
            component
                .relation_ids
                .is_disjoint(&target_component.relation_ids)
                && component.variables.is_disjoint(&target_component.variables)
        })
        .collect::<Vec<_>>();
    let influence_hash = hash_influence(target, &target_component, &independent_components);
    TargetInfluenceGraph {
        target,
        target_component,
        independent_components,
        influence_hash,
    }
}

fn bfs_target_component(h: &RelationVariableHypergraph, target: VariableId) -> HypergraphComponent {
    let mut relation_ids = BTreeSet::new();
    let mut variables = BTreeSet::from([target]);
    let mut variable_queue = VecDeque::from([target]);
    let mut relation_queue = VecDeque::new();

    while !variable_queue.is_empty() || !relation_queue.is_empty() {
        while let Some(variable) = variable_queue.pop_front() {
            for relation in variable_relations(h, variable) {
                if relation_ids.insert(relation) {
                    relation_queue.push_back(relation);
                }
            }
        }
        while let Some(relation) = relation_queue.pop_front() {
            for variable in relation_variables(h, relation) {
                if variables.insert(variable) {
                    variable_queue.push_back(variable);
                }
            }
        }
    }

    let component_hash = hash_component(&relation_ids, &variables);
    HypergraphComponent {
        relation_ids,
        variables,
        component_hash,
    }
}

fn hash_influence(
    target: VariableId,
    target_component: &HypergraphComponent,
    independent_components: &[HypergraphComponent],
) -> Hash {
    let mut chunks = vec![
        target.0.to_be_bytes().to_vec(),
        target_component.component_hash.0.to_vec(),
    ];
    for component in independent_components {
        chunks.push(component.component_hash.0.to_vec());
    }
    hash_sequence("target-influence", &chunks)
}

fn hash_component(
    relations: &BTreeSet<crate::types::ids::RelationId>,
    variables: &BTreeSet<VariableId>,
) -> Hash {
    let mut chunks = Vec::new();
    for relation in relations {
        chunks.push(relation.0.to_be_bytes().to_vec());
    }
    for variable in variables {
        chunks.push(variable.0.to_be_bytes().to_vec());
    }
    hash_sequence("hypergraph-component", &chunks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::hypergraph::build_relation_variable_hypergraph;
    use crate::preprocess::compression::pre_kernel_compress;
    use crate::problem::canonicalize::canonicalize_system;
    use crate::problem::input::make_problem;
    use crate::problem::validate::validate_input;
    use crate::solver::options::SolverOptions;
    use crate::types::polynomial::{constant_poly, poly_mul, poly_sub, variable_poly};
    use crate::types::rational::int_q;

    #[test]
    fn target_influence_bfs_starts_from_target_and_separates_isolated_component() {
        let t = VariableId(0);
        let x = VariableId(1);
        let u = VariableId(2);
        let problem = make_problem(
            vec![t, x, u],
            t,
            vec![
                poly_sub(
                    &poly_mul(&variable_poly(t), &variable_poly(x)),
                    &constant_poly(int_q(1)),
                ),
                poly_sub(
                    &poly_mul(&variable_poly(u), &variable_poly(u)),
                    &constant_poly(int_q(2)),
                ),
            ],
            Vec::new(),
        );
        let canonical = canonicalize_system(validate_input(problem).unwrap()).unwrap();
        let mut ctx = crate::problem::context::new_context(SolverOptions::default());
        let compressed = pre_kernel_compress(canonical, &mut ctx).unwrap();
        let h = build_relation_variable_hypergraph(&compressed);
        let influence = build_target_influence_graph(&h, t);
        assert!(influence.target_component.variables.contains(&t));
        assert!(influence.target_component.variables.contains(&x));
        assert!(!influence.target_component.variables.contains(&u));
        assert!(influence
            .independent_components
            .iter()
            .any(|component| component.variables.contains(&u)));
    }
}
