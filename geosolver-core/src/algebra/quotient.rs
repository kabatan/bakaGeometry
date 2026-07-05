use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::result::status::SolverError;
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::VariableId;
use crate::types::matrix::VectorQ;
use crate::types::polynomial::{
    constant_poly, poly_add, poly_mul, poly_variables, substitute_poly, variable_poly,
    SparsePolynomialQ, SubstitutionMap,
};
use crate::types::rational::{add_q, int_q, is_zero_q, mul_q, rational_to_bytes, zero_q};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BasisHandleId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BasisScope {
    TargetRelevant { variables: Vec<VariableId> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuotientHandleInput {
    pub basis_id: BasisHandleId,
    pub basis_scope: BasisScope,
    pub basis_polynomials: Vec<SparsePolynomialQ>,
    pub variable_action_columns: BTreeMap<VariableId, Vec<VectorQ>>,
    pub no_coordinate_roots_exported: bool,
    pub no_full_coordinate_rur_exported: bool,
}

pub trait TargetQuotientHandle {
    fn basis_id(&self) -> BasisHandleId;
    fn basis_size(&self) -> usize;
    fn basis_scope(&self) -> BasisScope;
    fn normal_form(&self, p: &SparsePolynomialQ) -> Result<VectorQ, SolverError>;
    fn multiply_by_variable(&self, v: &VectorQ, var: VariableId) -> Result<VectorQ, SolverError>;
    fn no_coordinate_solution_export(&self) -> bool;
    fn basis_polynomial(&self, index: usize) -> Option<SparsePolynomialQ>;
    fn basis_hash(&self) -> Hash;
    fn quotient_handle_hash(&self) -> Hash;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExplicitTargetQuotientHandle {
    input: QuotientHandleInput,
    basis_hash: Hash,
    handle_hash: Hash,
}

pub fn build_target_relevant_quotient_handle(
    input: QuotientHandleInput,
) -> Result<Box<dyn TargetQuotientHandle>, SolverError> {
    validate_input(&input)?;
    let basis_hash = hash_basis(&input.basis_polynomials);
    let handle_hash = hash_handle(&input, basis_hash);
    Ok(Box::new(ExplicitTargetQuotientHandle {
        input,
        basis_hash,
        handle_hash,
    }))
}

impl TargetQuotientHandle for ExplicitTargetQuotientHandle {
    fn basis_id(&self) -> BasisHandleId {
        self.input.basis_id
    }

    fn basis_size(&self) -> usize {
        self.input.basis_polynomials.len()
    }

    fn basis_scope(&self) -> BasisScope {
        self.input.basis_scope.clone()
    }

    fn normal_form(&self, p: &SparsePolynomialQ) -> Result<VectorQ, SolverError> {
        let subst = self
            .basis_variables()
            .into_iter()
            .map(|var| (var, variable_poly(var)))
            .collect::<SubstitutionMap>();
        let normalized = substitute_poly(p, &subst);
        let mut acc = zero_vector(self.basis_size());
        for term in &normalized.terms {
            let mut v = unit_vector(self.basis_size(), 0);
            for (var, exp) in &term.monomial.exponents {
                for _ in 0..*exp {
                    v = self.multiply_by_variable(&v, *var)?;
                }
            }
            acc = vector_add(&acc, &vector_scale(&v, &term.coeff));
        }
        Ok(acc)
    }

    fn multiply_by_variable(&self, v: &VectorQ, var: VariableId) -> Result<VectorQ, SolverError> {
        if v.entries.len() != self.basis_size() {
            return Err(SolverError::invalid_input(
                Some(var),
                "quotient vector dimension does not match basis size",
            ));
        }
        let columns = self
            .input
            .variable_action_columns
            .get(&var)
            .ok_or_else(|| {
                SolverError::invalid_input(Some(var), "missing quotient variable action")
            })?;
        let mut acc = zero_vector(self.basis_size());
        for (coeff, column) in v.entries.iter().zip(columns) {
            if is_zero_q(coeff) {
                continue;
            }
            acc = vector_add(&acc, &vector_scale(column, coeff));
        }
        Ok(acc)
    }

    fn no_coordinate_solution_export(&self) -> bool {
        self.input.no_coordinate_roots_exported && self.input.no_full_coordinate_rur_exported
    }

    fn basis_polynomial(&self, index: usize) -> Option<SparsePolynomialQ> {
        self.input.basis_polynomials.get(index).cloned()
    }

    fn basis_hash(&self) -> Hash {
        self.basis_hash
    }

    fn quotient_handle_hash(&self) -> Hash {
        self.handle_hash
    }
}

impl ExplicitTargetQuotientHandle {
    fn basis_variables(&self) -> Vec<VariableId> {
        let mut vars = poly_variables(
            &self
                .input
                .basis_polynomials
                .iter()
                .fold(constant_poly(zero_q()), |acc, p| poly_add(&acc, p)),
        )
        .into_iter()
        .collect::<Vec<_>>();
        vars.sort();
        vars
    }
}

pub fn unit_vector(width: usize, index: usize) -> VectorQ {
    let mut entries = vec![zero_q(); width];
    if index < width {
        entries[index] = int_q(1);
    }
    VectorQ { entries }
}

pub fn zero_vector(width: usize) -> VectorQ {
    VectorQ {
        entries: vec![zero_q(); width],
    }
}

pub fn vector_add(a: &VectorQ, b: &VectorQ) -> VectorQ {
    assert_eq!(a.entries.len(), b.entries.len(), "vector width mismatch");
    VectorQ {
        entries: a
            .entries
            .iter()
            .zip(&b.entries)
            .map(|(x, y)| add_q(x, y))
            .collect(),
    }
}

pub fn vector_scale(v: &VectorQ, c: &crate::types::rational::RationalQ) -> VectorQ {
    VectorQ {
        entries: v.entries.iter().map(|x| mul_q(x, c)).collect(),
    }
}

pub fn vector_hash(v: &VectorQ) -> Hash {
    hash_sequence(
        "vector-q",
        &v.entries.iter().map(rational_to_bytes).collect::<Vec<_>>(),
    )
}

fn validate_input(input: &QuotientHandleInput) -> Result<(), SolverError> {
    let n = input.basis_polynomials.len();
    if n == 0 {
        return Err(SolverError::invalid_input(
            None,
            "quotient basis must be nonempty",
        ));
    }
    if !input.no_coordinate_roots_exported || !input.no_full_coordinate_rur_exported {
        return Err(SolverError::invalid_input(
            None,
            "quotient handle must not expose coordinate roots or full coordinate RUR",
        ));
    }
    for (var, columns) in &input.variable_action_columns {
        if columns.len() != n {
            return Err(SolverError::invalid_input(
                Some(*var),
                "quotient action column count must match basis size",
            ));
        }
        for column in columns {
            if column.entries.len() != n {
                return Err(SolverError::invalid_input(
                    Some(*var),
                    "quotient action column width must match basis size",
                ));
            }
        }
    }
    Ok(())
}

fn hash_basis(basis: &[SparsePolynomialQ]) -> Hash {
    hash_sequence(
        "quotient-basis",
        &basis
            .iter()
            .map(|poly| poly.hash.0.to_vec())
            .collect::<Vec<_>>(),
    )
}

fn hash_handle(input: &QuotientHandleInput, basis_hash: Hash) -> Hash {
    let mut chunks = vec![
        input.basis_id.0.to_be_bytes().to_vec(),
        basis_hash.0.to_vec(),
    ];
    for (var, columns) in &input.variable_action_columns {
        chunks.push(var.0.to_be_bytes().to_vec());
        chunks.extend(columns.iter().map(|column| vector_hash(column).0.to_vec()));
    }
    hash_sequence("quotient-handle", &chunks)
}

pub fn monomial_basis_polynomials(var: VariableId, size: usize) -> Vec<SparsePolynomialQ> {
    let mut basis = Vec::with_capacity(size);
    let mut current = constant_poly(int_q(1));
    for _ in 0..size {
        basis.push(current.clone());
        current = poly_mul(&current, &variable_poly(var));
    }
    basis
}

#[cfg(test)]
mod tests {
    use crate::types::polynomial::{poly_mul, variable_poly};

    use super::*;

    #[test]
    fn explicit_handle_exposes_no_coordinate_solution_api_and_normal_forms_exactly() {
        let t = VariableId(7);
        let input = QuotientHandleInput {
            basis_id: BasisHandleId(1),
            basis_scope: BasisScope::TargetRelevant { variables: vec![t] },
            basis_polynomials: monomial_basis_polynomials(t, 2),
            variable_action_columns: BTreeMap::from([(
                t,
                vec![
                    VectorQ {
                        entries: vec![int_q(0), int_q(1)],
                    },
                    VectorQ {
                        entries: vec![int_q(-2), int_q(3)],
                    },
                ],
            )]),
            no_coordinate_roots_exported: true,
            no_full_coordinate_rur_exported: true,
        };
        let handle = build_target_relevant_quotient_handle(input).unwrap();
        let t_squared = poly_mul(&variable_poly(t), &variable_poly(t));

        assert!(handle.no_coordinate_solution_export());
        assert_eq!(
            handle.normal_form(&t_squared).unwrap(),
            VectorQ {
                entries: vec![int_q(-2), int_q(3)]
            }
        );
    }
}
