use geosolver_core::api::solve_target;
use geosolver_core::compose::compose::compose_projection_messages;
use geosolver_core::kernels::traits::KernelKind;
use geosolver_core::planner::relation_schedule::{
    build_dense_relation_search_schedule, DenseRelationSearchSchedule,
};
use geosolver_core::problem::context::new_context;
use geosolver_core::problem::input::{make_problem, RationalTargetProblem};
use geosolver_core::problem::semantic::{register_slack_encoding, RealConstraintKind};
use geosolver_core::result::output::TargetSolveResult;
use geosolver_core::result::status::SolverStatus;
use geosolver_core::roots::decode::hash_target_candidate;
use geosolver_core::solver::options::SolverOptions;
use geosolver_core::solver::pipeline::{
    step_build_dag, step_build_graphs, step_canonicalize, step_compress, step_validate,
};
use geosolver_core::types::hash::{hash_sequence, Hash};
use geosolver_core::types::ids::{RelationId, VariableId};
use geosolver_core::types::interval::interval_contains_q;
use geosolver_core::types::polynomial::{
    constant_poly, poly_add, poly_mul, poly_scale, poly_sub, variable_poly, SparsePolynomialQ,
};
use geosolver_core::types::rational::{div_q, int_q, RationalQ};
use geosolver_core::verify::replay_run_certificate;

fn v(id: u32) -> SparsePolynomialQ {
    variable_poly(VariableId(id))
}

fn c(value: i64) -> SparsePolynomialQ {
    constant_poly(int_q(value))
}

fn q(num: i64, den: i64) -> RationalQ {
    div_q(&int_q(num), &int_q(den)).expect("nonzero rational scale")
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
                let factor = match idx % 7 {
                    0 => q(-3, 5),
                    1 => q(8, 3),
                    2 => q(-7, 6),
                    3 => q(11, 4),
                    4 => q(-13, 9),
                    5 => q(17, 10),
                    _ => q(-19, 8),
                };
                let scaled = poly_scale(&relation, &factor);
                assert!(relation.terms.is_empty() || scaled != relation);
                scaled
            })
            .collect(),
        Vec::new(),
    )
}

fn options_prioritizing(kind: KernelKind) -> SolverOptions {
    SolverOptions {
        kernel_priority: vec![kind],
        ..SolverOptions::default()
    }
}

fn exact_options() -> SolverOptions {
    SolverOptions {
        exact_image_mode: true,
        ..SolverOptions::default()
    }
}

fn assert_support_producing_success(
    label: &str,
    problem: &RationalTargetProblem,
    result: &TargetSolveResult,
    required_kernel: Option<KernelKind>,
    allow_no_real_roots: bool,
) {
    assert_eq!(
        result.status,
        SolverStatus::CertifiedCandidateCover,
        "{label}: diagnostics={:?}",
        result.diagnostics
    );
    let support = result
        .support_polynomial
        .as_ref()
        .unwrap_or_else(|| panic!("{label}: missing support"));
    assert!(
        support.coeffs_low_to_high.len() > 1,
        "{label}: support must be nonconstant"
    );
    assert!(result.squarefree_support_polynomial.is_some());
    assert!(!result.projection_messages.is_empty());
    assert!(result.certificate.is_some());
    assert!(result.cost_trace.final_support_degree.is_some());
    assert!(result.cost_trace.certificate_size.is_some());
    assert!(
        allow_no_real_roots || !result.root_isolation.is_empty(),
        "{label}: support-producing case unexpectedly had no real roots"
    );
    assert_eq!(result.root_isolation.len(), result.decoded_candidates.len());
    let squarefree_hash = result.squarefree_support_polynomial.as_ref().unwrap().hash;
    for (root, candidate) in result.root_isolation.iter().zip(&result.decoded_candidates) {
        assert_eq!(candidate.target, problem.target);
        assert_eq!(candidate.support_hash, squarefree_hash);
        assert_eq!(candidate.root_index, root.root_index);
        assert_eq!(candidate.isolating_interval, root.isolating_interval);
        assert_eq!(
            candidate.candidate_hash,
            hash_target_candidate(
                problem.target,
                squarefree_hash,
                root.root_index,
                &root.isolating_interval
            )
        );
    }
    if let Some(kind) = required_kernel {
        assert!(
            result
                .projection_messages
                .iter()
                .any(|message| message.kernel_kind == kind),
            "{label}: expected {kind:?}, got {:?}",
            result
                .projection_messages
                .iter()
                .map(|message| message.kernel_kind)
                .collect::<Vec<_>>()
        );
    }
    let replay = replay_run_certificate(result, problem);
    assert!(replay.accepted, "{label}: replay failed: {replay:?}");
}

