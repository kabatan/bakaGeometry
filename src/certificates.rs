use crate::{
    ExactTargetImageCertificate, GuardCertificate, NullstellensatzCertificate, PolynomialQ,
    RealInfeasibilityCertificate, UniPolynomialQ,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExactIdentity {
    pub kind: ExactIdentityKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExactIdentityKind {
    IdealMembership,
    RadicalMembership,
    GuardedRadicalMembership,
    GuardProduct,
    AlgebraicInfeasibility,
    GuardedAlgebraicInfeasibility,
    CompressionReplay,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TargetCertificate {
    IdealMembership {
        support: UniPolynomialQ,
        multipliers: Vec<PolynomialQ>,
        identity: ExactIdentity,
    },
    RadicalMembership {
        support: UniPolynomialQ,
        power: usize,
        multipliers: Vec<PolynomialQ>,
        identity: ExactIdentity,
    },
    GuardedRadicalMembership {
        support: UniPolynomialQ,
        support_power: usize,
        guard_power: usize,
        guard_product: PolynomialQ,
        guard_certificates: Vec<GuardCertificate>,
        multipliers: Vec<PolynomialQ>,
        identity: ExactIdentity,
    },
    CompositeCover {
        support: UniPolynomialQ,
        children: Vec<TargetCertificate>,
        rule: CompositeRule,
        component_union_source: Option<ComponentUnionSource>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompositeRule {
    SameIdealGcd,
    ComponentUnionLcm,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentUnionSource {
    pub description: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SolverCertificate {
    TargetCover(TargetCertificate),
    ExactTargetImage(ExactTargetImageCertificate),
    EmptyAdmissibleSet(EmptyAdmissibleSetCertificate),
    NoNonzeroTargetEliminant(NoTargetEliminantCertificate),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EmptyAdmissibleSetCertificate {
    AlgebraicInfeasibility {
        multipliers: Vec<PolynomialQ>,
        identity: ExactIdentity,
    },
    GuardedAlgebraicInfeasibility {
        guard_product: PolynomialQ,
        guard_power: usize,
        guard_certificates: Vec<GuardCertificate>,
        multipliers: Vec<PolynomialQ>,
        identity: ExactIdentity,
    },
    RealInfeasibility {
        certificate: RealInfeasibilityCertificate,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoTargetEliminantCertificate {
    pub saturated_ideal_description: SaturatedIdealCertificate,
    pub elimination_certificate: EliminationZeroCertificate,
    pub guard_certificates: Vec<GuardCertificate>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaturatedIdealCertificate {
    pub guard_certificates: Vec<GuardCertificate>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EliminationZeroCertificate {
    pub identity: ExactIdentity,
}

pub(crate) fn nullstellensatz_identity_kind(
    certificate: &NullstellensatzCertificate,
) -> &ExactIdentityKind {
    &certificate.identity.kind
}
