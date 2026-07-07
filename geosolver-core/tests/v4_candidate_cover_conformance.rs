use geosolver_core::api::solve_target;
use geosolver_core::problem::input::{
    make_problem, make_problem_with_roles, RationalTargetProblem, VariableRoleRecord,
};
use geosolver_core::result::status::SolverStatus;
use geosolver_core::roots::decode::hash_target_candidate;
use geosolver_core::solver::options::{RootIsolationMethod, SolverOptions};
use geosolver_core::types::hash::Hash;
use geosolver_core::types::ids::VariableId;
use geosolver_core::types::monomial::normalize_monomial;
use geosolver_core::types::polynomial::{
    clear_denominators_primitive, constant_poly, normalize_poly, poly_mul, poly_scale, poly_sub,
    variable_poly, SparsePolynomialQ, TermQ,
};
use geosolver_core::types::rational::{div_q, int_q};
use geosolver_core::verify::replay_run_certificate;

fn v(id: u32) -> SparsePolynomialQ {
    variable_poly(VariableId(id))
}

fn c(value: i64) -> SparsePolynomialQ {
    constant_poly(int_q(value))
}

fn relation_scaled_problem(
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

fn simple_multivariate_problem(target: VariableId, x: VariableId) -> RationalTargetProblem {
    relation_scaled_problem(
        vec![x, target],
        target,
        vec![
            poly_sub(&poly_mul(&v(x.0), &v(x.0)), &c(1)),
            poly_sub(&v(target.0), &v(x.0)),
        ],
    )
}

fn assert_candidate_cover(
    label: &str,
    problem: &RationalTargetProblem,
    result: &geosolver_core::result::output::TargetSolveResult,
) {
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
    assert_eq!(
        result.root_isolation.len(),
        result.decoded_candidates.len(),
        "{label}: root/candidate count drift"
    );
    assert!(
        !result.projection_messages.is_empty(),
        "{label}: missing messages"
    );
    assert!(result.certificate.is_some(), "{label}: missing certificate");
    assert!(
        replay_run_certificate(result, problem).accepted,
        "{label}: replay failed"
    );
    assert!(result.cost_trace.final_support_degree.is_some());
    assert!(result.cost_trace.certificate_size.is_some());
}

#[test]
fn p18_public_api_ignores_roles_names_and_relation_order_for_mechanism() {
    let target = VariableId(1001);
    let x = VariableId(1009);
    let base = simple_multivariate_problem(target, x);
    let base_result = solve_target(base.clone(), SolverOptions::default());
    assert_candidate_cover("base", &base, &base_result);

    let role_decorated = make_problem_with_roles(
        vec![x, target],
        target,
        base.equations.clone(),
        Vec::new(),
        vec![
            VariableRoleRecord {
                variable: target,
                role_name: "expected_answer".to_owned(),
            },
            VariableRoleRecord {
                variable: x,
                role_name: "circle_center_coordinate".to_owned(),
            },
        ],
    );
    let role_result = solve_target(role_decorated.clone(), SolverOptions::default());
    assert_candidate_cover("role decorated", &role_decorated, &role_result);

    let relation_permuted = make_problem(
        vec![target, x],
        target,
        vec![base.equations[1].clone(), base.equations[0].clone()],
        Vec::new(),
    );
    let permuted_result = solve_target(relation_permuted.clone(), SolverOptions::default());
    assert_candidate_cover("relation permuted", &relation_permuted, &permuted_result);

    assert_eq!(
        base_result.cost_trace.final_support_degree,
        role_result.cost_trace.final_support_degree
    );
    assert_eq!(
        base_result.cost_trace.final_support_degree,
        permuted_result.cost_trace.final_support_degree
    );
    assert_eq!(
        base_result.decoded_candidates.len(),
        role_result.decoded_candidates.len()
    );
    assert_eq!(
        base_result.decoded_candidates.len(),
        permuted_result.decoded_candidates.len()
    );
}

#[test]
fn p18_descartes_option_decodes_hash_bound_candidates() {
    let target = VariableId(1103);
    let x = VariableId(1117);
    let problem = simple_multivariate_problem(target, x);
    let options = SolverOptions {
        root_isolation_method: RootIsolationMethod::Descartes,
        ..SolverOptions::default()
    };
    let result = solve_target(problem.clone(), options);

    assert_candidate_cover("descartes", &problem, &result);
    let squarefree_hash = result.squarefree_support_polynomial.as_ref().unwrap().hash;
    assert_eq!(result.decoded_candidates.len(), 2);
    for (root, candidate) in result.root_isolation.iter().zip(&result.decoded_candidates) {
        assert_eq!(candidate.target, target);
        assert_eq!(candidate.support_hash, squarefree_hash);
        assert_eq!(candidate.root_index, root.root_index);
        assert_eq!(candidate.isolating_interval, root.isolating_interval);
        assert_eq!(
            candidate.candidate_hash,
            hash_target_candidate(
                target,
                squarefree_hash,
                root.root_index,
                &root.isolating_interval
            )
        );
    }
    assert_eq!(
        result
            .certificate
            .as_ref()
            .map(|cert| cert.solver_options.root_isolation_method.clone()),
        Some(RootIsolationMethod::Descartes)
    );
}

#[test]
fn p18_exact_image_request_is_explicit_scope_guard_not_success() {
    let target = VariableId(1201);
    let x = VariableId(1213);
    let problem = simple_multivariate_problem(target, x);
    let result = solve_target(
        problem.clone(),
        SolverOptions {
            exact_image_mode: true,
            ..SolverOptions::default()
        },
    );

    assert_eq!(result.status, SolverStatus::CertificateDesignGap);
    assert!(result.support_polynomial.is_some());
    assert_eq!(result.decoded_candidates.len(), 2);
    assert!(result
        .certificate
        .as_ref()
        .is_some_and(|cert| cert.exact_image_certificate_hash.is_none()));
    assert!(result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.name == "ExactImageOutOfScope"));
    assert!(replay_run_certificate(&result, &problem).accepted);
}

