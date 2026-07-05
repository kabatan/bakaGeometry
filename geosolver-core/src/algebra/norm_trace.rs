use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::algebra::resultant::{
    build_sparse_resultant_template, compute_resultant_relation, ModularOptions, ResultantInput,
};
use crate::result::status::{AlgebraicReason, FailureKind, SolverError, SolverErrorKind, StageId};
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::VariableId;
use crate::types::monomial::{monomial_to_bytes, normalize_monomial, Monomial};
use crate::types::polynomial::{
    clear_denominators_primitive, normalize_poly, poly_variables, SparsePolynomialQ, TermQ,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TowerDescription {
    pub algebraic_variable: VariableId,
    pub exported_variables: Vec<VariableId>,
    pub minimal_polynomial: SparsePolynomialQ,
    pub target_minus_expression: SparsePolynomialQ,
    pub source_relation_hashes: Vec<Hash>,
    pub tower_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UniOrMultiPolynomialQ {
    Multivariate(SparsePolynomialQ),
}

impl UniOrMultiPolynomialQ {
    pub fn as_multivariate(&self) -> &SparsePolynomialQ {
        match self {
            UniOrMultiPolynomialQ::Multivariate(poly) => poly,
        }
    }

    pub fn into_multivariate(self) -> SparsePolynomialQ {
        match self {
            UniOrMultiPolynomialQ::Multivariate(poly) => poly,
        }
    }
}

pub fn detect_explicit_tower(
    relations: &[SparsePolynomialQ],
    exported: &[VariableId],
) -> Option<TowerDescription> {
    let exported_variables = canonical_variables(exported)?;
    let exported_set: BTreeSet<_> = exported_variables.iter().copied().collect();
    let normalized_relations = relations
        .iter()
        .map(|relation| normalize_poly(relation.clone()))
        .filter(|relation| !relation.terms.is_empty())
        .collect::<Vec<_>>();
    if normalized_relations.len() < 2 {
        return None;
    }

    let all_variables = normalized_relations
        .iter()
        .flat_map(poly_variables)
        .collect::<BTreeSet<_>>();
    let algebraic_variables = all_variables
        .difference(&exported_set)
        .copied()
        .collect::<Vec<_>>();
    if algebraic_variables.len() != 1 {
        return None;
    }
    let algebraic_variable = algebraic_variables[0];

    let mut minimal_candidates = normalized_relations
        .iter()
        .filter(|relation| {
            let vars = poly_variables(relation);
            !vars.is_empty()
                && vars.iter().all(|var| *var == algebraic_variable)
                && degree_in_variable(relation, algebraic_variable) > 0
        })
        .cloned()
        .collect::<Vec<_>>();
    minimal_candidates
        .sort_by_key(|poly| (degree_in_variable(poly, algebraic_variable), poly.hash));
    let minimal_polynomial = minimal_candidates.first()?.clone();

    let allowed: BTreeSet<_> = exported_set
        .iter()
        .copied()
        .chain(std::iter::once(algebraic_variable))
        .collect();
    let mut expression_candidates = normalized_relations
        .iter()
        .filter(|relation| relation.hash != minimal_polynomial.hash)
        .filter(|relation| {
            let vars = poly_variables(relation);
            vars.iter().all(|var| allowed.contains(var))
                && vars.iter().any(|var| exported_set.contains(var))
        })
        .cloned()
        .collect::<Vec<_>>();
    expression_candidates.sort_by_key(|poly| (poly_variables(poly).len(), poly.hash));
    let target_minus_expression = expression_candidates.first()?.clone();

    let source_relation_hashes = normalized_relations
        .iter()
        .map(|relation| relation.hash)
        .collect::<Vec<_>>();
    let tower_hash = hash_tower(
        algebraic_variable,
        &exported_variables,
        &minimal_polynomial,
        &target_minus_expression,
        &source_relation_hashes,
    );
    Some(TowerDescription {
        algebraic_variable,
        exported_variables,
        minimal_polynomial,
        target_minus_expression,
        source_relation_hashes,
        tower_hash,
    })
}

pub fn norm_of_target_minus_expression(
    tower: &TowerDescription,
    target_expr: SparsePolynomialQ,
) -> Result<UniOrMultiPolynomialQ, SolverError> {
    let target_expr = normalize_poly(target_expr);
    validate_tower_expression(tower, &target_expr)?;
    let dim = degree_in_variable(&tower.minimal_polynomial, tower.algebraic_variable)
        + degree_in_variable(&target_expr, tower.algebraic_variable);
    if dim == 0 {
        return Err(algorithmic_hard_case(
            tower.algebraic_variable,
            "NormTraceResultant",
            "norm requires positive degree in the algebraic variable",
        ));
    }
    let template = build_sparse_resultant_template(ResultantInput {
        polynomials: vec![tower.minimal_polynomial.clone(), target_expr],
        eliminate: tower.algebraic_variable,
        keep_variables: tower.exported_variables.clone(),
        max_matrix_dim: dim as usize,
    })?;
    let relation = compute_resultant_relation(&template, ModularOptions::default())?;
    let primitive = clear_denominators_primitive(&relation.relation);
    if primitive.terms.is_empty() {
        return Err(algorithmic_hard_case(
            tower.algebraic_variable,
            "NormTraceResultant",
            "norm computation produced the zero polynomial",
        ));
    }
    Ok(UniOrMultiPolynomialQ::Multivariate(primitive))
}

pub fn verify_norm_relation(tower: &TowerDescription, relation: &SparsePolynomialQ) -> bool {
    let Ok(recomputed) =
        norm_of_target_minus_expression(tower, tower.target_minus_expression.clone())
    else {
        return false;
    };
    clear_denominators_primitive(recomputed.as_multivariate())
        == clear_denominators_primitive(&normalize_poly(relation.clone()))
}

fn validate_tower_expression(
    tower: &TowerDescription,
    expression: &SparsePolynomialQ,
) -> Result<(), SolverError> {
    let allowed: BTreeSet<_> = tower
        .exported_variables
        .iter()
        .copied()
        .chain(std::iter::once(tower.algebraic_variable))
        .collect();
    if poly_variables(expression)
        .iter()
        .any(|var| !allowed.contains(var))
    {
        return Err(SolverError::invalid_input(
            Some(tower.algebraic_variable),
            "target expression contains variables outside the detected algebraic tower",
        ));
    }
    if degree_in_variable(expression, tower.algebraic_variable) == 0 {
        return Err(algorithmic_hard_case(
            tower.algebraic_variable,
            "NormTraceResultant",
            "target expression is independent of the algebraic variable",
        ));
    }
    Ok(())
}

fn canonical_variables(vars: &[VariableId]) -> Option<Vec<VariableId>> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    let mut previous = None;
    for var in vars {
        if previous.is_some_and(|last| last > *var) || !seen.insert(*var) {
            return None;
        }
        out.push(*var);
        previous = Some(*var);
    }
    Some(out)
}

fn degree_in_variable(poly: &SparsePolynomialQ, var: VariableId) -> u32 {
    poly.terms
        .iter()
        .map(|term| exponent_of(&term.monomial, var))
        .max()
        .unwrap_or(0)
}

fn exponent_of(monomial: &Monomial, var: VariableId) -> u32 {
    monomial
        .exponents
        .iter()
        .find(|(v, _)| *v == var)
        .map_or(0, |(_, exp)| *exp)
}

fn hash_tower(
    algebraic_variable: VariableId,
    exported_variables: &[VariableId],
    minimal_polynomial: &SparsePolynomialQ,
    target_minus_expression: &SparsePolynomialQ,
    source_relation_hashes: &[Hash],
) -> Hash {
    let mut chunks = Vec::new();
    chunks.push(algebraic_variable.0.to_be_bytes().to_vec());
    for var in exported_variables {
        chunks.push(var.0.to_be_bytes().to_vec());
    }
    chunks.push(poly_bytes(minimal_polynomial));
    chunks.push(poly_bytes(target_minus_expression));
    for hash in source_relation_hashes {
        chunks.push(hash.0.to_vec());
    }
    hash_sequence("tower-description", &chunks)
}

fn poly_bytes(poly: &SparsePolynomialQ) -> Vec<u8> {
    let mut chunks = Vec::new();
    for term in &poly.terms {
        chunks.extend_from_slice(&crate::types::rational::rational_to_bytes(&term.coeff));
        chunks.extend_from_slice(&monomial_to_bytes(&term.monomial));
    }
    chunks
}

fn algorithmic_hard_case(target: VariableId, stage: &str, reason: &str) -> SolverError {
    SolverError {
        target: Some(target),
        kind: SolverErrorKind::Failure(FailureKind::AlgorithmicHardCase {
            stage: StageId(stage.to_owned()),
            reason: AlgebraicReason(reason.to_owned()),
            minimal_block_hash: hash_sequence(
                "p3f-norm-trace-hard-case",
                &[reason.as_bytes().to_vec()],
            ),
        }),
    }
}

#[allow(dead_code)]
fn term(coeff: crate::types::rational::RationalQ, entries: Vec<(VariableId, u32)>) -> TermQ {
    TermQ {
        coeff,
        monomial: normalize_monomial(entries),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::hash::hash_sequence;
    use crate::types::polynomial::{constant_poly, poly_add, poly_mul, poly_sub, variable_poly};
    use crate::types::rational::int_q;

    fn alpha() -> VariableId {
        VariableId(1)
    }

    fn target() -> VariableId {
        VariableId(2)
    }

    #[test]
    fn detects_explicit_tower_by_algebraic_form() {
        let a = variable_poly(alpha());
        let t = variable_poly(target());
        let min = poly_sub(&poly_mul(&a, &a), &constant_poly(int_q(2)));
        let expr = poly_sub(&t, &a);
        let tower = detect_explicit_tower(&[min.clone(), expr.clone()], &[target()]).unwrap();
        assert_eq!(tower.algebraic_variable, alpha());
        assert_eq!(tower.minimal_polynomial, min);
        assert_eq!(tower.target_minus_expression, expr);
    }

    #[test]
    fn norm_relation_is_exactly_recomputed() {
        let a = variable_poly(alpha());
        let t = variable_poly(target());
        let min = poly_sub(&poly_mul(&a, &a), &constant_poly(int_q(2)));
        let expr = poly_sub(&t, &a);
        let tower = detect_explicit_tower(&[min, expr.clone()], &[target()]).unwrap();
        let relation = norm_of_target_minus_expression(&tower, expr)
            .unwrap()
            .into_multivariate();
        let expected = poly_sub(&poly_mul(&t, &t), &constant_poly(int_q(2)));
        assert_eq!(
            clear_denominators_primitive(&relation),
            clear_denominators_primitive(&expected)
        );
        assert!(verify_norm_relation(&tower, &relation));
        assert!(!verify_norm_relation(
            &tower,
            &poly_add(&relation, &constant_poly(int_q(1)))
        ));
    }

    #[test]
    fn noncanonical_exported_variables_do_not_detect_a_tower() {
        let a = variable_poly(alpha());
        let t = variable_poly(target());
        let min = poly_sub(&poly_mul(&a, &a), &constant_poly(int_q(2)));
        let expr = poly_sub(&t, &a);
        assert!(detect_explicit_tower(&[min, expr], &[target(), target()]).is_none());
    }

    #[test]
    fn tower_hash_is_stable() {
        let a = variable_poly(alpha());
        let t = variable_poly(target());
        let min = poly_sub(&poly_mul(&a, &a), &constant_poly(int_q(2)));
        let expr = poly_sub(&t, &a);
        let first = detect_explicit_tower(&[min.clone(), expr.clone()], &[target()]).unwrap();
        let second = detect_explicit_tower(&[min, expr], &[target()]).unwrap();
        assert_eq!(first.tower_hash, second.tower_hash);
        assert_ne!(first.tower_hash, hash_sequence("tower-description", &[]));
    }
}
