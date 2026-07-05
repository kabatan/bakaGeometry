use geosolver_core::algebra::sign::SignDetermination;
use geosolver_core::api::solve_target;
use geosolver_core::fiber::exact_image::{FiberCandidateDisposition, SemanticGuardSource};
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
    assert!(result.exact_image_certificate.is_none());
    assert!(result.decoded_candidates.len() >= 2);
    assert!(replay_run_certificate(&result, &problem).accepted);
}

#[test]
fn p13_exact_image_filters_spurious_slack_root_with_certificates() {
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
        SolverStatus::CertifiedExactTargetImage,
        "diagnostics={:?}",
        result.diagnostics
    );
    assert_eq!(result.decoded_candidates.len(), 1);
    assert!(interval_contains_q(
        &result.decoded_candidates[0].isolating_interval,
        &int_q(1)
    ));
    assert!(replay_run_certificate(&result, &problem).accepted);

    let classification = result
        .exact_image_certificate
        .as_ref()
        .expect("P13 certificate");
    assert_eq!(classification.records.len(), 2);
    assert_eq!(classification.exact_candidates.len(), 1);
    assert_eq!(classification.rejected_candidates.len(), 1);
    assert!(classification.records.iter().any(|record| {
        record.disposition == FiberCandidateDisposition::RejectedBySemantic
            && record.semantic_decisions.iter().any(|decision| {
                decision.guard_source == SemanticGuardSource::SquareSlackEquation
                    && decision.sign_certificate.sign == SignDetermination::Zero
                    && !decision.accepted
            })
    }));
    assert!(
        classification.records.iter().any(|record| {
            record.disposition == FiberCandidateDisposition::Realizable
                && record
                    .hermite_certificate
                    .as_ref()
                    .is_some_and(|cert| cert.real_root_count == 2)
                && record
                    .semantic_decisions
                    .iter()
                    .any(|decision| decision.sign_certificate.sign == SignDetermination::Positive)
        }),
        "records={:?}",
        classification.records
    );
}

#[test]
fn p13_exact_image_distinguishes_empty_real_target_image() {
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
        SolverStatus::CertifiedEmptyRealTargetImage,
        "diagnostics={:?}",
        result.diagnostics
    );
    assert!(result.decoded_candidates.is_empty());
    assert!(result.root_isolation.is_empty());
    assert!(replay_run_certificate(&result, &problem).accepted);
    let classification = result
        .exact_image_certificate
        .as_ref()
        .expect("P13 certificate");
    assert_eq!(classification.exact_candidates.len(), 0);
    assert_eq!(classification.rejected_candidates.len(), 2);
    assert!(classification.records.iter().all(|record| {
        record.disposition == FiberCandidateDisposition::RejectedBySemantic
            && record
                .semantic_decisions
                .iter()
                .any(|decision| decision.sign_certificate.sign == SignDetermination::Negative)
    }));
}

#[test]
fn p13_branch_choice_semantics_affect_exact_classification() {
    let target = VariableId(0);
    let slack = VariableId(1);
    let problem = problem_with_semantic(
        target,
        slack,
        support_t_times_t_minus_one(target),
        square_slack_relation(v(target.0), slack),
        RealConstraintKind::BranchChoice,
    );

    let result = solve_target(problem, exact_options());

    assert_eq!(
        result.status,
        SolverStatus::CertifiedExactTargetImage,
        "diagnostics={:?}",
        result.diagnostics
    );
    let classification = result
        .exact_image_certificate
        .as_ref()
        .expect("P13 certificate");
    assert!(classification.records.iter().any(|record| {
        record
            .semantic_decisions
            .iter()
            .any(|decision| decision.kind == RealConstraintKind::BranchChoice)
    }));
    assert_eq!(classification.exact_candidates.len(), 1);
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

    let result = solve_target(problem, exact_options());

    assert_eq!(result.status, SolverStatus::CertifiedNonFiniteTargetImage);
    assert!(result.support_polynomial.is_none());
    assert!(result.certificate.is_none());
    assert!(result.exact_image_certificate.is_none());
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.name == "CertifiedNonFiniteTargetImage"
            && diagnostic
                .message
                .contains("ZeroTargetEliminationWithRealWitness")
            && diagnostic.message.contains("real_certificate_hash Some")
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

    let result = solve_target(problem, exact_options());

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

    let result = solve_target(problem, exact_options());

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
