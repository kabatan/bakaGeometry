use geosolver_core::algebra::groebner::{groebner_elimination_basis, GroebnerOptions};
use geosolver_core::algebra::monomial_order::elimination_order;
use geosolver_core::algebra::resultant::{build_sparse_resultant_template, ResultantInput};
use geosolver_core::api::solve_target;
use geosolver_core::compose::final_support::FinalSupportComputation;
use geosolver_core::kernels::traits::KernelKind;
use geosolver_core::problem::context::new_context;
use geosolver_core::problem::input::{make_problem, RationalTargetProblem};
use geosolver_core::problem::semantic::{register_slack_encoding, RealConstraintKind};
use geosolver_core::result::cost_trace::GlobalCostTrace;
use geosolver_core::result::output::TargetSolveResult;
use geosolver_core::result::status::{FailureKind, SolverError, SolverErrorKind, SolverStatus};
use geosolver_core::solver::options::SolverOptions;
use geosolver_core::solver::pipeline::{
    step_build_dag, step_build_graphs, step_canonicalize, step_compose, step_compress,
    step_core_certificate, step_cost_trace, step_execute, step_plan, step_roots, step_support,
    step_validate, step_verify_messages,
};
use geosolver_core::types::hash::Hash;
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

fn problem(
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
            .map(|(idx, relation)| poly_scale(&relation, &int_q((idx as i64) + 2)))
            .collect(),
        Vec::new(),
    )
}

fn multivariate_candidate_problem() -> RationalTargetProblem {
    let t = VariableId(0);
    let x = VariableId(1);
    problem(
        vec![x, t],
        t,
        vec![
            poly_sub(&poly_mul(&v(x.0), &v(x.0)), &c(1)),
            poly_sub(&v(t.0), &v(x.0)),
        ],
    )
}

fn exact_semantic_problem(
    kind: RealConstraintKind,
    guard: SparsePolynomialQ,
) -> RationalTargetProblem {
    let t = VariableId(0);
    let support = poly_mul(&v(t.0), &poly_sub(&v(t.0), &c(1)));
    exact_semantic_problem_with_support(kind, support, guard)
}

fn exact_semantic_problem_with_support(
    kind: RealConstraintKind,
    support: SparsePolynomialQ,
    guard: SparsePolynomialQ,
) -> RationalTargetProblem {
    let t = VariableId(0);
    let s = VariableId(1);
    let square_slack = poly_sub(&guard, &poly_mul(&v(s.0), &v(s.0)));
    make_problem(
        vec![t, s],
        t,
        vec![
            poly_scale(&support, &int_q(2)),
            poly_scale(&square_slack, &int_q(3)),
        ],
        vec![register_slack_encoding(kind, vec![RelationId(1)], vec![s])],
    )
}

fn assert_public_candidate_success(result: &TargetSolveResult, problem: &RationalTargetProblem) {
    assert_eq!(
        result.status,
        SolverStatus::CertifiedCandidateCover,
        "diagnostics={:?}",
        result.diagnostics
    );
    assert_eq!(result.target, problem.target);
    assert!(result.support_polynomial.is_some());
    assert!(result.squarefree_support_polynomial.is_some());
    assert!(!result.root_isolation.is_empty());
    assert_eq!(result.root_isolation.len(), result.decoded_candidates.len());
    assert!(!result.projection_messages.is_empty());
    assert!(result.certificate.is_some());
    assert!(result
        .certificate
        .as_ref()
        .is_some_and(|cert| cert.exact_image_certificate_hash.is_none()));
    assert!(replay_run_certificate(result, problem).accepted);
    assert_result_cost_trace(result, problem.variables.len(), problem.equations.len());
    assert_truthful_runtime_invariants(result);
}

