use std::collections::BTreeSet;

use crate::univariate::UniPolynomialFp;
use crate::window::CertificateWindow;
use crate::{compression::CertifiedSystemQ, Monomial, UniPolynomialQ};

pub trait CandidateOracle {
    fn generate(
        &self,
        system: &CertifiedSystemQ,
        window: &CertificateWindow,
    ) -> Vec<TargetCandidate>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetCandidate {
    pub support_mod_primes: Vec<UniPolynomialFp>,
    pub reconstructed: Option<UniPolynomialQ>,
    pub origin: CandidateOrigin,
    pub origin_evidence: BTreeSet<CandidateOrigin>,
    pub traces: Vec<CandidateTrace>,
}

impl TargetCandidate {
    pub(crate) fn from_origin(
        support_mod_primes: Vec<UniPolynomialFp>,
        reconstructed: Option<UniPolynomialQ>,
        origin: CandidateOrigin,
        traces: Vec<CandidateTrace>,
    ) -> Self {
        Self {
            support_mod_primes,
            reconstructed,
            origin,
            origin_evidence: BTreeSet::from([origin]),
            traces,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CandidateOrigin {
    DirectTargetEquation,
    NormTraceTower,
    ResidualCyclic,
    TargetCyclicKrylov,
    HiddenVariableSparseResultant,
    SliceSpecialization,
    LocalizedSchur,
    CompleteTargetElimination,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CandidateTrace {
    DirectEquation { equation_index: usize },
    ModularWitness(ModularWitnessTrace),
    SparseResultantWitness(SparseResultantWitnessTrace),
    RouteWitness(RouteWitnessTrace),
    SliceWitness(SliceWitnessTrace),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SparseResultantTemplateKind {
    SquareSparseResultant,
    OverdeterminedSparseEliminantMinor,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SparseResultantWitnessTrace {
    pub prime: u64,
    pub template_kind: SparseResultantTemplateKind,
    pub eliminated_variable_count: usize,
    pub equation_indices: Vec<usize>,
    pub support_sums: Vec<Vec<Monomial>>,
    pub row_support_size: usize,
    pub multiplier_support_sizes: Vec<usize>,
    pub minor_or_determinant_degree: usize,
    pub active_multiplier_supports: Vec<Vec<Monomial>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModularWitnessTrace {
    pub prime: u64,
    pub active_multiplier_supports: Vec<Vec<Monomial>>,
    pub relation_coefficients: Vec<u64>,
    pub residual_vectors: Vec<Vec<u64>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RouteWitnessTrace {
    pub origin: CandidateOrigin,
    pub equation_indices: Vec<usize>,
    pub support_size: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SliceWitnessTrace {
    pub prime: u64,
    pub assignments: Vec<SliceAssignment>,
    pub affine_equations: Vec<SliceAffineEquation>,
    pub equation_index: usize,
    pub equation_indices: Vec<usize>,
    pub internal_origin: CandidateOrigin,
    pub relation_coefficients: Vec<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SliceAssignment {
    pub variable_index: usize,
    pub value: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SliceAffineEquation {
    pub coefficients: Vec<SliceAffineCoefficient>,
    pub constant: u64,
    pub denominator_admissible: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SliceAffineCoefficient {
    pub variable_index: usize,
    pub coefficient: u64,
}
