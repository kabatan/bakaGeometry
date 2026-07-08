use std::collections::BTreeMap;

use geosolver_core::{
    verify_certificate, ComponentUnionSource, CompositeRule, ExactIdentity, ExactIdentityKind,
    ExactTargetImageCertificate, GuardCertificate, GuardKind, GuardProvenance, GuardRecord,
    Monomial, NoTargetEliminantCertificate, PolynomialQ, Rational, SaturatedIdealCertificate,
    SolverCertificate, TargetCertificate, TargetProblemQ, UniPolynomialQ, Variable,
    VerificationResult,
};
use num_bigint::BigInt;
use num_rational::BigRational;

fn variable(symbol: &str) -> Variable {
    Variable {
        symbol: symbol.to_string(),
    }
}

fn rational(value: i64) -> Rational {
    BigRational::from_integer(BigInt::from(value))
}

fn uni(variable: &Variable, coefficients: &[i64]) -> UniPolynomialQ {
    let mut polynomial = UniPolynomialQ {
        variable: variable.clone(),
        coefficients: coefficients.iter().map(|value| rational(*value)).collect(),
    };
    polynomial.normalize();
    polynomial
}

fn poly_from_uni(variables: &[Variable], coefficients: &[i64]) -> PolynomialQ {
    uni(&variables[0], coefficients).to_multivariate(variables)
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

fn malformed_constant_with_extra_exponent(variables: &[Variable], value: i64) -> PolynomialQ {
    let mut terms = BTreeMap::new();
    terms.insert(
        Monomial {
            exponents: vec![0; variables.len() + 1],
        },
        rational(value),
    );
    PolynomialQ {
        variables: variables.to_vec(),
        terms,
    }
}

fn problem(equations: Vec<PolynomialQ>, guards: Vec<GuardRecord>) -> TargetProblemQ {
    let target = variable("T");
    TargetProblemQ {
        equations,
        variables: vec![target.clone()],
        target,
        semantic_guards: guards,
    }
}

fn problem_with_variables(
    variables: Vec<Variable>,
    target: Variable,
    equations: Vec<PolynomialQ>,
) -> TargetProblemQ {
    TargetProblemQ {
        equations,
        variables,
        target,
        semantic_guards: Vec::new(),
    }
}

fn identity(kind: ExactIdentityKind) -> ExactIdentity {
    ExactIdentity { kind }
}

#[test]
fn ideal_membership_target_certificate_recomputes_identity() {
    let t = variable("T");
    let variables = vec![t.clone()];
    let equation = poly_from_uni(&variables, &[-2, 0, 1]);
    let problem = problem(vec![equation], Vec::new());

    let certificate = SolverCertificate::TargetCover(TargetCertificate::IdealMembership {
        support: uni(&t, &[-2, 0, 1]),
        multipliers: vec![constant(&variables, 1)],
        identity: identity(ExactIdentityKind::IdealMembership),
    });
    assert_eq!(
        verify_certificate(problem.clone(), certificate),
        VerificationResult::Verified
    );

    let bad = SolverCertificate::TargetCover(TargetCertificate::IdealMembership {
        support: uni(&t, &[-3, 0, 1]),
        multipliers: vec![constant(&variables, 1)],
        identity: identity(ExactIdentityKind::IdealMembership),
    });
    assert!(matches!(
        verify_certificate(problem, bad),
        VerificationResult::Rejected { .. }
    ));
}

#[test]
fn ideal_membership_rejects_multiplier_tamper() {
    let t = variable("T");
    let variables = vec![t.clone()];
    let equation = poly_from_uni(&variables, &[-2, 0, 1]);
    let problem = problem(vec![equation], Vec::new());

    let bad = SolverCertificate::TargetCover(TargetCertificate::IdealMembership {
        support: uni(&t, &[-2, 0, 1]),
        multipliers: vec![constant(&variables, 0)],
        identity: identity(ExactIdentityKind::IdealMembership),
    });

    assert!(matches!(
        verify_certificate(problem, bad),
        VerificationResult::Rejected { .. }
    ));
}

#[test]
fn target_certificate_rejects_malformed_multiplier_arity() {
    let t = variable("T");
    let variables = vec![t.clone()];
    let equation = poly_from_uni(&variables, &[-2, 0, 1]);
    let input = problem(vec![equation], Vec::new());

    let malformed = SolverCertificate::TargetCover(TargetCertificate::IdealMembership {
        support: uni(&t, &[-2, 0, 1]),
        multipliers: vec![malformed_constant_with_extra_exponent(&variables, 1)],
        identity: identity(ExactIdentityKind::IdealMembership),
    });

    assert!(matches!(
        verify_certificate(input, malformed),
        VerificationResult::Rejected { .. }
    ));
}

#[test]
fn target_certificates_reject_wrong_identity_kind() {
    let t = variable("T");
    let variables = vec![t.clone()];
    let equation = poly_from_uni(&variables, &[-2, 0, 1]);
    let input = problem(vec![equation], Vec::new());

    let wrong_kind = SolverCertificate::TargetCover(TargetCertificate::IdealMembership {
        support: uni(&t, &[-2, 0, 1]),
        multipliers: vec![constant(&variables, 1)],
        identity: identity(ExactIdentityKind::RadicalMembership),
    });

    assert!(matches!(
        verify_certificate(input, wrong_kind),
        VerificationResult::Rejected { .. }
    ));
}

#[test]
fn radical_membership_requires_positive_power_and_exact_identity() {
    let t = variable("T");
    let variables = vec![t.clone()];
    let problem = problem(vec![poly_from_uni(&variables, &[0, 0, 1])], Vec::new());

    let certificate = SolverCertificate::TargetCover(TargetCertificate::RadicalMembership {
        support: uni(&t, &[0, 1]),
        power: 2,
        multipliers: vec![constant(&variables, 1)],
        identity: identity(ExactIdentityKind::RadicalMembership),
    });
    assert_eq!(
        verify_certificate(problem.clone(), certificate),
        VerificationResult::Verified
    );

    let bad = SolverCertificate::TargetCover(TargetCertificate::RadicalMembership {
        support: uni(&t, &[0, 1]),
        power: 0,
        multipliers: vec![constant(&variables, 1)],
        identity: identity(ExactIdentityKind::RadicalMembership),
    });
    assert!(matches!(
        verify_certificate(problem, bad),
        VerificationResult::Rejected { .. }
    ));
}

#[test]
fn guarded_radical_refuses_missing_guard_certificate() {
    let t = variable("T");
    let variables = vec![t.clone()];
    let guard = poly_from_uni(&variables, &[0, 1]);
    let equation = poly_from_uni(&variables, &[0, -1, 1]);
    let record = GuardRecord {
        polynomial: guard.clone(),
        kind: GuardKind::NonZero,
        provenance: GuardProvenance {
            description: "nonzero input".to_string(),
        },
    };
    let problem = problem(vec![equation], vec![record]);

    let bad = SolverCertificate::TargetCover(TargetCertificate::GuardedRadicalMembership {
        support: uni(&t, &[-1, 1]),
        support_power: 1,
        guard_power: 1,
        guard_product: guard,
        guard_certificates: Vec::new(),
        multipliers: vec![constant(&variables, 1)],
        identity: identity(ExactIdentityKind::GuardedRadicalMembership),
    });
    assert!(matches!(
        verify_certificate(problem, bad),
        VerificationResult::Rejected { .. }
    ));
}

#[test]
fn guarded_radical_recomputes_guard_product_and_guard_certificate() {
    let t = variable("T");
    let variables = vec![t.clone()];
    let guard = poly_from_uni(&variables, &[0, 1]);
    let equation = poly_from_uni(&variables, &[0, -1, 1]);
    let record = GuardRecord {
        polynomial: guard.clone(),
        kind: GuardKind::NonZero,
        provenance: GuardProvenance {
            description: "nonzero input".to_string(),
        },
    };
    let problem = problem(vec![equation], vec![record.clone()]);
    let guard_certificate = GuardCertificate::InputSemanticNonzero {
        guard: guard.clone(),
        record: record.clone(),
    };

    let good = SolverCertificate::TargetCover(TargetCertificate::GuardedRadicalMembership {
        support: uni(&t, &[-1, 1]),
        support_power: 1,
        guard_power: 1,
        guard_product: guard.clone(),
        guard_certificates: vec![guard_certificate.clone()],
        multipliers: vec![constant(&variables, 1)],
        identity: identity(ExactIdentityKind::GuardedRadicalMembership),
    });
    assert_eq!(
        verify_certificate(problem.clone(), good),
        VerificationResult::Verified
    );

    let bad_product = SolverCertificate::TargetCover(TargetCertificate::GuardedRadicalMembership {
        support: uni(&t, &[-1, 1]),
        support_power: 1,
        guard_power: 1,
        guard_product: constant(&variables, 1),
        guard_certificates: vec![guard_certificate.clone()],
        multipliers: vec![constant(&variables, 1)],
        identity: identity(ExactIdentityKind::GuardedRadicalMembership),
    });
    assert!(matches!(
        verify_certificate(problem.clone(), bad_product),
        VerificationResult::Rejected { .. }
    ));

    let altered_record = GuardRecord {
        kind: GuardKind::Positive,
        ..record
    };
    let bad_guard = SolverCertificate::TargetCover(TargetCertificate::GuardedRadicalMembership {
        support: uni(&t, &[-1, 1]),
        support_power: 1,
        guard_power: 1,
        guard_product: guard,
        guard_certificates: vec![GuardCertificate::InputSemanticNonzero {
            guard: poly_from_uni(&variables, &[0, 1]),
            record: altered_record,
        }],
        multipliers: vec![constant(&variables, 1)],
        identity: identity(ExactIdentityKind::GuardedRadicalMembership),
    });
    assert!(matches!(
        verify_certificate(problem, bad_guard),
        VerificationResult::Rejected { .. }
    ));
}

#[test]
fn empty_certificates_reject_wrong_identity_kind() {
    let t = variable("T");
    let variables = vec![t.clone()];
    let input = problem(vec![constant(&variables, 1)], Vec::new());

    let wrong_kind = SolverCertificate::EmptyAdmissibleSet(
        geosolver_core::EmptyAdmissibleSetCertificate::AlgebraicInfeasibility {
            multipliers: vec![constant(&variables, 1)],
            identity: identity(ExactIdentityKind::IdealMembership),
        },
    );

    assert!(matches!(
        verify_certificate(input, wrong_kind),
        VerificationResult::Rejected { .. }
    ));
}

#[test]
fn same_ideal_composite_uses_gcd_not_product() {
    let t = variable("T");
    let variables = vec![t.clone()];
    let equation = poly_from_uni(&variables, &[0, -1, 1]);
    let problem = problem(vec![equation], Vec::new());
    let child_a = TargetCertificate::IdealMembership {
        support: uni(&t, &[0, -1, 1]),
        multipliers: vec![constant(&variables, 1)],
        identity: identity(ExactIdentityKind::IdealMembership),
    };
    let child_b = TargetCertificate::IdealMembership {
        support: uni(&t, &[0, -1, 0, 1]),
        multipliers: vec![poly_from_uni(&variables, &[1, 1])],
        identity: identity(ExactIdentityKind::IdealMembership),
    };

    let good = SolverCertificate::TargetCover(TargetCertificate::CompositeCover {
        support: uni(&t, &[0, -1, 1]),
        children: vec![child_a.clone(), child_b.clone()],
        rule: CompositeRule::SameIdealGcd,
        component_union_source: None,
    });
    assert_eq!(
        verify_certificate(problem.clone(), good),
        VerificationResult::Verified
    );

    let bad = SolverCertificate::TargetCover(TargetCertificate::CompositeCover {
        support: uni(&t, &[0, 0, 1, -1, -1, 1]),
        children: vec![child_a, child_b],
        rule: CompositeRule::SameIdealGcd,
        component_union_source: None,
    });
    assert!(matches!(
        verify_certificate(problem, bad),
        VerificationResult::Rejected { .. }
    ));
}

#[test]
fn composite_rule_tamper_recomputes_lcm_instead_of_trusting_rule_label() {
    let t = variable("T");
    let variables = vec![t.clone()];
    let problem = problem(
        vec![
            poly_from_uni(&variables, &[0, 1]),
            poly_from_uni(&variables, &[-1, 1]),
        ],
        Vec::new(),
    );
    let child_a = TargetCertificate::IdealMembership {
        support: uni(&t, &[0, 1]),
        multipliers: vec![constant(&variables, 1), constant(&variables, 0)],
        identity: identity(ExactIdentityKind::IdealMembership),
    };
    let child_b = TargetCertificate::IdealMembership {
        support: uni(&t, &[-1, 1]),
        multipliers: vec![constant(&variables, 0), constant(&variables, 1)],
        identity: identity(ExactIdentityKind::IdealMembership),
    };

    let tampered_rule = SolverCertificate::TargetCover(TargetCertificate::CompositeCover {
        support: uni(&t, &[1]),
        children: vec![child_a, child_b],
        rule: CompositeRule::ComponentUnionLcm,
        component_union_source: Some(ComponentUnionSource {
            description: "tampered rule".to_string(),
        }),
    });

    assert!(matches!(
        verify_certificate(problem, tampered_rule),
        VerificationResult::Rejected { .. }
    ));
}

#[test]
fn component_union_lcm_without_replay_source_is_design_gap() {
    let t = variable("T");
    let variables = vec![t.clone()];
    let problem = problem(
        vec![
            poly_from_uni(&variables, &[0, 1]),
            poly_from_uni(&variables, &[-1, 1]),
        ],
        Vec::new(),
    );
    let child_a = TargetCertificate::IdealMembership {
        support: uni(&t, &[0, 1]),
        multipliers: vec![constant(&variables, 1), constant(&variables, 0)],
        identity: identity(ExactIdentityKind::IdealMembership),
    };
    let child_b = TargetCertificate::IdealMembership {
        support: uni(&t, &[-1, 1]),
        multipliers: vec![constant(&variables, 0), constant(&variables, 1)],
        identity: identity(ExactIdentityKind::IdealMembership),
    };

    let description_only_source =
        SolverCertificate::TargetCover(TargetCertificate::CompositeCover {
            support: uni(&t, &[0, -1, 1]),
            children: vec![child_a.clone(), child_b.clone()],
            rule: CompositeRule::ComponentUnionLcm,
            component_union_source: Some(ComponentUnionSource {
                description: "explicit component union".to_string(),
            }),
        });
    assert!(matches!(
        verify_certificate(problem.clone(), description_only_source),
        VerificationResult::CertificateDesignGap { .. }
    ));

    let missing_source = SolverCertificate::TargetCover(TargetCertificate::CompositeCover {
        support: uni(&t, &[0, -1, 1]),
        children: vec![child_a, child_b],
        rule: CompositeRule::ComponentUnionLcm,
        component_union_source: None,
    });
    assert!(matches!(
        verify_certificate(problem, missing_source),
        VerificationResult::Rejected { .. }
    ));
}

#[test]
fn target_certificate_rejects_zero_support() {
    let t = variable("T");
    let problem = problem(Vec::new(), Vec::new());
    let zero_support = UniPolynomialQ::zero(t);

    let bad = SolverCertificate::TargetCover(TargetCertificate::IdealMembership {
        support: zero_support,
        multipliers: Vec::new(),
        identity: identity(ExactIdentityKind::IdealMembership),
    });

    assert!(matches!(
        verify_certificate(problem, bad),
        VerificationResult::Rejected { .. }
    ));
}

#[test]
fn guard_nullstellensatz_rejects_wrong_identity_kind() {
    let t = variable("T");
    let variables = vec![t.clone()];
    let equation = poly_from_uni(&variables, &[-1, 1]);
    let guard = poly_from_uni(&variables, &[0, 1]);
    let input = problem(vec![equation], Vec::new());

    let wrong_algebraic = GuardCertificate::AlgebraicNonvanishing {
        guard: guard.clone(),
        certificate: geosolver_core::NullstellensatzCertificate {
            multipliers: vec![constant(&variables, -1)],
            guard_multiplier: constant(&variables, 1),
            identity: identity(ExactIdentityKind::IdealMembership),
        },
    };
    let wrong_algebraic_certificate =
        SolverCertificate::TargetCover(TargetCertificate::GuardedRadicalMembership {
            support: uni(&t, &[-1, 1]),
            support_power: 1,
            guard_power: 0,
            guard_product: guard.clone(),
            guard_certificates: vec![wrong_algebraic],
            multipliers: vec![constant(&variables, 1)],
            identity: identity(ExactIdentityKind::GuardedRadicalMembership),
        });
    assert!(matches!(
        verify_certificate(input.clone(), wrong_algebraic_certificate),
        VerificationResult::Rejected { .. }
    ));

    let wrong_real = GuardCertificate::RealAdmissibleNonvanishing {
        guard: guard.clone(),
        certificate:
            geosolver_core::RealInfeasibilityCertificate::VerifiedByExactAlgebraicCertificate(
                geosolver_core::NullstellensatzCertificate {
                    multipliers: vec![constant(&variables, -1)],
                    guard_multiplier: constant(&variables, 1),
                    identity: identity(ExactIdentityKind::IdealMembership),
                },
            ),
    };
    let wrong_real_certificate =
        SolverCertificate::TargetCover(TargetCertificate::GuardedRadicalMembership {
            support: uni(&t, &[-1, 1]),
            support_power: 1,
            guard_power: 0,
            guard_product: guard,
            guard_certificates: vec![wrong_real],
            multipliers: vec![constant(&variables, 1)],
            identity: identity(ExactIdentityKind::GuardedRadicalMembership),
        });
    assert!(matches!(
        verify_certificate(input, wrong_real_certificate),
        VerificationResult::Rejected { .. }
    ));
}

#[test]
fn target_certificate_rejects_non_target_support_variable() {
    let x = variable("X");
    let t = variable("T");
    let variables = vec![x.clone(), t.clone()];
    let equation = uni(&x, &[0, 1]).to_multivariate(&variables);
    let problem = problem_with_variables(variables.clone(), t, vec![equation]);

    let bad = SolverCertificate::TargetCover(TargetCertificate::IdealMembership {
        support: uni(&x, &[0, 1]),
        multipliers: vec![constant(&variables, 1)],
        identity: identity(ExactIdentityKind::IdealMembership),
    });

    assert!(matches!(
        verify_certificate(problem, bad),
        VerificationResult::Rejected { .. }
    ));
}

#[test]
fn exact_target_image_shell_checks_cover_and_squarefree_before_design_gap() {
    let t = variable("T");
    let variables = vec![t.clone()];
    let support = uni(&t, &[1, 0, 1]);
    let problem = problem(vec![support.to_multivariate(&variables)], Vec::new());
    let cover = TargetCertificate::IdealMembership {
        support: support.clone(),
        multipliers: vec![constant(&variables, 1)],
        identity: identity(ExactIdentityKind::IdealMembership),
    };

    let certificate = SolverCertificate::ExactTargetImage(ExactTargetImageCertificate {
        cover: cover.clone(),
        squarefree_support: support.squarefree_part(),
        root_classifications: Vec::new(),
    });
    assert!(matches!(
        verify_certificate(problem.clone(), certificate),
        VerificationResult::CertificateDesignGap { .. }
    ));

    let bad_squarefree = SolverCertificate::ExactTargetImage(ExactTargetImageCertificate {
        cover,
        squarefree_support: uni(&t, &[1, 1]),
        root_classifications: Vec::new(),
    });
    assert!(matches!(
        verify_certificate(problem, bad_squarefree),
        VerificationResult::Rejected { .. }
    ));
}

#[test]
fn no_target_eliminant_is_p15_design_gap_not_monomial_acceptance() {
    let x = variable("X");
    let t = variable("T");
    let variables = vec![x.clone(), t.clone()];
    let input = TargetProblemQ {
        equations: vec![PolynomialQ::from_term(
            variables.clone(),
            rational(1),
            Monomial {
                exponents: vec![1, 0],
            },
        )],
        variables,
        target: t,
        semantic_guards: Vec::new(),
    };
    let certificate = SolverCertificate::NoNonzeroTargetEliminant(NoTargetEliminantCertificate {
        saturated_ideal_description: SaturatedIdealCertificate {
            guard_certificates: Vec::new(),
        },
        elimination_certificate: geosolver_core::EliminationZeroCertificate {
            identity: identity(ExactIdentityKind::CompressionReplay),
        },
        guard_certificates: Vec::new(),
    });

    assert!(matches!(
        verify_certificate(input, certificate),
        VerificationResult::CertificateDesignGap { .. }
    ));
}
