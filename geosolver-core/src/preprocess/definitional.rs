use serde::{Deserialize, Serialize};

use crate::preprocess::compression::{
    affine_parts_in_variable, CompressionState, SubstitutionKind,
};
use crate::problem::canonicalize::CanonicalSystemQ;
use crate::problem::context::SolverContext;
use crate::result::status::SolverError;
use crate::types::ids::{RelationId, VariableId};
use crate::types::polynomial::{poly_scale, poly_variables, SparsePolynomialQ};
use crate::types::rational::{div_q, int_q, is_zero_q, neg_q, RationalQ};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefinitionalCandidate {
    pub variable: VariableId,
    pub source_relation_id: RelationId,
    pub coefficient: RationalQ,
    pub expression: SparsePolynomialQ,
    pub score: usize,
}

pub fn find_definitional_relations(system: &CanonicalSystemQ) -> Vec<DefinitionalCandidate> {
    let mut candidates = Vec::new();
    for relation in &system.relations {
        let variables = poly_variables(&relation.polynomial);
        for variable in variables {
            if variable == system.target {
                continue;
            }
            let Some((coefficient_poly, rest)) =
                affine_parts_in_variable(&relation.polynomial, variable)
            else {
                continue;
            };
            let Some(coefficient) = constant_coefficient(&coefficient_poly) else {
                continue;
            };
            if is_zero_q(coefficient) {
                continue;
            }
            let Ok(inv_coeff) = div_q(&int_q(-1), coefficient) else {
                continue;
            };
            let expression = poly_scale(&rest, &inv_coeff);
            if poly_variables(&expression).contains(&variable) {
                continue;
            }
            candidates.push(DefinitionalCandidate {
                variable,
                source_relation_id: relation.id,
                coefficient: coefficient.clone(),
                expression,
                score: relation.polynomial.terms.len(),
            });
        }
    }
    candidates.sort_by_key(|candidate| {
        (
            candidate.score,
            candidate.variable,
            candidate.source_relation_id,
            candidate.expression.hash,
        )
    });
    candidates
}

pub fn apply_definitional_elimination(
    mut state: CompressionState,
    _candidates: &[DefinitionalCandidate],
    _ctx: &mut SolverContext,
) -> Result<CompressionState, SolverError> {
    loop {
        let current = state.to_canonical_system();
        let Some(candidate) = find_definitional_relations(&current)
            .into_iter()
            .find(|candidate| {
                candidate.variable != state.target
                    && state.variables.contains(&candidate.variable)
                    && state
                        .relations
                        .iter()
                        .any(|relation| relation.id == candidate.source_relation_id)
            })
        else {
            break;
        };
        state.apply_polynomial_substitution(
            candidate.variable,
            &candidate.expression,
            candidate.source_relation_id,
        );
        state.add_substitution(
            candidate.variable,
            candidate.expression,
            None,
            candidate.source_relation_id,
            SubstitutionKind::Definitional,
        );
    }
    Ok(state)
}

fn constant_coefficient(poly: &SparsePolynomialQ) -> Option<&RationalQ> {
    if poly.terms.len() == 1 && poly.terms[0].monomial.exponents.is_empty() {
        Some(&poly.terms[0].coeff)
    } else {
        None
    }
}

#[allow(dead_code)]
fn neg_poly(poly: &SparsePolynomialQ) -> SparsePolynomialQ {
    poly_scale(poly, &neg_q(&int_q(1)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preprocess::compression::CompressionState;
    use crate::problem::canonicalize::canonicalize_system;
    use crate::problem::context::new_context;
    use crate::problem::input::make_problem;
    use crate::problem::validate::validate_input;
    use crate::solver::options::SolverOptions;
    use crate::types::polynomial::{constant_poly, poly_sub, variable_poly};
    use crate::types::rational::int_q;

    #[test]
    fn finds_constant_pivot_definitions_without_using_target() {
        let t = VariableId(0);
        let x = VariableId(1);
        let y = VariableId(2);
        let problem = make_problem(
            vec![t, x, y],
            t,
            vec![poly_sub(&variable_poly(y), &constant_poly(int_q(3)))],
            Vec::new(),
        );
        let canonical = canonicalize_system(validate_input(problem).unwrap()).unwrap();
        let candidates = find_definitional_relations(&canonical);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].variable, y);
        assert_eq!(candidates[0].expression, constant_poly(int_q(3)));
    }

    #[test]
    fn cascading_definitions_are_recomputed_after_each_substitution() {
        let t = VariableId(0);
        let s = VariableId(1);
        let x = VariableId(2);
        let canonical = canonicalize_system(
            validate_input(make_problem(
                vec![t, s, x],
                t,
                vec![
                    poly_sub(&variable_poly(x), &constant_poly(int_q(2))),
                    poly_sub(&variable_poly(s), &variable_poly(x)),
                    poly_sub(&variable_poly(t), &variable_poly(s)),
                ],
                Vec::new(),
            ))
            .unwrap(),
        )
        .unwrap();
        let state = CompressionState::from_system(canonical);
        let mut ctx = new_context(SolverOptions::default());
        let state = apply_definitional_elimination(state, &[], &mut ctx).unwrap();

        assert!(
            state.relations.iter().any(|relation| relation.polynomial
                == poly_sub(&constant_poly(int_q(2)), &variable_poly(t))),
            "relations={:?}",
            state.relations
        );
    }
}