fn run_support_case(
    label: &str,
    problem: RationalTargetProblem,
    options: SolverOptions,
    required_kernel: Option<KernelKind>,
    allow_no_real_roots: bool,
) -> TargetSolveResult {
    let result = solve_target(problem.clone(), options);
    assert_support_producing_success(
        label,
        &problem,
        &result,
        required_kernel,
        allow_no_real_roots,
    );
    result
}

fn multiseparator_problem() -> RationalTargetProblem {
    let t = VariableId(347);
    let u = VariableId(349);
    let w = VariableId(353);
    let x = VariableId(359);
    let y = VariableId(367);
    let x2 = poly_mul(&v(x.0), &v(x.0));
    let u2 = poly_mul(&v(u.0), &v(u.0));
    let w2 = poly_mul(&v(w.0), &v(w.0));
    let y2 = poly_mul(&v(y.0), &v(y.0));
    let y3 = poly_mul(&y2, &v(y.0));
    scaled_problem(
        vec![w, x, t, y, u],
        t,
        vec![
            poly_sub(&y3, &c(2)),
            poly_sub(&u2, &v(t.0)),
            poly_sub(&y2, &x2),
            poly_sub(&w2, &u2),
            poly_sub(&x2, &w2),
        ],
    )
}

#[test]
fn p15_support_producing_candidate_cover_suite() {
    let t = VariableId(307);
    let x = VariableId(311);
    let y = VariableId(313);
    run_support_case(
        "renamed one-block no initial target-only relation",
        scaled_problem(
            vec![y, t, x],
            t,
            vec![
                poly_sub(&poly_mul(&v(x.0), &v(x.0)), &c(3)),
                poly_sub(&v(t.0), &v(y.0)),
                poly_sub(&v(y.0), &v(x.0)),
            ],
        ),
        SolverOptions::default(),
        None,
        false,
    );

    let t = VariableId(317);
    let x = VariableId(331);
    let y = VariableId(337);
    let xy = poly_mul(&v(x.0), &v(y.0));
    let mut action_options = options_prioritizing(KernelKind::TargetActionKrylov);
    action_options.max_relation_search_export_degree = Some(0);
    run_support_case(
        "multivariate quotient action with nonlinear target",
        scaled_problem(
            vec![y, t, x],
            t,
            vec![
                poly_sub(&poly_sub(&v(t.0), &xy), &v(x.0)),
                poly_sub(&poly_mul(&v(y.0), &v(y.0)), &c(7)),
                poly_sub(&poly_mul(&v(x.0), &v(x.0)), &c(5)),
            ],
        ),
        action_options,
        Some(KernelKind::TargetActionKrylov),
        false,
    );

    let multi_sep_problem = multiseparator_problem();
    let multi_sep = run_support_case(
        "multiple eliminated variables and separators",
        multi_sep_problem,
        SolverOptions::default(),
        None,
        false,
    );
    assert!(
        multi_sep.projection_messages.len() >= 2,
        "multi-separator support case must exercise composition"
    );

    let t = VariableId(373);
    let x = VariableId(379);
    let x2 = poly_mul(&v(x.0), &v(x.0));
    let x3 = poly_mul(&x2, &v(x.0));
    run_support_case(
        "sparse resultant eliminant",
        scaled_problem(
            vec![t, x],
            t,
            vec![
                poly_sub(&x3, &c(7)),
                poly_sub(&x2, &poly_add(&v(t.0), &c(1))),
            ],
        ),
        options_prioritizing(KernelKind::SparseResultantProjection),
        Some(KernelKind::SparseResultantProjection),
        false,
    );

    let t = VariableId(383);
    let u = VariableId(389);
    let w = VariableId(397);
    let x = VariableId(401);
    let x2 = poly_mul(&v(x.0), &v(x.0));
    let u2 = poly_mul(&v(u.0), &v(u.0));
    run_support_case(
        "specialization interpolation multiseparator",
        scaled_problem(
            vec![w, t, x, u],
            t,
            vec![
                poly_sub(&poly_mul(&v(w.0), &v(w.0)), &c(2)),
                poly_sub(&x2, &c(1)),
                poly_sub(&x2, &poly_add(&v(t.0), &u2)),
                poly_sub(&poly_mul(&v(u.0), &v(w.0)), &c(1)),
            ],
        ),
        options_prioritizing(KernelKind::SpecializationInterpolation),
        Some(KernelKind::SpecializationInterpolation),
        false,
    );

    let t = VariableId(409);
    let x = VariableId(419);
    let y = VariableId(421);
    let s = VariableId(431);
    let denominator = poly_add(&v(x.0), &c(2));
    let witness = poly_sub(&poly_mul(&denominator, &v(s.0)), &c(1));
    let affine = poly_sub(
        &poly_mul(&denominator, &v(y.0)),
        &poly_add(&v(t.0), &poly_mul(&c(2), &v(x.0))),
    );
    run_support_case(
        "guarded rational affine preprocessing",
        scaled_problem(
            vec![s, y, t, x],
            t,
            vec![
                poly_sub(&v(x.0), &c(1)),
                witness,
                poly_sub(&poly_mul(&v(y.0), &v(y.0)), &c(3)),
                affine,
            ],
        ),
        SolverOptions::default(),
        None,
        false,
    );

    let t = VariableId(2003);
    let s = VariableId(2011);
    let semantic_positive = run_support_case(
        "semantic guard and slack provenance remains candidate cover",
        semantic_problem(
            t,
            s,
            poly_mul(&v(t.0), &poly_sub(&v(t.0), &c(1))),
            v(t.0),
            RealConstraintKind::Positive,
        ),
        SolverOptions::default(),
        None,
        false,
    );
    assert!(semantic_positive.exact_image_certificate.is_none());
    assert!(semantic_positive.decoded_candidates.len() >= 2);

    let t = VariableId(2017);
    let s = VariableId(2027);
    let semantic_branch = run_support_case(
        "semantic branch and slack provenance remains candidate cover",
        semantic_problem(
            t,
            s,
            poly_sub(&poly_mul(&v(t.0), &v(t.0)), &c(1)),
            v(t.0),
            RealConstraintKind::BranchChoice,
        ),
        SolverOptions::default(),
        None,
        false,
    );
    assert!(semantic_branch.exact_image_certificate.is_none());

    let t = VariableId(2039);
    let a = VariableId(2053);
    let b = VariableId(2063);
    let c_var = VariableId(2069);
    let d = VariableId(2081);
    let oriented_area = poly_sub(&poly_mul(&v(a.0), &v(d.0)), &poly_mul(&v(b.0), &v(c_var.0)));
    run_support_case(
        "determinant oriented bilinear support",
        scaled_problem(
            vec![d, b, t, c_var, a],
            t,
            vec![
                poly_sub(&v(a.0), &c(1)),
                poly_sub(&v(b.0), &c(2)),
                poly_sub(&v(c_var.0), &c(3)),
                poly_sub(&v(d.0), &c(5)),
                poly_sub(&v(t.0), &oriented_area),
            ],
        ),
        SolverOptions::default(),
        None,
        false,
    );

    let t = VariableId(2087);
    let x = VariableId(2089);
    let y = VariableId(2099);
    let u = VariableId(2111);
    let w = VariableId(2113);
    let gram_dot = poly_add(&poly_mul(&v(x.0), &v(u.0)), &poly_mul(&v(y.0), &v(w.0)));
    run_support_case(
        "dot gram bilinear support",
        scaled_problem(
            vec![w, x, t, u, y],
            t,
            vec![
                poly_sub(&v(x.0), &c(2)),
                poly_sub(&v(y.0), &c(-3)),
                poly_sub(&v(u.0), &c(5)),
                poly_sub(&v(w.0), &c(7)),
                poly_sub(&v(t.0), &gram_dot),
            ],
        ),
        SolverOptions::default(),
        None,
        false,
    );

    let t = VariableId(433);
    let z = VariableId(439);
    run_support_case(
        "target independent feasibility obligation",
        scaled_problem(
            vec![z, t],
            t,
            vec![
                poly_sub(&poly_mul(&v(z.0), &v(z.0)), &c(9)),
                poly_sub(&poly_mul(&v(t.0), &v(t.0)), &c(13)),
            ],
        ),
        SolverOptions::default(),
        None,
        false,
    );

    let t = VariableId(443);
    let x = VariableId(449);
    let y = VariableId(457);
    let mut universal_options = options_prioritizing(KernelKind::UniversalTargetElimination);
    universal_options.max_relation_search_export_degree = Some(0);
    run_support_case(
        "one large block universal projection",
        scaled_problem(
            vec![y, x, t],
            t,
            vec![
                poly_sub(&poly_sub(&v(t.0), &v(x.0)), &v(y.0)),
                poly_sub(&poly_add(&poly_mul(&v(x.0), &v(x.0)), &v(y.0)), &c(2)),
                poly_sub(&poly_sub(&poly_mul(&v(y.0), &v(y.0)), &v(x.0)), &c(1)),
            ],
        ),
        universal_options,
        Some(KernelKind::UniversalTargetElimination),
        false,
    );

    let t = VariableId(461);
    let y = VariableId(463);
    run_support_case(
        "regular-chain projection",
        scaled_problem(
            vec![t, y],
            t,
            vec![
                poly_sub(&poly_add(&v(t.0), &c(2)), &v(y.0)),
                poly_sub(&poly_mul(&v(y.0), &v(y.0)), &c(5)),
            ],
        ),
        options_prioritizing(KernelKind::RegularChainProjection),
        Some(KernelKind::RegularChainProjection),
        false,
    );

    let t = VariableId(467);
    let a = VariableId(479);
    let b = VariableId(487);
    run_support_case(
        "norm-trace two-step tower",
        scaled_problem(
            vec![b, t, a],
            t,
            vec![
                poly_sub(&poly_mul(&v(b.0), &v(b.0)), &v(a.0)),
                poly_sub(&poly_mul(&v(t.0), &v(b.0)), &c(2)),
                poly_sub(&poly_mul(&v(a.0), &v(a.0)), &c(3)),
            ],
        ),
        options_prioritizing(KernelKind::NormTraceProjection),
        Some(KernelKind::NormTraceProjection),
        false,
    );

    let t = VariableId(491);
    run_support_case(
        "non-real support remains candidate cover",
        scaled_problem(
            vec![t],
            t,
            vec![poly_add(&poly_mul(&v(t.0), &v(t.0)), &c(4))],
        ),
        SolverOptions::default(),
        None,
        true,
    );
}

