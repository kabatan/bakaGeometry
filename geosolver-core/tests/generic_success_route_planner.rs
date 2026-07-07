use geosolver_core::api::solve_target;
use geosolver_core::kernels::traits::KernelKind;
use geosolver_core::planner::kernel_plan::KernelPlan;
use geosolver_core::planner::relation_schedule::monomial_count_total_degree_leq_saturating;
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
    constant_poly, poly_add, poly_mul, poly_sub, variable_poly, SparsePolynomialQ,
};
use geosolver_core::types::rational::int_q;
use geosolver_core::verify::replay_run_certificate;

fn v(var: VariableId) -> SparsePolynomialQ {
    variable_poly(var)
}

fn c(value: i64) -> SparsePolynomialQ {
    constant_poly(int_q(value))
}

fn options_prioritizing(kind: KernelKind) -> SolverOptions {
    SolverOptions {
        kernel_priority: vec![kind],
        ..SolverOptions::default()
    }
}

fn auxiliary_variables(base: u32, count: usize) -> Vec<VariableId> {
    (0..count)
        .map(|idx| VariableId(base + 100 + idx as u32 * 5))
        .collect()
}

fn sparse_large_local_relation(target: VariableId, aux: &[VariableId]) -> SparsePolynomialQ {
    aux.iter().enumerate().fold(v(target), |acc, (idx, var)| {
        let square = poly_mul(&v(*var), &v(*var));
        poly_add(&acc, &poly_mul(&c((idx as i64 % 7) + 2), &square))
    })
}

fn large_action_problem(base: u32) -> RationalTargetProblem {
    let target = VariableId(base + 1);
    let x = VariableId(base + 3);
    let aux = auxiliary_variables(base, 24);
    let mut variables = vec![target, x];
    variables.extend(aux.iter().copied());
    make_problem(
        variables,
        target,
        vec![
            poly_sub(&poly_mul(&v(x), &v(x)), &c(2)),
            poly_sub(&v(target), &v(x)),
            poly_sub(&sparse_large_local_relation(target, &aux), &c(5)),
        ],
        Vec::new(),
    )
}

fn large_sparse_problem(base: u32) -> RationalTargetProblem {
    let target = VariableId(base + 1);
    let x = VariableId(base + 7);
    let aux = auxiliary_variables(base, 24);
    let x2 = poly_mul(&v(x), &v(x));
    let x3 = poly_mul(&x2, &v(x));
    let mut variables = vec![target, x];
    variables.extend(aux.iter().copied());
    make_problem(
        variables,
        target,
        vec![
            poly_sub(&x2, &v(target)),
            poly_sub(&x3, &c(2)),
            poly_sub(&sparse_large_local_relation(target, &aux), &c(9)),
        ],
        Vec::new(),
    )
}

fn separator_rich_problem(base: u32) -> RationalTargetProblem {
    let target = VariableId(base + 1);
    let u = VariableId(base + 3);
    let w = VariableId(base + 5);
    let x = VariableId(base + 7);
    let y = VariableId(base + 11);
    let x2 = poly_mul(&v(x), &v(x));
    let u2 = poly_mul(&v(u), &v(u));
    let w2 = poly_mul(&v(w), &v(w));
    let y2 = poly_mul(&v(y), &v(y));
    let y3 = poly_mul(&y2, &v(y));
    make_problem(
        vec![w, x, target, y, u],
        target,
        vec![
            poly_sub(&y3, &c(2)),
            poly_sub(&u2, &v(target)),
            poly_sub(&y2, &x2),
            poly_sub(&w2, &u2),
            poly_sub(&x2, &w2),
        ],
        Vec::new(),
    )
}

fn public_universal_later_strategy_problem(base: u32) -> RationalTargetProblem {
    let target = VariableId(base + 1);
    let x = VariableId(base + 3);
    let y = VariableId(base + 5);
    let z = VariableId(base + 7);
    let xy = poly_mul(&v(x), &v(y));
    make_problem(
        vec![target, z, y, x],
        target,
        vec![
            poly_sub(&poly_mul(&v(x), &v(x)), &c(2)),
            poly_sub(&poly_mul(&v(y), &v(y)), &c(3)),
            poly_sub(&poly_mul(&v(z), &v(z)), &c(5)),
            poly_sub(&poly_sub(&poly_sub(&v(target), &xy), &v(x)), &v(z)),
        ],
        Vec::new(),
    )
}

