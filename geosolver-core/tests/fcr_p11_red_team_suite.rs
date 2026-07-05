use std::collections::BTreeSet;

use geosolver_core::api::solve_target;
use geosolver_core::compose::compose::compose_projection_messages;
use geosolver_core::graph::hypergraph::build_relation_variable_hypergraph;
use geosolver_core::graph::influence::build_target_influence_graph;
use geosolver_core::graph::projection_dag::build_target_projection_dag;
use geosolver_core::graph::separators::CostModel;
use geosolver_core::graph::tree_decomposition::build_target_rooted_decomposition;
use geosolver_core::graph::weighted_primal::build_weighted_primal_graph;
use geosolver_core::kernels::traits::KernelKind;
use geosolver_core::preprocess::compression::pre_kernel_compress;
use geosolver_core::problem::canonicalize::canonicalize_system;
use geosolver_core::problem::context::new_context;
use geosolver_core::problem::input::{make_problem, RationalTargetProblem};
use geosolver_core::problem::validate::validate_input;
use geosolver_core::result::output::TargetSolveResult;
use geosolver_core::result::status::SolverStatus;
use geosolver_core::roots::decode::hash_target_candidate;
use geosolver_core::solver::options::SolverOptions;
use geosolver_core::types::hash::Hash;
use geosolver_core::types::ids::VariableId;
use geosolver_core::types::polynomial::{
    constant_poly, poly_add, poly_mul, poly_scale, poly_sub, poly_variables, variable_poly,
    SparsePolynomialQ,
};
use geosolver_core::types::rational::{div_q, int_q, RationalQ};
use geosolver_core::verify::replay_run_certificate;

fn public_candidate_cover(problem: RationalTargetProblem) -> TargetSolveResult {
    let result = solve_target(problem.clone(), SolverOptions::default());
    assert_candidate_cover(&result, &problem);
    result
}

fn public_candidate_cover_with_options(
    problem: RationalTargetProblem,
    options: SolverOptions,
) -> TargetSolveResult {
    let result = solve_target(problem.clone(), options);
    assert_candidate_cover(&result, &problem);
    result
}