#[test]
fn p15_renamed_permuted_structure_keeps_mechanism() {
    let t = VariableId(701);
    let x = VariableId(709);
    let y = VariableId(719);
    let xy = poly_mul(&v(x.0), &v(y.0));
    let mut options = options_prioritizing(KernelKind::TargetActionKrylov);
    options.max_relation_search_export_degree = Some(0);
    let base = run_support_case(
        "base nonlinear action structure",
        scaled_problem(
            vec![y, t, x],
            t,
            vec![
                poly_sub(&poly_mul(&v(x.0), &v(x.0)), &c(2)),
                poly_sub(&poly_mul(&v(y.0), &v(y.0)), &c(3)),
                poly_sub(&poly_sub(&v(t.0), &xy), &v(x.0)),
            ],
        ),
        options.clone(),
        Some(KernelKind::TargetActionKrylov),
        false,
    );

    let target = VariableId(809);
    let a = VariableId(811);
    let b = VariableId(821);
    let ab = poly_mul(&v(a.0), &v(b.0));
    let renamed = run_support_case(
        "renamed and relation-permuted nonlinear action structure",
        scaled_problem(
            vec![b, target, a],
            target,
            vec![
                poly_sub(&poly_sub(&v(target.0), &ab), &v(a.0)),
                poly_sub(&poly_mul(&v(b.0), &v(b.0)), &c(3)),
                poly_sub(&poly_mul(&v(a.0), &v(a.0)), &c(2)),
            ],
        ),
        options,
        Some(KernelKind::TargetActionKrylov),
        false,
    );

    assert_eq!(
        base.cost_trace.final_support_degree,
        renamed.cost_trace.final_support_degree
    );
    assert_eq!(
        base.projection_messages
            .iter()
            .map(|message| message.kernel_kind)
            .collect::<Vec<_>>(),
        renamed
            .projection_messages
            .iter()
            .map(|message| message.kernel_kind)
            .collect::<Vec<_>>()
    );
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
            poly_scale(&support, &int_q(5)),
            poly_scale(
                &poly_sub(&guard, &poly_mul(&v(slack.0), &v(slack.0))),
                &int_q(-7),
            ),
        ],
        vec![register_slack_encoding(
            kind,
            vec![RelationId(1)],
            vec![slack],
        )],
    )
}

