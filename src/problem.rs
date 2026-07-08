use std::collections::BTreeSet;

use crate::{GuardCertificate, PolynomialQ, Variable};

#[derive(Clone, Debug)]
pub struct TargetProblemQ {
    pub equations: Vec<PolynomialQ>,
    pub variables: Vec<Variable>,
    pub target: Variable,
    pub semantic_guards: Vec<GuardRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuardRecord {
    pub polynomial: PolynomialQ,
    pub kind: GuardKind,
    pub provenance: GuardProvenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuardKind {
    NonZero,
    Positive,
    Negative,
    NonNegative,
    NonPositive,
    OtherSemanticCondition,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuardProvenance {
    pub description: String,
}

impl TargetProblemQ {
    pub(crate) fn is_well_formed(&self) -> bool {
        self.variables
            .iter()
            .filter(|variable| *variable == &self.target)
            .count()
            == 1
            && self.variables.iter().collect::<BTreeSet<_>>().len() == self.variables.len()
            && self.equations.iter().all(|equation| {
                equation.variables == self.variables && polynomial_terms_have_valid_arity(equation)
            })
            && self.semantic_guards.iter().all(|record| {
                record.polynomial.variables == self.variables
                    && polynomial_terms_have_valid_arity(&record.polynomial)
            })
    }
}

pub(crate) fn polynomial_terms_have_valid_arity(polynomial: &PolynomialQ) -> bool {
    polynomial
        .terms
        .keys()
        .all(|monomial| monomial.exponents.len() == polynomial.variables.len())
}

pub(crate) fn verified_guard_count(guards: &[GuardCertificate]) -> usize {
    guards.len()
}
