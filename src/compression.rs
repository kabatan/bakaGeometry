use crate::{ExactIdentity, GuardCertificate, PolynomialQ, Rational, Variable};

#[derive(Clone, Debug)]
pub struct CertifiedSystemQ {
    pub equations: Vec<PolynomialQ>,
    pub variables: Vec<Variable>,
    pub target: Variable,
    pub guard_certificates: Vec<GuardCertificate>,
    pub replay: CompressionReplayCertificate,
}

#[derive(Clone, Debug)]
pub struct CompressionReplayCertificate {
    pub steps: Vec<CompressionStepCertificate>,
}

#[derive(Clone, Debug)]
pub enum CompressionStepCertificate {
    DefinitionSubstitution {
        variable: Variable,
        expression: PolynomialQ,
        identity: ExactIdentity,
    },
    AffineElimination {
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