#[test]
fn p15_exact_image_semantics_suite() {
    let t = VariableId(503);
    let s = VariableId(509);
    let result = solve_target(
        semantic_problem(
            t,
            s,
            poly_mul(&v(t.0), &poly_sub(&v(t.0), &c(1))),
            v(t.0),
            RealConstraintKind::Positive,
        ),
        exact_options(),
    );
    assert_eq!(result.status, SolverStatus::CertifiedExactTargetImage);
    assert_eq!(result.decoded_candidates.len(), 1);
    assert!(interval_contains_q(
        &result.decoded_candidates[0].isolating_interval,
        &int_q(1)
    ));
    assert!(result.exact_image_certificate.is_some());

    let t = VariableId(521);
    let s = VariableId(523);
    let result = solve_target(
        semantic_problem(
            t,
            s,
            poly_sub(&poly_mul(&v(t.0), &v(t.0)), &c(1)),
            poly_scale(&poly_mul(&v(t.0), &v(t.0)), &int_q(-1)),
            RealConstraintKind::NonNegative,
        ),
        exact_options(),
    );
    assert_eq!(result.status, SolverStatus::CertifiedEmptyRealTargetImage);
    assert!(result.support_polynomial.is_some());
    assert!(result.decoded_candidates.is_empty());
    assert!(result.exact_image_certificate.is_some());

    let t = VariableId(541);
    let s = VariableId(547);
    let result = solve_target(
        semantic_problem(
            t,
            s,
            poly_mul(&v(t.0), &poly_sub(&v(t.0), &c(1))),
            v(t.0),
            RealConstraintKind::BranchChoice,
        ),
        exact_options(),
    );
    assert_eq!(result.status, SolverStatus::CertifiedExactTargetImage);
    assert_eq!(result.decoded_candidates.len(), 1);
    assert!(result.exact_image_certificate.is_some());
}

