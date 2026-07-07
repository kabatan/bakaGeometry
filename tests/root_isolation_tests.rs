use geosolver_core::{
    solve_target, ExactImageMode, GuardRecord, Monomial, PolynomialQ, Rational, ResourceLimits,
    SolverOptions, SolverStatus, TargetProblemQ, Variable,
};
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{Signed, Zero};

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
fn quadratic_support_returns_two_rational_root_intervals() {
    let t = variable("T");
    let variables = vec![t.clone()];
    let equations = vec![polynomial(&variables, &[(1, vec![2]), (-2, vec![0])])];
    let problem = TargetProblemQ {
        equations,
        variables,
        target: t,
        semantic_guards: Vec::<GuardRecord>::new(),
    };

    let result = solve_target(problem, options());

    assert_eq!(result.status, SolverStatus::CertifiedCandidateCover);
    let cover = result.cover.unwrap();
    assert_eq!(cover.real_roots.len(), 2);
    for root in &cover.real_roots {
        assert!(root.isolating_interval.lower < root.isolating_interval.upper);
        let lower_value = square_minus_two(&root.isolating_interval.lower);
        let upper_value = square_minus_two(&root.isolating_interval.upper);
        assert!(!lower_value.is_zero());
        assert!(!upper_value.is_zero());
        assert_ne!(lower_value.signum(), upper_value.signum());
    }
}

fn square_minus_two(value: &Rational) -> Rational {
    value.clone() * value.clone() - rational(2)
}
