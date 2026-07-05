use serde::{Deserialize, Serialize};

use crate::preprocess::compression::{CompressionState, GuardKind};
use crate::problem::context::SolverContext;
use crate::problem::semantic::RealConstraintEncoding;
use crate::result::status::SolverError;
use crate::types::ids::{RelationId, VariableId};
use crate::types::monomial::normalize_monomial;
use crate::types::polynomial::{constant_poly, normalize_poly, SparsePolynomialQ, TermQ};
use crate::types::rational::{int_q, is_zero_q, neg_q};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExplicitNonzeroWitness {
    pub factor: SparsePolynomialQ,
    pub witness_relation_id: RelationId,
    pub slack_variable: VariableId,
}

pub fn apply_explicit_saturations(
    mut state: CompressionState,
    _ctx: &mut SolverContext,
) -> Result<CompressionState, SolverError> {
    let witnesses = explicit_nonzero_witnesses(&state);
    for witness in witnesses {
        state.add_guard(
            witness.factor.clone(),
            vec![witness.witness_relation_id],
            GuardKind::ExplicitNonZeroWitness,
        );
        state.add_saturation(
            witness.factor,
            witness.witness_relation_id,
            witness.slack_variable,
        );
    }
    Ok(state)
}

pub fn is_explicit_nonzero_factor(
    factor: &SparsePolynomialQ,
    _semantics: &[RealConstraintEncoding],
) -> bool {
    is_nonzero_constant(factor)
}

pub fn find_explicit_nonzero_witness(
    state: &CompressionState,
    factor: &SparsePolynomialQ,
) -> Option<ExplicitNonzeroWitness> {
    let normalized = normalize_poly(factor.clone());
    explicit_nonzero_witnesses(state)
        .into_iter()
        .find(|witness| witness.factor == normalized)
}

pub fn explicit_nonzero_witnesses(state: &CompressionState) -> Vec<ExplicitNonzeroWitness> {
    let mut witnesses = state
        .relations
        .iter()
        .filter_map(|relation| {
            extract_nonzero_witness(&relation.polynomial).map(|(factor, slack_variable)| {
                ExplicitNonzeroWitness {
                    factor,
                    witness_relation_id: relation.id,
                    slack_variable,
                }
            })
        })
        .collect::<Vec<_>>();
    witnesses.sort_by_key(|witness| {
        (
            witness.factor.hash,
            witness.witness_relation_id,
            witness.slack_variable,
        )
    });
    witnesses.dedup_by_key(|witness| {
        (
            witness.factor.hash,
            witness.witness_relation_id,
            witness.slack_variable,
        )
    });
    witnesses
}

fn extract_nonzero_witness(poly: &SparsePolynomialQ) -> Option<(SparsePolynomialQ, VariableId)> {
    let mut variables = crate::types::polynomial::poly_variables(poly)
        .into_iter()
        .collect::<Vec<_>>();
    variables.sort_by(|a, b| b.cmp(a));
    for slack_variable in variables {
        let mut constant_part = constant_poly(int_q(0));
        let mut factor_terms = Vec::new();
        let mut valid = true;
        for term in &poly.terms {
            let exponent = term
                .monomial
                .exponents
                .iter()
                .find(|(var, _)| *var == slack_variable)
                .map_or(0, |(_, exp)| *exp);
            match exponent {
                0 => {
                    if !term.monomial.exponents.is_empty() {
                        valid = false;
                        break;
                    }
                    constant_part = normalize_poly(SparsePolynomialQ {
                        terms: vec![term.clone()],
                        hash: crate::types::hash::hash_sequence("poly", &[]),
                    });
                }
                1 => {
                    let reduced = normalize_monomial(
                        term.monomial
                            .exponents
                            .iter()
                            .filter_map(|(var, exp)| {
                                if *var == slack_variable {
                                    None
                                } else {
                                    Some((*var, *exp))
                                }
                            })
                            .collect(),
                    );
                    factor_terms.push(TermQ {
                        coeff: term.coeff.clone(),
                        monomial: reduced,
                    });
                }
                _ => {
                    valid = false;
                    break;
                }
            }
        }
        if !valid || factor_terms.is_empty() {
            continue;
        }
        let factor = normalize_poly(SparsePolynomialQ {
            terms: factor_terms,
            hash: crate::types::hash::hash_sequence("poly", &[]),
        });
        if factor.terms.is_empty() {
            continue;
        }
        if constant_part == constant_poly(int_q(-1)) {
            return Some((factor, slack_variable));
        }
        if constant_part == constant_poly(int_q(1)) {
            let neg_factor = normalize_poly(SparsePolynomialQ {
                terms: factor
                    .terms
                    .iter()
                    .map(|term| TermQ {
                        coeff: neg_q(&term.coeff),
                        monomial: term.monomial.clone(),
                    })
                    .collect(),
                hash: crate::types::hash::hash_sequence("poly", &[]),
            });
            return Some((neg_factor, slack_variable));
        }
    }
    None
}

fn is_nonzero_constant(poly: &SparsePolynomialQ) -> bool {
    poly.terms.len() == 1
        && poly.terms[0].monomial.exponents.is_empty()
        && !is_zero_q(&poly.terms[0].coeff)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::problem::canonicalize::canonicalize_system;
    use crate::problem::input::make_problem;
    use crate::problem::validate::validate_input;
    use crate::solver::options::SolverOptions;
    use crate::types::ids::VariableId;
    use crate::types::polynomial::{constant_poly, poly_mul, poly_sub, variable_poly};
    use crate::types::rational::int_q;

    #[test]
    fn saturation_records_only_explicit_nonzero_witnesses() {
        let t = VariableId(0);
        let a = VariableId(1);
        let s = VariableId(2);
        let witness = poly_sub(
            &poly_mul(&variable_poly(a), &variable_poly(s)),
            &constant_poly(int_q(1)),
        );
        let canonical = canonicalize_system(
            validate_input(make_problem(vec![t, a, s], t, vec![witness], Vec::new())).unwrap(),
        )
        .unwrap();
        let state = CompressionState::from_system(canonical);
        let mut ctx = crate::problem::context::new_context(SolverOptions::default());
        let state = apply_explicit_saturations(state, &mut ctx).unwrap();
        assert_eq!(state.saturations.len(), 1);
        assert_eq!(state.guards.len(), 1);
    }
}