#[test]
fn p15_failure_and_nonfinite_semantics_suite() {
    let invalid_target = VariableId(557);
    let x = VariableId(563);
    let result = solve_target(
        make_problem(vec![x], invalid_target, vec![v(x.0)], Vec::new()),
        SolverOptions::default(),
    );
    assert_eq!(result.status, SolverStatus::InvalidInput);
    assert!(result.support_polynomial.is_none());

    let t = VariableId(569);
    let x = VariableId(571);
    let mut bounded = options_prioritizing(KernelKind::TargetRelationSearch);
    bounded.max_relation_search_export_degree = Some(2);
    bounded.max_matrix_rows = Some(1);
    bounded.max_matrix_cols = Some(1);
    let result = solve_target(
        scaled_problem(
            vec![x, t],
            t,
            vec![poly_sub(&poly_mul(&v(x.0), &v(x.0)), &v(t.0))],
        ),
        bounded,
    );
    assert!(matches!(
        result.status,
        SolverStatus::AlgorithmicHardCase
            | SolverStatus::FiniteResourceFailure
            | SolverStatus::CertificateDesignGap
    ));
    assert_ne!(result.status, SolverStatus::CertifiedNonFiniteTargetImage);
    assert!(result.support_polynomial.is_none());
    assert!(!result.cost_trace.block_traces.is_empty());

    let t = VariableId(577);
    let x = VariableId(587);
    let result = solve_target(
        scaled_problem(vec![x, t], t, vec![poly_sub(&v(x.0), &c(3))]),
        SolverOptions::default(),
    );
    assert_eq!(result.status, SolverStatus::CertifiedNonFiniteTargetImage);
    assert!(result.support_polynomial.is_none());
    assert!(result.certificate.is_none());
    assert!(result.nonfinite_certificate.is_some());
    assert!(
        replay_run_certificate(
            &result,
            &scaled_problem(vec![x, t], t, vec![poly_sub(&v(x.0), &c(3))])
        )
        .accepted
    );

    let t = VariableId(593);
    let x = VariableId(599);
    let s = VariableId(601);
    let semantic_relation = poly_sub(&v(x.0), &poly_mul(&v(s.0), &v(s.0)));
    let result = solve_target(
        make_problem(
            vec![t, x, s],
            t,
            vec![poly_sub(&v(x.0), &c(1)), semantic_relation],
            vec![register_slack_encoding(
                RealConstraintKind::NonNegative,
                vec![RelationId(1)],
                vec![s],
            )],
        ),
        exact_options(),
    );
    assert_eq!(result.status, SolverStatus::CertificateDesignGap);
    assert_ne!(result.status, SolverStatus::CertifiedNonFiniteTargetImage);
}

