use geosolver_core::api::solve_target;
use geosolver_core::compose::message::{hash_projection_message, ProjectionMessage};
use geosolver_core::kernels::traits::{KernelContext, KernelKind};
use geosolver_core::planner::kernel_plan::{KernelPlan, UniversalStrategy};
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
use geosolver_core::types::hash::hash_sequence;
use geosolver_core::types::ids::VariableId;
use geosolver_core::types::polynomial::{
    constant_poly, poly_add, poly_mul, poly_sub, variable_poly, SparsePolynomialQ,
};
use geosolver_core::types::rational::int_q;
use geosolver_core::verify::certificates::{
    kernel_certificate_binding_hash, KernelCertificatePayload,
};
use geosolver_core::verify::replay_run_certificate;
use geosolver_core::verify::verify_message::verify_projection_message;

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
    let x4 = poly_mul(&x2, &x2);
    let x6 = poly_mul(&x4, &x2);
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
            poly_sub(&x6, &c(4)),
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
    expected_kernel: KernelKind,
    expect_dense_cost_prohibited: bool,
) {
    assert_eq!(
        result.status,
        SolverStatus::CertifiedCandidateCover,
        "{label}: diagnostics={:?}",
        result.diagnostics
    );
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

fn assert_universal_later_strategy_trace(label: &str, message: &ProjectionMessage) {
    assert_eq!(message.kernel_kind, KernelKind::UniversalTargetElimination);
    let KernelCertificatePayload::Universal(proof) = &message.certificate.payload else {
        panic!("{label}: Universal message has non-Universal payload");
    };
    assert_ne!(
        proof.chosen_strategy,
        UniversalStrategy::TargetRelationSearchEscalated,
        "{label}: expected a later strategy after dense TRS decline"
    );
    assert!(
        !proof.failed_strategy_hashes.is_empty(),
        "{label}: expected failed internal strategy trace"
    );
}

fn assert_rehashed_universal_failed_strategy_tamper_is_rejected(
    problem: &RationalTargetProblem,
    options: SolverOptions,
    result: &TargetSolveResult,
) {
    let mut tampered = result.clone();
    {
        let message = tampered
            .projection_messages
            .iter_mut()
            .find(|message| message.kernel_kind == KernelKind::UniversalTargetElimination)
            .expect("expected Universal message to tamper");
        let KernelCertificatePayload::Universal(proof) = &mut message.certificate.payload else {
            panic!("Universal message has non-Universal payload");
        };
        assert!(
            !proof.failed_strategy_hashes.is_empty(),
            "tamper test requires a failed strategy prefix"
        );
        proof.failed_strategy_hashes[0] = hash_sequence("tampered-universal-failed-strategy", &[]);

        let stage_certificate_hash = proof.stage_certificate_hash;
        let mut chunks = vec![stage_certificate_hash.0.to_vec()];
        for relation in &message.relation_generators {
            chunks.push(relation.hash.0.to_vec());
        }
        chunks.push(format!("{:?}", message.certificate.payload).into_bytes());
        message.certificate.certificate_hash =
            hash_sequence("universal-elimination-certificate", &chunks);
        message.certificate.binding_hash = kernel_certificate_binding_hash(&message.certificate);
        message.package_hash = hash_projection_message(message);
    }

    let message = tampered
        .projection_messages
        .iter()
        .find(|message| message.kernel_kind == KernelKind::UniversalTargetElimination)
        .expect("expected Universal message to tamper");
    let kctx = kernel_context_for_message(problem.clone(), options, &tampered, message);
    assert!(
        verify_projection_message(message, &kctx).is_err(),
        "direct Universal failed-strategy tamper verification must be rejected"
    );
    let replay = replay_run_certificate(&tampered, problem);
    assert!(
        !replay.accepted,
        "rehashed Universal failed-strategy tamper must be rejected"
    );
}

fn kernel_context_for_message(
    problem: RationalTargetProblem,
    options: SolverOptions,
    result: &TargetSolveResult,
    message: &ProjectionMessage,
) -> KernelContext {
    let mut ctx = new_context(options);
    let validated = step_validate(problem, &mut ctx).unwrap();
    let canonical = step_canonicalize(validated, &mut ctx).unwrap();
    let compressed = step_compress(canonical, &mut ctx).unwrap();
    let graphs = step_build_graphs(&compressed, &mut ctx).unwrap();
    let dag = step_build_dag(&graphs, &compressed, &mut ctx).unwrap();
    let block = dag
        .blocks
        .into_iter()
        .find(|block| block.block_id == message.block_id)
        .expect("message block must exist in rebuilt DAG");
    let child_messages = result
        .projection_messages
        .iter()
        .filter(|candidate| block.child_block_ids.contains(&candidate.block_id))
        .cloned()
        .collect();
    KernelContext {
        block,
        system: compressed,
        child_messages,
    }
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
    assert!(plan.admissions.iter().any(|admission| {
        admission.kind == KernelKind::TargetRelationSearch
            && admission.execution_plan.as_ref().is_some_and(|plan| {
                plan.support_plan.dense_relation_search_schedule.is_none()
                    && plan.support_plan.sparse_relation_search_schedule.is_some()
            })
    }));
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
            KernelKind::TargetActionKrylov,
            true,
        ),
        (
            "G2 sparse large footprint",
            large_sparse_problem(62_000),
            options_prioritizing(KernelKind::SparseResultantProjection),
            KernelKind::SparseResultantProjection,
            true,
        ),
        (
            "G3 separator-rich composition",
            separator_rich_problem(63_000),
            options_prioritizing(KernelKind::TargetRelationSearch),
            KernelKind::TargetRelationSearch,
            false,
        ),
        (
            "G4 public Universal after route failures",
            public_universal_later_strategy_problem(64_000),
            options_universal_with_dense_cap(),
            KernelKind::UniversalTargetElimination,
            true,
        ),
        (
            "G5 one-large-block large action footprint",
            large_action_problem(65_000),
            options_prioritizing(KernelKind::TargetActionKrylov),
            KernelKind::TargetActionKrylov,
            true,
        ),
    ];

    for (label, problem, options, expected_kernel, expect_dense_cost_prohibited) in cases {
        let result = solve_target(problem.clone(), options);
        assert_success_route(
            label,
            &problem,
            &result,
            expected_kernel,
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
            let universal_message = result
                .projection_messages
                .iter()
                .find(|message| message.kernel_kind == KernelKind::UniversalTargetElimination)
                .expect("G4 public result must execute Universal");
            assert_universal_later_strategy_trace(label, universal_message);
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
        KernelKind::UniversalTargetElimination,
        true,
    );
    let universal_message = result
        .projection_messages
        .iter()
        .find(|message| message.kernel_kind == KernelKind::UniversalTargetElimination)
        .expect("public result must execute Universal");
    assert_universal_later_strategy_trace("public Universal internal strategy", universal_message);
    assert_rehashed_universal_failed_strategy_tamper_is_rejected(
        &problem,
        options.clone(),
        &result,
    );
    assert_one_large_block_with_universal_ladder(problem, options);
}