fn assert_candidate_cover(result: &TargetSolveResult, problem: &RationalTargetProblem) {
    assert_eq!(
        result.status,
        SolverStatus::CertifiedCandidateCover,
        "unexpected status diagnostics={:?}",
        result.diagnostics
    );
    let support = result.support_polynomial.as_ref().expect("support");
    assert!(support.coeffs_low_to_high.len() > 1);
    assert!(result.squarefree_support_polynomial.is_some());
    assert!(!result.projection_messages.is_empty());
    assert!(result.certificate.is_some());
    assert_eq!(result.root_isolation.len(), result.decoded_candidates.len());
    let squarefree_hash = result.squarefree_support_polynomial.as_ref().unwrap().hash;
    for (root, candidate) in result.root_isolation.iter().zip(&result.decoded_candidates) {
        assert_eq!(root.support_hash, squarefree_hash);
        assert_eq!(candidate.target, problem.target);
        assert_eq!(candidate.support_hash, squarefree_hash);
        assert_eq!(candidate.root_index, root.root_index);
        assert_eq!(candidate.isolating_interval, root.isolating_interval);
        assert_ne!(candidate.candidate_hash, Hash([0; 32]));
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
    let replay = replay_run_certificate(result, problem);
    assert!(replay.accepted, "replay failed: {replay:?}");
}

fn assert_executed_kernel(result: &TargetSolveResult, kind: KernelKind) {
    assert!(
        result
            .projection_messages
            .iter()
            .any(|message| message.kernel_kind == kind),
        "expected {kind:?}; executed kernels={:?}",
        result
            .projection_messages
            .iter()
            .map(|message| message.kernel_kind)
            .collect::<Vec<_>>()
    );
}

fn assert_public_composition_uses_multiple_messages(
    problem: &RationalTargetProblem,
    result: &TargetSolveResult,
) {
    assert!(
        result.projection_messages.len() >= 2,
        "red-team composition case requires multiple projection messages: {:?}",
        result.projection_messages
    );
    let target_only = BTreeSet::from([problem.target]);
    assert!(
        result.projection_messages.iter().any(|message| message
            .relation_generators
            .iter()
            .any(|relation| !poly_variables(relation).is_subset(&target_only))),
        "composition must include separator-bearing message relations"
    );
    let canonical = canonicalize_system(validate_input(problem.clone()).unwrap()).unwrap();
    let mut ctx = new_context(SolverOptions::default());
    let compressed = pre_kernel_compress(canonical, &mut ctx).unwrap();
    let hypergraph = build_relation_variable_hypergraph(&compressed);
    let influence = build_target_influence_graph(&hypergraph, problem.target);
    let weighted = build_weighted_primal_graph(&compressed, &influence);
    let decomposition =
        build_target_rooted_decomposition(&weighted, problem.target, &CostModel::default());
    let dag = build_target_projection_dag(&compressed, &influence, &decomposition).unwrap();
    let full = compose_projection_messages(
        &dag,
        result.projection_messages.clone(),
        problem.target,
        &mut new_context(SolverOptions::default()),
    )
    .expect("fresh red-team message composition must replay");
    assert!(
        !full.separator_elimination_messages.is_empty(),
        "composition must perform message-only separator elimination"
    );
}

fn options_prioritizing(kind: KernelKind) -> SolverOptions {
    SolverOptions {
        kernel_priority: vec![kind],
        ..SolverOptions::default()
    }
}

fn problem(
    variables: Vec<VariableId>,
    target: VariableId,
    relations: Vec<SparsePolynomialQ>,
) -> RationalTargetProblem {
    make_problem(variables, target, scale_relations(relations), Vec::new())
}

fn v(id: u32) -> SparsePolynomialQ {
    variable_poly(VariableId(id))
}

fn c(n: i64) -> SparsePolynomialQ {
    constant_poly(int_q(n))
}

fn q(num: i64, den: i64) -> RationalQ {
    div_q(&int_q(num), &int_q(den)).expect("nonzero scale denominator")
}

fn scale_relations(relations: Vec<SparsePolynomialQ>) -> Vec<SparsePolynomialQ> {
    relations
        .into_iter()
        .enumerate()
        .map(|(index, relation)| {
            let factor = match index % 7 {
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
        .collect()
}

#[test]
fn fcr_p11_red_team_01_multivariate_action_not_alias_univariate() {
    let t = VariableId(101);
    let x = VariableId(103);
    let y = VariableId(107);
    let xy = poly_mul(&v(x.0), &v(y.0));
    let mut options = options_prioritizing(KernelKind::TargetActionKrylov);
    options.max_relation_search_export_degree = Some(0);
    let result = public_candidate_cover_with_options(
        problem(
            vec![y, t, x],
            t,
            vec![
                poly_sub(&poly_mul(&v(y.0), &v(y.0)), &c(5)),
                poly_sub(&poly_sub(&v(t.0), &xy), &v(y.0)),
                poly_sub(&poly_mul(&v(x.0), &v(x.0)), &c(3)),
            ],
        ),
        options,
    );
    assert!(!result.root_isolation.is_empty());
    assert_executed_kernel(&result, KernelKind::TargetActionKrylov);
}

#[test]
fn fcr_p11_red_team_02_two_separator_public_composition() {
    let t = VariableId(109);
    let u = VariableId(113);
    let w = VariableId(127);
    let x = VariableId(131);
    let y = VariableId(137);
    let u2 = poly_mul(&v(u.0), &v(u.0));
    let w2 = poly_mul(&v(w.0), &v(w.0));
    let x2 = poly_mul(&v(x.0), &v(x.0));
    let y2 = poly_mul(&v(y.0), &v(y.0));
    let y3 = poly_mul(&y2, &v(y.0));
    let input = problem(
        vec![w, y, t, x, u],
        t,
        vec![
            poly_sub(&w2, &u2),
            poly_sub(&y3, &c(3)),
            poly_sub(&u2, &v(t.0)),
            poly_sub(&y2, &x2),
            poly_sub(&x2, &w2),
        ],
    );
    let result = public_candidate_cover(input.clone());
    assert_public_composition_uses_multiple_messages(&input, &result);
}

#[test]
fn fcr_p11_red_team_03_sparse_resultant_higher_degree_eliminant() {
    let t = VariableId(139);
    let x = VariableId(149);
    let x2 = poly_mul(&v(x.0), &v(x.0));
    let x3 = poly_mul(&x2, &v(x.0));
    let result = public_candidate_cover_with_options(
        problem(
            vec![t, x],
            t,
            vec![
                poly_sub(&x2, &poly_add(&v(t.0), &c(1))),
                poly_sub(&x3, &c(5)),
            ],
        ),
        options_prioritizing(KernelKind::SparseResultantProjection),
    );
    assert!(!result.root_isolation.is_empty());
    assert_executed_kernel(&result, KernelKind::SparseResultantProjection);
}

#[test]
fn fcr_p11_red_team_04_guarded_rational_affine_nonconstant_denominator() {
    let t = VariableId(151);
    let x = VariableId(157);
    let y = VariableId(163);
    let s = VariableId(167);
    let denominator = poly_add(&v(x.0), &c(2));
    let witness = poly_sub(&poly_mul(&denominator, &v(s.0)), &c(1));
    let affine = poly_sub(
        &poly_mul(&denominator, &v(y.0)),
        &poly_add(&v(t.0), &poly_mul(&c(3), &v(x.0))),
    );
    let result = public_candidate_cover(problem(
        vec![s, t, y, x],
        t,
        vec![
            poly_sub(&v(x.0), &c(2)),
            poly_sub(&poly_mul(&v(y.0), &v(y.0)), &c(5)),
            affine,
            witness,
        ],
    ));
    assert!(!result.root_isolation.is_empty());
}

#[test]
fn fcr_p11_red_team_05_one_large_block_universal_path() {
    let t = VariableId(173);
    let x = VariableId(179);
    let y = VariableId(181);
    let mut options = options_prioritizing(KernelKind::UniversalTargetElimination);
    options.max_relation_search_export_degree = Some(0);
    let result = public_candidate_cover_with_options(
        problem(
            vec![y, x, t],
            t,
            vec![
                poly_sub(&poly_add(&poly_mul(&v(x.0), &v(x.0)), &v(y.0)), &c(3)),
                poly_sub(&poly_sub(&poly_mul(&v(y.0), &v(y.0)), &v(x.0)), &c(1)),
                poly_sub(&poly_sub(&v(t.0), &v(x.0)), &v(y.0)),
            ],
        ),
        options,
    );
    assert!(!result.root_isolation.is_empty());
    assert_executed_kernel(&result, KernelKind::UniversalTargetElimination);
}

#[test]
fn fcr_p11_red_team_06_target_independent_feasibility_component() {
    let t = VariableId(193);
    let z = VariableId(197);
    let result = public_candidate_cover(problem(
        vec![z, t],
        t,
        vec![
            poly_sub(&poly_mul(&v(z.0), &v(z.0)), &c(4)),
            poly_sub(&poly_mul(&v(t.0), &v(t.0)), &c(11)),
        ],
    ));
    assert_eq!(result.root_isolation.len(), 2);
}

#[test]
fn fcr_p11_red_team_07_positive_nonfinite_kept_out_of_candidate_cover_claim() {
    let t = VariableId(199);
    let x = VariableId(211);
    let result = solve_target(
        problem(vec![x, t], t, vec![poly_sub(&v(x.0), &c(2))]),
        SolverOptions::default(),
    );
    assert_eq!(result.status, SolverStatus::CertifiedNonFiniteTargetImage);
    assert!(result.support_polynomial.is_none());
    assert!(
        result.certificate.is_none(),
        "public nonfinite is not replay-bound yet and must stay out of candidate-cover readiness"
    );
}

#[test]
fn fcr_p11_red_team_08_target_free_without_positive_witness_is_not_nonfinite() {
    let t = VariableId(223);
    let x = VariableId(227);
    let y = VariableId(229);
    let mut options = options_prioritizing(KernelKind::TargetRelationSearch);
    options.max_relation_search_export_degree = Some(2);
    options.max_matrix_rows = Some(1);
    options.max_matrix_cols = Some(1);
    let result = solve_target(
        problem(
            vec![y, t, x],
            t,
            vec![
                poly_sub(&poly_mul(&v(x.0), &v(y.0)), &v(t.0)),
                poly_sub(
                    &poly_add(&poly_mul(&v(x.0), &v(x.0)), &poly_mul(&v(y.0), &v(y.0))),
                    &c(3),
                ),
            ],
        ),
        options,
    );
    assert_ne!(result.status, SolverStatus::CertifiedNonFiniteTargetImage);
    assert!(matches!(
        result.status,
        SolverStatus::AlgorithmicHardCase
            | SolverStatus::FiniteResourceFailure
            | SolverStatus::CertificateDesignGap
    ));
    assert!(
        !result.cost_trace.block_traces.is_empty(),
        "bounded no-positive-proof case must retain trace: {:?}",
        result.cost_trace
    );
}

#[test]
fn fcr_p11_red_team_09_regular_chain_fresh_input() {
    let t = VariableId(233);
    let y = VariableId(239);
    let result = public_candidate_cover_with_options(
        problem(
            vec![y, t],
            t,
            vec![
                poly_sub(&poly_add(&v(t.0), &c(1)), &v(y.0)),
                poly_sub(&poly_mul(&v(y.0), &v(y.0)), &c(3)),
            ],
        ),
        options_prioritizing(KernelKind::RegularChainProjection),
    );
    assert!(!result.root_isolation.is_empty());
    assert_executed_kernel(&result, KernelKind::RegularChainProjection);
}

#[test]
fn fcr_p11_red_team_10_norm_trace_fresh_two_step_tower() {
    let t = VariableId(251);
    let a = VariableId(257);
    let b = VariableId(263);
    let result = public_candidate_cover_with_options(
        problem(
            vec![b, a, t],
            t,
            vec![
                poly_sub(&poly_mul(&v(a.0), &v(a.0)), &c(3)),
                poly_sub(&poly_mul(&v(b.0), &v(b.0)), &v(a.0)),
                poly_sub(&poly_mul(&v(t.0), &v(b.0)), &c(2)),
            ],
        ),
        options_prioritizing(KernelKind::NormTraceProjection),
    );
    assert!(!result.root_isolation.is_empty());
    assert_executed_kernel(&result, KernelKind::NormTraceProjection);
}