fn assert_result_cost_trace(
    result: &TargetSolveResult,
    expected_variable_count: usize,
    expected_relation_count: usize,
) {
    let trace = &result.cost_trace;
    assert!(trace.total_variable_count > 0);
    assert!(trace.total_variable_count <= expected_variable_count);
    assert!(trace.total_relation_count > 0);
    assert!(trace.total_relation_count <= expected_relation_count);
    assert!(trace.total_monomial_count >= trace.total_relation_count);
    assert!(trace.max_total_degree >= 1);
    assert!(trace.max_coefficient_height_bits >= 1);
    assert!(trace.max_block_width >= 1);
    assert!(trace.max_separator_width >= 1);
    assert!(trace.final_support_degree.is_some_and(|degree| degree >= 1));
    assert!(trace.certificate_size.is_some_and(|size| size > 0));
    assert_eq!(trace.block_traces.len(), result.projection_messages.len());
    assert_eq!(
        trace
            .block_traces
            .iter()
            .map(|block| block.block_id)
            .collect::<Vec<_>>(),
        result
            .projection_messages
            .iter()
            .map(|message| message.block_id)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        trace
            .block_traces
            .iter()
            .map(|block| block.kernel_kind)
            .collect::<Vec<_>>(),
        result
            .projection_messages
            .iter()
            .map(|message| message.kernel_kind)
            .collect::<Vec<_>>()
    );
    for block in &trace.block_traces {
        assert!(
            block.coefficient_height_before_bits > 0,
            "block trace must record input coefficient height: {block:?}"
        );
        assert!(
            block.coefficient_height_after_bits > 0,
            "block trace must record output coefficient height: {block:?}"
        );
    }
    assert!(trace.composition_trace.relation_count_before > 0);
    assert!(trace.composition_trace.relation_count_after > 0);
    assert!(trace.verification_trace.checked_relation_count > 0);
}

fn assert_truthful_runtime_invariants(result: &TargetSolveResult) {
    let cert = result.certificate.as_ref().expect("core run certificate");
    assert_eq!(
        cert.projection_message_hashes.len(),
        result.projection_messages.len()
    );
    assert_eq!(
        cert.kernel_plan_hashes.len(),
        result.projection_messages.len()
    );
    assert!(cert.global_support_certificate_hash.is_some());
    assert!(cert.final_dag_replay_evidence_hash.is_some());
    assert!(cert.final_dag_replay_evidence.is_some());
    assert!(cert.invariants.p11_replay_enforced());
    assert!(!cert.invariants.no_geometry_dispatch);
    assert!(!cert.invariants.no_problem_id_dispatch);
    assert!(!cert.invariants.no_expected_answer_dispatch);
    assert!(!cert.invariants.no_qe_cad);
    assert_ne!(cert.run_hash, Hash([0; 32]));
}

fn assert_finite_resource_height(err: SolverError, expected_stage: &str) {
    match err.kind {
        SolverErrorKind::Failure(FailureKind::FiniteResourceFailure {
            stage,
            coefficient_height_bits,
            ..
        }) => {
            assert_eq!(stage.0, expected_stage);
            assert!(
                coefficient_height_bits.is_some_and(|height| height > 0),
                "finite-resource error must carry observed coefficient height"
            );
        }
        other => panic!("expected finite-resource failure, got {other:?}"),
    }
}

