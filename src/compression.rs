use crate::problem::polynomial_terms_have_valid_arity;
use crate::{
    ExactIdentity, GuardCertificate, GuardKind, PolynomialQ, Rational, TargetProblemQ, Variable,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CertifiedSystemQ {
    pub equations: Vec<PolynomialQ>,
    pub variables: Vec<Variable>,
    pub target: Variable,
    pub guard_certificates: Vec<GuardCertificate>,
    pub replay: CompressionReplayCertificate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompressionReplayCertificate {
    pub steps: Vec<CompressionStepCertificate>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompressionStepCertificate {
    IdentityInput,
    DefinitionSubstitution {
        variable: Variable,
        expression: PolynomialQ,
        identity: ExactIdentity,
    },
    AffineElimination {
        eliminated: Variable,
        pivot: PolynomialQ,
        pivot_guard: GuardCertificate,
        identity: ExactIdentity,
    },
    ExplicitGuardSaturation {
        guard: GuardCertificate,
        identity: ExactIdentity,
    },
    PrimitiveNormalization {
        before: PolynomialQ,
        after: PolynomialQ,
        multiplier: Rational,
    },
    ZeroEquationRemoval {
        removed: PolynomialQ,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProblemValidationError {
    TargetMissingOrDuplicated,
    DuplicateVariable,
    EquationVariableMismatch { equation_index: usize },
    EquationMonomialArityMismatch { equation_index: usize },
    GuardVariableMismatch { guard_index: usize },
    GuardMonomialArityMismatch { guard_index: usize },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompressionVerificationError {
    InvalidProblem(ProblemValidationError),
    CertifiedFieldMismatch,
    ReplayMismatch,
    GuardCertificateRejected,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CompressionEquationLift {
    pub source_index: usize,
    pub multiplier: Rational,
}

pub fn validate_target_problem(problem: &TargetProblemQ) -> Result<(), ProblemValidationError> {
    if problem
        .variables
        .iter()
        .filter(|variable| *variable == &problem.target)
        .count()
        != 1
    {
        return Err(ProblemValidationError::TargetMissingOrDuplicated);
    }
    let mut seen = std::collections::BTreeSet::new();
    for variable in &problem.variables {
        if !seen.insert(variable) {
            return Err(ProblemValidationError::DuplicateVariable);
        }
    }
    for (equation_index, equation) in problem.equations.iter().enumerate() {
        if equation.variables != problem.variables {
            return Err(ProblemValidationError::EquationVariableMismatch { equation_index });
        }
        if !polynomial_terms_have_valid_arity(equation) {
            return Err(ProblemValidationError::EquationMonomialArityMismatch { equation_index });
        }
    }
    for (guard_index, record) in problem.semantic_guards.iter().enumerate() {
        if record.polynomial.variables != problem.variables {
            return Err(ProblemValidationError::GuardVariableMismatch { guard_index });
        }
        if !polynomial_terms_have_valid_arity(&record.polynomial) {
            return Err(ProblemValidationError::GuardMonomialArityMismatch { guard_index });
        }
    }
    Ok(())
}

pub fn certified_system_from_problem(
    problem: &TargetProblemQ,
) -> Result<CertifiedSystemQ, ProblemValidationError> {
    validate_target_problem(problem)?;
    let mut equations = Vec::new();
    let mut steps = Vec::new();

    for equation in &problem.equations {
        let Some((canonical, normalized, multiplier)) = normalize_input_equation(equation) else {
            steps.push(CompressionStepCertificate::ZeroEquationRemoval {
                removed: equation.clone(),
            });
            continue;
        };
        if normalized != canonical {
            steps.push(CompressionStepCertificate::PrimitiveNormalization {
                before: canonical,
                after: normalized.clone(),
                multiplier,
            });
        }
        equations.push(normalized);
    }

    let guard_certificates = problem
        .semantic_guards
        .iter()
        .filter(|record| record.kind == GuardKind::NonZero)
        .map(|record| GuardCertificate::InputSemanticNonzero {
            guard: record.polynomial.clone(),
            record: record.clone(),
        })
        .collect();

    Ok(CertifiedSystemQ {
        equations,
        variables: problem.variables.clone(),
        target: problem.target.clone(),
        guard_certificates,
        replay: CompressionReplayCertificate { steps },
    })
}

pub(crate) fn lift_multipliers_to_original_problem(
    problem: &TargetProblemQ,
    compressed_multipliers: &[PolynomialQ],
) -> Option<Vec<PolynomialQ>> {
    let lifts = compression_equation_lifts(problem).ok()?;
    if compressed_multipliers.len() != lifts.len() {
        return None;
    }
    let mut original_multipliers = problem
        .equations
        .iter()
        .map(|equation| PolynomialQ::zero(equation.variables.clone()))
        .collect::<Vec<_>>();
    for (multiplier, lift) in compressed_multipliers.iter().zip(lifts) {
        if multiplier.variables != problem.variables {
            return None;
        }
        let lifted = multiplier.scale(&lift.multiplier);
        original_multipliers[lift.source_index] =
            original_multipliers[lift.source_index].add(&lifted);
    }
    Some(original_multipliers)
}

pub(crate) fn compression_equation_lifts(
    problem: &TargetProblemQ,
) -> Result<Vec<CompressionEquationLift>, ProblemValidationError> {
    validate_target_problem(problem)?;
    let mut lifts = Vec::new();
    for (source_index, equation) in problem.equations.iter().enumerate() {
        if let Some((_, _, multiplier)) = normalize_input_equation(equation) {
            lifts.push(CompressionEquationLift {
                source_index,
                multiplier,
            });
        }
    }
    Ok(lifts)
}

fn normalize_input_equation(
    equation: &PolynomialQ,
) -> Option<(PolynomialQ, PolynomialQ, Rational)> {
    let mut canonical = equation.clone();
    canonical.normalize();
    if canonical.is_zero() {
        return None;
    }
    let (normalized, multiplier) = canonical.primitive_integer_normalized_with_multiplier();
    Some((canonical, normalized, multiplier))
}

pub fn verify_compression_replay(
    problem: &TargetProblemQ,
    certified: &CertifiedSystemQ,
) -> Result<(), CompressionVerificationError> {
    validate_target_problem(problem).map_err(CompressionVerificationError::InvalidProblem)?;
    let replayed = certified_system_from_problem(problem)
        .map_err(CompressionVerificationError::InvalidProblem)?;
    if certified.variables != replayed.variables || certified.target != replayed.target {
        return Err(CompressionVerificationError::CertifiedFieldMismatch);
    }
    if certified.equations != replayed.equations || certified.replay != replayed.replay {
        return Err(CompressionVerificationError::ReplayMismatch);
    }
    if certified.guard_certificates != replayed.guard_certificates {
        return Err(CompressionVerificationError::GuardCertificateRejected);
    }
    for certificate in &certified.guard_certificates {
        if crate::verifier::verify_guard_certificate(problem, certificate)
            != crate::VerificationResult::Verified
        {
            return Err(CompressionVerificationError::GuardCertificateRejected);
        }
    }
    Ok(())
}
