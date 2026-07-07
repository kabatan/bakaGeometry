use geosolver_core::algebra::resultant::ResultantBackendKind;
use geosolver_core::api::solve_target;
use geosolver_core::compose::compose::compose_projection_messages;
use geosolver_core::kernels::traits::KernelKind;
use geosolver_core::planner::planner::plan_all_blocks;
use geosolver_core::problem::context::new_context;
use geosolver_core::problem::input::{make_problem, RationalTargetProblem};
use geosolver_core::result::output::TargetSolveResult;
use geosolver_core::result::status::SolverStatus;
use geosolver_core::solver::options::SolverOptions;
use geosolver_core::solver::pipeline::{
    step_build_dag, step_build_graphs, step_canonicalize, step_compress, step_validate,
    step_verify_messages,
};
use geosolver_core::types::ids::VariableId;
use geosolver_core::types::polynomial::{
    constant_poly, poly_add, poly_mul, poly_scale, poly_sub, variable_poly, SparsePolynomialQ,
};
use geosolver_core::types::rational::{div_q, int_q, RationalQ};
use geosolver_core::verify::certificates::KernelCertificatePayload;
use geosolver_core::verify::{replay_run_certificate, verify_global_support};

#[derive(Debug, Clone, Copy)]
struct Variant {
    base: u32,
    permute_variable_ids: bool,
    permute_relations: bool,
    scale_relations: bool,
}

#[derive(Debug, Clone, Copy)]
struct Expectations {
    successful_route: KernelKind,
    dense_trs_materialization_allowed: Option<bool>,
    sparse_resultant_status: Option<&'static str>,
    sparse_resultant_cost_class: Option<&'static str>,
    sparse_resultant_backend: Option<ResultantBackendKind>,
    require_sparse_resultant_bounded_failure_probe: bool,
    require_graph_split: bool,
    require_one_large_block: bool,
    required_universal_internal_failures: usize,
}

fn variants() -> [Variant; 3] {
    [
        Variant {
            base: 10_000,
            permute_variable_ids: false,
            permute_relations: false,
            scale_relations: true,
        },
        Variant {
            base: 10_000,
            permute_variable_ids: true,
            permute_relations: false,
            scale_relations: true,
        },
        Variant {
            base: 10_000,
            permute_variable_ids: false,
            permute_relations: true,
            scale_relations: true,
        },
    ]
}

#[test]
fn acr_p9_s1_target_action_large_block() {
    run_family("S1_target_action_large_block", s1_target_action_large_block);
}

#[test]
fn acr_p9_s2_sparse_relation_search() {
    run_family("S2_sparse_relation_search", s2_sparse_relation_search);
}

#[test]
fn acr_p9_s3_sparse_resultant_feasible() {
    run_family("S3_sparse_resultant_feasible", s3_sparse_resultant_feasible);
}

#[test]
fn acr_p9_s4_sparse_resultant_prohibited_later_route() {
    run_family(
        "S4_sparse_resultant_prohibited_later_route",
        s4_sparse_resultant_prohibited,
    );
}

#[test]
fn acr_p9_s5_specialization_interpolation_after_prohibitions() {
    run_family(
        "S5_specialization_interpolation_after_prohibitions",
        s5_specialization_interpolation,
    );
}

#[test]
fn acr_p9_s6_universal_after_internal_failures() {
    run_family(
        "S6_universal_after_internal_failures",
        s6_universal_internal_failures,
    );
}

#[test]
fn acr_p9_s7_graph_decomposition_separator() {
    run_family(
        "S7_graph_decomposition_separator",
        s7_graph_decomposition_separator,
    );
}

#[test]
fn acr_p9_s8_one_large_block_universal() {
    run_family("S8_one_large_block_universal", s8_one_large_block_universal);
}