fn assert_dense_schedule_replay_evidence(
    label: &str,
    first: &DenseRelationSearchSchedule,
    second: &DenseRelationSearchSchedule,
    expected_eliminated: Vec<VariableId>,
    expected_exported: Vec<VariableId>,
    expected_d_max: usize,
) {
    assert_eq!(
        first.schedule_hash, second.schedule_hash,
        "{label}: schedule hash changed under input permutation"
    );
    assert_eq!(first.eliminated_variables, expected_eliminated);
    assert_eq!(first.exported_variables, expected_exported);
    assert_eq!(first.d_max, expected_d_max);
    assert!(first.z_seed >= 1, "{label}: z seed must be positive");
    assert!(
        first.e_cap >= first.z_seed,
        "{label}: export cap must admit at least one stage"
    );
    assert!(!first.stages.is_empty(), "{label}: missing stages");
    assert_eq!(first.stages.len(), second.stages.len());

    for (idx, (left, right)) in first.stages.iter().zip(second.stages.iter()).enumerate() {
        assert_eq!(
            left.export_degree,
            first.z_seed + idx,
            "{label}: stage export degree order drifted"
        );
        assert_eq!(
            left.multiplier_total_degree,
            left.export_degree + first.d_max,
            "{label}: multiplier bound no longer follows export degree plus d_max"
        );
        assert_eq!(
            left.export_support_hash, right.export_support_hash,
            "{label}: export support hash changed at stage {idx}"
        );
        assert_eq!(
            left.multiplier_support_hashes, right.multiplier_support_hashes,
            "{label}: multiplier support hashes changed at stage {idx}"
        );
        assert_eq!(
            left.row_monomial_hash, right.row_monomial_hash,
            "{label}: row monomial hash changed at stage {idx}"
        );
        assert_eq!(
            (left.matrix_rows, left.matrix_cols),
            (right.matrix_rows, right.matrix_cols),
            "{label}: matrix dimensions changed at stage {idx}"
        );
        assert_ne!(left.export_support_hash, Hash([0; 32]));
        assert_ne!(left.row_monomial_hash, Hash([0; 32]));
        assert_eq!(left.row_monomial_count, left.matrix_rows);
        assert!(left.matrix_rows > 0, "{label}: empty row support");
        assert!(left.matrix_cols > 0, "{label}: empty matrix columns");
        assert!(!left.multiplier_support_hashes.is_empty());
        assert_eq!(
            left.stage_hash,
            geosolver_core::planner::relation_schedule::hash_relation_search_stage(left),
            "{label}: stage hash is not replayable"
        );
    }
}

