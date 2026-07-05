use geosolver_core::api::solve_target;
use geosolver_core::problem::input::make_problem;
use geosolver_core::result::status::SolverStatus;
use geosolver_core::solver::options::SolverOptions;
use geosolver_core::types::ids::VariableId;
use geosolver_core::types::interval::interval_contains_q;
use geosolver_core::types::polynomial::{
    constant_poly, poly_add, poly_mul, poly_scale, poly_sub, variable_poly,
};
use geosolver_core::types::rational::int_q;
use geosolver_core::verify::replay_run_certificate;

#[test]
fn p3_public_api_runs_candidate_cover_pipeline_for_target_only_case() {
    let target = VariableId(0);
    let t = variable_poly(target);
    let support_relation = poly_add(
        &poly_sub(&poly_mul(&t, &t), &poly_scale(&t, &int_q(4))),
        &constant_poly(int_q(3)),
    );
    let problem = make_problem(vec![target], target, vec![support_relation], Vec::new());

    let result = solve_target(problem.clone(), SolverOptions::default());

    assert_eq!(result.status, SolverStatus::CertifiedCandidateCover);
    assert_eq!(result.target, target);
    assert!(result.support_polynomial.is_some());
    assert!(result.squarefree_support_polynomial.is_some());
    assert_eq!(result.projection_messages.len(), 1);
    assert_eq!(result.root_isolation.len(), 2);
    assert_eq!(result.decoded_candidates.len(), 2);
    assert!(result.certificate.is_some());

    let cert = result.certificate.as_ref().unwrap();
    assert_eq!(cert.target_variable, target);
    assert_eq!(
        cert.projection_message_hashes.len(),
        result.projection_messages.len()
    );
    assert!(cert.global_support_certificate_hash.is_some());
    assert!(cert.final_dag_replay_evidence_hash.is_some());
    assert!(cert.final_dag_replay_evidence.is_some());
    assert!(replay_run_certificate(&result, &problem).accepted);
    assert_eq!(
        cert.global_support_hash,
        result
            .support_polynomial
            .as_ref()
            .map(|support| support.hash)
    );

    assert_eq!(result.cost_trace.block_traces.len(), 1);
    assert!(result.cost_trace.verification_trace.checked_relation_count > 0);
    assert!(interval_contains_q(
        &result.root_isolation[0].isolating_interval,
        &int_q(1)
    ));
    assert!(interval_contains_q(
        &result.root_isolation[1].isolating_interval,
        &int_q(3)
    ));
}

#[test]
fn fcr_p9_public_api_returns_empty_candidate_cover_for_support_with_no_real_roots() {
    let target = VariableId(0);
    let t = variable_poly(target);
    let support_relation = poly_add(&poly_mul(&t, &t), &constant_poly(int_q(1)));
    let problem = make_problem(vec![target], target, vec![support_relation], Vec::new());

    let result = solve_target(problem.clone(), SolverOptions::default());

    assert_eq!(result.status, SolverStatus::CertifiedCandidateCover);
    assert!(result.support_polynomial.is_some());
    assert!(result.squarefree_support_polynomial.is_some());
    assert!(result.root_isolation.is_empty());
    assert!(result.decoded_candidates.is_empty());
    assert!(result.certificate.is_some());
    assert!(result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.name == "EmptyRealCandidateCover"));
    assert!(replay_run_certificate(&result, &problem).accepted);
}