fn run_family(
    family: &str,
    build: fn(Variant) -> (RationalTargetProblem, SolverOptions, Expectations),
) {
    for (variant_index, variant) in variants().into_iter().enumerate() {
        let label = format!("{family}::variant_{variant_index}");
        eprintln!("running {label}");
        let (problem, options, expectations) = build(variant);
        assert_graph_expectations(&label, &problem, expectations);
        if expectations.require_sparse_resultant_bounded_failure_probe {
            assert_sparse_resultant_bounded_failure_probe(&label, &problem);
        }
        let result = solve_target(problem.clone(), options);
        assert_support_producing_success(&label, &problem, &result, expectations);
    }
}

fn assert_support_producing_success(
    label: &str,
    problem: &RationalTargetProblem,
    result: &TargetSolveResult,
    expectations: Expectations,
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
        support
            .coeffs_low_to_high
            .iter()
            .any(|coeff| coeff != &int_q(0)),
        "{label}: support polynomial is zero"
    );
    assert!(
        support.coeffs_low_to_high.len() > 1,
        "{label}: support polynomial must be nonconstant"
    );
    assert!(result.squarefree_support_polynomial.is_some());
    assert!(result.exact_image_certificate.is_none());
    assert!(result.nonfinite_certificate.is_none());
    assert!(!result.projection_messages.is_empty());
    assert!(result.certificate.is_some());
    assert!(result.cost_trace.final_support_degree.is_some());
    assert!(result.cost_trace.certificate_size.is_some());
    assert!(!result.cost_trace.block_traces.is_empty());
    assert!(result
        .cost_trace
        .block_traces
        .iter()
        .any(|trace| trace.route_cost.is_some()));
    assert!(result.diagnostics.iter().any(|record| {
        record.name == "ExactImageFilteringNotRequested"
            || record.name == "CandidateCoverMayContainSpuriousRoots"
    }));

    let composed = compose_for_problem(problem, result.projection_messages.clone())
        .unwrap_or_else(|| panic!("{label}: failed to recompose projection messages"));
    verify_global_support(support, &composed)
        .unwrap_or_else(|err| panic!("{label}: support verification failed: {err:?}"));

    let replay = replay_run_certificate(result, problem);
    assert!(replay.accepted, "{label}: replay failed: {replay:?}");

    verify_messages_for_problem(label, problem, &result.projection_messages);
    let message_routes = result
        .projection_messages
        .iter()
        .map(|message| message.kernel_kind)
        .collect::<Vec<_>>();
    assert!(
        result.projection_messages.iter().any(|message| {
            message.kernel_kind == expectations.successful_route
                && message_routes.contains(&message.kernel_kind)
        }),
        "{label}: required verified route {:?}, messages={:?}, diagnostics={:?}",
        expectations.successful_route,
        message_routes,
        result.diagnostics
    );
    assert!(
        result.diagnostics.iter().any(|record| {
            record.name == "BlockProjectionRouteSuccess"
                && record.details.get("kernel_kind").map(String::as_str)
                    == Some(kernel_name(expectations.successful_route))
        }),
        "{label}: missing success route trace for {:?}",
        expectations.successful_route
    );

    if let Some(required_dense_allowed) = expectations.dense_trs_materialization_allowed {
        assert!(
            result.diagnostics.iter().any(|record| {
                record.name == "KernelRouteTrace"
                    && record.details.get("kernel_kind").map(String::as_str)
                        == Some("TargetRelationSearch")
                    && record
                        .details
                        .get("materialization_allowed")
                        .map(String::as_str)
                        == Some(if required_dense_allowed {
                            "true"
                        } else {
                            "false"
                        })
            }),
            "{label}: missing dense TRS materialization diagnostic"
        );
    }
    if let Some(required_status) = expectations.sparse_resultant_status {
        assert!(
            result.diagnostics.iter().any(|record| {
                record.name == "KernelRouteTrace"
                    && record.details.get("kernel_kind").map(String::as_str)
                        == Some("SparseResultantProjection")
                    && record.details.get("admission_status").map(String::as_str)
                        == Some(required_status)
            }),
            "{label}: missing SparseResultant status {required_status}; diagnostics={:?}",
            result.diagnostics
        );
    }
    if let Some(required_cost_class) = expectations.sparse_resultant_cost_class {
        assert!(
            result.diagnostics.iter().any(|record| {
                record.name == "KernelRouteTrace"
                    && record.details.get("kernel_kind").map(String::as_str)
                        == Some("SparseResultantProjection")
                    && record.details.get("cost_class").map(String::as_str)
                        == Some(required_cost_class)
            }),
            "{label}: missing SparseResultant cost_class {required_cost_class}; diagnostics={:?}",
            result.diagnostics
        );
    }
    if let Some(required_backend) = expectations.sparse_resultant_backend {
        assert_sparse_resultant_backend(label, result, required_backend);
    }
    if expectations.required_universal_internal_failures > 0 {
        let universal = result
            .projection_messages
            .iter()
            .find(|message| message.kernel_kind == KernelKind::UniversalTargetElimination)
            .unwrap_or_else(|| panic!("{label}: missing universal message"));
        let KernelCertificatePayload::Universal(proof) = &universal.certificate.payload else {
            panic!("{label}: universal message lacks universal payload");
        };
        assert!(
            proof.executed_failed_strategy_hashes.len()
                >= expectations.required_universal_internal_failures,
            "{label}: required at least {} bounded attempted internal Universal failures, got {:?}",
            expectations.required_universal_internal_failures,
            proof
                .strategy_records
                .iter()
                .map(|record| (
                    record.strategy,
                    record.enabled,
                    proof
                        .executed_failed_strategy_hashes
                        .contains(&record.stage_hash),
                    record.cost_class,
                    record.skip_reason.clone()
                ))
                .collect::<Vec<_>>()
        );
    }
}

