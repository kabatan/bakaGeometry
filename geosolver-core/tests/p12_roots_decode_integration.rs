use geosolver_core::compose::finalize_candidate_cover_result;
use geosolver_core::result::cost_trace::GlobalCostTrace;
use geosolver_core::result::status::SolverStatus;
use geosolver_core::roots::decode::hash_target_candidate;
use geosolver_core::solver::options::RootIsolationMethod;
use geosolver_core::types::hash::{hash_sequence, Hash};
use geosolver_core::types::ids::VariableId;
use geosolver_core::types::interval::interval_contains_q;
use geosolver_core::types::rational::int_q;
use geosolver_core::types::univariate::{normalize_univariate, UniPolynomialQ};

fn support_poly(t: VariableId) -> UniPolynomialQ {
    normalize_univariate(UniPolynomialQ {
        variable: t,
        coeffs_low_to_high: vec![int_q(3), int_q(-4), int_q(1)],
        hash: hash_sequence("univariate", &[]),
    })
}

#[test]
fn p12_candidate_cover_finalizer_returns_exact_roots_and_nonplaceholder_candidates() {
    let t = VariableId(0);
    let result = finalize_candidate_cover_result(
        t,
        support_poly(t),
        Vec::new(),
        GlobalCostTrace::default(),
        RootIsolationMethod::Sturm,
    )
    .unwrap();

    assert_eq!(result.status, SolverStatus::CertifiedCandidateCover);
    assert!(result.support_polynomial.is_some());
    let squarefree = result.squarefree_support_polynomial.as_ref().unwrap();
    assert_eq!(result.root_isolation.len(), 2);
    assert_eq!(result.decoded_candidates.len(), 2);
    assert!(interval_contains_q(
        &result.root_isolation[0].isolating_interval,
        &int_q(1)
    ));
    assert!(interval_contains_q(
        &result.root_isolation[1].isolating_interval,
        &int_q(3)
    ));

    for (root, candidate) in result
        .root_isolation
        .iter()
        .zip(result.decoded_candidates.iter())
    {
        assert_eq!(root.support_hash, squarefree.hash);
        assert_eq!(candidate.target, t);
        assert_eq!(candidate.support_hash, squarefree.hash);
        assert_eq!(candidate.root_index, root.root_index);
        assert_eq!(candidate.isolating_interval, root.isolating_interval);
        assert_ne!(candidate.candidate_hash, Hash([0; 32]));
        assert_eq!(
            candidate.candidate_hash,
            hash_target_candidate(
                t,
                squarefree.hash,
                root.root_index,
                &root.isolating_interval
            )
        );
    }
}