#[test]
fn p14_stage_trace_executes_appendix_29_pipeline_in_order() {
    let problem = multivariate_candidate_problem();
    let target = problem.target;
    let public_result = solve_target(problem.clone(), SolverOptions::default());
    assert_public_candidate_success(&public_result, &problem);

    let mut ctx = new_context(SolverOptions::default());
    let validated = step_validate(problem.clone(), &mut ctx).expect("validate");
    let canonical = step_canonicalize(validated, &mut ctx).expect("canonicalize");
    let compressed = step_compress(canonical.clone(), &mut ctx).expect("compress");
    let graphs = step_build_graphs(&compressed, &mut ctx).expect("graphs");
    let dag = step_build_dag(&graphs, &compressed, &mut ctx).expect("dag");
    let plans = step_plan(&dag, &compressed, &mut ctx).expect("plan");
    let messages = step_execute(&dag, &plans, &compressed, &mut ctx).expect("execute");
    step_verify_messages(&dag, &messages, &compressed).expect("verify messages");
    let composed = step_compose(&dag, messages.clone(), target, &mut ctx).expect("compose");
    let support_outcome = step_support(&composed, &compressed, target, &mut ctx).expect("support");
    let support = match support_outcome {
        FinalSupportComputation::Support(support) => support,
        FinalSupportComputation::CertifiedNonFinite(_) => {
            panic!("candidate problem must produce finite support")
        }
    };
    let support_certificate =
        geosolver_core::verify::verify_support::verify_global_support(&support, &composed)
            .expect("support certificate");
    let roots = step_roots(&support, target, &mut ctx).expect("roots");
    let certificate = step_core_certificate(
        &problem,
        &ctx.options,
        &canonical,
        &compressed,
        &graphs,
        &dag,
        &plans,
        &messages,
        Some(&support),
        &roots,
        None,
        Some(&support_certificate),
    );
    let cost_trace = step_cost_trace(
        &compressed,
        &dag,
        &messages,
        Some(&composed),
        Some(&support),
        Some(&certificate),
    );

    assert_eq!(public_result.support_polynomial.as_ref(), Some(&support));
    assert_eq!(
        public_result.squarefree_support_polynomial.as_ref(),
        Some(&roots.squarefree_support)
    );
    assert_eq!(public_result.root_isolation, roots.root_isolation);
    assert_eq!(public_result.decoded_candidates, roots.decoded_candidates);
    assert_eq!(public_result.projection_messages, messages);
    assert_eq!(public_result.cost_trace, cost_trace);
    assert_eq!(
        public_result.certificate.as_ref().map(|cert| cert.run_hash),
        Some(certificate.run_hash)
    );
}

#[test]
fn p14_empty_relation_nonfinite_still_runs_appendix_29_stages() {
    let t = VariableId(5);
    let x = VariableId(6);
    let problem = problem(vec![x, t], t, vec![poly_sub(&v(x.0), &c(1))]);
    let public_result = solve_target(problem.clone(), SolverOptions::default());
    assert_eq!(
        public_result.status,
        SolverStatus::CertifiedNonFiniteTargetImage
    );

    let mut ctx = new_context(SolverOptions::default());
    let validated = step_validate(problem.clone(), &mut ctx).expect("validate");
    let canonical = step_canonicalize(validated, &mut ctx).expect("canonicalize");
    let compressed = step_compress(canonical, &mut ctx).expect("compress");
    assert!(
        compressed.relations.is_empty(),
        "test must cover empty compressed relation branch: {:?}",
        compressed.relations
    );
    let graphs = step_build_graphs(&compressed, &mut ctx).expect("graphs");
    let dag = step_build_dag(&graphs, &compressed, &mut ctx).expect("dag");
    let plans = step_plan(&dag, &compressed, &mut ctx).expect("plan");
    assert!(plans.is_empty());
    let messages = step_execute(&dag, &plans, &compressed, &mut ctx).expect("execute");
    assert!(messages.is_empty());
    step_verify_messages(&dag, &messages, &compressed).expect("verify messages");
    let composed = step_compose(&dag, messages.clone(), t, &mut ctx).expect("compose");
    assert!(composed.message_relations.is_empty());
    assert!(composed.root_relations.is_empty());
    let support_outcome = step_support(&composed, &compressed, t, &mut ctx).expect("support");
    assert!(matches!(
        support_outcome,
        FinalSupportComputation::CertifiedNonFinite(_)
    ));
    let cost_trace = step_cost_trace(&compressed, &dag, &messages, Some(&composed), None, None);
    assert!(cost_trace.final_support_degree.is_none());
    assert!(cost_trace.certificate_size.is_none());

    assert_eq!(public_result.projection_messages, messages);
    assert_eq!(public_result.cost_trace, cost_trace);
}

