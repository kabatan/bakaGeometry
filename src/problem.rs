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
        self.variables.contains(&self.target)
            && self
                .equations
                .iter()
                .all(|equation| equation.variables == self.variables)
            && self
                .semantic_guards
                .iter()
                .all(|record| record.polynomial.variables == self.variables)
    }
}

pub(crate) fn verified_guard_count(guards: &[GuardCertificate]) -> usize {
    guards.len()
}
