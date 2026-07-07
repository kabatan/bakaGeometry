use geosolver_core::{
    solve_target, verify_certificate, ExactImageMode, GuardRecord, Monomial, PolynomialQ, Rational,
    ResourceLimits, SolverOptions, SolverStatus, TargetProblemQ, Variable, VerificationResult,
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

fn monomial(exponents: &[u32]) -> Monomial {
    Monomial {
        exponents: exponents.to_vec(),
    }
}

fn term(variables: &[Variable], coefficient: i64, exponents: &[u32]) -> PolynomialQ {
    PolynomialQ::from_term(
        variables.to_vec(),
        rational(coefficient),
        monomial(exponents),
    )
}

fn polynomial(variables: &[Variable], terms: &[(i64, Vec<u32>)]) -> PolynomialQ {
    terms.iter().fold(
        PolynomialQ::zero(variables.to_vec()),
        |accumulator, entry| accumulator.add(&term(variables, entry.0, &entry.1)),
    )
}

fn problem(
    equations: Vec<PolynomialQ>,
    variables: Vec<Variable>,
    target: Variable,
) -> TargetProblemQ {
    TargetProblemQ {
        equations,
        variables,
        target,
        semantic_guards: Vec::<GuardRecord>::new(),
    }
}

fn options() -> SolverOptions {
    SolverOptions {
        resource_limits: ResourceLimits {
            max_window_degree: Some(2),
            max_proof_weight: None,
            max_matrix_rows: None,
            max_matrix_cols: None,
            max_candidate_count: None,
        },
        exact_image_mode: ExactImageMode::CoverOnly,
    }
}

#[test]
fn direct_candidate_route_returns_verified_cover_before_complete_fallback() {
    let t = variable("T");
    let x = variable("X");
    let variables = vec![t.clone(), x.clone()];
    let equations = vec![
        polynomial(&variables, &[(1, vec![2, 0]), (-2, vec![0, 0])]),
        polynomial(&variables, &[(1, vec![0, 2]), (-3, vec![0, 0])]),
    ];

    let input = problem(equations, variables, t);
    let result = solve_target(input.clone(), options());

    assert_eq!(result.status, SolverStatus::CertifiedCandidateCover);
    assert!(result.cover.is_some());
    assert!(result.certificate.is_some());
    assert_eq!(
        verify_certificate(input, result.certificate.clone().unwrap()),
        VerificationResult::Verified
    );
    assert!(result
        .trace
        .events
        .iter()
        .any(|event| event.starts_with("candidate:")));
    assert!(result
        .trace
        .events
        .iter()
        .any(|event| event.starts_with("proof_try:")));
    assert_no_complete_fallback(&result.trace.events);
}

#[test]
fn certified_candidate_cover_does_not_use_complete_fallback() {
    let x = variable("X");
    let t = variable("T");
    let variables = vec![x.clone(), t.clone()];
    let equations = vec![
        polynomial(&variables, &[(1, vec![2, 0]), (-2, vec![0, 0])]),
        polynomial(&variables, &[(-1, vec![1, 0]), (1, vec![0, 1])]),
    ];

    let result = solve_target(problem(equations, variables, t), options());

    assert_eq!(result.status, SolverStatus::CertifiedCandidateCover);
    assert!(result.cover.is_some());
    assert!(result.certificate.is_some());
    assert!(result
        .trace
        .events
        .iter()
        .any(|event| event.starts_with("candidate:")));
    assert!(result
        .trace
        .events
        .iter()
        .any(|event| event.starts_with("proof_try:")));
    assert_no_complete_fallback(&result.trace.events);
}

#[test]
fn ranking_changes_attempt_order_before_certified_success() {
    let t = variable("T");
    let x = variable("X");
    let variables = vec![t.clone(), x.clone()];
    let equations = vec![
        polynomial(&variables, &[(1, vec![3, 0]), (-1, vec![1, 0])]),
        polynomial(&variables, &[(-1, vec![0, 0]), (1, vec![1, 0])]),
        polynomial(&variables, &[(1, vec![0, 2]), (-3, vec![0, 0])]),
    ];

    let result = solve_target(problem(equations, variables, t), options());
    let candidate_events = result
        .trace
        .events
        .iter()
        .filter(|event| event.starts_with("candidate:"))
        .collect::<Vec<_>>();

    assert_eq!(result.status, SolverStatus::CertifiedCandidateCover);
    assert!(result.cover.is_some());
    assert!(candidate_events
        .first()
        .is_some_and(|event| event.contains("degree=1")));
    assert_no_complete_fallback(&result.trace.events);
}

#[test]
fn tower_family_returns_verified_cover_before_complete_fallback() {
    let y = variable("Y");
    let x = variable("X");
    let t = variable("T");
    let variables = vec![y.clone(), x.clone(), t.clone()];
    let equations = vec![
        polynomial(&variables, &[(1, vec![2, 0, 0]), (-2, vec![0, 0, 0])]),
        polynomial(&variables, &[(1, vec![0, 2, 0]), (-1, vec![1, 0, 0])]),
        polynomial(&variables, &[(1, vec![0, 0, 1]), (-1, vec![0, 1, 0])]),
    ];

    let result = solve_target(problem(equations, variables, t), options());

    assert_eq!(result.status, SolverStatus::CertifiedCandidateCover);
    assert!(result.cover.is_some());
    assert!(result
        .trace
        .events
        .iter()
        .any(|event| event.starts_with("candidate:")));
    assert!(result
        .trace
        .events
        .iter()
        .any(|event| event.starts_with("proof_try:")));
    assert_no_complete_fallback(&result.trace.events);
}

#[test]
fn krylov_family_returns_verified_cover_before_complete_fallback() {
    let x = variable("X");
    let t = variable("T");
    let variables = vec![x.clone(), t.clone()];
    let equations = vec![
        polynomial(&variables, &[(1, vec![2, 0]), (-2, vec![0, 0])]),
        polynomial(&variables, &[(-1, vec![1, 0]), (1, vec![0, 1])]),
    ];

    let result = solve_target(problem(equations, variables, t), options());

    assert_eq!(result.status, SolverStatus::CertifiedCandidateCover);
    assert!(result.cover.is_some());
    assert!(result
        .trace
        .events
        .iter()
        .any(|event| event.starts_with("candidate:")));
    assert!(result
        .trace
        .events
        .iter()
        .any(|event| event.starts_with("proof_try:")));
    assert_no_complete_fallback(&result.trace.events);
}

#[test]
fn resultant_family_returns_verified_cover_before_complete_fallback() {
    let x = variable("X");
    let y = variable("Y");
    let t = variable("T");
    let variables = vec![x.clone(), y.clone(), t.clone()];
    let equations = vec![
        polynomial(&variables, &[(1, vec![1, 0, 0]), (-1, vec![0, 1, 0])]),
        polynomial(&variables, &[(1, vec![0, 1, 0]), (-1, vec![0, 0, 1])]),
        polynomial(&variables, &[(1, vec![1, 0, 0]), (-2, vec![0, 0, 0])]),
    ];

    let result = solve_target(problem(equations, variables, t), options());

    assert_eq!(result.status, SolverStatus::CertifiedCandidateCover);
    assert!(result.cover.is_some());
    assert!(result
        .trace
        .events
        .iter()
        .any(|event| event.starts_with("candidate:")));
    assert!(result
        .trace
        .events
        .iter()
        .any(|event| event.starts_with("proof_try:")));
    assert_no_complete_fallback(&result.trace.events);
}

#[test]
fn slice_candidate_route_does_not_adopt_without_fixed_proof() {
    let x = variable("X");
    let t = variable("T");
    let variables = vec![x.clone(), t.clone()];
    let equations = vec![polynomial(
        &variables,
        &[(1, vec![0, 2]), (1, vec![1, 0]), (-2, vec![0, 0])],
    )];

    let result = solve_target(problem(equations, variables, t), options());

    assert_eq!(result.status, SolverStatus::NoVerifiedTargetCertificate);
    assert!(result.cover.is_none());
    assert!(result
        .trace
        .events
        .iter()
        .any(|event| event.contains("candidate:SliceSpecialization")));
    assert!(result
        .trace
        .events
        .iter()
        .any(|event| event.contains("proof_try:SliceSpecialization")));
}

fn assert_no_complete_fallback(events: &[String]) {
    assert!(!events
        .iter()
        .any(|event| event.starts_with("target_elimination:")));
}