#[test]
fn p15_relation_search_schedule_reproducibility_suite() {
    let x = VariableId(607);
    let y = VariableId(613);
    let t = VariableId(617);
    let relations = vec![
        poly_sub(&poly_mul(&v(x.0), &v(x.0)), &v(t.0)),
        poly_sub(&poly_add(&v(y.0), &v(x.0)), &c(1)),
    ];
    let exported_a = vec![t, y];
    let eliminated_a = vec![x];
    let exported_b = vec![y, t];
    let eliminated_b = vec![x];
    let options = SolverOptions {
        max_relation_search_export_degree: Some(5),
        ..SolverOptions::default()
    };

    let first =
        build_dense_relation_search_schedule(&relations, &eliminated_a, &exported_a, &options);
    let second =
        build_dense_relation_search_schedule(&relations, &eliminated_b, &exported_b, &options);
    assert_dense_schedule_replay_evidence(
        "one-eliminated two-exported quadratic",
        &first,
        &second,
        vec![x],
        vec![y, t],
        2,
    );

    let a = VariableId(619);
    let b = VariableId(631);
    let t = VariableId(641);
    let b2 = poly_mul(&v(b.0), &v(b.0));
    let b3 = poly_mul(&b2, &v(b.0));
    let relations = vec![
        poly_sub(&poly_mul(&v(a.0), &v(a.0)), &v(b.0)),
        poly_sub(&b3, &v(t.0)),
        poly_sub(&poly_mul(&v(t.0), &v(t.0)), &v(a.0)),
    ];
    let first = build_dense_relation_search_schedule(&relations, &[b, a], &[t], &options);
    let second = build_dense_relation_search_schedule(&relations, &[a, b], &[t], &options);
    assert_dense_schedule_replay_evidence(
        "two-eliminated one-exported cubic",
        &first,
        &second,
        vec![a, b],
        vec![t],
        3,
    );

    let x = VariableId(643);
    let y = VariableId(647);
    let z = VariableId(653);
    let u = VariableId(659);
    let v_var = VariableId(661);
    let t = VariableId(673);
    let x2 = poly_mul(&v(x.0), &v(x.0));
    let x4 = poly_mul(&x2, &x2);
    let relations = vec![
        poly_sub(&x4, &v(u.0)),
        poly_sub(&poly_mul(&v(y.0), &v(y.0)), &v(v_var.0)),
        poly_sub(&poly_mul(&v(z.0), &v(z.0)), &v(t.0)),
        poly_sub(&poly_mul(&v(x.0), &v(y.0)), &v(z.0)),
    ];
    let first =
        build_dense_relation_search_schedule(&relations, &[z, x, y], &[t, u, v_var], &options);
    let second =
        build_dense_relation_search_schedule(&relations, &[y, z, x], &[v_var, t, u], &options);
    assert_dense_schedule_replay_evidence(
        "three-eliminated three-exported quartic",
        &first,
        &second,
        vec![x, y, z],
        vec![u, v_var, t],
        4,
    );
}

