use std::{collections::BTreeMap, panic};

use geosolver_core::{
    certified_system_from_problem, solve_target, validate_target_problem, verify_certificate,
    verify_compression_replay, CompressionStepCertificate, ExactImageMode, GuardCertificate,
    GuardKind, GuardProvenance, GuardRecord, Monomial, PolynomialQ, Rational, ResourceLimits,
    SolverOptions, SolverStatus, TargetProblemQ, Variable, VerificationResult,
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

fn term(variables: &[Variable], coefficient: i64, exponents: &[u32]) -> PolynomialQ {
    PolynomialQ::from_term(
        variables.to_vec(),
        rational(coefficient),
        Monomial {
            exponents: exponents.to_vec(),
        },
    )
}

fn problem_with_nonzero_guard() -> TargetProblemQ {
    let x = variable("X");
    let t = variable("T");
    let variables = vec![x, t.clone()];
    let guard = term(&variables, 1, &[1, 0]);
    let record = GuardRecord {
        polynomial: guard,
        kind: GuardKind::NonZero,
        provenance: GuardProvenance {
            description: "input nonzero".to_string(),
        },
    };

    TargetProblemQ {
        equations: vec![
            term(&variables, 2, &[1, 0]).add(&term(&variables, -2, &[0, 1])),
            PolynomialQ::zero(variables.clone()),
        ],
        variables,
        target: t,
        semantic_guards: vec![record],
    }
}

#[test]
fn nonzero_semantic_guard_becomes_verifiable_input_guard_certificate() {
    let problem = problem_with_nonzero_guard();

    validate_target_problem(&problem).unwrap();
    let system = certified_system_from_problem(&problem).unwrap();

    assert_eq!(system.guard_certificates.len(), 1);
    assert!(matches!(
        &system.guard_certificates[0],
        GuardCertificate::InputSemanticNonzero { record, .. }
            if record.provenance.description == "input nonzero"
    ));
    assert!(system.replay.steps.iter().any(|step| matches!(
        step,
        CompressionStepCertificate::PrimitiveNormalization { .. }
    )));
    assert!(system
        .replay
        .steps
        .iter()
        .any(|step| matches!(step, CompressionStepCertificate::ZeroEquationRemoval { .. })));
    assert!(verify_compression_replay(&problem, &system).is_ok());
}

#[test]
fn compression_replay_rejects_guard_and_replay_tampering() {
    let problem = problem_with_nonzero_guard();
    let mut system = certified_system_from_problem(&problem).unwrap();

    let mut bad_guard_system = system.clone();
    let GuardCertificate::InputSemanticNonzero { record, .. } =
        &mut bad_guard_system.guard_certificates[0]
    else {
        panic!("expected input guard certificate");
    };
    record.kind = GuardKind::Positive;
    assert!(verify_compression_replay(&problem, &bad_guard_system).is_err());

    system.replay.steps.retain(|step| {
        !matches!(
            step,
            CompressionStepCertificate::PrimitiveNormalization { .. }
        )
    });
    assert!(verify_compression_replay(&problem, &system).is_err());
}

#[test]
fn invalid_problem_rejects_missing_or_duplicated_target() {
    let t = variable("T");
    let duplicated = TargetProblemQ {
        equations: Vec::new(),
        variables: vec![t.clone(), t.clone()],
        target: t,
        semantic_guards: Vec::new(),
    };

    assert!(validate_target_problem(&duplicated).is_err());
}

#[test]
fn malformed_public_polynomial_arity_is_invalid_input_not_panic() {
    let t = variable("T");
    let variables = vec![t.clone()];
    let mut terms = BTreeMap::new();
    terms.insert(
        Monomial {
            exponents: vec![1, 0],
        },
        rational(1),
    );
    let malformed = PolynomialQ {
        variables: variables.clone(),
        terms,
    };
    let input = TargetProblemQ {
        equations: vec![malformed],
        variables,
        target: t,
        semantic_guards: Vec::new(),
    };

    assert!(validate_target_problem(&input).is_err());
    let result = panic::catch_unwind(|| {
        solve_target(
            input,
            SolverOptions {
                resource_limits: ResourceLimits {
                    max_window_degree: Some(1),
                    max_proof_weight: Some(1),
                    max_matrix_rows: None,
                    max_matrix_cols: None,
                    max_candidate_count: None,
                },
                exact_image_mode: ExactImageMode::CoverOnly,
            },
        )
    });

    assert!(result.is_ok());
    assert_eq!(result.unwrap().status, SolverStatus::InvalidInput);
}

#[test]
fn solver_certificate_from_normalized_system_verifies_against_original_problem() {
    let t = variable("T");
    let x = variable("X");
    let variables = vec![t.clone(), x.clone()];
    let input = TargetProblemQ {
        equations: vec![
            term(&variables, 2, &[2, 0]).add(&term(&variables, -4, &[0, 0])),
            term(&variables, 1, &[0, 2]).add(&term(&variables, -3, &[0, 0])),
            PolynomialQ::zero(variables.clone()),
        ],
        variables,
        target: t,
        semantic_guards: Vec::new(),
    };
    let result = solve_target(
        input.clone(),
        SolverOptions {
            resource_limits: ResourceLimits {
                max_window_degree: Some(2),
                max_proof_weight: Some(2),
                max_matrix_rows: None,
                max_matrix_cols: None,
                max_candidate_count: None,
            },
            exact_image_mode: ExactImageMode::CoverOnly,
        },
    );

    assert_eq!(
        result.status,
        SolverStatus::CertifiedCandidateCover,
        "{:?}",
        result.trace.events
    );
    assert_eq!(
        verify_certificate(input, result.certificate.unwrap()),
        VerificationResult::Verified
    );
}