fn options_universal_with_dense_cap() -> SolverOptions {
    SolverOptions {
        max_relation_search_export_degree: Some(0),
        kernel_priority: vec![KernelKind::UniversalTargetElimination],
        ..SolverOptions::default()
    }
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

fn assert_success_route(
    label: &str,
    problem: &RationalTargetProblem,
    result: &TargetSolveResult,
    expected_kernel: Option<KernelKind>,
    required_failed_kernel: Option<KernelKind>,
    expect_dense_cost_prohibited: bool,
) {
    assert_eq!(
        result.status,
        SolverStatus::CertifiedCandidateCover,
        "{label}: diagnostics={:?}",
        result.diagnostics
    );
    if let Some(expected_kernel) = expected_kernel {
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
    }
    if let Some(required_failed_kernel) = required_failed_kernel {
        assert!(
            result.diagnostics.iter().any(|record| {
                record.name == "BlockProjectionFailureTrace"
                    && record.details.get("kernel_kind").map(String::as_str)
                        == Some(kernel_name(required_failed_kernel))
                    && record.details.get("route_event").map(String::as_str)
                        == Some("route_allowed_failure")
            }),
            "{label}: missing allowed failed route trace for {:?}; diagnostics={:?}",
            required_failed_kernel,
            result.diagnostics
        );
    }
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.name == "KernelRouteTrace"),
        "{label}: missing route trace diagnostics"
    );
    assert!(
        !result.cost_trace.block_traces.is_empty(),
        "{label}: missing projection cost trace"
    );
    if expect_dense_cost_prohibited {
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.name == "CostProhibitedDenseRoute"
                    && diagnostic.details.get("decision").map(String::as_str)
                        == Some("CostProhibitedDenseRoute")),
            "{label}: dense route was not cost-prohibited: {:?}",
            result.diagnostics
        );
    }
    let replay = replay_run_certificate(result, problem);
    assert!(replay.accepted, "{label}: replay failed: {replay:?}");
}

fn assert_one_large_block_with_universal_ladder(
    problem: RationalTargetProblem,
    options: SolverOptions,
) {
    let explicit_universal_priority = options
        .kernel_priority
        .contains(&KernelKind::UniversalTargetElimination);
    let (_compressed, _ctx, plans) = planning_artifacts(problem, options);
    assert_eq!(
        plans.len(),
        1,
        "expected one relation-bearing projection block"
    );
    let plan = &plans[0];
    assert!(
        plan.declared_ladder
            .iter()
            .any(|entry| entry.kernel_kind == KernelKind::UniversalTargetElimination),
        "Universal must be present in one-large-block ladder"
    );
    if !explicit_universal_priority {
        assert_eq!(
            plan.declared_ladder.last().map(|entry| entry.kernel_kind),
            Some(KernelKind::UniversalTargetElimination),
            "Universal must be declared last without explicit priority"
        );
    }
}

#[test]
fn gsr_p1_saturating_counts_match_small_values_and_saturate() {
    assert_eq!(
        monomial_count_total_degree_leq_saturating(2, 3).value,
        Some(10)
    );
    assert_eq!(
        monomial_count_total_degree_leq_saturating(3, 2).value,
        Some(10)
    );
    assert!(monomial_count_total_degree_leq_saturating(100, 100).saturated);
}

#[test]
fn gsr_p6_public_success_routes_remain_available_after_dense_decline() {
    let cases = vec![
        (
            "G1 action large footprint",
            large_action_problem(61_000),
            options_prioritizing(KernelKind::TargetActionKrylov),
            Some(KernelKind::TargetActionKrylov),
            None,
            true,
        ),
        (
            "G2 sparse large footprint",
            large_sparse_problem(62_000),
            options_prioritizing(KernelKind::SparseResultantProjection),
            Some(KernelKind::SparseResultantProjection),
            None,
            true,
        ),
        (
            "G3 separator-rich composition",
            separator_rich_problem(63_000),
            SolverOptions::default(),
            None,
            None,
            false,
        ),
        (
            "G4 public Universal after route failures",
            public_universal_later_strategy_problem(64_000),
            options_universal_with_dense_cap(),
            Some(KernelKind::TargetActionKrylov),
            Some(KernelKind::UniversalTargetElimination),
            true,
        ),
        (
            "G5 one-large-block large action footprint",
            large_action_problem(65_000),
            options_prioritizing(KernelKind::TargetActionKrylov),
            Some(KernelKind::TargetActionKrylov),
            None,
            true,
        ),
    ];

    for (
        label,
        problem,
        options,
        expected_kernel,
        required_failed_kernel,
        expect_dense_cost_prohibited,
    ) in cases
    {
        let result = solve_target(problem.clone(), options);
        assert_success_route(
            label,
            &problem,
            &result,
            expected_kernel,
            required_failed_kernel,
            expect_dense_cost_prohibited,
        );
        if label.starts_with("G3") {
            assert!(
                result.projection_messages.len() >= 2,
                "{label}: expected separator-rich multi-message composition"
            );
        }
        if label.starts_with("G4") {
            let options = options_universal_with_dense_cap();
            assert_one_large_block_with_universal_ladder(problem.clone(), options);
        }
        if label.starts_with("G5") {
            assert_one_large_block_with_universal_ladder(problem, SolverOptions::default());
        }
    }
}

#[test]
fn gsr_p4_universal_internal_later_strategy_records_trace() {
    let problem = public_universal_later_strategy_problem(66_000);
    let options = options_universal_with_dense_cap();
    let result = solve_target(problem.clone(), options.clone());
    assert_success_route(
        "public Universal internal later strategy evidence",
        &problem,
        &result,
        Some(KernelKind::TargetActionKrylov),
        Some(KernelKind::UniversalTargetElimination),
        true,
    );
    assert_one_large_block_with_universal_ladder(problem, options);
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
