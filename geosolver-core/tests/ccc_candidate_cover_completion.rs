use geosolver_core::api::solve_target;
use geosolver_core::problem::input::{make_problem, RationalTargetProblem};
use geosolver_core::problem::semantic::{register_slack_encoding, RealConstraintKind};
use geosolver_core::result::output::TargetSolveResult;
use geosolver_core::result::status::SolverStatus;
use geosolver_core::solver::options::SolverOptions;
use geosolver_core::types::ids::{RelationId, VariableId};
use geosolver_core::types::interval::interval_contains_q;
use geosolver_core::types::polynomial::{
    constant_poly, poly_add, poly_mul, poly_scale, poly_sub, variable_poly, SparsePolynomialQ,
};
use geosolver_core::types::rational::int_q;
use geosolver_core::verify::replay_run_certificate;

fn v(id: u32) -> SparsePolynomialQ {
    variable_poly(VariableId(id))
}

fn c(value: i64) -> SparsePolynomialQ {
    constant_poly(int_q(value))
}

fn scaled_problem(
    variables: Vec<VariableId>,
    target: VariableId,
    relations: Vec<SparsePolynomialQ>,
) -> RationalTargetProblem {
    make_problem(
        variables,
        target,
        relations
            .into_iter()
            .enumerate()
            .map(|(idx, relation)| {
                let scale = int_q(match idx % 5 {
                    0 => -3,
                    1 => 5,
                    2 => -7,
                    3 => 11,
                    _ => -13,
                });
                poly_scale(&relation, &scale)
            })
            .collect(),
        Vec::new(),
    )
}

fn support_product(target: VariableId, roots: &[i64]) -> SparsePolynomialQ {
    roots.iter().fold(c(1), |acc, root| {
        poly_mul(&acc, &poly_sub(&v(target.0), &c(*root)))
    })
}

fn square_slack_relation(guard: SparsePolynomialQ, slack: VariableId) -> SparsePolynomialQ {
    poly_sub(&guard, &poly_mul(&v(slack.0), &v(slack.0)))
}

fn semantic_problem(
    target: VariableId,
    slack: VariableId,
    support: SparsePolynomialQ,
    guard: SparsePolynomialQ,
    kind: RealConstraintKind,
) -> RationalTargetProblem {
    make_problem(
        vec![target, slack],
        target,
        vec![
            poly_scale(&support, &int_q(2)),
            square_slack_relation(guard, slack),
        ],
        vec![register_slack_encoding(
            kind,
            vec![RelationId(1)],
            vec![slack],
        )],
    )
}

fn assert_candidate_cover(
    label: &str,
    problem: RationalTargetProblem,
    allow_empty_real_roots: bool,
) -> TargetSolveResult {
    let result = solve_target(problem.clone(), SolverOptions::default());
    assert_eq!(
        result.status,
        SolverStatus::CertifiedCandidateCover,
        "{label}: diagnostics={:?}",
        result.diagnostics
    );
    assert!(
        result.support_polynomial.is_some(),
        "{label}: missing support"
    );
    assert!(
        result.squarefree_support_polynomial.is_some(),
        "{label}: missing squarefree support"
    );
    assert!(
        allow_empty_real_roots || !result.decoded_candidates.is_empty(),
        "{label}: expected real candidates"
    );
    assert!(
        result.certificate.is_some(),
        "{label}: missing run certificate"
    );
    assert!(result.exact_image_certificate.is_none());
    assert!(result.nonfinite_certificate.is_none());
    assert!(result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.name == "ExactImageFilteringNotRequested"));
    assert!(result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.name == "CandidateCoverMayContainSpuriousRoots"));
    assert!(
        replay_run_certificate(&result, &problem).accepted,
        "{label}: replay rejected"
    );
    result
}

fn assert_spurious_cover_kept(label: &str, problem: RationalTargetProblem, retained_root: i64) {
    let result = assert_candidate_cover(label, problem, false);
    assert!(
        result.decoded_candidates.len() >= 2,
        "{label}: candidate-cover mode must keep all roots of S(T)"
    );
    assert!(
        result
            .decoded_candidates
            .iter()
            .any(|candidate| interval_contains_q(
                &candidate.isolating_interval,
                &int_q(retained_root)
            )),
        "{label}: expected retained spurious/root candidate {retained_root}"
    );
}

#[test]
fn ccc_p11_a12_spurious_roots_are_allowed_in_candidate_cover_mode() {
    let target = VariableId(700);
    let slack = VariableId(701);
    assert_spurious_cover_kept(
        "positive slack keeps zero root until exact-image mode",
        semantic_problem(
            target,
            slack,
            support_product(target, &[0, 1]),
            v(target.0),
            RealConstraintKind::Positive,
        ),
        0,
    );

    let target = VariableId(702);
    let slack = VariableId(703);
    assert_spurious_cover_kept(
        "positive slack keeps negative root until exact-image mode",
        semantic_problem(
            target,
            slack,
            support_product(target, &[-1, 2]),
            v(target.0),
            RealConstraintKind::Positive,
        ),
        -1,
    );

    let target = VariableId(704);
    let slack = VariableId(705);
    assert_spurious_cover_kept(
        "nonnegative shifted guard keeps invalid negative root",
        semantic_problem(
            target,
            slack,
            support_product(target, &[-2, 2]),
            poly_sub(&v(target.0), &c(1)),
            RealConstraintKind::NonNegative,
        ),
        -2,
    );

    let target = VariableId(706);
    let slack = VariableId(707);
    assert_spurious_cover_kept(
        "branch-choice slack keeps unclassified branch root",
        semantic_problem(
            target,
            slack,
            support_product(target, &[-3, 0]),
            poly_add(&v(target.0), &c(3)),
            RealConstraintKind::BranchChoice,
        ),
        -3,
    );
}