fn compose_for_problem(
    problem: &RationalTargetProblem,
    messages: Vec<geosolver_core::compose::message::ProjectionMessage>,
) -> Option<geosolver_core::compose::compose::ComposedProjection> {
    let mut ctx = new_context(SolverOptions::default());
    let validated = step_validate(problem.clone(), &mut ctx).ok()?;
    let canonical = step_canonicalize(validated, &mut ctx).ok()?;
    let compressed = step_compress(canonical, &mut ctx).ok()?;
    let graphs = step_build_graphs(&compressed, &mut ctx).ok()?;
    let dag = step_build_dag(&graphs, &compressed, &mut ctx).ok()?;
    compose_projection_messages(&dag, messages, problem.target, &mut ctx).ok()
}

#[test]
fn p15_anti_decoration_replay_rejects_tamper_and_deletion() {
    let t = VariableId(631);
    let x = VariableId(641);
    let problem = scaled_problem(
        vec![x, t],
        t,
        vec![
            poly_sub(&poly_mul(&v(x.0), &v(x.0)), &c(2)),
            poly_sub(&v(t.0), &v(x.0)),
        ],
    );
    let result = solve_target(problem.clone(), SolverOptions::default());
    assert_support_producing_success("anti-decoration baseline", &problem, &result, None, false);

    let mut dag_tampered = result.clone();
    let cert = dag_tampered.certificate.as_mut().expect("certificate");
    cert.target_projection_dag_hash = hash_sequence("p15-dag-tamper", &[]);
    assert!(!replay_run_certificate(&dag_tampered, &problem).accepted);

    let mut plan_tampered = result.clone();
    let cert = plan_tampered.certificate.as_mut().expect("certificate");
    cert.kernel_plan_hashes[0] = hash_sequence("p15-plan-tamper", &[]);
    assert!(!replay_run_certificate(&plan_tampered, &problem).accepted);

    let mut authorization_tampered = result.clone();
    let cert = authorization_tampered
        .certificate
        .as_mut()
        .expect("certificate");
    let evidence = cert
        .final_dag_replay_evidence
        .as_mut()
        .expect("final DAG replay evidence");
    evidence.block_authorization_hashes[0] = hash_sequence("p15-authorization-tamper", &[]);
    assert!(!replay_run_certificate(&authorization_tampered, &problem).accepted);

    let mut message_tampered = result.clone();
    message_tampered.projection_messages[0].package_hash = Hash([3; 32]);
    assert!(!replay_run_certificate(&message_tampered, &problem).accepted);

    let mut message_deleted = result.clone();
    message_deleted.projection_messages.remove(0);
    assert!(!replay_run_certificate(&message_deleted, &problem).accepted);

    let mut certificate_tampered = result;
    certificate_tampered.projection_messages[0]
        .certificate
        .binding_hash = hash_sequence("p15-certificate-binding-tamper", &[]);
    assert!(!replay_run_certificate(&certificate_tampered, &problem).accepted);

    let multi_problem = multiseparator_problem();
    let multi_result = solve_target(multi_problem.clone(), SolverOptions::default());
    assert_support_producing_success(
        "anti-decoration multiseparator baseline",
        &multi_problem,
        &multi_result,
        None,
        false,
    );
    assert!(multi_result.projection_messages.len() >= 2);
    let full = compose_for_problem(&multi_problem, multi_result.projection_messages.clone())
        .expect("baseline message composition");
    let mut reduced_messages = multi_result.projection_messages.clone();
    reduced_messages.remove(0);
    let reduced = compose_for_problem(&multi_problem, reduced_messages);
    assert!(
        reduced
            .as_ref()
            .is_none_or(|projection| projection.root_relations != full.root_relations),
        "removing a child projection message must fail or change composed support"
    );
}
