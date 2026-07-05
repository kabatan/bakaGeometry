use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::preprocess::compression::{
    component_hash, obligation_hash, polynomial_variable_map, sort_dedup_variables, Component,
    CompressionState, FeasibilityObligation,
};
use crate::problem::context::SolverContext;
use crate::result::status::SolverError;
use crate::types::ids::VariableId;
use crate::types::polynomial::poly_variables;

pub fn mark_target_independent_components(
    mut state: CompressionState,
    _ctx: &mut SolverContext,
) -> Result<CompressionState, SolverError> {
    let components = build_components(&state);
    let obligations = compute_component_feasibility_obligations(&components);
    let independent_relation_ids = components
        .iter()
        .filter(|component| !component.contains_target)
        .flat_map(|component| component.relation_ids.iter().copied())
        .collect::<BTreeSet<_>>();
    state.target_independent_components = components;
    state.feasibility_obligations = obligations;
    state.relations = state
        .relations
        .into_iter()
        .filter(|relation| !independent_relation_ids.contains(&relation.id))
        .collect();
    state.relation_order = state.relations.iter().map(|relation| relation.id).collect();
    Ok(state)
}

pub fn compute_component_feasibility_obligations(
    components: &[Component],
) -> Vec<FeasibilityObligation> {
    components
        .iter()
        .filter(|component| !component.contains_target)
        .map(|component| FeasibilityObligation {
            component_id: component.component_id,
            relation_ids: component.relation_ids.clone(),
            variables: component.variables.clone(),
            obligation_hash: obligation_hash(
                component.component_id,
                &component.relation_ids,
                &component.variables,
            ),
        })
        .collect()
}

fn build_components(state: &CompressionState) -> Vec<Component> {
    let variable_to_relations = polynomial_variable_map(&state.relations);
    let relation_to_variables = state
        .relations
        .iter()
        .map(|relation| {
            (
                relation.id,
                poly_variables(&relation.polynomial)
                    .into_iter()
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut relation_ids = state
        .relations
        .iter()
        .map(|relation| relation.id)
        .collect::<BTreeSet<_>>();
    let mut components = Vec::new();
    while let Some(start) = relation_ids.iter().next().copied() {
        let mut queue = VecDeque::from([start]);
        let mut component_relations = BTreeSet::new();
        let mut component_variables = BTreeSet::new();
        relation_ids.remove(&start);
        while let Some(relation_id) = queue.pop_front() {
            component_relations.insert(relation_id);
            for variable in relation_to_variables
                .get(&relation_id)
                .cloned()
                .unwrap_or_default()
            {
                component_variables.insert(variable);
                for neighbor in variable_to_relations
                    .get(&variable)
                    .cloned()
                    .unwrap_or_default()
                {
                    if relation_ids.remove(&neighbor) {
                        queue.push_back(neighbor);
                    }
                }
            }
        }
        let component_id = components.len();
        let relation_ids = component_relations.into_iter().collect::<Vec<_>>();
        let variables = sort_dedup_variables(component_variables);
        let contains_target = variables.contains(&state.target);
        let component_hash =
            component_hash(component_id, &relation_ids, &variables, contains_target);
        components.push(Component {
            component_id,
            relation_ids,
            variables,
            contains_target,
            component_hash,
        });
    }
    components
}

#[allow(dead_code)]
fn vars(v: impl IntoIterator<Item = VariableId>) -> Vec<VariableId> {
    sort_dedup_variables(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::problem::canonicalize::canonicalize_system;
    use crate::problem::input::make_problem;
    use crate::problem::validate::validate_input;
    use crate::types::ids::VariableId;
    use crate::types::polynomial::{constant_poly, poly_sub, variable_poly};
    use crate::types::rational::int_q;

    #[test]
    fn target_independent_component_becomes_feasibility_obligation() {
        let t = VariableId(0);
        let x = VariableId(1);
        let u = VariableId(2);
        let canonical = canonicalize_system(
            validate_input(make_problem(
                vec![t, x, u],
                t,
                vec![
                    poly_sub(&variable_poly(t), &variable_poly(x)),
                    poly_sub(&variable_poly(u), &constant_poly(int_q(1))),
                ],
                Vec::new(),
            ))
            .unwrap(),
        )
        .unwrap();
        let state = CompressionState::from_system(canonical);
        let mut ctx =
            crate::problem::context::new_context(crate::solver::options::SolverOptions::default());
        let state = mark_target_independent_components(state, &mut ctx).unwrap();
        assert_eq!(state.feasibility_obligations.len(), 1);
        assert_eq!(state.relations.len(), 1);
        assert!(state.relations[0].polynomial == poly_sub(&variable_poly(t), &variable_poly(x)));
    }
}
