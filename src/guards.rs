use crate::{ExactIdentity, GuardRecord, PolynomialQ};

#[derive(Clone, Debug)]
pub enum GuardCertificate {
    InputSemanticNonzero {
        guard: PolynomialQ,
        record: GuardRecord,
    },
    AlgebraicNonvanishing {
        guard: PolynomialQ,
        certificate: NullstellensatzCertificate,
    },
    RealAdmissibleNonvanishing {
        guard: PolynomialQ,
        certificate: RealInfeasibilityCertificate,
    },
    DerivedProduct {
        product: PolynomialQ,
        factors: Vec<GuardCertificate>,
        identity: ExactIdentity,
    },
}

#[derive(Clone, Debug)]
pub struct NullstellensatzCertificate {
    pub multipliers: Vec<PolynomialQ>,
    pub guard_multiplier: PolynomialQ,
    pub identity: ExactIdentity,
}

#[derive(Clone, Debug)]
pub enum RealInfeasibilityCertificate {
    VerifiedByExactAlgebraicCertificate(NullstellensatzCertificate),
    VerifiedByExternalReplay { replay: String },
}
