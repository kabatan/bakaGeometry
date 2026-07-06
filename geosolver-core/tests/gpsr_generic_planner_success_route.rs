use geosolver_core::api::solve_target;
use geosolver_core::kernels::traits::KernelKind;
use geosolver_core::planner::admission::KernelAdmissionStatus;
use geosolver_core::planner::kernel_plan::KernelPlan;
use geosolver_core::planner::relation_schedule::{
    build_dense_relation_search_schedule, estimate_dense_relation_search_schedule,
};
use geosolver_core::preprocess::compression::CompressedSystemQ;
use geosolver_core::problem::context::{new_context, SolverContext};
use geosolver_core::problem::input::{make_problem, RationalTargetProblem};
use geosolver_core::result::output::TargetSolveResult;
use geosolver_core::result::status::SolverStatus;
use geosolver_core::solver::options::SolverOptions;
use geosolver_core::solver::pipeline::{
    step_build_dag, step_build_graphs, step_canonicalize, step_compress, step_plan, step_validate,
};
use geosolver_core::types::ids::VariableId;
use geosolver_core::types::polynomial::{
    constant_poly, poly_add, poly_mul, poly_scale, poly_sub, variable_poly, SparsePolynomialQ,
};
use geosolver_core::types::rational::{div_q, int_q, RationalQ};
use geosolver_core::verify::replay_run_certificate;

fn v(var: VariableId) -> SparsePolynomialQ {
    variable_poly(var)
}

fn c(value: i64) -> SparsePolynomialQ {
    constant_poly(int_q(value))
}

fn q(num: i64, den: i64) -> RationalQ {
    div_q(&int_q(num), &int_q(den)).expect("nonzero rational scale")
}

fn options_prioritizing(kind: KernelKind) -> SolverOptions {
    SolverOptions {
        kernel_priority: vec![kind],
        ..SolverOptions::default()
    }
}

fn large_auxiliary_variables(base: u32, count: usize) -> Vec<VariableId> {
    (0..count)
        .map(|idx| VariableId(base + 100 + idx as u32 * 3))
        .collect()
}

fn large_sparse_footprint(target: VariableId, aux: &[VariableId]) -> SparsePolynomialQ {
    let mut relation = v(target);
    for (idx, var) in aux.iter().enumerate() {
        let square = poly_mul(&v(*var), &v(*var));
        let weight = int_q((idx as i64 % 11) + 2);
        relation = poly_add(&relation, &poly_scale(&square, &weight));
    }
    poly_sub(&relation, &c(5))
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
                let scale = match idx % 6 {
                    0 => q(5, 3),
                    1 => q(-7, 4),
                    2 => q(11, 5),
                    3 => q(-13, 6),
                    4 => q(17, 7),
                    _ => q(-19, 8),
                };
                poly_scale(&relation, &scale)
            })
            .collect(),
        Vec::new(),
    )
}

fn dense_decline_action_problem(base: u32) -> RationalTargetProblem {
    let target = VariableId(base + 1);
    let x = VariableId(base + 5);
    let aux = large_auxiliary_variables(base, 24);
    let x2 = poly_mul(&v(x), &v(x));
    let mut variables = vec![target, x];
    variables.extend(aux.iter().copied());
    scaled_problem(
        variables,
        target,
        vec![
            poly_sub(&x2, &c(2)),
            poly_sub(&v(target), &v(x)),
            large_sparse_footprint(target, &aux),
        ],
    )
}

fn dense_decline_sparse_problem(base: u32) -> RationalTargetProblem {
    let target = VariableId(base + 1);
    let x = VariableId(base + 7);
    let aux = large_auxiliary_variables(base, 24);
    let x2 = poly_mul(&v(x), &v(x));
    let x3 = poly_mul(&x2, &v(x));
    let mut variables = vec![target, x];
    variables.extend(aux.iter().copied());
    scaled_problem(
        variables,
        target,
        vec![
            poly_sub(&x2, &v(target)),
            poly_sub(&x3, &c(2)),
            large_sparse_footprint(target, &aux),
        ],
    )
}

fn planning_artifacts(
    problem: RationalTargetProblem,
    options: SolverOptions,
) -> (CompressedSystemQ, SolverContext, Vec<KernelPlan>) {
    let mut ctx = new_context(options);
    let validated = step_validate(problem, &mut ctx).unwrap();
    let canonical = step_canonicalize(validated, &mut ctx).unwrap();
    let compressed = step_compress(canonical, &mut ctx).unwrap();
    let graphs = step_build_graphs(&compressed, &mut ctx).unwrap();
    let dag = step_build_dag(&graphs, &compressed, &mut ctx).unwrap();
    let plans = step_plan(&dag, &compressed, &mut ctx).unwrap();
    (compressed, ctx, plans)
}

fn dense_route_declined(plan: &KernelPlan) -> bool {
    plan.admissions.iter().any(|admission| {
        admission.kind == KernelKind::TargetRelationSearch
            && matches!(
                &admission.status,
                KernelAdmissionStatus::Declined { reason }
                    if reason.contains("CostProhibitedDenseRoute")
            )
    })
}