#[test]
fn p14_groebner_resource_error_carries_coefficient_height() {
    let x = VariableId(71);
    let y = VariableId(72);
    let relations = vec![
        poly_sub(&poly_mul(&v(y.0), &v(y.0)), &v(x.0)),
        poly_sub(&poly_mul(&poly_mul(&v(y.0), &v(y.0)), &v(y.0)), &c(2)),
    ];
    let order = elimination_order(&[y], &[x]);

    let err = groebner_elimination_basis(
        &relations,
        &order,
        GroebnerOptions {
            max_pairs: 0,
            max_basis_size: 1,
        },
    )
    .unwrap_err();

    assert_finite_resource_height(err, "GroebnerLocalPairLimit");
}

#[test]
fn p14_sparse_resultant_resource_error_carries_coefficient_height() {
    let x = VariableId(81);
    let y = VariableId(82);
    let input = ResultantInput {
        polynomials: vec![
            poly_sub(&poly_mul(&v(y.0), &v(y.0)), &v(x.0)),
            poly_sub(&poly_mul(&poly_mul(&v(y.0), &v(y.0)), &v(y.0)), &c(2)),
        ],
        eliminate: y,
        keep_variables: vec![x],
        max_matrix_dim: 1,
    };

    let err = build_sparse_resultant_template(input).unwrap_err();

    assert_finite_resource_height(err, "SparseResultantTemplateMatrixCap");
}

#[test]
fn p14_public_candidate_cover_success_has_all_result_fields_and_trace() {
    let problem = multivariate_candidate_problem();
    let result = solve_target(problem.clone(), SolverOptions::default());

    assert_public_candidate_success(&result, &problem);
    assert_eq!(result.root_isolation.len(), 2);
    assert!(result
        .root_isolation
        .iter()
        .any(|root| interval_contains_q(&root.isolating_interval, &int_q(-1))));
    assert!(result
        .root_isolation
        .iter()
        .any(|root| interval_contains_q(&root.isolating_interval, &int_q(1))));
}

#[test]
fn p16_public_exact_image_request_returns_scope_guard_with_candidates() {
    let problem = exact_semantic_problem(RealConstraintKind::Positive, v(0));
    let result = solve_target(problem.clone(), exact_options());

    assert_eq!(
        result.status,
        SolverStatus::CertificateDesignGap,
        "diagnostics={:?}",
        result.diagnostics
    );
    assert!(result.support_polynomial.is_some());
    assert!(result.squarefree_support_polynomial.is_some());
    assert_eq!(result.root_isolation.len(), 2);
    assert_eq!(result.decoded_candidates.len(), 2);
    assert!(result
        .certificate
        .as_ref()
        .is_some_and(|cert| cert.exact_image_certificate_hash.is_none()));
    assert!(result.certificate.is_some());
    assert!(replay_run_certificate(&result, &problem).accepted);
    assert!(result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.name == "ExactImageOutOfScope"));
    assert_result_cost_trace(&result, problem.variables.len(), problem.equations.len());
}

#[test]
fn p16_public_exact_image_empty_semantics_do_not_filter_candidates() {
    let negative_square_guard = poly_scale(&poly_mul(&v(0), &v(0)), &int_q(-1));
    let support = poly_sub(&poly_mul(&v(0), &v(0)), &c(1));
    let problem = exact_semantic_problem_with_support(
        RealConstraintKind::NonNegative,
        support,
        negative_square_guard,
    );
    let result = solve_target(problem.clone(), exact_options());

    assert_eq!(
        result.status,
        SolverStatus::CertificateDesignGap,
        "diagnostics={:?}",
        result.diagnostics
    );
    assert!(result.support_polynomial.is_some());
    assert!(result.squarefree_support_polynomial.is_some());
    assert_eq!(result.root_isolation.len(), 2);
    assert_eq!(result.decoded_candidates.len(), 2);
    assert!(result
        .certificate
        .as_ref()
        .is_some_and(|cert| cert.exact_image_certificate_hash.is_none()));
    assert!(result.certificate.is_some());
    assert!(replay_run_certificate(&result, &problem).accepted);
    assert!(result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.name == "ExactImageOutOfScope"));
    assert_result_cost_trace(&result, problem.variables.len(), problem.equations.len());
}

