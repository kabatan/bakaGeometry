use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::preprocess::compression::{relation_with_polynomial, CompressionState};
use crate::problem::context::SolverContext;
use crate::result::status::SolverError;
use crate::types::ids::RelationId;
use crate::types::polynomial::{clear_denominators_primitive, SparsePolynomialQ};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BinomialCandidate {
    pub source_relation_id: RelationId,
    pub relation: SparsePolynomialQ,
    pub term_count: usize,
}

pub fn detect_binomial_relations(state: &CompressionState) -> Vec<BinomialCandidate> {
    state
        .relations
        .iter()
        .filter_map(|relation| {
            let primitive = clear_denominators_primitive(&relation.polynomial);
            let term_count = primitive.terms.len();
            if (1..=2).contains(&term_count) {
                Some(BinomialCandidate {
                    source_relation_id: relation.id,
                    relation: primitive,
                    term_count,
                })
            } else {
                None
            }
        })
        .collect()
}

pub fn simplify_binomial_relations(
    mut state: CompressionState,
    _ctx: &mut SolverContext,
) -> Result<CompressionState, SolverError> {
    let mut by_hash = BTreeMap::new();
    for relation in &state.relations {
        let primitive = clear_denominators_primitive(&relation.polynomial);
        let normalized_relation =
            relation_with_polynomial(relation.id, primitive, relation.source.clone());
        by_hash
            .entry(normalized_relation.polynomial.hash)
            .or_insert(normalized_relation);
    }
    let mut relations = by_hash.into_values().collect::<Vec<_>>();
    relations.sort_by_key(|relation| relation.id);
    state.replace_relations(relations);
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::problem::canonicalize::canonicalize_system;
    use crate::problem::input::make_problem;
    use crate::problem::validate::validate_input;
    use crate::types::ids::VariableId;
    use crate::types::polynomial::{poly_scale, poly_sub, variable_poly};
    use crate::types::rational::int_q;

    #[test]
    fn binomial_simplification_removes_scaled_duplicates_without_factor_split() {
        let t = VariableId(0);
        let x = VariableId(1);
        let y = VariableId(2);
        let binomial = poly_sub(&variable_poly(x), &variable_poly(y));
        let duplicate = poly_scale(&binomial, &int_q(3));
        let canonical = canonicalize_system(
            validate_input(make_problem(
                vec![t, x, y],
                t,
                vec![binomial.clone(), duplicate],
                Vec::new(),
            ))
            .unwrap(),
        )
        .unwrap();
        let state = CompressionState::from_system(canonical);
        let mut ctx =
            crate::problem::context::new_context(crate::solver::options::SolverOptions::default());
        let state = simplify_binomial_relations(state, &mut ctx).unwrap();
        assert_eq!(state.relations.len(), 1);
        assert_eq!(state.relations[0].polynomial, binomial);
    }
}