fn assert_cost_prohibited_diagnostic(result: &TargetSolveResult) {
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.name == "CostProhibitedDenseRoute")
        .unwrap_or_else(|| panic!("missing dense route diagnostic: {:?}", result.diagnostics));
    assert_eq!(
        diagnostic.details.get("kernel").map(String::as_str),
        Some("TargetRelationSearch")
    );
    assert_eq!(
        diagnostic.details.get("route").map(String::as_str),
        Some("DenseTotalDegree")
    );
    assert_eq!(
        diagnostic.details.get("decision").map(String::as_str),
        Some("CostProhibitedDenseRoute")
    );
    for key in [
        "block_id",
        "stage_count",
        "materialized_stage_cap",
        "matrix_col_cap",
        "matrix_row_cap",
        "memory_cap_bytes",
        "first_export_degree",
        "estimated_matrix_cols",
        "estimated_rows",
        "estimated_memory_bytes",
        "first_prohibited_stage",
    ] {
        assert!(
            diagnostic.details.contains_key(key),
            "missing diagnostic detail {key}: {diagnostic:?}"
        );
    }
}

fn assert_support_success(
    label: &str,
    problem: &RationalTargetProblem,
    result: &TargetSolveResult,
    expected_kernel: KernelKind,
) {
    assert_eq!(
        result.status,
        SolverStatus::CertifiedCandidateCover,
        "{label}: diagnostics={:?}",
        result.diagnostics
    );
    assert!(result.support_polynomial.is_some(), "{label}: no support");
    assert!(
        result.squarefree_support_polynomial.is_some(),
        "{label}: no squarefree support"
    );
    assert!(!result.root_isolation.is_empty(), "{label}: no real roots");
    assert!(result.exact_image_certificate.is_none());
    assert!(
        result
            .projection_messages
            .iter()
            .any(|message| message.kernel_kind == expected_kernel),
        "{label}: expected {expected_kernel:?}, got {:?}",
        result
            .projection_messages
            .iter()
            .map(|message| message.kernel_kind)
            .collect::<Vec<_>>()
    );
    assert_cost_prohibited_diagnostic(result);
    let replay = replay_run_certificate(result, problem);
    assert!(replay.accepted, "{label}: replay failed: {replay:?}");
}

#[test]
fn gpsr_dense_trs_large_footprint_preflight_is_descriptor_only() {
    let problem = dense_decline_action_problem(10_000);
    let relations = problem.equations.clone();
    let target = problem.target;
    let eliminated = problem
        .variables
        .iter()
        .copied()
        .filter(|var| *var != target)
        .collect::<Vec<_>>();
    let exported = vec![target];

    let preflight = estimate_dense_relation_search_schedule(
        &relations,
        &eliminated,
        &exported,
        &SolverOptions::default(),
    );
    assert!(!preflight.materialization_allowed);
    assert!(preflight.first_prohibited_stage.is_some());
    assert!(
        preflight.planned_stage_count.value.unwrap()
            > preflight.caps.max_materialized_stages as u128
    );
    assert!(!preflight.stage_estimates.is_empty());

    let schedule = build_dense_relation_search_schedule(
        &relations,
        &eliminated,
        &exported,
        &SolverOptions::default(),
    );
    assert_eq!(schedule.preflight, preflight);
    assert!(schedule.stages.is_empty());
    assert!(!schedule.support_descriptors.is_empty());
}

#[test]
fn gpsr_admission_isolation_keeps_later_kernels_after_dense_decline() {
    let (_compressed, ctx, plans) = planning_artifacts(
        dense_decline_sparse_problem(20_000),
        SolverOptions::default(),
    );
    let plan = plans
        .iter()
        .find(|plan| dense_route_declined(plan))
        .expect("expected at least one plan with dense TRS cost decline");
    let target_relation_index = plan
        .admissions
        .iter()
        .position(|admission| admission.kind == KernelKind::TargetRelationSearch)
        .unwrap();
    assert!(plan.admissions[target_relation_index + 1..]
        .iter()
        .any(|admission| admission.kind == KernelKind::SparseResultantProjection));
    assert!(plan.admissions[target_relation_index + 1..]
        .iter()
        .any(|admission| admission.kind == KernelKind::UniversalTargetElimination));
    assert!(!plan
        .declared_ladder
        .iter()
        .any(|entry| entry.kernel_kind == KernelKind::TargetRelationSearch));
    assert!(plan
        .declared_ladder
        .iter()
        .any(|entry| entry.kernel_kind == KernelKind::SparseResultantProjection));
    assert!(plan
        .declared_ladder
        .iter()
        .any(|entry| entry.kernel_kind == KernelKind::UniversalTargetElimination));
    assert!(ctx
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.name == "CostProhibitedDenseRoute"
            && diagnostic.details.get("decision").map(String::as_str)
                == Some("CostProhibitedDenseRoute")));
}

#[test]
fn gpsr_large_footprint_action_route_still_produces_candidate_cover() {
    let problem = dense_decline_action_problem(30_000);
    let result = solve_target(
        problem.clone(),
        options_prioritizing(KernelKind::TargetActionKrylov),
    );
    assert_support_success(
        "large footprint action route",
        &problem,
        &result,
        KernelKind::TargetActionKrylov,
    );
}

#[test]
fn gpsr_large_footprint_sparse_route_still_produces_candidate_cover() {
    let problem = dense_decline_sparse_problem(40_000);
    let result = solve_target(
        problem.clone(),
        options_prioritizing(KernelKind::SparseResultantProjection),
    );
    assert_support_success(
        "large footprint sparse route",
        &problem,
        &result,
        KernelKind::SparseResultantProjection,
    );
}

#[test]
fn gpsr_universal_ladder_survives_internal_dense_decline() {
    let problem = dense_decline_sparse_problem(50_000);
    let result = solve_target(
        problem.clone(),
        options_prioritizing(KernelKind::UniversalTargetElimination),
    );
    assert_support_success(
        "universal ladder after dense decline",
        &problem,
        &result,
        KernelKind::UniversalTargetElimination,
    );
}
