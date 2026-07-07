use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::preprocess::compression::{relation_with_polynomial, CompressionState};
use crate::problem::context::SolverContext;
use crate::problem::semantic::semantic_relations;
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
    let semantic_relation_ids = semantic_relations(&state.semantic_encodings);
    let mut by_hash = BTreeMap::new();
    for relation in &state.relations {
        let primitive = clear_denominators_primitive(&relation.polynomial);
        let normalized_relation =
            relation_with_polynomial(relation.id, primitive, relation.source.clone());
        by_hash
            .entry(normalized_relation.polynomial.hash)
            .or_insert_with(Vec::new)
            .push(normalized_relation);
    }
    let mut relations = Vec::new();
    for (_, mut group) in by_hash {
        group.sort_by_key(|relation| relation.id);
        if group
            .iter()
            .any(|relation| semantic_relation_ids.contains(&relation.id))
        {
            relations.extend(group);
        } else if let Some(first) = group.into_iter().next() {
            relations.push(first);
        }
    }
    relations.sort_by_key(|relation| relation.id);
    state.replace_relations(relations);
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preprocess::saturation::apply_explicit_saturations;
    use crate::problem::canonicalize::canonicalize_system;
    use crate::problem::input::make_problem;
    use crate::problem::semantic::{register_slack_encoding, RealConstraintKind};
    use crate::problem::validate::validate_input;
    use crate::types::ids::VariableId;
    use crate::types::polynomial::{constant_poly, poly_mul, poly_scale, poly_sub, variable_poly};
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

    #[test]
    fn binomial_dedup_keeps_semantically_referenced_duplicate_relation_id() {
        let t = VariableId(0);
        let a = VariableId(1);
        let s = VariableId(2);
        let witness = poly_sub(
            &poly_mul(&variable_poly(a), &variable_poly(s)),
            &constant_poly(int_q(1)),
        );
        let duplicate = poly_scale(&witness, &int_q(3));
        let canonical = canonicalize_system(
            validate_input(make_problem(
                vec![t, a, s],
                t,
                vec![witness, duplicate],
                vec![register_slack_encoding(
                    RealConstraintKind::NonZero,
                    vec![RelationId(1)],
                    vec![s],
                )],
            ))
            .unwrap(),
        )
        .unwrap();
        let state = CompressionState::from_system(canonical);
        let mut ctx =
            crate::problem::context::new_context(crate::solver::options::SolverOptions::default());
        let state = simplify_binomial_relations(state, &mut ctx).unwrap();
        assert!(state
            .relations
            .iter()
            .any(|relation| relation.id == RelationId(1)));

        let state = apply_explicit_saturations(state, &mut ctx).unwrap();
        assert_eq!(state.saturations.len(), 1);
    }
}
