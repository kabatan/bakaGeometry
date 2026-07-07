use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::algebra::monomial_order::lex_order;
use crate::algebra::polynomial_ops::reduce_by_set;
use crate::preprocess::compression::{
    affine_parts_in_variable, rational_expression, sort_dedup_variables, CompressionState,
    GuardKind, SubstitutionKind,
};
use crate::preprocess::saturation::{find_explicit_nonzero_witness, ExplicitNonzeroWitness};
use crate::problem::context::SolverContext;
use crate::result::diagnostics::DiagnosticRecord;
use crate::result::status::SolverError;
use crate::result::status::StageId;
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
    pub denominator_nonzero_witness: Option<ExplicitNonzeroWitness>,
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
    pub numerator: SparsePolynomialQ,
    pub expression: AffinePivotExpression,
    pub denominator_is_constant_nonzero: bool,
    pub denominator_has_recorded_nonzero_semantics: bool,
    pub denominator_nonzero_witness: Option<ExplicitNonzeroWitness>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AffinePivotExpression {
    Polynomial(SparsePolynomialQ),
    Rational {
        numerator: SparsePolynomialQ,
        denominator: SparsePolynomialQ,
    },
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
            let denominator_nonzero_witness = find_explicit_nonzero_witness(state, &denominator);
            if denominator_nonzero_witness.as_ref().is_some_and(|witness| {
                witness.witness_relation_id == relation.id && witness.slack_variable == variable
            }) {
                continue;
            }
            let denominator_has_recorded_nonzero_semantics = denominator_nonzero_witness.is_some();
            let polynomial_expression = exact_polynomial_quotient(&numerator, &denominator);
            candidates.push(LinearAffineCandidate {
                variable,
                source_relation_id: relation.id,
                denominator,
                numerator,
                denominator_is_constant_nonzero,
                denominator_has_recorded_nonzero_semantics,
                denominator_nonzero_witness,
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
    let variables_with_unguarded_nonconstant_denominator = candidates
        .iter()
        .filter(|candidate| {
            !candidate.denominator_is_constant_nonzero
                && !candidate.denominator_has_recorded_nonzero_semantics
        })
        .map(|candidate| candidate.variable)
        .collect::<BTreeSet<_>>();
    candidates
        .iter()
        .filter_map(|candidate| {
            if variables_with_unguarded_nonconstant_denominator.contains(&candidate.variable) {
                return None;
            }
            if !candidate.denominator_is_constant_nonzero
                && !candidate.denominator_has_recorded_nonzero_semantics
            {
                return None;
            }
            let expression = match &candidate.polynomial_expression {
                Some(poly) => AffinePivotExpression::Polynomial(poly.clone()),
                None if candidate.denominator_has_recorded_nonzero_semantics => {
                    AffinePivotExpression::Rational {
                        numerator: candidate.numerator.clone(),
                        denominator: candidate.denominator.clone(),
                    }
                }
                None => return None,
            };
            Some(AffinePivot {
                variable: candidate.variable,
                source_relation_id: candidate.source_relation_id,
                denominator: candidate.denominator.clone(),
                numerator: candidate.numerator.clone(),
                expression,
                denominator_is_constant_nonzero: candidate.denominator_is_constant_nonzero,
                denominator_has_recorded_nonzero_semantics: candidate
                    .denominator_has_recorded_nonzero_semantics,
                denominator_nonzero_witness: candidate.denominator_nonzero_witness.clone(),
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
            if candidates.iter().any(|candidate| {
                !candidate.denominator_is_constant_nonzero
                    && !candidate.denominator_has_recorded_nonzero_semantics
            }) {
                state.diagnostics.push(DiagnosticRecord::new(
                    "UnsafeAffinePivotRejected",
                    "linear affine elimination rejected nonconstant denominator without recorded nonzero guard",
                    Some(StageId("PreKernelAlgebraicCompression::LinearAffineElimination".to_owned())),
                ));
            }
            break;
        };
        let guard_record = if pivot.denominator_is_constant_nonzero {
            state.add_guard(
                pivot.denominator.clone(),
                vec![pivot.source_relation_id],
                GuardKind::ConstantNonZeroPivot,
            )
        } else {
            let mut source_ids = vec![pivot.source_relation_id];
            if let Some(witness) = &pivot.denominator_nonzero_witness {
                source_ids.push(witness.witness_relation_id);
            }
            state.add_guard(
                pivot.denominator.clone(),
                source_ids,
                GuardKind::AffineDenominator,
            )
        };
        let guard = if pivot.denominator_is_constant_nonzero {
            None
        } else {
            Some(pivot.denominator.clone())
        };
        match pivot.expression {
            AffinePivotExpression::Polynomial(expression) => {
                state.apply_polynomial_substitution(
                    pivot.variable,
                    &expression,
                    pivot.source_relation_id,
                );
                state.add_substitution(
                    pivot.variable,
                    expression,
                    guard,
                    pivot.source_relation_id,
                    if pivot.denominator_is_constant_nonzero {
                        SubstitutionKind::LinearAffineConstantPivot
                    } else {
                        SubstitutionKind::LinearAffineGuardedPivot
                    },
                );
            }
            AffinePivotExpression::Rational {
                numerator,
                denominator,
            } => {
                let witness_ids = pivot
                    .denominator_nonzero_witness
                    .as_ref()
                    .map(|witness| vec![witness.witness_relation_id])
                    .unwrap_or_default();
                state.apply_rational_affine_substitution(
                    pivot.variable,
                    &numerator,
                    &denominator,
                    pivot.source_relation_id,
                    guard_record.clone(),
                    witness_ids.clone(),
                );
                let expression =
                    rational_expression(numerator, denominator, guard_record, witness_ids);
                state.add_rational_substitution(
                    pivot.variable,
                    expression,
                    guard,
                    pivot.source_relation_id,
                    SubstitutionKind::LinearAffineGuardedPivot,
                );
            }
        }
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
        (false, true, false) => 0,
        (false, true, true) => 1,
        (true, _, true) => 2,
        _ => 3,
    }
}

#[allow(dead_code)]
fn one() -> SparsePolynomialQ {
    constant_poly(int_q(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algebra::groebner::{groebner_elimination_basis, GroebnerOptions};
    use crate::algebra::monomial_order::lex_order;
    use crate::algebra::polynomial_ops::reduce_by_set;
    use crate::preprocess::compression::{
        rational_affine_transformation_certificate, CompressionExpression,
    };
    use crate::problem::canonicalize::canonicalize_system;
    use crate::problem::input::make_problem;
    use crate::problem::semantic::{register_slack_encoding, RealConstraintKind};
    use crate::problem::validate::validate_input;
    use crate::types::polynomial::{
        clear_denominators_primitive, constant_poly, poly_add, poly_mul, poly_scale, poly_sub,
        variable_poly,
    };
    use crate::types::rational::int_q;

    fn state_from_relations(relations: Vec<SparsePolynomialQ>) -> CompressionState {
        let t = VariableId(0);
        let vars = vec![t, VariableId(1), VariableId(2), VariableId(3)];
        state_from_relations_with_vars(t, vars, relations)
    }

    fn state_from_relations_with_vars(
        target: VariableId,
        vars: Vec<VariableId>,
        relations: Vec<SparsePolynomialQ>,
    ) -> CompressionState {
        let canonical = canonicalize_system(
            validate_input(make_problem(vars, target, relations, Vec::new())).unwrap(),
        )
        .unwrap();
        CompressionState::from_system(canonical)
    }

    fn state_from_relations_with_nonzero_semantic(
        relations: Vec<SparsePolynomialQ>,
        witness_relation: RelationId,
        slack: VariableId,
    ) -> CompressionState {
        let t = VariableId(0);
        let vars = vec![t, VariableId(1), VariableId(2), VariableId(3)];
        let canonical = canonicalize_system(
            validate_input(make_problem(
                vars,
                t,
                relations,
                vec![register_slack_encoding(
                    RealConstraintKind::NonZero,
                    vec![witness_relation],
                    vec![slack],
                )],
            ))
            .unwrap(),
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
        let state =
            state_from_relations_with_nonzero_semantic(vec![witness, relation], RelationId(0), s);
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

    #[test]
    fn guarded_rational_affine_non_polynomial_case_clears_denominator() {
        let t = VariableId(0);
        let x = VariableId(1);
        let y = VariableId(2);
        let s = VariableId(3);
        let denominator = poly_add(&variable_poly(x), &constant_poly(int_q(1)));
        let witness = poly_sub(
            &poly_mul(&denominator, &variable_poly(s)),
            &constant_poly(int_q(1)),
        );
        let affine = poly_sub(
            &poly_mul(&denominator, &variable_poly(y)),
            &poly_add(&variable_poly(t), &variable_poly(x)),
        );
        let y_minus_two = poly_sub(&variable_poly(y), &constant_poly(int_q(2)));
        let state = state_from_relations_with_nonzero_semantic(
            vec![witness.clone(), affine.clone(), y_minus_two.clone()],
            RelationId(0),
            s,
        );
        let witness_relation = state
            .relations
            .iter()
            .find(|relation| relation.polynomial == clear_denominators_primitive(&witness))
            .expect("witness relation is present after canonicalization")
            .clone();
        let pivot_relation = state
            .relations
            .iter()
            .find(|relation| relation.polynomial == clear_denominators_primitive(&affine))
            .expect("affine pivot relation is present after canonicalization")
            .clone();
        let y_minus_two_relation = state
            .relations
            .iter()
            .find(|relation| relation.polynomial == clear_denominators_primitive(&y_minus_two))
            .expect("y - 2 relation is present after canonicalization")
            .clone();
        let candidates = find_linear_affine_candidates(&state);
        let rational_candidate = candidates
            .iter()
            .find(|candidate| {
                candidate.variable == y
                    && candidate.denominator_has_recorded_nonzero_semantics
                    && candidate.polynomial_expression.is_none()
            })
            .expect("guarded non-polynomial rational affine pivot is selectable");
        let expected_pivot_numerator = rational_candidate.numerator.clone();
        let expected_pivot_denominator = rational_candidate.denominator.clone();

        let mut ctx =
            crate::problem::context::new_context(crate::solver::options::SolverOptions::default());
        let state = eliminate_linear_affine_variables(state, &mut ctx).unwrap();

        assert!(state.substitutions.iter().any(|sub| {
            sub.kind == SubstitutionKind::LinearAffineGuardedPivot
                && matches!(sub.expression, CompressionExpression::Rational(_))
        }));
        assert!(state
            .guards
            .iter()
            .any(|guard| guard.guard_kind == GuardKind::AffineDenominator));
        assert!(state
            .relations
            .iter()
            .all(|relation| !poly_variables(&relation.polynomial).contains(&y)));
        let expected = poly_sub(
            &poly_sub(
                &poly_mul(&constant_poly(int_q(2)), &denominator),
                &variable_poly(t),
            ),
            &variable_poly(x),
        );
        let transformed_y_minus_two_relation = state
            .relations
            .iter()
            .find(|relation| relation.id == y_minus_two_relation.id)
            .expect("the transformed y - 2 relation remains relation-id bound");
        assert_eq!(transformed_y_minus_two_relation.polynomial, expected);
        let cert = state
            .rational_affine_transformations
            .iter()
            .find(|cert| cert.original_relation_id == y_minus_two_relation.id)
            .expect("the transformed y - 2 relation has a rational affine certificate");
        assert_eq!(cert.original_relation_id, y_minus_two_relation.id);
        assert_eq!(
            cert.transformed_relation_id,
            transformed_y_minus_two_relation.id
        );
        assert_eq!(cert.pivot_relation_id, pivot_relation.id);
        assert_eq!(cert.eliminated_variable, y);
        assert_eq!(cert.numerator, expected_pivot_numerator);
        assert_eq!(cert.denominator, expected_pivot_denominator);
        assert_eq!(cert.denominator_clearing_power, 1);
        assert_eq!(cert.source_witness_relation_ids, vec![witness_relation.id]);
        assert_eq!(
            cert.denominator_guard.guard_kind,
            GuardKind::AffineDenominator
        );
        assert_eq!(
            cert.denominator_guard.factor,
            clear_denominators_primitive(&cert.denominator)
        );
        assert_eq!(
            cert.denominator_guard.source_relation_ids,
            vec![pivot_relation.id, witness_relation.id]
        );
        assert_eq!(cert.original_relation_hash, y_minus_two_relation.hash);
        assert_eq!(
            cert.transformed_relation_hash,
            transformed_y_minus_two_relation.hash
        );
        let recomputed = rational_affine_transformation_certificate(
            y_minus_two_relation.id,
            transformed_y_minus_two_relation.id,
            pivot_relation.id,
            y,
            cert.numerator.clone(),
            cert.denominator.clone(),
            1,
            cert.denominator_guard.clone(),
            vec![witness_relation.id],
            y_minus_two_relation.hash,
            transformed_y_minus_two_relation.hash,
        );
        assert_eq!(cert.transformation_hash, recomputed.transformation_hash);
        let tampered = rational_affine_transformation_certificate(
            y_minus_two_relation.id,
            transformed_y_minus_two_relation.id,
            pivot_relation.id,
            y,
            cert.numerator.clone(),
            cert.denominator.clone(),
            0,
            cert.denominator_guard.clone(),
            vec![witness_relation.id],
            y_minus_two_relation.hash,
            transformed_y_minus_two_relation.hash,
        );
        assert_ne!(cert.transformation_hash, tampered.transformation_hash);
    }

    #[test]
    fn p12g_g5_guarded_rational_affine_records_guard_and_preserves_target_support() {
        let t = VariableId(0);
        let x = VariableId(1);
        let y = VariableId(2);
        let s = VariableId(3);
        let denominator = poly_add(&variable_poly(x), &constant_poly(int_q(1)));
        let witness = poly_sub(
            &poly_mul(&denominator, &variable_poly(s)),
            &constant_poly(int_q(1)),
        );
        let affine = poly_sub(
            &poly_mul(&denominator, &variable_poly(y)),
            &poly_add(&variable_poly(t), &variable_poly(x)),
        );
        let y_square_minus_two = poly_sub(
            &poly_mul(&variable_poly(y), &variable_poly(y)),
            &constant_poly(int_q(2)),
        );
        let x_minus_one = poly_sub(&variable_poly(x), &constant_poly(int_q(1)));
        let state = state_from_relations_with_nonzero_semantic(
            vec![witness, affine, y_square_minus_two, x_minus_one.clone()],
            RelationId(0),
            s,
        );

        let mut ctx =
            crate::problem::context::new_context(crate::solver::options::SolverOptions::default());
        let state = eliminate_linear_affine_variables(state, &mut ctx).unwrap();
        let expected_target_support = poly_sub(
            &poly_mul(
                &poly_add(&variable_poly(t), &constant_poly(int_q(1))),
                &poly_add(&variable_poly(t), &constant_poly(int_q(1))),
            ),
            &constant_poly(int_q(8)),
        );
        let target_plus_x = poly_add(&variable_poly(t), &variable_poly(x));
        let expected_guarded_relation = poly_sub(
            &poly_mul(
                &constant_poly(int_q(2)),
                &poly_mul(&denominator, &denominator),
            ),
            &poly_mul(&target_plus_x, &target_plus_x),
        );

        assert!(state
            .guards
            .iter()
            .any(|guard| guard.guard_kind == GuardKind::AffineDenominator));
        assert!(state.substitutions.iter().any(|sub| {
            sub.kind == SubstitutionKind::LinearAffineGuardedPivot
                && matches!(sub.expression, CompressionExpression::Rational(_))
        }));
        assert!(
            state.relations.iter().any(|relation| same_poly_up_to_sign(
                &relation.polynomial,
                &expected_guarded_relation
            )),
            "relations={:?}",
            state
                .relations
                .iter()
                .map(|relation| &relation.polynomial)
                .collect::<Vec<_>>()
        );
        assert!(state
            .relations
            .iter()
            .any(|relation| same_poly_up_to_sign(&relation.polynomial, &x_minus_one)));
        assert!(same_poly_up_to_sign(
            &substitute_x_equals_one(&expected_guarded_relation, x),
            &expected_target_support
        ));
    }

    #[test]
    fn p12g_g3_bilinear_structure_preprocesses_to_target_support_under_permutation() {
        let t = VariableId(0);
        let a = VariableId(3);
        let b = VariableId(11);
        let u = VariableId(5);
        let v = VariableId(2);
        let bilinear = poly_sub(
            &poly_sub(
                &poly_mul(&variable_poly(a), &variable_poly(u)),
                &poly_mul(&variable_poly(b), &variable_poly(v)),
            ),
            &variable_poly(t),
        );
        let state = state_from_relations_with_vars(
            t,
            vec![v, a, u, t, b],
            vec![
                poly_sub(&variable_poly(v), &constant_poly(int_q(5))),
                poly_sub(&variable_poly(b), &constant_poly(int_q(2))),
                bilinear,
                poly_sub(&variable_poly(u), &constant_poly(int_q(3))),
                poly_sub(&variable_poly(a), &constant_poly(int_q(1))),
            ],
        );
        let mut ctx =
            crate::problem::context::new_context(crate::solver::options::SolverOptions::default());
        let state = eliminate_linear_affine_variables(state, &mut ctx).unwrap();
        let state = eliminate_linear_affine_variables(state, &mut ctx).unwrap();

        let support = poly_add(&variable_poly(t), &constant_poly(int_q(7)));
        assert!(relations_imply_support(
            &state,
            &support,
            vec![v, b, u, a, t]
        ));
    }

    #[test]
    fn p12g_g4_quadratic_structure_preprocesses_to_target_support() {
        let t = VariableId(0);
        let p = VariableId(4);
        let q = VariableId(2);
        let r = VariableId(6);
        let quadratic = poly_sub(
            &poly_add(
                &poly_mul(&variable_poly(p), &variable_poly(p)),
                &poly_mul(&variable_poly(q), &variable_poly(q)),
            ),
            &variable_poly(r),
        );
        let state = state_from_relations_with_vars(
            t,
            vec![t, p, q, r],
            vec![
                poly_sub(&variable_poly(q), &constant_poly(int_q(1))),
                quadratic,
                poly_sub(&variable_poly(r), &constant_poly(int_q(5))),
                poly_sub(&variable_poly(p), &variable_poly(t)),
            ],
        );
        let mut ctx =
            crate::problem::context::new_context(crate::solver::options::SolverOptions::default());
        let state = eliminate_linear_affine_variables(state, &mut ctx).unwrap();
        let state = eliminate_linear_affine_variables(state, &mut ctx).unwrap();

        let support = poly_sub(
            &poly_mul(&variable_poly(t), &variable_poly(t)),
            &constant_poly(int_q(4)),
        );
        assert!(relations_imply_support(&state, &support, vec![q, r, p, t]));
    }

    fn same_poly_up_to_sign(
        a: &crate::types::polynomial::SparsePolynomialQ,
        b: &crate::types::polynomial::SparsePolynomialQ,
    ) -> bool {
        a == b || *a == poly_scale(b, &int_q(-1))
    }

    fn substitute_x_equals_one(
        poly: &crate::types::polynomial::SparsePolynomialQ,
        x: VariableId,
    ) -> crate::types::polynomial::SparsePolynomialQ {
        let mut subst = crate::types::polynomial::SubstitutionMap::new();
        subst.insert(x, constant_poly(int_q(1)));
        crate::types::polynomial::substitute_poly(poly, &subst)
    }

    fn relations_imply_support(
        state: &CompressionState,
        support: &crate::types::polynomial::SparsePolynomialQ,
        variables: Vec<VariableId>,
    ) -> bool {
        let relations = state
            .relations
            .iter()
            .map(|relation| relation.polynomial.clone())
            .collect::<Vec<_>>();
        let basis = groebner_elimination_basis(
            &relations,
            &lex_order(&variables),
            GroebnerOptions {
                max_pairs: 1024,
                max_basis_size: 128,
            },
        )
        .unwrap()
        .basis
        .into_iter()
        .map(|certified| certified.polynomial)
        .collect::<Vec<_>>();
        reduce_by_set(support, &basis, &lex_order(&variables))
            .remainder
            .terms
            .is_empty()
    }

    #[test]
    fn unsafe_nonconstant_rational_affine_candidate_is_left_in_system() {
        let t = VariableId(0);
        let x = VariableId(1);
        let y = VariableId(2);
        let denominator = poly_add(&variable_poly(x), &constant_poly(int_q(1)));
        let affine = poly_sub(
            &poly_mul(&denominator, &variable_poly(y)),
            &poly_add(&variable_poly(t), &variable_poly(x)),
        );
        let y_minus_two = poly_sub(&variable_poly(y), &constant_poly(int_q(2)));
        let state = state_from_relations(vec![affine.clone(), y_minus_two]);
        let candidates = find_linear_affine_candidates(&state);
        assert!(candidates.iter().any(|candidate| {
            candidate.variable == y
                && !candidate.denominator_has_recorded_nonzero_semantics
                && candidate.polynomial_expression.is_none()
        }));
        assert!(
            select_safe_affine_pivots(&candidates, PivotPolicy::MarkowitzLikePolicy).is_empty()
        );

        let mut ctx =
            crate::problem::context::new_context(crate::solver::options::SolverOptions::default());
        let state = eliminate_linear_affine_variables(state, &mut ctx).unwrap();
        assert!(state.substitutions.is_empty());
        assert!(state
            .diagnostics
            .iter()
            .any(|diag| diag.name == "UnsafeAffinePivotRejected"));
        assert!(state.relations.iter().any(|relation| {
            let vars = poly_variables(&relation.polynomial);
            vars.contains(&y) && vars.contains(&x) && vars.contains(&t)
        }));
    }
}
