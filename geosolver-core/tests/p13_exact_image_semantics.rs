use geosolver_core::api::solve_target;
use geosolver_core::problem::input::{make_problem, RationalTargetProblem};
use geosolver_core::problem::semantic::{register_slack_encoding, RealConstraintKind};
use geosolver_core::result::status::SolverStatus;
use geosolver_core::solver::options::SolverOptions;
use geosolver_core::types::ids::{RelationId, VariableId};
use geosolver_core::types::interval::interval_contains_q;
use geosolver_core::types::polynomial::{
    constant_poly, poly_mul, poly_scale, poly_sub, variable_poly, SparsePolynomialQ,
};
use geosolver_core::types::rational::int_q;
use geosolver_core::verify::replay_run_certificate;

fn v(id: u32) -> SparsePolynomialQ {
    variable_poly(VariableId(id))
}

fn c(value: i64) -> SparsePolynomialQ {
    constant_poly(int_q(value))
}

fn exact_options() -> SolverOptions {
    SolverOptions {
        exact_image_mode: true,
        ..SolverOptions::default()
    }
}

fn support_t_times_t_minus_one(t: VariableId) -> SparsePolynomialQ {
    poly_mul(&v(t.0), &poly_sub(&v(t.0), &c(1)))
}

fn support_t_squared_minus_one(t: VariableId) -> SparsePolynomialQ {
    poly_sub(&poly_mul(&v(t.0), &v(t.0)), &c(1))
}

fn square_slack_relation(guard: SparsePolynomialQ, slack: VariableId) -> SparsePolynomialQ {
    poly_sub(&guard, &poly_mul(&v(slack.0), &v(slack.0)))
}

fn problem_with_semantic(
    target: VariableId,
    slack: VariableId,
    support: SparsePolynomialQ,
    slack_relation: SparsePolynomialQ,
    kind: RealConstraintKind,
) -> RationalTargetProblem {
    make_problem(
        vec![target, slack],
        target,
        vec![
            poly_scale(&support, &int_q(2)),
            poly_scale(&slack_relation, &int_q(3)),
        ],
        vec![register_slack_encoding(
            kind,
            vec![RelationId(1)],
            vec![slack],
        )],
    )
}

#[test]
fn p13_candidate_cover_mode_does_not_claim_exact_image_for_semantic_problem() {
    let target = VariableId(0);
    let slack = VariableId(1);
    let problem = problem_with_semantic(
        target,
        slack,
        support_t_times_t_minus_one(target),
        square_slack_relation(v(target.0), slack),
        RealConstraintKind::Positive,
    );

    let result = solve_target(problem.clone(), SolverOptions::default());

    assert_eq!(result.status, SolverStatus::CertifiedCandidateCover);
    assert!(result
        .certificate
        .as_ref()
        .is_some_and(|cert| cert.exact_image_certificate_hash.is_none()));
    assert!(result.decoded_candidates.len() >= 2);
    assert!(result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.name == "ExactImageFilteringNotRequested"));
    assert!(result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.name == "CandidateCoverMayContainSpuriousRoots"));
    assert!(replay_run_certificate(&result, &problem).accepted);
}

#[test]
fn p16_exact_image_request_returns_scope_guard_without_filtering_slack_root() {
    let target = VariableId(0);
    let slack = VariableId(1);
    let problem = problem_with_semantic(
        target,
        slack,
        support_t_times_t_minus_one(target),
        square_slack_relation(v(target.0), slack),
        RealConstraintKind::Positive,
    );

    let result = solve_target(problem.clone(), exact_options());

    assert_eq!(
        result.status,
        SolverStatus::CertificateDesignGap,
        "diagnostics={:?}",
        result.diagnostics
    );
    assert_eq!(result.decoded_candidates.len(), 2);
    assert!(result
        .decoded_candidates
        .iter()
        .any(|candidate| interval_contains_q(&candidate.isolating_interval, &int_q(0))));
    assert!(result
        .decoded_candidates
        .iter()
        .any(|candidate| interval_contains_q(&candidate.isolating_interval, &int_q(1))));
    assert!(replay_run_certificate(&result, &problem).accepted);
    assert!(result
        .certificate
        .as_ref()
        .is_some_and(|cert| cert.exact_image_certificate_hash.is_none()));
    assert!(result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.name == "ExactImageOutOfScope"));
}

