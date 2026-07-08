use crate::problem::polynomial_terms_have_valid_arity;
use crate::{
    AlgebraicRealRoot, CompositeRule, EmptyAdmissibleSetCertificate, ExactIdentityKind,
    ExactTargetImageCertificate, GuardCertificate, GuardKind, NoTargetEliminantCertificate,
    NullstellensatzCertificate, PolynomialQ, RealInfeasibilityCertificate,
    RealRootFiberCertificate, SolverCertificate, TargetCertificate, TargetProblemQ, UniPolynomialQ,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VerificationResult {
    Verified,
    Rejected { reason: String },
    CertificateDesignGap { reason: String },
}

pub fn verify_certificate(problem: TargetProblemQ, cert: SolverCertificate) -> VerificationResult {
    if !problem.is_well_formed() {
        return reject("invalid problem");
    }

    match cert {
        SolverCertificate::EmptyAdmissibleSet(certificate) => {
            verify_empty_certificate(&problem, &certificate)
        }
        SolverCertificate::NoNonzeroTargetEliminant(certificate) => {
            verify_no_target_eliminant_certificate(&problem, &certificate)
        }
        SolverCertificate::TargetCover(certificate) => {
            verify_target_certificate(&problem, &certificate)
        }
        SolverCertificate::ExactTargetImage(certificate) => {
            verify_exact_target_image_certificate(&problem, &certificate)
        }
    }
}

pub(crate) fn verify_target_certificate(
    problem: &TargetProblemQ,
    cert: &TargetCertificate,
) -> VerificationResult {
    match cert {
        TargetCertificate::IdealMembership {
            support,
            multipliers,
            identity,
        } => {
            if identity.kind != ExactIdentityKind::IdealMembership {
                return reject("identity kind mismatch");
            }
            if let Some(rejection) = reject_invalid_target_support(problem, support) {
                return rejection;
            }
            let target = support.to_multivariate(&problem.variables);
            verify_linear_combination_equals(problem, multipliers, &target)
        }
        TargetCertificate::RadicalMembership {
            support,
            power,
            multipliers,
            identity,
        } => {
            if identity.kind != ExactIdentityKind::RadicalMembership {
                return reject("identity kind mismatch");
            }
            if let Some(rejection) = reject_invalid_target_support(problem, support) {
                return rejection;
            }
            if *power == 0 {
                return reject("radical power must be positive");
            }
            let target = support.pow(*power).to_multivariate(&problem.variables);
            verify_linear_combination_equals(problem, multipliers, &target)
        }
        TargetCertificate::GuardedRadicalMembership {
            support,
            support_power,
            guard_power,
            guard_product,
            guard_certificates,
            multipliers,
            identity,
        } => {
            if identity.kind != ExactIdentityKind::GuardedRadicalMembership {
                return reject("identity kind mismatch");
            }
            if let Some(rejection) = reject_invalid_target_support(problem, support) {
                return rejection;
            }
            if !polynomial_matches_problem(problem, guard_product) {
                return reject("guard product malformed");
            }
            if *support_power == 0 {
                return reject("support power must be positive");
            }
            let Some(computed_product) = product_of_verified_guards(problem, guard_certificates)
            else {
                return reject("guard factor rejected");
            };
            if &computed_product != guard_product {
                return reject("guard product mismatch");
            }
            let support_part = support
                .pow(*support_power)
                .to_multivariate(&problem.variables);
            let target = guard_product.pow(*guard_power).mul(&support_part);
            verify_linear_combination_equals(problem, multipliers, &target)
        }
        TargetCertificate::CompositeCover {
            support,
            children,
            rule,
            component_union_source,
        } => verify_composite_target_certificate(
            problem,
            support,
            children,
            *rule,
            component_union_source.as_ref(),
        ),
    }
}

fn reject_invalid_target_support(
    problem: &TargetProblemQ,
    support: &UniPolynomialQ,
) -> Option<VerificationResult> {
    if support.is_zero() {
        return Some(reject("target support is zero"));
    }
    if support.variable != problem.target {
        return Some(reject("target support variable mismatch"));
    }
    None
}

pub(crate) fn verify_guard_certificate(
    problem: &TargetProblemQ,
    cert: &GuardCertificate,
) -> VerificationResult {
    if !problem.is_well_formed() {
        return reject("invalid problem");
    }
    match verified_guard_polynomial(problem, cert) {
        Some(_) => VerificationResult::Verified,
        None => reject("guard certificate rejected"),
    }
}

pub(crate) fn verify_empty_certificate(
    problem: &TargetProblemQ,
    cert: &EmptyAdmissibleSetCertificate,
) -> VerificationResult {
    match cert {
        EmptyAdmissibleSetCertificate::AlgebraicInfeasibility {
            multipliers,
            identity,
        } => {
            if identity.kind != ExactIdentityKind::AlgebraicInfeasibility {
                return reject("identity kind mismatch");
            }
            verify_linear_combination_equals(
                problem,
                multipliers,
                &PolynomialQ::one(problem.variables.clone()),
            )
        }
        EmptyAdmissibleSetCertificate::GuardedAlgebraicInfeasibility {
            guard_product,
            guard_power,
            guard_certificates,
            multipliers,
            identity,
        } => {
            if identity.kind != ExactIdentityKind::GuardedAlgebraicInfeasibility {
                return reject("identity kind mismatch");
            }
            if !polynomial_matches_problem(problem, guard_product) {
                return reject("guard product malformed");
            }
            let Some(computed_product) = product_of_verified_guards(problem, guard_certificates)
            else {
                return reject("guard factor rejected");
            };
            if &computed_product != guard_product {
                return reject("guard product mismatch");
            }
            let target = guard_product.pow(*guard_power);
            verify_linear_combination_equals(problem, multipliers, &target)
        }
        EmptyAdmissibleSetCertificate::RealInfeasibility { .. } => {
            design_gap("real infeasibility replay requires P16 exact real replay")
        }
    }
}

pub(crate) fn verify_no_target_eliminant_certificate(
    problem: &TargetProblemQ,
    cert: &NoTargetEliminantCertificate,
) -> VerificationResult {
    for certificate in &cert.guard_certificates {
        if verify_guard_certificate(problem, certificate) != VerificationResult::Verified {
            return reject("guard certificate rejected");
        }
    }
    for certificate in &cert.saturated_ideal_description.guard_certificates {
        if verify_guard_certificate(problem, certificate) != VerificationResult::Verified {
            return reject("saturation guard certificate rejected");
        }
    }
    design_gap("no-target eliminant exact elimination-zero replay requires P15")
}

fn verify_exact_target_image_certificate(
    problem: &TargetProblemQ,
    cert: &ExactTargetImageCertificate,
) -> VerificationResult {
    if verify_target_certificate(problem, &cert.cover) != VerificationResult::Verified {
        return reject("exact image cover certificate rejected");
    }
    let support = target_certificate_support(&cert.cover);
    if cert.squarefree_support != support.squarefree_part() {
        return reject("exact image squarefree support mismatch");
    }
    let roots = crate::roots::isolate_real_roots_squarefree(&cert.squarefree_support);
    if cert.root_classifications.len() != roots.len() {
        return reject("exact image root classification count mismatch");
    }
    for root in &roots {
        let matches = cert
            .root_classifications
            .iter()
            .filter(|classification| classified_root(classification) == root)
            .count();
        if matches != 1 {
            return reject("exact image root classification mismatch");
        }
    }
    for classification in &cert.root_classifications {
        match classification {
            RealRootFiberCertificate::Nonempty { certificate, .. } => {
                if certificate.description.is_empty() {
                    return reject("nonempty real fiber replay missing");
                }
            }
            RealRootFiberCertificate::Empty { certificate, .. } => {
                if certificate.description.is_empty() {
                    return reject("empty real fiber replay missing");
                }
            }
        }
    }
    design_gap("exact real fiber replay requires P16")
}

fn classified_root(classification: &RealRootFiberCertificate) -> &AlgebraicRealRoot {
    match classification {
        RealRootFiberCertificate::Nonempty { root, .. }
        | RealRootFiberCertificate::Empty { root, .. } => root,
    }
}

fn verify_composite_target_certificate(
    problem: &TargetProblemQ,
    support: &UniPolynomialQ,
    children: &[TargetCertificate],
    rule: CompositeRule,
    component_union_source: Option<&crate::ComponentUnionSource>,
) -> VerificationResult {
    if children.is_empty() {
        return reject("composite certificate has no children");
    }
    if let Some(rejection) = reject_invalid_target_support(problem, support) {
        return rejection;
    }
    for child in children {
        if verify_target_certificate(problem, child) != VerificationResult::Verified {
            return reject("child target certificate rejected");
        }
    }

    let mut combined = target_certificate_support(&children[0]).clone();
    match rule {
        CompositeRule::SameIdealGcd => {
            for child in &children[1..] {
                combined = combined.gcd(target_certificate_support(child));
            }
        }
        CompositeRule::ComponentUnionLcm => {
            for child in &children[1..] {
                combined = combined.lcm(target_certificate_support(child));
            }
            if combined.primitive_integer_normalized() != support.primitive_integer_normalized() {
                return reject("composite support mismatch");
            }
            let _ = component_union_source;
            return design_gap(
                "component union source replay requires P5/P16 replay-verifiable data",
            );
        }
    }

    if combined.primitive_integer_normalized() == support.primitive_integer_normalized() {
        VerificationResult::Verified
    } else {
        reject("composite support mismatch")
    }
}

fn target_certificate_support(cert: &TargetCertificate) -> &UniPolynomialQ {
    match cert {
        TargetCertificate::IdealMembership { support, .. }
        | TargetCertificate::RadicalMembership { support, .. }
        | TargetCertificate::GuardedRadicalMembership { support, .. }
        | TargetCertificate::CompositeCover { support, .. } => support,
    }
}

fn verified_guard_polynomial(
    problem: &TargetProblemQ,
    cert: &GuardCertificate,
) -> Option<PolynomialQ> {
    match cert {
        GuardCertificate::InputSemanticNonzero { guard, record } => {
            if record.kind == GuardKind::NonZero
                && polynomial_matches_problem(problem, guard)
                && &record.polynomial == guard
                && problem.semantic_guards.contains(record)
            {
                Some(guard.clone())
            } else {
                None
            }
        }
        GuardCertificate::AlgebraicNonvanishing { guard, certificate } => {
            verify_nullstellensatz_nonvanishing(problem, guard, certificate).then(|| guard.clone())
        }
        GuardCertificate::RealAdmissibleNonvanishing { guard, certificate } => match certificate {
            RealInfeasibilityCertificate::VerifiedByExactAlgebraicCertificate(nullstellensatz) => {
                verify_nullstellensatz_nonvanishing(problem, guard, nullstellensatz)
                    .then(|| guard.clone())
            }
            RealInfeasibilityCertificate::VerifiedByExternalReplay { .. } => None,
        },
        GuardCertificate::DerivedProduct {
            product,
            factors,
            identity,
        } => {
            if identity.kind != ExactIdentityKind::GuardProduct {
                return None;
            }
            if !polynomial_matches_problem(problem, product) {
                return None;
            }
            let computed = product_of_verified_guards(problem, factors)?;
            (&computed == product).then(|| product.clone())
        }
    }
}

fn product_of_verified_guards(
    problem: &TargetProblemQ,
    certificates: &[GuardCertificate],
) -> Option<PolynomialQ> {
    let mut product = PolynomialQ::one(problem.variables.clone());
    for certificate in certificates {
        let guard = verified_guard_polynomial(problem, certificate)?;
        product = product.mul(&guard);
    }
    Some(product)
}

fn verify_nullstellensatz_nonvanishing(
    problem: &TargetProblemQ,
    guard: &PolynomialQ,
    certificate: &NullstellensatzCertificate,
) -> bool {
    if certificate.identity.kind != ExactIdentityKind::AlgebraicInfeasibility {
        return false;
    }
    if !polynomial_matches_problem(problem, guard)
        || !polynomial_matches_problem(problem, &certificate.guard_multiplier)
        || certificate.multipliers.len() != problem.equations.len()
    {
        return false;
    }
    let Some(mut sum) = linear_combination(problem, &certificate.multipliers) else {
        return false;
    };
    sum = sum.add(&certificate.guard_multiplier.mul(guard));
    PolynomialQ::one(problem.variables.clone())
        .sub(&sum)
        .is_zero()
}

fn verify_linear_combination_equals(
    problem: &TargetProblemQ,
    multipliers: &[PolynomialQ],
    target: &PolynomialQ,
) -> VerificationResult {
    if !polynomial_matches_problem(problem, target) {
        return reject("target variables mismatch");
    }
    let Some(sum) = linear_combination(problem, multipliers) else {
        return reject("linear combination malformed");
    };
    if target.sub(&sum).is_zero() {
        VerificationResult::Verified
    } else {
        reject("polynomial identity is nonzero")
    }
}

fn linear_combination(
    problem: &TargetProblemQ,
    multipliers: &[PolynomialQ],
) -> Option<PolynomialQ> {
    if multipliers.len() != problem.equations.len() {
        return None;
    }
    let mut sum = PolynomialQ::zero(problem.variables.clone());
    for (multiplier, equation) in multipliers.iter().zip(&problem.equations) {
        if !polynomial_matches_problem(problem, multiplier)
            || !polynomial_matches_problem(problem, equation)
        {
            return None;
        }
        sum = sum.add(&multiplier.mul(equation));
    }
    Some(sum)
}

fn polynomial_matches_problem(problem: &TargetProblemQ, polynomial: &PolynomialQ) -> bool {
    polynomial.variables == problem.variables && polynomial_terms_have_valid_arity(polynomial)
}

fn reject(reason: &str) -> VerificationResult {
    VerificationResult::Rejected {
        reason: reason.to_string(),
    }
}

fn design_gap(reason: &str) -> VerificationResult {
    VerificationResult::CertificateDesignGap {
        reason: reason.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        EmptyAdmissibleSetCertificate, ExactIdentity, ExactIdentityKind, GuardCertificate,
        GuardKind, GuardProvenance, GuardRecord, Monomial, NullstellensatzCertificate, PolynomialQ,
        SolverCertificate, Variable,
    };
    use num_bigint::BigInt;
    use num_rational::BigRational;

    fn variable(symbol: &str) -> Variable {
        Variable {
            symbol: symbol.to_string(),
        }
    }

    fn rational(value: i64) -> crate::Rational {
        BigRational::from_integer(BigInt::from(value))
    }

    fn constant(variables: &[Variable], value: i64) -> PolynomialQ {
        PolynomialQ::from_term(
            variables.to_vec(),
            rational(value),
            Monomial {
                exponents: vec![0; variables.len()],
            },
        )
    }

    fn target_power(variables: &[Variable], coefficient: i64, degree: u32) -> PolynomialQ {
        PolynomialQ::from_term(
            variables.to_vec(),
            rational(coefficient),
            Monomial {
                exponents: vec![degree],
            },
        )
    }

    fn identity(kind: ExactIdentityKind) -> ExactIdentity {
        ExactIdentity { kind }
    }

    fn one_variable_problem(
        equations: Vec<PolynomialQ>,
        guards: Vec<GuardRecord>,
    ) -> TargetProblemQ {
        let target = variable("T");
        TargetProblemQ {
            equations,
            variables: vec![target.clone()],
            target,
            semantic_guards: guards,
        }
    }

    #[test]
    fn input_semantic_nonzero_requires_identical_record() {
        let t = variable("T");
        let variables = vec![t.clone()];
        let guard = target_power(&variables, 1, 1);
        let record = GuardRecord {
            polynomial: guard.clone(),
            kind: GuardKind::NonZero,
            provenance: GuardProvenance {
                description: "input guard".to_string(),
            },
        };
        let problem = one_variable_problem(Vec::new(), vec![record.clone()]);

        assert_eq!(
            verify_guard_certificate(
                &problem,
                &GuardCertificate::InputSemanticNonzero {
                    guard: guard.clone(),
                    record: record.clone(),
                }
            ),
            VerificationResult::Verified
        );

        let altered_record = GuardRecord {
            provenance: GuardProvenance {
                description: "altered guard".to_string(),
            },
            ..record
        };
        assert!(matches!(
            verify_guard_certificate(
                &problem,
                &GuardCertificate::InputSemanticNonzero {
                    guard,
                    record: altered_record,
                }
            ),
            VerificationResult::Rejected { .. }
        ));
    }

    #[test]
    fn derived_product_recomputes_factor_product() {
        let t = variable("T");
        let variables = vec![t.clone()];
        let guard_t = target_power(&variables, 1, 1);
        let guard_t_minus_one = target_power(&variables, 1, 1).sub(&constant(&variables, 1));
        let product = guard_t.mul(&guard_t_minus_one);
        let record_t = GuardRecord {
            polynomial: guard_t.clone(),
            kind: GuardKind::NonZero,
            provenance: GuardProvenance {
                description: "first guard".to_string(),
            },
        };
        let record_t_minus_one = GuardRecord {
            polynomial: guard_t_minus_one.clone(),
            kind: GuardKind::NonZero,
            provenance: GuardProvenance {
                description: "second guard".to_string(),
            },
        };
        let problem = one_variable_problem(
            Vec::new(),
            vec![record_t.clone(), record_t_minus_one.clone()],
        );

        let derived = GuardCertificate::DerivedProduct {
            product,
            factors: vec![
                GuardCertificate::InputSemanticNonzero {
                    guard: guard_t,
                    record: record_t,
                },
                GuardCertificate::InputSemanticNonzero {
                    guard: guard_t_minus_one,
                    record: record_t_minus_one,
                },
            ],
            identity: identity(ExactIdentityKind::GuardProduct),
        };

        assert_eq!(
            verify_guard_certificate(&problem, &derived),
            VerificationResult::Verified
        );

        let bad_product = GuardCertificate::DerivedProduct {
            product: constant(&variables, 1),
            factors: match derived {
                GuardCertificate::DerivedProduct { factors, .. } => factors,
                _ => unreachable!(),
            },
            identity: identity(ExactIdentityKind::GuardProduct),
        };
        assert!(matches!(
            verify_guard_certificate(&problem, &bad_product),
            VerificationResult::Rejected { .. }
        ));
    }

    #[test]
    fn algebraic_nonvanishing_recomputes_nullstellensatz_identity() {
        let t = variable("T");
        let variables = vec![t.clone()];
        let equation = target_power(&variables, 1, 1).sub(&constant(&variables, 1));
        let guard = target_power(&variables, 1, 1);
        let problem = one_variable_problem(vec![equation], Vec::new());

        let certificate = GuardCertificate::AlgebraicNonvanishing {
            guard: guard.clone(),
            certificate: NullstellensatzCertificate {
                multipliers: vec![constant(&variables, -1)],
                guard_multiplier: constant(&variables, 1),
                identity: identity(ExactIdentityKind::AlgebraicInfeasibility),
            },
        };

        assert_eq!(
            verify_guard_certificate(&problem, &certificate),
            VerificationResult::Verified
        );

        let bad_certificate = GuardCertificate::AlgebraicNonvanishing {
            guard,
            certificate: NullstellensatzCertificate {
                multipliers: vec![constant(&variables, -1)],
                guard_multiplier: constant(&variables, 0),
                identity: identity(ExactIdentityKind::AlgebraicInfeasibility),
            },
        };
        assert!(matches!(
            verify_guard_certificate(&problem, &bad_certificate),
            VerificationResult::Rejected { .. }
        ));
    }

    #[test]
    fn empty_algebraic_certificate_verifies_and_tampering_fails() {
        let t = variable("T");
        let variables = vec![t.clone()];
        let problem = one_variable_problem(vec![constant(&variables, 1)], Vec::new());
        let good = SolverCertificate::EmptyAdmissibleSet(
            EmptyAdmissibleSetCertificate::AlgebraicInfeasibility {
                multipliers: vec![constant(&variables, 1)],
                identity: identity(ExactIdentityKind::AlgebraicInfeasibility),
            },
        );

        assert_eq!(
            verify_certificate(problem.clone(), good),
            VerificationResult::Verified
        );

        let bad = SolverCertificate::EmptyAdmissibleSet(
            EmptyAdmissibleSetCertificate::AlgebraicInfeasibility {
                multipliers: vec![constant(&variables, 0)],
                identity: identity(ExactIdentityKind::AlgebraicInfeasibility),
            },
        );
        assert!(matches!(
            verify_certificate(problem, bad),
            VerificationResult::Rejected { .. }
        ));
    }
}