fn assert_sparse_resultant_bounded_failure_probe(label: &str, problem: &RationalTargetProblem) {
    let mut ctx = new_context(SolverOptions {
        kernel_priority: vec![KernelKind::SparseResultantProjection],
        max_matrix_rows: Some(1),
        ..SolverOptions::default()
    });
    let validated = step_validate(problem.clone(), &mut ctx).unwrap();
    let canonical = step_canonicalize(validated, &mut ctx).unwrap();
    let compressed = step_compress(canonical, &mut ctx).unwrap();
    let graphs = step_build_graphs(&compressed, &mut ctx).unwrap();
    let dag = step_build_dag(&graphs, &compressed, &mut ctx).unwrap();
    let _ = plan_all_blocks(&dag, &compressed, &mut ctx);
    assert!(
        ctx.diagnostics.iter().any(|record| {
            record.name == "KernelRouteTrace"
                && record.details.get("kernel_kind").map(String::as_str)
                    == Some("SparseResultantProjection")
                && matches!(
                    record.details.get("admission_status").map(String::as_str),
                    Some("PlanProbeFailed" | "CostProhibited")
                )
        }),
        "{label}: missing bounded SparseResultant planning failure diagnostics: {:?}",
        ctx.diagnostics
    );
}

fn assert_sparse_resultant_backend(
    label: &str,
    result: &TargetSolveResult,
    required_backend: ResultantBackendKind,
) {
    let sparse = result
        .projection_messages
        .iter()
        .find(|message| message.kernel_kind == KernelKind::SparseResultantProjection)
        .unwrap_or_else(|| panic!("{label}: missing SparseResultant projection message"));
    let KernelCertificatePayload::SparseResultant(payload) = &sparse.certificate.payload else {
        panic!("{label}: SparseResultant message has wrong payload");
    };
    assert!(
        payload
            .resultant_certificates
            .iter()
            .any(|certificate| certificate.backend == required_backend),
        "{label}: missing SparseResultant backend {:?}; certificates={:?}",
        required_backend,
        payload.resultant_certificates
    );
}