#[test]
fn p14_public_certified_nonfinite_is_finalized_without_panic() {
    let t = VariableId(5);
    let x = VariableId(6);
    let problem = problem(vec![x, t], t, vec![poly_sub(&v(x.0), &c(1))]);

    let result = solve_target(problem.clone(), SolverOptions::default());

    assert_eq!(result.status, SolverStatus::CertifiedNonFiniteTargetImage);
    assert_eq!(result.target, t);
    assert!(result.support_polynomial.is_none());
    assert!(result.squarefree_support_polynomial.is_none());
    assert!(result.root_isolation.is_empty());
    assert!(result.decoded_candidates.is_empty());
    assert!(result.certificate.is_none());
    assert!(replay_run_certificate(&result, &problem).accepted);
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.name == "CertifiedNonFiniteTargetImage"
            && diagnostic.message.contains("ZeroTargetElimination")
    }));
    assert!(result.cost_trace.total_variable_count <= problem.variables.len());
    assert!(result.cost_trace.total_relation_count <= problem.equations.len());
    assert_eq!(
        result.cost_trace.block_traces.len(),
        result.projection_messages.len()
    );
}

#[test]
fn p14_public_bounded_hard_case_has_status_and_resource_trace() {
    let t = VariableId(59);
    let x = VariableId(39);
    let mut options = SolverOptions {
        kernel_priority: vec![KernelKind::TargetRelationSearch],
        ..SolverOptions::default()
    };
    options.max_relation_search_export_degree = Some(2);
    options.max_matrix_rows = Some(1);
    options.max_matrix_cols = Some(1);
    let problem = problem(
        vec![x, t],
        t,
        vec![poly_sub(&poly_mul(&v(x.0), &v(x.0)), &v(t.0))],
    );

    let result = solve_target(problem, options);

    assert!(matches!(
        result.status,
        SolverStatus::AlgorithmicHardCase
            | SolverStatus::FiniteResourceFailure
            | SolverStatus::CertificateDesignGap
    ));
    assert_eq!(result.target, t);
    assert!(result.support_polynomial.is_none());
    assert!(result.cost_trace.total_variable_count > 0);
    assert!(result.cost_trace.total_relation_count > 0);
    assert!(result.cost_trace.total_monomial_count > 0);
    assert!(result.cost_trace.max_total_degree > 0);
    assert!(result.cost_trace.max_coefficient_height_bits > 0);
    assert!(result.cost_trace.max_block_width > 0);
    assert!(result.cost_trace.max_separator_width > 0);
    assert!(result.cost_trace.final_support_degree.is_none());
    assert!(result.cost_trace.certificate_size.is_none());
    assert_eq!(
        result.cost_trace.verification_trace.checked_relation_count,
        0
    );
    assert!(!result.cost_trace.block_traces.is_empty());
    assert!(result.cost_trace.block_traces.iter().any(|trace| {
        trace.kernel_kind == KernelKind::TargetRelationSearch
            && (trace.matrix_rows.is_some() || trace.matrix_cols.is_some())
            && trace.local_variable_count > 0
            && trace.local_relation_count > 0
            && trace.local_monomial_count > 0
            && trace.coefficient_height_before_bits > 0
            && trace.coefficient_height_after_bits > 0
    }));
    assert!(result.certificate.is_none());
}

#[test]
fn p14_public_invalid_input_maps_to_result_not_panic() {
    let target = VariableId(0);
    let x = VariableId(1);
    let problem = make_problem(vec![x], target, vec![v(x.0)], Vec::new());

    let result = solve_target(problem, SolverOptions::default());

    assert_eq!(result.status, SolverStatus::InvalidInput);
    assert_eq!(result.target, target);
    assert!(result.support_polynomial.is_none());
    assert!(result.squarefree_support_polynomial.is_none());
    assert!(result.root_isolation.is_empty());
    assert!(result.decoded_candidates.is_empty());
    assert!(result.projection_messages.is_empty());
    assert!(result.certificate.is_none());
    assert_eq!(result.cost_trace, GlobalCostTrace::default());
    assert!(result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.name == "InvalidInput"));
}
