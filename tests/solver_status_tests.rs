use geosolver_core::{
    solve_target, ExactImageMode, GuardRecord, Monomial, PolynomialQ, Rational, ResourceLimits,
    SolverOptions, SolverStatus, TargetProblemQ, Variable,
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

fn problem() -> TargetProblemQ {
    let x = variable("X");
    let t = variable("T");
    let variables = vec![x.clone(), t.clone()];
    let equations = vec![
        polynomial(&variables, &[(1, vec![2, 0]), (-2, vec![0, 0])]),
        polynomial(&variables, &[(-1, vec![1, 0]), (1, vec![0, 1])]),
    ];
    TargetProblemQ {
        equations,
        variables,
        target: t,
        semantic_guards: Vec::<GuardRecord>::new(),
    }
}

fn options(mode: ExactImageMode) -> SolverOptions {
    SolverOptions {
        resource_limits: ResourceLimits {
            max_window_degree: Some(2),
            max_proof_weight: Some(2),
            max_matrix_rows: None,
            max_matrix_cols: None,
            max_candidate_count: None,
        },
        exact_image_mode: mode,
    }
}

#[test]
fn try_exact_image_keeps_candidate_cover_when_classifier_incomplete() {
    let result = solve_target(problem(), options(ExactImageMode::TryExactImage));

    assert_eq!(result.status, SolverStatus::CertifiedCandidateCover);
    assert!(result.cover.is_some());
    assert!(result.exact_image.is_none());
    assert!(result.certificate.is_some());
    assert!(result
        .trace
        .events
        .iter()
        .any(|event| event.starts_with("exact_image:incomplete")));
}

#[test]
fn require_exact_image_fails_closed_when_classifier_incomplete() {
    let result = solve_target(problem(), options(ExactImageMode::RequireExactImage));

    assert_eq!(result.status, SolverStatus::NoVerifiedTargetCertificate);
    assert!(result.cover.is_none());
    assert!(result.exact_image.is_none());
    assert!(result.certificate.is_none());
    assert!(result
        .trace
        .events
        .iter()
        .any(|event| event.starts_with("exact_image:incomplete")));
}
