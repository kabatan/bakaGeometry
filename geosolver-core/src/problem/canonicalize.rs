use serde::{Deserialize, Serialize};

use crate::problem::semantic::RealConstraintEncoding;
use crate::problem::validate::ValidatedProblem;
use crate::result::diagnostics::DiagnosticRecord;
use crate::result::status::{SolverError, StageId};
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::{RelationId, VariableId};
use crate::types::polynomial::{
    clear_denominators_primitive, poly_monomial_count, SparsePolynomialQ,
};
use crate::types::rational::is_zero_q;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalSystemQ {
    pub variables: Vec<VariableId>,
    pub target: VariableId,
    pub relations: Vec<CanonicalRelationQ>,
    pub relation_order: Vec<RelationId>,
    pub variable_order: VariableOrder,
    pub semantic_encodings: Vec<RealConstraintEncoding>,
    pub canonical_hash: Hash,
    pub diagnostics: Vec<DiagnosticRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalRelationQ {
    pub id: RelationId,
    pub polynomial: SparsePolynomialQ,
    pub source: RelationSource,
    pub hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VariableOrder {
    pub variables: Vec<VariableId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationSource {
    InputEquation,
    SemanticEncoding,
}

pub fn canonicalize_system(validated: ValidatedProblem) -> Result<CanonicalSystemQ, SolverError> {
    let variables = validated.problem.variables.clone();
    let mut relations = Vec::new();
    let mut diagnostics = Vec::new();
    for (idx, polynomial) in validated.problem.equations.into_iter().enumerate() {
        let relation = canonicalize_relation(RelationId(idx as u32), polynomial);
        if relation.polynomial.terms.is_empty() {
            diagnostics.push(DiagnosticRecord::new(
                "ZeroRelationRemoved",
                "zero relation removed during canonicalization",
                Some(StageId("CanonicalizeSystem".to_string())),
            ));
            continue;
        }
        if is_nonzero_constant(&relation.polynomial) {
            return Err(SolverError::invalid_input(
                Some(validated.problem.target),
                "nonzero constant relation makes the algebraic system contradictory",
            ));
        }
        relations.push(relation);
    }
    let relation_order = relations
        .iter()
        .map(|relation| relation.id)
        .collect::<Vec<_>>();
    let canonical_hash = hash_sequence(
        "canonical-system",
        &relations
            .iter()
            .map(|relation| relation.hash.0.to_vec())
            .collect::<Vec<_>>(),
    );
    Ok(CanonicalSystemQ {
        variables: variables.clone(),
        target: validated.problem.target,
        relations,
        relation_order,
        variable_order: canonical_variable_order(&variables, validated.problem.target),
        semantic_encodings: validated.problem.semantic_encodings,
        canonical_hash,
        diagnostics,
    })
}

pub fn canonicalize_relation(id: RelationId, p: SparsePolynomialQ) -> CanonicalRelationQ {
    let polynomial = clear_denominators_primitive(&p);
    let hash = hash_sequence(
        "canonical-relation",
        &[id.0.to_be_bytes().to_vec(), polynomial.hash.0.to_vec()],
    );
    CanonicalRelationQ {
        id,
        polynomial,
        source: RelationSource::InputEquation,
        hash,
    }
}

pub fn canonical_variable_order(vars: &[VariableId], target: VariableId) -> VariableOrder {
    let mut variables = vars.to_vec();
    variables.sort();
    variables.retain(|v| *v != target);
    variables.insert(0, target);
    VariableOrder { variables }
}

fn is_nonzero_constant(p: &SparsePolynomialQ) -> bool {
    poly_monomial_count(p) == 1
        && p.terms[0].monomial.exponents.is_empty()
        && !is_zero_q(&p.terms[0].coeff)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::problem::input::make_problem;
    use crate::problem::validate::validate_input;
    use crate::types::ids::VariableId;
    use crate::types::polynomial::{constant_poly, variable_poly};
    use crate::types::rational::{int_q, new_q};
    use num_bigint::BigInt;

    #[test]
    fn denominator_normalization_is_preserved() {
        let problem = make_problem(
            vec![VariableId(0), VariableId(1)],
            VariableId(0),
            vec![constant_poly(new_q(BigInt::from(2), BigInt::from(4)))],
            Vec::new(),
        );
        assert!(canonicalize_system(validate_input(problem).unwrap()).is_err());

        let problem = make_problem(
            vec![VariableId(0), VariableId(1)],
            VariableId(0),
            vec![crate::types::polynomial::poly_scale(
                &variable_poly(VariableId(1)),
                &new_q(BigInt::from(2), BigInt::from(4)),
            )],
            Vec::new(),
        );
        let canonical = canonicalize_system(validate_input(problem).unwrap()).unwrap();
        assert_eq!(
            canonical.relations[0].polynomial,
            variable_poly(VariableId(1))
        );
    }

    #[test]
    fn zero_relations_are_removed_with_diagnostics() {
        let problem = make_problem(
            vec![VariableId(0)],
            VariableId(0),
            vec![constant_poly(int_q(0))],
            Vec::new(),
        );
        let canonical = canonicalize_system(validate_input(problem).unwrap()).unwrap();
        assert!(canonical.relations.is_empty());
        assert_eq!(canonical.diagnostics[0].name, "ZeroRelationRemoved");
    }

    #[test]
    fn nonzero_constant_relation_is_contradiction() {
        let problem = make_problem(
            vec![VariableId(0)],
            VariableId(0),
            vec![constant_poly(int_q(1))],
            Vec::new(),
        );
        assert!(canonicalize_system(validate_input(problem).unwrap()).is_err());
    }
}
