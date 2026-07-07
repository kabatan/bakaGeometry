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
            max_proof_weight: Some(2),
            max_matrix_rows: None,
            max_matrix_cols: None,
            max_candidate_count: None,
        },
        exact_image_mode: ExactImageMode::CoverOnly,
    }
}

#[test]
fn solver_returns_empty_certificate_for_constant_infeasibility() {
    let t = variable("T");
    let variables = vec![t.clone()];
    let equations = vec![PolynomialQ::one(variables.clone())];
    let input = problem(equations, variables, t);

    let result = solve_target(input.clone(), options());

    assert_eq!(result.status, SolverStatus::CertifiedEmptyAdmissibleSet);
    assert!(result.cover.is_none());
    assert!(result.exact_image.is_none());
    assert!(result
        .trace
        .events
        .iter()
        .any(|event| event == "early_empty:certified"));
    assert_eq!(
        verify_certificate(input, result.certificate.unwrap()),
        VerificationResult::Verified
    );
}

#[test]
fn solver_no_target_eliminant_is_algebraic_only() {
    let x = variable("X");
    let t = variable("T");
    let variables = vec![x.clone(), t.clone()];
    let equations = vec![polynomial(&variables, &[(1, vec![1, 0])])];
    let input = problem(equations, variables, t);

    let result = solve_target(input.clone(), options());

    assert_eq!(
        result.status,
        SolverStatus::CertifiedNoNonzeroTargetEliminant
    );
    assert!(result.cover.is_none());
    assert!(result.exact_image.is_none());
    assert!(result
        .trace
        .events
        .iter()
        .any(|event| event == "target_elimination:no_target_eliminant"));
    assert_eq!(
        verify_certificate(input, result.certificate.unwrap()),
        VerificationResult::Verified
    );
}
