use geosolver_core::api::solve_target;
use geosolver_core::kernels::traits::KernelKind;
use geosolver_core::problem::input::{make_problem, RationalTargetProblem};
use geosolver_core::result::status::SolverStatus;
use geosolver_core::solver::options::SolverOptions;
use geosolver_core::types::hash::hash_sequence;
use geosolver_core::types::ids::VariableId;
use geosolver_core::types::polynomial::{
    constant_poly, poly_mul, poly_scale, poly_sub, variable_poly, SparsePolynomialQ,
};
use geosolver_core::types::rational::{div_q, int_q, RationalQ};
use geosolver_core::verify::replay_run_certificate;

// Finite candidate-cover scope keeps nonfinite as a guarded replayable status,
// without exposing an extra public nonfinite certificate object on TargetSolveResult.

fn v(id: u32) -> SparsePolynomialQ {
    variable_poly(VariableId(id))
}

fn c(value: i64) -> SparsePolynomialQ {
    constant_poly(int_q(value))
}

fn problem(
    variables: Vec<VariableId>,
    target: VariableId,
    relations: Vec<SparsePolynomialQ>,
) -> RationalTargetProblem {
    let scaled = relations
        .into_iter()
        .enumerate()
        .map(|(idx, relation)| poly_scale(&relation, &scale_factor(idx)))
        .collect::<Vec<_>>();
    make_problem(variables, target, scaled, Vec::new())
}

fn scale_factor(idx: usize) -> RationalQ {
    let factors = [q(2, 3), q(-5, 2), q(7, 4), q(-11, 5)];
    factors[idx % factors.len()].clone()
}

fn q(num: i64, den: i64) -> RationalQ {
    div_q(&int_q(num), &int_q(den)).expect("nonzero scale denominator")
}

#[test]
fn fcr_final_nonfinite_public_certified_nonfinite_requires_positive_proof() {
    let t = VariableId(53);
    let x = VariableId(35);
    let nonfinite_problem = problem(vec![x, t], t, vec![poly_sub(&v(x.0), &c(1))]);
    let result = solve_target(nonfinite_problem.clone(), SolverOptions::default());
    assert_eq!(result.status, SolverStatus::CertifiedNonFiniteTargetImage);
    assert!(result.support_polynomial.is_none());
    assert!(result.certificate.is_none());
    let replay = replay_run_certificate(&result, &nonfinite_problem);
    assert!(replay.accepted);
    assert_eq!(
        replay.replay_hash,
        hash_sequence(
            "core-run-replay",
            &[
                nonfinite_problem.input_hash.0.to_vec(),
                result.target.0.to_be_bytes().to_vec(),
                vec![0xfe],
                vec![1],
            ],
        )
    );

    let mut tampered = result.clone();
    tampered.target = VariableId(97);
    assert!(!replay_run_certificate(&tampered, &nonfinite_problem).accepted);

    let finite_t = VariableId(61);
    let finite_problem = problem(
        vec![finite_t],
        finite_t,
        vec![poly_sub(&poly_mul(&v(finite_t.0), &v(finite_t.0)), &c(1))],
    );
    let mut finite_result = solve_target(finite_problem.clone(), SolverOptions::default());
    assert_ne!(
        finite_result.status,
        SolverStatus::CertifiedNonFiniteTargetImage
    );
    assert!(replay_run_certificate(&finite_result, &finite_problem).accepted);
    finite_result.status = SolverStatus::CertifiedNonFiniteTargetImage;
    assert!(!replay_run_certificate(&finite_result, &finite_problem).accepted);
}

#[test]
fn fcr_final_nonfinite_bounded_search_failure_is_not_nonfinite() {
    let t = VariableId(59);
    let x = VariableId(39);
    let mut options = SolverOptions {
        kernel_priority: vec![KernelKind::TargetRelationSearch],
        ..SolverOptions::default()
    };
    options.max_relation_search_export_degree = Some(2);
    options.max_matrix_rows = Some(1);
    options.max_matrix_cols = Some(1);
    let result = solve_target(
        problem(
            vec![x, t],
            t,
            vec![poly_sub(&poly_mul(&v(x.0), &v(x.0)), &v(t.0))],
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
        "bounded failure must retain cost trace evidence: {:?}",
        result.cost_trace
    );
}
