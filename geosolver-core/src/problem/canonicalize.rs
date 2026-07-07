use serde::{Deserialize, Serialize};

use crate::problem::semantic::{verify_semantic_references, RealConstraintEncoding};
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
    let target = validated.problem.target;
    let semantic_encodings = validated.problem.semantic_encodings;
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
                Some(target),
                "nonzero constant relation makes the algebraic system contradictory",
            ));
        }
        relations.push(relation);
    }
    relations.sort_by_key(|relation| relation.polynomial.hash);
    if semantic_encodings.is_empty() {
        relations = relations
            .into_iter()
            .enumerate()
            .map(|(idx, relation)| {
                canonicalize_relation(RelationId(idx as u32), relation.polynomial)
            })
            .collect();
    }
    let relation_order = relations
        .iter()
        .map(|relation| relation.id)
        .collect::<Vec<_>>();
    verify_semantic_references(&semantic_encodings, &relation_order, &variables).map_err(|_| {
        SolverError::invalid_input(
            Some(target),
            "semantic encoding became inconsistent during canonicalization",
        )
    })?;
    let variable_order = canonical_variable_order(&variables, target);
    let canonical_hash = hash_canonical_system(
        &variable_order,
        &relation_order,
        &relations,
        &semantic_encodings,
    );
    Ok(CanonicalSystemQ {
        variables: variables.clone(),
        target,
        relations,
        relation_order,
        variable_order,
        semantic_encodings,
        canonical_hash,
        diagnostics,
    })
}

fn hash_canonical_system(
    variable_order: &VariableOrder,
    relation_order: &[RelationId],
    relations: &[CanonicalRelationQ],
    semantic_encodings: &[RealConstraintEncoding],
) -> Hash {
    let mut chunks = Vec::new();
    chunks.push(
        variable_order
            .variables
            .iter()
            .flat_map(|variable| variable.0.to_be_bytes())
            .collect(),
    );
    for relation_id in relation_order {
        chunks.push(relation_id.0.to_be_bytes().to_vec());
    }
    for relation in relations {
        chunks.push(relation.id.0.to_be_bytes().to_vec());
        chunks.push(relation.hash.0.to_vec());
    }
    for encoding in semantic_encodings {
        chunks.push(encoding.semantic_hash.0.to_vec());
    }
    hash_sequence("canonical-system", &chunks)
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
    use crate::problem::semantic::{register_slack_encoding, RealConstraintKind};
    use crate::problem::validate::validate_input;
    use crate::types::ids::{RelationId, VariableId};
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

    #[test]
    fn zero_relation_removal_cannot_drop_semantic_provenance() {
        let t = VariableId(0);
        let s = VariableId(1);
        let semantic =
            register_slack_encoding(RealConstraintKind::Positive, vec![RelationId(0)], vec![s]);
        let problem = make_problem(vec![t, s], t, vec![constant_poly(int_q(0))], vec![semantic]);

        assert!(canonicalize_system(validate_input(problem).unwrap()).is_err());
    }

    #[test]
    fn canonical_hash_binds_semantic_provenance() {
        let t = VariableId(0);
        let s = VariableId(1);
        let equation = variable_poly(t);
        let without_semantic = make_problem(vec![t, s], t, vec![equation.clone()], Vec::new());
        let with_semantic = make_problem(
            vec![t, s],
            t,
            vec![equation],
            vec![register_slack_encoding(
                RealConstraintKind::NonNegative,
                vec![RelationId(0)],
                vec![s],
            )],
        );

        let without_semantic =
            canonicalize_system(validate_input(without_semantic).unwrap()).unwrap();
        let with_semantic = canonicalize_system(validate_input(with_semantic).unwrap()).unwrap();
        assert_ne!(
            without_semantic.canonical_hash,
            with_semantic.canonical_hash
        );
        assert_eq!(with_semantic.semantic_encodings.len(), 1);
    }

    #[test]
    fn relation_order_is_content_canonicalized() {
        let x = VariableId(0);
        let y = VariableId(1);
        let first = variable_poly(x);
        let second = variable_poly(y);
        let forward = make_problem(
            vec![x, y],
            x,
            vec![first.clone(), second.clone()],
            Vec::new(),
        );
        let reversed = make_problem(vec![x, y], x, vec![second, first], Vec::new());

        let forward = canonicalize_system(validate_input(forward).unwrap()).unwrap();
        let reversed = canonicalize_system(validate_input(reversed).unwrap()).unwrap();

        assert_eq!(
            forward
                .relations
                .iter()
                .map(|relation| relation.polynomial.hash)
                .collect::<Vec<_>>(),
            reversed
                .relations
                .iter()
                .map(|relation| relation.polynomial.hash)
                .collect::<Vec<_>>()
        );
    }
}