#[test]
fn ccc_p12_red_team_runs_twelve_fresh_public_inputs() {
    let t = VariableId(710);
    let x = VariableId(711);
    assert_candidate_cover(
        "fresh 01 nonlinear alias-free target chain",
        scaled_problem(
            vec![x, t],
            t,
            vec![
                poly_sub(&poly_mul(&v(x.0), &v(x.0)), &c(2)),
                poly_sub(&v(t.0), &v(x.0)),
            ],
        ),
        false,
    );

    let t = VariableId(712);
    let x = VariableId(713);
    let y = VariableId(714);
    assert_candidate_cover(
        "fresh 02 two eliminated variables",
        scaled_problem(
            vec![y, t, x],
            t,
            vec![
                poly_sub(&poly_mul(&v(x.0), &v(x.0)), &c(3)),
                poly_sub(&v(y.0), &v(x.0)),
                poly_sub(&v(t.0), &v(y.0)),
            ],
        ),
        false,
    );

    let t = VariableId(715);
    let x = VariableId(716);
    assert_candidate_cover(
        "fresh 03 higher-degree sparse eliminant",
        scaled_problem(
            vec![t, x],
            t,
            vec![
                poly_sub(&poly_mul(&v(x.0), &v(x.0)), &poly_add(&v(t.0), &c(1))),
                poly_sub(&poly_mul(&poly_mul(&v(x.0), &v(x.0)), &v(x.0)), &c(5)),
            ],
        ),
        false,
    );

    let t = VariableId(717);
    assert_candidate_cover(
        "fresh 04 nonreal support gives empty candidate cover",
        scaled_problem(
            vec![t],
            t,
            vec![poly_add(&poly_mul(&v(t.0), &v(t.0)), &c(1))],
        ),
        true,
    );

    let t = VariableId(718);
    let z = VariableId(719);
    assert_candidate_cover(
        "fresh 05 target-independent feasible component retained",
        scaled_problem(
            vec![z, t],
            t,
            vec![
                poly_sub(&poly_mul(&v(z.0), &v(z.0)), &c(4)),
                support_product(t, &[-2, 2]),
            ],
        ),
        false,
    );

    let t = VariableId(720);
    let x = VariableId(721);
    let y = VariableId(722);
    assert_candidate_cover(
        "fresh 06 mixed bilinear target relation",
        scaled_problem(
            vec![y, x, t],
            t,
            vec![
                poly_sub(&poly_mul(&v(x.0), &v(y.0)), &v(t.0)),
                poly_sub(&poly_mul(&v(x.0), &v(x.0)), &c(2)),
                poly_sub(&v(y.0), &c(3)),
            ],
        ),
        false,
    );

    let t = VariableId(723);
    let x = VariableId(724);
    let y = VariableId(725);
    assert_candidate_cover(
        "fresh 07 additive target from two radicals",
        scaled_problem(
            vec![x, t, y],
            t,
            vec![
                poly_sub(&poly_mul(&v(x.0), &v(x.0)), &c(2)),
                poly_sub(&poly_mul(&v(y.0), &v(y.0)), &c(3)),
                poly_sub(&v(t.0), &poly_add(&v(x.0), &v(y.0))),
            ],
        ),
        false,
    );

    let t = VariableId(726);
    let x = VariableId(727);
    let a = VariableId(728);
    assert_candidate_cover(
        "fresh 08 guarded affine denominator witness",
        scaled_problem(
            vec![a, x, t],
            t,
            vec![
                poly_sub(&v(x.0), &c(2)),
                poly_sub(&poly_mul(&poly_add(&v(x.0), &c(1)), &v(a.0)), &c(1)),
                poly_sub(&v(t.0), &poly_mul(&c(3), &v(a.0))),
            ],
        ),
        false,
    );

    let t = VariableId(729);
    let result = solve_target(
        scaled_problem(vec![t], t, Vec::new()),
        SolverOptions::default(),
    );
    assert_eq!(result.status, SolverStatus::CertifiedNonFiniteTargetImage);
    assert!(result.nonfinite_certificate.is_some());
    assert!(replay_run_certificate(&result, &scaled_problem(vec![t], t, Vec::new())).accepted);

    let t = VariableId(730);
    let x = VariableId(731);
    let options = SolverOptions {
        max_matrix_rows: Some(1),
        max_matrix_cols: Some(1),
        ..SolverOptions::default()
    };
    let result = solve_target(
        scaled_problem(
            vec![x, t],
            t,
            vec![poly_sub(
                &poly_mul(&poly_mul(&v(x.0), &v(x.0)), &v(x.0)),
                &v(t.0),
            )],
        ),
        options,
    );
    assert_ne!(result.status, SolverStatus::CertifiedNonFiniteTargetImage);

    let t = VariableId(732);
    let x = VariableId(733);
    assert_candidate_cover(
        "fresh 11 quartic support through helper variable",
        scaled_problem(
            vec![x, t],
            t,
            vec![
                poly_sub(&poly_mul(&v(x.0), &v(x.0)), &c(2)),
                poly_sub(&v(t.0), &poly_mul(&v(x.0), &v(x.0))),
            ],
        ),
        false,
    );

    let t = VariableId(734);
    let x = VariableId(735);
    let y = VariableId(736);
    assert_candidate_cover(
        "fresh 12 triangular cubic chain",
        scaled_problem(
            vec![t, y, x],
            t,
            vec![
                poly_sub(&poly_mul(&v(x.0), &v(x.0)), &c(2)),
                poly_sub(&v(y.0), &poly_mul(&v(x.0), &v(x.0))),
                poly_sub(&v(t.0), &poly_mul(&v(y.0), &v(x.0))),
            ],
        ),
        false,
    );
}