#[test]
fn p18_normalization_and_hash_binding_are_order_independent() {
    let x = VariableId(1301);
    let y = VariableId(1303);
    let p = normalize_poly(SparsePolynomialQ {
        terms: vec![
            TermQ {
                coeff: int_q(2),
                monomial: normalize_monomial(vec![(x, 1), (y, 1)]),
            },
            TermQ {
                coeff: int_q(3),
                monomial: normalize_monomial(vec![(y, 1), (x, 1)]),
            },
            TermQ {
                coeff: int_q(-5),
                monomial: normalize_monomial(vec![(x, 1), (y, 1)]),
            },
        ],
        hash: Hash([7; 32]),
    });
    assert!(p.terms.is_empty(), "duplicate terms should cancel exactly");

    let half = div_q(&int_q(1), &int_q(2)).unwrap();
    let third = div_q(&int_q(1), &int_q(3)).unwrap();
    let rational_poly = normalize_poly(SparsePolynomialQ {
        terms: vec![
            TermQ {
                coeff: half,
                monomial: normalize_monomial(vec![(x, 1)]),
            },
            TermQ {
                coeff: third,
                monomial: normalize_monomial(Vec::new()),
            },
        ],
        hash: Hash([9; 32]),
    });
    let primitive = clear_denominators_primitive(&rational_poly);
    let expected = normalize_poly(SparsePolynomialQ {
        terms: vec![
            TermQ {
                coeff: int_q(3),
                monomial: normalize_monomial(vec![(x, 1)]),
            },
            TermQ {
                coeff: int_q(2),
                monomial: normalize_monomial(Vec::new()),
            },
        ],
        hash: Hash([0; 32]),
    });
    assert_eq!(primitive, expected);
}

#[test]
fn p18_bounded_failure_returns_evidence_cost_trace_not_unsupported() {
    let target = VariableId(1409);
    let x = VariableId(1423);
    let options = SolverOptions {
        max_relation_search_export_degree: Some(2),
        max_matrix_rows: Some(1),
        max_matrix_cols: Some(1),
        ..SolverOptions::default()
    };
    let problem = relation_scaled_problem(
        vec![x, target],
        target,
        vec![poly_sub(&poly_mul(&v(x.0), &v(x.0)), &v(target.0))],
    );

    let result = solve_target(problem, options);

    assert!(matches!(
        result.status,
        SolverStatus::FiniteResourceFailure
            | SolverStatus::AlgorithmicHardCase
            | SolverStatus::CertificateDesignGap
    ));
    assert!(result.support_polynomial.is_none());
    assert!(result.cost_trace.total_variable_count > 0);
    assert!(result.cost_trace.total_relation_count > 0);
    assert!(result.cost_trace.total_monomial_count > 0);
    assert!(result
        .cost_trace
        .block_traces
        .iter()
        .any(|trace| trace.matrix_rows.is_some() || trace.matrix_cols.is_some()));
}
