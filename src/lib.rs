#![forbid(unsafe_code)]

mod arith;
mod candidate_direct;
mod candidate_krylov;
mod candidate_residual;
mod candidate_resultant;
mod candidate_slice;
mod candidate_tower;
mod candidates;
mod certificates;
mod compression;
mod crt;
mod dependency_dag;
mod error;
mod exact_image;
mod fallback_elimination;
mod finite_field;
mod guards;
mod linear_fp;
mod linear_q;
mod matrix_fp;
mod matrix_q;
mod monomial;
mod normalize;
mod options;
mod polynomial;
mod problem;
mod proof;
mod proof_learning;
mod proof_schedule;
mod rational_reconstruction;
mod repair_multiple;
mod repair_schur;
mod residual;
mod roots;
mod solver;
mod trace;
mod univariate;
mod variable;
mod verifier;
mod window;

#[cfg(test)]
mod test_support;

pub use crate::compression::{
    certified_system_from_problem, validate_target_problem, verify_compression_replay,
    CertifiedSystemQ, CompressionReplayCertificate, CompressionStepCertificate,
    CompressionVerificationError, ProblemValidationError,
};
pub use certificates::{
    ComponentUnionSource, CompositeRule, EliminationZeroCertificate, EmptyAdmissibleSetCertificate,
    ExactIdentity, ExactIdentityKind, NoTargetEliminantCertificate, SaturatedIdealCertificate,
    SolverCertificate, TargetCertificate,
};
pub use exact_image::{
    CertifiedExactTargetImage, ExactTargetImageCertificate, RealFiberEmptyCertificate,
    RealFiberNonemptyCertificate, RealRootFiberCertificate,
};
pub use guards::{GuardCertificate, NullstellensatzCertificate, RealInfeasibilityCertificate};
pub use monomial::Monomial;
pub use options::{ExactImageMode, ResourceLimits, SolverOptions};
pub use polynomial::{PolynomialQ, Rational};
pub use problem::{GuardKind, GuardProvenance, GuardRecord, TargetProblemQ};
pub use roots::{AlgebraicRealRoot, RationalInterval};
pub use solver::{solve_target, CertifiedCandidateCover, SolverStatus, TargetSolveResult};
pub use trace::SolverTrace;
pub use univariate::{
    FactorizationFailure, FactorizationLimits, FactorizationResult, FactorizationStatus,
    FactorizationTrace, UniPolynomialQ,
};
pub use variable::Variable;
pub use verifier::{verify_certificate, VerificationResult};