fn assert_graph_expectations(
    label: &str,
    problem: &RationalTargetProblem,
    expectations: Expectations,
) {
    if !expectations.require_graph_split && !expectations.require_one_large_block {
        return;
    }
    let mut ctx = new_context(SolverOptions::default());
    let validated = step_validate(problem.clone(), &mut ctx).unwrap();
    let canonical = step_canonicalize(validated, &mut ctx).unwrap();
    let compressed = step_compress(canonical, &mut ctx).unwrap();
    let graphs = step_build_graphs(&compressed, &mut ctx).unwrap();
    if expectations.require_graph_split {
        assert!(
            !graphs.decomposition.root.children.is_empty(),
            "{label}: required graph decomposition split"
        );
        assert!(graphs.decomposition.diagnostics.iter().any(|diagnostic| {
            diagnostic.selected && diagnostic.reason == "selected_cost_improving_separator"
        }));
    }
    if expectations.require_one_large_block {
        assert!(
            graphs.decomposition.root.children.is_empty(),
            "{label}: required one large block with no useful separator"
        );
    }
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

fn verify_messages_for_problem(
    label: &str,
    problem: &RationalTargetProblem,
    messages: &[geosolver_core::compose::message::ProjectionMessage],
) {
    let mut ctx = new_context(SolverOptions::default());
    let validated = step_validate(problem.clone(), &mut ctx).unwrap();
    let canonical = step_canonicalize(validated, &mut ctx).unwrap();
    let compressed = step_compress(canonical, &mut ctx).unwrap();
    let graphs = step_build_graphs(&compressed, &mut ctx).unwrap();
    let dag = step_build_dag(&graphs, &compressed, &mut ctx).unwrap();
    step_verify_messages(&dag, messages, &compressed)
        .unwrap_or_else(|err| panic!("{label}: projection message verification failed: {err:?}"));
}

fn options_prioritizing(kind: KernelKind) -> SolverOptions {
    SolverOptions {
        kernel_priority: vec![kind],
        ..SolverOptions::default()
    }
}

fn options_prioritizing_order(kinds: Vec<KernelKind>) -> SolverOptions {
    SolverOptions {
        kernel_priority: kinds,
        ..SolverOptions::default()
    }
}

fn options_prioritizing_with_dense_cap(kind: KernelKind, cap: usize) -> SolverOptions {
    SolverOptions {
        kernel_priority: vec![kind],
        max_relation_search_export_degree: Some(cap),
        ..SolverOptions::default()
    }
}

fn s1_target_action_large_block(
    variant: Variant,
) -> (RationalTargetProblem, SolverOptions, Expectations) {
    let [t, x, y] = ids(variant, [1, 5, 11]);
    let xy = poly_mul(&vp(x), &vp(y));
    let relations = variant_relations(
        variant,
        vec![
            poly_sub(&poly_mul(&vp(x), &vp(x)), &cp(2)),
            poly_sub(&poly_mul(&vp(y), &vp(y)), &cp(3)),
            poly_sub(&poly_sub(&vp(t), &xy), &vp(x)),
        ],
    );
    (
        make_problem(vec![y, t, x], t, relations, Vec::new()),
        options_prioritizing_with_dense_cap(KernelKind::TargetActionKrylov, 0),
        Expectations {
            successful_route: KernelKind::TargetActionKrylov,
            dense_trs_materialization_allowed: Some(false),
            sparse_resultant_status: None,
            sparse_resultant_cost_class: None,
            sparse_resultant_backend: None,
            require_sparse_resultant_bounded_failure_probe: false,
            require_graph_split: false,
            require_one_large_block: false,
            required_universal_internal_failures: 0,
        },
    )
}

fn s2_sparse_relation_search(
    variant: Variant,
) -> (RationalTargetProblem, SolverOptions, Expectations) {
    let [t, x, y] = ids(variant, [17, 23, 29]);
    let relations = variant_relations(
        variant,
        vec![
            poly_sub(&vp(x), &vp(t)),
            poly_sub(&poly_mul(&vp(x), &vp(x)), &cp(1)),
            poly_sub(&vp(y), &vp(x)),
        ],
    );
    (
        make_problem(vec![t, x, y], t, relations, Vec::new()),
        options_prioritizing_with_dense_cap(KernelKind::TargetRelationSearch, 0),
        Expectations {
            successful_route: KernelKind::TargetRelationSearch,
            dense_trs_materialization_allowed: Some(false),
            sparse_resultant_status: None,
            sparse_resultant_cost_class: None,
            sparse_resultant_backend: None,
            require_sparse_resultant_bounded_failure_probe: false,
            require_graph_split: false,
            require_one_large_block: false,
            required_universal_internal_failures: 0,
        },
    )
}

fn s3_sparse_resultant_feasible(
    variant: Variant,
) -> (RationalTargetProblem, SolverOptions, Expectations) {
    let [t, x] = ids(variant, [31, 37]);
    let x2 = poly_mul(&vp(x), &vp(x));
    let relations = variant_relations(
        variant,
        vec![
            poly_sub(&x2, &poly_add(&vp(t), &cp(1))),
            poly_sub(&x2, &cp(7)),
        ],
    );
    (
        make_problem(vec![t, x], t, relations, Vec::new()),
        options_prioritizing(KernelKind::SparseResultantProjection),
        Expectations {
            successful_route: KernelKind::SparseResultantProjection,
            dense_trs_materialization_allowed: None,
            sparse_resultant_status: Some("Admitted"),
            sparse_resultant_cost_class: None,
            sparse_resultant_backend: Some(ResultantBackendKind::QuadraticSubresultant),
            require_sparse_resultant_bounded_failure_probe: false,
            require_graph_split: false,
            require_one_large_block: false,
            required_universal_internal_failures: 0,
        },
    )
}

fn s4_sparse_resultant_prohibited(
    variant: Variant,
) -> (RationalTargetProblem, SolverOptions, Expectations) {
    let [t, x, y] = ids(variant, [41, 43, 47]);
    let keep_variables = ids(
        variant,
        [
            149, 151, 157, 163, 167, 173, 179, 181, 191, 193, 197, 199, 211, 223, 227, 229, 233,
            239, 241, 251, 257, 263, 269, 271, 277,
        ],
    );
    let keep_product = product_poly(&keep_variables);
    let relations = variant_relations(
        variant,
        vec![
            poly_sub(&vp(x), &vp(t)),
            poly_sub(&poly_mul(&vp(x), &vp(x)), &cp(2)),
            poly_sub(&poly_mul(&vp(y), &vp(y)), &vp(x)),
            poly_sub(
                &poly_mul(&poly_mul(&vp(y), &vp(y)), &vp(y)),
                &poly_mul(&vp(x), &vp(y)),
            ),
            poly_add(&vp(y), &keep_product),
            poly_add(&poly_mul(&vp(y), &vp(y)), &keep_product),
        ],
    );
    let mut variables = vec![t, x, y];
    variables.extend(keep_variables);
    (
        make_problem(variables, t, relations, Vec::new()),
        SolverOptions {
            max_relation_search_export_degree: Some(0),
            ..options_prioritizing_order(vec![
                KernelKind::SparseResultantProjection,
                KernelKind::TargetRelationSearch,
            ])
        },
        Expectations {
            successful_route: KernelKind::TargetRelationSearch,
            dense_trs_materialization_allowed: Some(false),
            sparse_resultant_status: None,
            sparse_resultant_cost_class: Some("CostProhibited"),
            sparse_resultant_backend: None,
            require_sparse_resultant_bounded_failure_probe: false,
            require_graph_split: false,
            require_one_large_block: false,
            required_universal_internal_failures: 0,
        },
    )
}

fn s5_specialization_interpolation(
    variant: Variant,
) -> (RationalTargetProblem, SolverOptions, Expectations) {
    let [t, u, w, x] = ids(variant, [53, 59, 61, 67]);
    let x2 = poly_mul(&vp(x), &vp(x));
    let u2 = poly_mul(&vp(u), &vp(u));
    let relations = variant_relations(
        variant,
        vec![
            poly_sub(&poly_mul(&vp(w), &vp(w)), &cp(2)),
            poly_sub(&x2, &cp(1)),
            poly_sub(&x2, &poly_add(&vp(t), &u2)),
            poly_sub(&poly_mul(&vp(u), &vp(w)), &cp(1)),
        ],
    );
    (
        make_problem(vec![w, t, x, u], t, relations, Vec::new()),
        SolverOptions {
            max_relation_search_export_degree: Some(1),
            ..options_prioritizing_order(vec![KernelKind::SpecializationInterpolation])
        },
        Expectations {
            successful_route: KernelKind::SpecializationInterpolation,
            dense_trs_materialization_allowed: Some(false),
            sparse_resultant_status: None,
            sparse_resultant_cost_class: None,
            sparse_resultant_backend: None,
            require_sparse_resultant_bounded_failure_probe: true,
            require_graph_split: false,
            require_one_large_block: false,
            required_universal_internal_failures: 0,
        },
    )
}

fn s6_universal_internal_failures(
    variant: Variant,
) -> (RationalTargetProblem, SolverOptions, Expectations) {
    let [t, x, y, z] = ids(variant, [71, 73, 79, 81]);
    let xy = poly_mul(&vp(x), &vp(y));
    let relations = variant_relations(
        variant,
        vec![
            poly_sub(&poly_mul(&vp(x), &vp(x)), &cp(2)),
            poly_sub(&poly_mul(&vp(y), &vp(y)), &cp(3)),
            poly_sub(&poly_mul(&vp(z), &vp(z)), &cp(5)),
            poly_sub(&poly_sub(&poly_sub(&vp(t), &xy), &vp(x)), &vp(z)),
        ],
    );
    (
        make_problem(vec![t, z, y, x], t, relations, Vec::new()),
        options_prioritizing_with_dense_cap(KernelKind::UniversalTargetElimination, 0),
        Expectations {
            successful_route: KernelKind::UniversalTargetElimination,
            dense_trs_materialization_allowed: Some(false),
            sparse_resultant_status: None,
            sparse_resultant_cost_class: None,
            sparse_resultant_backend: None,
            require_sparse_resultant_bounded_failure_probe: false,
            require_graph_split: false,
            require_one_large_block: false,
            required_universal_internal_failures: 2,
        },
    )
}

fn s7_graph_decomposition_separator(
    variant: Variant,
) -> (RationalTargetProblem, SolverOptions, Expectations) {
    let [t, u, w, x, y] = ids(variant, [83, 89, 97, 101, 103]);
    let x2 = poly_mul(&vp(x), &vp(x));
    let x4 = poly_mul(&x2, &x2);
    let x6 = poly_mul(&x4, &x2);
    let u2 = poly_mul(&vp(u), &vp(u));
    let w2 = poly_mul(&vp(w), &vp(w));
    let y2 = poly_mul(&vp(y), &vp(y));
    let y3 = poly_mul(&y2, &vp(y));
    let relations = variant_relations(
        variant,
        vec![
            poly_sub(&y3, &cp(2)),
            poly_sub(&u2, &vp(t)),
            poly_sub(&y2, &x2),
            poly_sub(&w2, &u2),
            poly_sub(&x2, &w2),
            poly_sub(&x6, &cp(4)),
        ],
    );
    (
        make_problem(vec![w, x, t, y, u], t, relations, Vec::new()),
        options_prioritizing(KernelKind::TargetRelationSearch),
        Expectations {
            successful_route: KernelKind::TargetRelationSearch,
            dense_trs_materialization_allowed: None,
            sparse_resultant_status: None,
            sparse_resultant_cost_class: None,
            sparse_resultant_backend: None,
            require_sparse_resultant_bounded_failure_probe: false,
            require_graph_split: true,
            require_one_large_block: false,
            required_universal_internal_failures: 0,
        },
    )
}

fn s8_one_large_block_universal(
    variant: Variant,
) -> (RationalTargetProblem, SolverOptions, Expectations) {
    let [t, x, y] = ids(variant, [107, 109, 113]);
    let relations = variant_relations(
        variant,
        vec![
            poly_sub(&poly_sub(&vp(t), &vp(x)), &vp(y)),
            poly_sub(&poly_add(&poly_mul(&vp(x), &vp(x)), &vp(y)), &cp(2)),
            poly_sub(&poly_sub(&poly_mul(&vp(y), &vp(y)), &vp(x)), &cp(1)),
        ],
    );
    (
        make_problem(vec![y, x, t], t, relations, Vec::new()),
        options_prioritizing_with_dense_cap(KernelKind::UniversalTargetElimination, 0),
        Expectations {
            successful_route: KernelKind::UniversalTargetElimination,
            dense_trs_materialization_allowed: None,
            sparse_resultant_status: None,
            sparse_resultant_cost_class: None,
            sparse_resultant_backend: None,
            require_sparse_resultant_bounded_failure_probe: false,
            require_graph_split: false,
            require_one_large_block: true,
            required_universal_internal_failures: 0,
        },
    )
}

fn variant_relations(
    variant: Variant,
    relations: Vec<SparsePolynomialQ>,
) -> Vec<SparsePolynomialQ> {
    let mut out = relations
        .into_iter()
        .enumerate()
        .map(|(idx, relation)| {
            if variant.scale_relations {
                poly_scale(&relation, &scale_for(idx))
            } else {
                relation
            }
        })
        .collect::<Vec<_>>();
    if variant.permute_relations {
        if out.len() > 1 {
            out.swap(0, 1);
        }
    }
    out
}

fn ids<const N: usize>(variant: Variant, offsets: [u32; N]) -> [VariableId; N] {
    std::array::from_fn(|index| {
        let mapped = if variant.permute_variable_ids {
            offsets[index] * 3 + (index as u32 + 1)
        } else {
            offsets[index]
        };
        VariableId(variant.base + mapped)
    })
}

fn vp(variable: VariableId) -> SparsePolynomialQ {
    variable_poly(variable)
}

fn cp(value: i64) -> SparsePolynomialQ {
    constant_poly(int_q(value))
}

fn product_poly<const N: usize>(variables: &[VariableId; N]) -> SparsePolynomialQ {
    variables
        .iter()
        .fold(cp(1), |acc, variable| poly_mul(&acc, &vp(*variable)))
}

fn scale_for(index: usize) -> RationalQ {
    let num = match index % 7 {
        0 => -3,
        1 => 2,
        2 => -5,
        3 => 7,
        4 => -11,
        5 => 13,
        _ => -17,
    };
    div_q(&int_q(num), &int_q(1)).expect("nonzero rational scale")
}

fn kernel_name(kind: KernelKind) -> &'static str {
    match kind {
        KernelKind::TargetUnivariate => "TargetUnivariate",
        KernelKind::LinearAffine => "LinearAffine",
        KernelKind::TargetRelationSearch => "TargetRelationSearch",
        KernelKind::SparseResultantProjection => "SparseResultantProjection",
        KernelKind::TargetActionKrylov => "TargetActionKrylov",
        KernelKind::NormTraceProjection => "NormTraceProjection",
        KernelKind::RegularChainProjection => "RegularChainProjection",
        KernelKind::SpecializationInterpolation => "SpecializationInterpolation",
        KernelKind::UniversalTargetElimination => "UniversalTargetElimination",
    }
}
