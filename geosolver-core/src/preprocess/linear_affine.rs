use serde::{Deserialize, Serialize};

use crate::algebra::monomial_order::lex_order;
use crate::algebra::polynomial_ops::reduce_by_set;
use crate::preprocess::compression::{
    affine_parts_in_variable, sort_dedup_variables, CompressionState, GuardKind, SubstitutionKind,
};
use crate::preprocess::saturation::find_explicit_nonzero_witness;
use crate::problem::context::SolverContext;
use crate::result::status::SolverError;
use crate::types::ids::{RelationId, VariableId};
use crate::types::polynomial::{
    constant_poly, poly_scale, poly_variables, zero_poly, SparsePolynomialQ,
};
use crate::types::rational::{div_q, int_q, is_zero_q, RationalQ};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinearAffineCandidate {
    pub variable: VariableId,
    pub source_relation_id: RelationId,
    pub denominator: SparsePolynomialQ,
    pub numerator: SparsePolynomialQ,
    pub denominator_is_constant_nonzero: bool,
    pub denominator_has_recorded_nonzero_semantics: bool,
    pub polynomial_expression: Option<SparsePolynomialQ>,
    pub score: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PivotPolicy {
    MarkowitzLikePolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AffinePivot {
    pub variable: VariableId,
    pub source_relation_id: RelationId,
    pub denominator: SparsePolynomialQ,
    pub expression: SparsePolynomialQ,
    pub denominator_is_constant_nonzero: bool,
    pub denominator_has_recorded_nonzero_semantics: bool,
}

pub fn find_linear_affine_candidates(state: &CompressionState) -> Vec<LinearAffineCandidate> {
    let mut candidates = Vec::new();
    for relation in &state.relations {
        let variables = poly_variables(&relation.polynomial);
        for variable in variables {
            if variable == state.target {
                continue;
            }
            let Some((denominator, rest)) =
                affine_parts_in_variable(&relation.polynomial, variable)
            else {
                continue;
            };
            if denominator.terms.is_empty() || poly_variables(&rest).contains(&variable) {
                continue;
            }
            let numerator = poly_scale(&rest, &int_q(-1));
            let denominator_is_constant_nonzero = constant_nonzero(&denominator).is_some();
            let denominator_has_recorded_nonzero_semantics =
                find_explicit_nonzero_witness(state, &denominator).is_some();
            let polynomial_expression = exact_polynomial_quotient(&numerator, &denominator);
            candidates.push(LinearAffineCandidate {
                variable,
                source_relation_id: relation.id,
                denominator,
                numerator,
                denominator_is_constant_nonzero,
                denominator_has_recorded_nonzero_semantics,
                polynomial_expression,
                score: relation.polynomial.terms.len(),
            });
        }
    }
    candidates.sort_by_key(|candidate| {
        (
            safe_rank(candidate),
            candidate.score,
            candidate.variable,
            candidate.source_relation_id,
            candidate.denominator.hash,
        )
    });
    candidates
}

pub fn select_safe_affine_pivots(
    candidates: &[LinearAffineCandidate],
    _policy: PivotPolicy,
) -> Vec<AffinePivot> {
    candidates
        .iter()
        .filter_map(|candidate| {
            let expression = candidate.polynomial_expression.clone()?;
            if !candidate.denominator_is_constant_nonzero
                && !candidate.denominator_has_recorded_nonzero_semantics
            {
                return None;
            }
            Some(AffinePivot {
                variable: candidate.variable,
                source_relation_id: candidate.source_relation_id,
                denominator: candidate.denominator.clone(),
                expression,
                denominator_is_constant_nonzero: candidate.denominator_is_constant_nonzero,
                denominator_has_recorded_nonzero_semantics: candidate
                    .denominator_has_recorded_nonzero_semantics,
            })
        })
        .collect()
}

pub fn eliminate_linear_affine_variables(
    mut state: CompressionState,
    _ctx: &mut SolverContext,
) -> Result<CompressionState, SolverError> {
    loop {
        let candidates = find_linear_affine_candidates(&state);
        let pivots = select_safe_affine_pivots(&candidates, PivotPolicy::MarkowitzLikePolicy);
        let Some(pivot) = pivots
            .into_iter()
            .find(|pivot| state.variables.contains(&pivot.variable))
        else {
            break;
        };
        let guard = if pivot.denominator_is_constant_nonzero {
            state.add_guard(
                pivot.denominator.clone(),
                vec![pivot.source_relation_id],
                GuardKind::ConstantNonZeroPivot,
            );
            None
        } else {
            state.add_guard(
                pivot.denominator.clone(),
                vec![pivot.source_relation_id],
                GuardKind::AffineDenominator,
            );
            Some(pivot.denominator.clone())
        };
        state.apply_polynomial_substitution(
            pivot.variable,
            &pivot.expression,
            pivot.source_relation_id,
        );
        state.add_substitution(
            pivot.variable,
            pivot.expression,
            guard,
            pivot.source_relation_id,
            if pivot.denominator_is_constant_nonzero {
                SubstitutionKind::LinearAffineConstantPivot
            } else {
                SubstitutionKind::LinearAffineGuardedPivot
            },
        );
    }
    Ok(state)
}

fn exact_polynomial_quotient(
    numerator: &SparsePolynomialQ,
    denominator: &SparsePolynomialQ,
) -> Option<SparsePolynomialQ> {
    if numerator.terms.is_empty() {
        return Some(zero_poly());
    }
    if let Some(constant) = constant_nonzero(denominator) {
        let scale = div_q(&int_q(1), constant).ok()?;
        return Some(poly_scale(numerator, &scale));
    }
    let variables = sort_dedup_variables(
        poly_variables(numerator)
            .into_iter()
            .chain(poly_variables(denominator)),
    );
    let reduction = reduce_by_set(numerator, &[denominator.clone()], &lex_order(&variables));
    if reduction.remainder.terms.is_empty() {
        Some(reduction.quotients[0].clone())
    } else {
        None
    }
}

fn constant_nonzero(poly: &SparsePolynomialQ) -> Option<&RationalQ> {
    if poly.terms.len() == 1
        && poly.terms[0].monomial.exponents.is_empty()
        && !is_zero_q(&poly.terms[0].coeff)
    {
        Some(&poly.terms[0].coeff)
    } else {
        None
    }
}

fn safe_rank(candidate: &LinearAffineCandidate) -> usize {
    match (
        candidate.denominator_is_constant_nonzero,
        candidate.denominator_has_recorded_nonzero_semantics,
        candidate.polynomial_expression.is_some(),
    ) {
        (true, _, true) => 0,
        (false, true, true) => 1,
        _ => 2,
    }
}

#[allow(dead_code)]
fn one() -> SparsePolynomialQ {
    constant_poly(int_q(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::problem::canonicalize::canonicalize_system;
    use crate::problem::input::make_problem;
    use crate::problem::validate::validate_input;
    use crate::types::polynomial::{constant_poly, poly_mul, poly_sub, variable_poly};
    use crate::types::rational::int_q;

    fn state_from_relations(relations: Vec<SparsePolynomialQ>) -> CompressionState {
        let t = VariableId(0);
        let vars = vec![t, VariableId(1), VariableId(2), VariableId(3)];
        let canonical = canonicalize_system(
            validate_input(make_problem(vars, t, relations, Vec::new())).unwrap(),
        )
        .unwrap();
        CompressionState::from_system(canonical)
    }

    #[test]
    fn rejects_nonconstant_affine_without_explicit_guard() {
        let x = VariableId(1);
        let y = VariableId(2);
        let relation = poly_mul(
            &poly_sub(&variable_poly(x), &constant_poly(int_q(1))),
            &poly_sub(&variable_poly(y), &constant_poly(int_q(2))),
        );
        let state = state_from_relations(vec![relation]);
        let candidates = find_linear_affine_candidates(&state);
        assert!(!candidates.is_empty());
        assert!(
            select_safe_affine_pivots(&candidates, PivotPolicy::MarkowitzLikePolicy).is_empty()
        );
    }

    #[test]
    fn guarded_affine_pivot_records_denominator_guard() {
        let x = VariableId(1);
        let y = VariableId(2);
        let s = VariableId(3);
        let factor = poly_sub(&variable_poly(x), &constant_poly(int_q(1)));
        let witness = poly_sub(
            &poly_mul(&factor, &variable_poly(s)),
            &constant_poly(int_q(1)),
        );
        let relation = poly_mul(
            &factor,
            &poly_sub(&variable_poly(y), &constant_poly(int_q(2))),
        );
        let state = state_from_relations(vec![witness, relation]);
        let mut ctx =
            crate::problem::context::new_context(crate::solver::options::SolverOptions::default());
        let state = eliminate_linear_affine_variables(state, &mut ctx).unwrap();
        assert!(state
            .substitutions
            .iter()
            .any(|sub| sub.kind == SubstitutionKind::LinearAffineGuardedPivot));
        assert!(state
            .guards
            .iter()
            .any(|guard| guard.guard_kind == GuardKind::AffineDenominator));
    }
}