#[test]
fn p16_exact_image_empty_case_keeps_unfiltered_candidate_cover() {
    let target = VariableId(0);
    let slack = VariableId(1);
    let negative_square_guard = poly_scale(&poly_mul(&v(target.0), &v(target.0)), &int_q(-1));
    let problem = problem_with_semantic(
        target,
        slack,
        support_t_squared_minus_one(target),
        square_slack_relation(negative_square_guard, slack),
        RealConstraintKind::NonNegative,
    );

    let result = solve_target(problem.clone(), exact_options());

    assert_eq!(
        result.status,
        SolverStatus::CertificateDesignGap,
        "diagnostics={:?}",
        result.diagnostics
    );
    assert_eq!(result.decoded_candidates.len(), 2);
    assert_eq!(result.root_isolation.len(), 2);
    assert!(replay_run_certificate(&result, &problem).accepted);
    assert!(result
        .certificate
        .as_ref()
        .is_some_and(|cert| cert.exact_image_certificate_hash.is_none()));
    assert!(result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.name == "ExactImageOutOfScope"));
}

#[test]
fn p16_branch_choice_semantics_do_not_filter_candidate_cover() {
    let target = VariableId(0);
    let slack = VariableId(1);
    let problem = problem_with_semantic(
        target,
        slack,
        support_t_times_t_minus_one(target),
        square_slack_relation(v(target.0), slack),
        RealConstraintKind::BranchChoice,
    );

    let result = solve_target(problem.clone(), exact_options());

    assert_eq!(
        result.status,
        SolverStatus::CertificateDesignGap,
        "diagnostics={:?}",
        result.diagnostics
    );
    assert_eq!(result.decoded_candidates.len(), 2);
    assert!(result
        .certificate
        .as_ref()
        .is_some_and(|cert| cert.exact_image_certificate_hash.is_none()));
    assert!(result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.name == "ExactImageOutOfScope"));
}

#[test]
fn p13_exact_image_nonfinite_requires_real_nonfinite_certificate() {
    let target = VariableId(0);
    let x = VariableId(1);
    let problem = make_problem(
        vec![target, x],
        target,
        vec![poly_sub(&v(x.0), &c(1))],
        Vec::new(),
    );

    let result = solve_target(problem.clone(), exact_options());

    assert_eq!(result.status, SolverStatus::CertificateDesignGap);
    assert!(result.support_polynomial.is_none());
    assert!(result.certificate.is_none());
    assert!(!replay_run_certificate(&result, &problem).accepted);
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.name == "ExactImageOutOfScope"
            && diagnostic
                .details
                .contains_key("nonfinite_certificate_hash")
    }));
}

#[test]
fn p13_exact_image_nonfinite_with_semantics_returns_gap_without_real_semantic_proof() {
    let target = VariableId(0);
    let x = VariableId(1);
    let slack = VariableId(2);
    let semantic_relation = square_slack_relation(v(x.0), slack);
    let problem = make_problem(
        vec![target, x, slack],
        target,
        vec![poly_sub(&v(x.0), &c(1)), semantic_relation],
        vec![register_slack_encoding(
            RealConstraintKind::NonNegative,
            vec![RelationId(1)],
            vec![slack],
        )],
    );

    let result = solve_target(problem.clone(), exact_options());

    assert_eq!(
        result.status,
        SolverStatus::CertificateDesignGap,
        "diagnostics={:?}",
        result.diagnostics
    );
    assert!(result.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("RealNonFiniteSemanticGuardSaturationCertificate")));
}

#[test]
fn p13_exact_image_nonfinite_with_guard_or_saturation_returns_gap_without_real_proof() {
    let target = VariableId(0);
    let x = VariableId(1);
    let a = VariableId(2);
    let slack = VariableId(3);
    let nonzero_witness = poly_sub(&poly_mul(&v(a.0), &v(slack.0)), &c(1));
    let problem = make_problem(
        vec![target, x, a, slack],
        target,
        vec![poly_sub(&v(x.0), &c(1)), nonzero_witness],
        Vec::new(),
    );

    let result = solve_target(problem.clone(), exact_options());

    assert_eq!(
        result.status,
        SolverStatus::CertificateDesignGap,
        "diagnostics={:?}",
        result.diagnostics
    );
    assert!(result.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("RealNonFiniteSemanticGuardSaturationCertificate")));
}
