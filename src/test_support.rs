use std::collections::BTreeSet;

use crate::candidates::CandidateOrigin;

pub(crate) struct TestRouteForcing {
    pub enabled_origins: BTreeSet<CandidateOrigin>,
    pub allow_complete_fallback: bool,
}

impl TestRouteForcing {
    pub(crate) fn only(origin: CandidateOrigin) -> Self {
        Self {
            enabled_origins: BTreeSet::from([origin]),
            allow_complete_fallback: false,
        }
    }
}

pub(crate) fn test_only_route_control_enabled() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use num_bigint::BigInt;
    use num_rational::BigRational;

    use super::*;
    use crate::{
        solver::{collect_candidates_for_test, solve_target_for_test},
        ExactImageMode, GuardRecord, Monomial, PolynomialQ, Rational, ResourceLimits,
        SolverOptions, SolverStatus, TargetProblemQ, Variable,
    };

    fn variable(symbol: &str) -> Variable {
        Variable {
            symbol: symbol.to_string(),
        }
    }

    fn rational(value: i64) -> Rational {
        BigRational::from_integer(BigInt::from(value))
    }

    fn monomial(exponents: &[u32]) -> Monomial {
        Monomial {
            exponents: exponents.to_vec(),
        }
    }

    fn term(variables: &[Variable], coefficient: i64, exponents: &[u32]) -> PolynomialQ {
        PolynomialQ::from_term(
            variables.to_vec(),
            rational(coefficient),
            monomial(exponents),
        )
    }

    fn polynomial(variables: &[Variable], terms: &[(i64, Vec<u32>)]) -> PolynomialQ {
        terms.iter().fold(
            PolynomialQ::zero(variables.to_vec()),
            |accumulator, entry| accumulator.add(&term(variables, entry.0, &entry.1)),
        )
    }

    fn problem(
        equations: Vec<PolynomialQ>,
        variables: Vec<Variable>,
        target: Variable,
    ) -> TargetProblemQ {
        TargetProblemQ {
            equations,
            variables,
            target,
            semantic_guards: Vec::<GuardRecord>::new(),
        }
    }

    fn limits() -> ResourceLimits {
        SolverOptions {
            resource_limits: ResourceLimits {
                max_window_degree: Some(2),
                max_proof_weight: Some(2),
                max_matrix_rows: None,
                max_matrix_cols: None,
                max_candidate_count: None,
            },
            exact_image_mode: ExactImageMode::CoverOnly,
        }
        .resource_limits
    }

    fn solver_options() -> SolverOptions {
        SolverOptions {
            resource_limits: limits(),
            exact_image_mode: ExactImageMode::CoverOnly,
        }
    }

    fn route_result(
        equations: Vec<PolynomialQ>,
        variables: Vec<Variable>,
        target: Variable,
        origin: CandidateOrigin,
    ) -> crate::TargetSolveResult {
        solve_target_for_test(
            problem(equations, variables, target),
            solver_options(),
            &TestRouteForcing::only(origin),
        )
    }

    fn assert_route_only_cover(result: crate::TargetSolveResult, origin: CandidateOrigin) {
        assert_eq!(result.status, SolverStatus::CertifiedCandidateCover);
        assert!(result.cover.is_some());
        assert!(result.certificate.is_some());
        assert!(result
            .trace
            .events
            .iter()
            .any(|event| event.contains(&format!("candidate:{origin:?}"))));
        assert!(result
            .trace
            .events
            .iter()
            .any(|event| event.contains(&format!("proof_try:{origin:?}"))));
        assert!(!result
            .trace
            .events
            .iter()
            .any(|event| event.starts_with("target_elimination:")));
    }

    #[test]
    fn direct_route_forcing_selects_only_direct_candidates() {
        let t = variable("T");
        let x = variable("X");
        let variables = vec![t.clone(), x.clone()];
        let equations = vec![
            polynomial(&variables, &[(1, vec![2, 0]), (-2, vec![0, 0])]),
            polynomial(&variables, &[(1, vec![0, 2]), (-3, vec![0, 0])]),
        ];

        let candidates = collect_candidates_for_test(
            problem(equations, variables, t),
            &limits(),
            &TestRouteForcing::only(CandidateOrigin::DirectTargetEquation),
        );

        assert!(!candidates.is_empty());
        assert!(candidates
            .iter()
            .all(|candidate| candidate.origin == CandidateOrigin::DirectTargetEquation));
    }

    #[test]
    fn direct_route_forcing_solves_without_other_routes_or_complete_fallback() {
        let t = variable("T");
        let x = variable("X");
        let variables = vec![t.clone(), x.clone()];
        let equations = vec![
            polynomial(&variables, &[(1, vec![2, 0]), (-2, vec![0, 0])]),
            polynomial(&variables, &[(1, vec![0, 2]), (-3, vec![0, 0])]),
        ];

        let result = route_result(
            equations,
            variables,
            t,
            CandidateOrigin::DirectTargetEquation,
        );

        assert_route_only_cover(result, CandidateOrigin::DirectTargetEquation);
    }

    #[test]
    fn residual_route_forcing_selects_only_residual_candidates() {
        let x = variable("X");
        let t = variable("T");
        let variables = vec![x.clone(), t.clone()];
        let equations = vec![
            polynomial(&variables, &[(1, vec![2, 0]), (-2, vec![0, 0])]),
            polynomial(&variables, &[(-1, vec![1, 0]), (1, vec![0, 1])]),
        ];

        let candidates = collect_candidates_for_test(
            problem(equations, variables, t),
            &limits(),
            &TestRouteForcing::only(CandidateOrigin::ResidualCyclic),
        );

        assert!(!candidates.is_empty());
        assert!(candidates
            .iter()
            .all(|candidate| candidate.origin == CandidateOrigin::ResidualCyclic));
    }

    #[test]
    fn residual_route_forcing_solves_without_other_routes_or_complete_fallback() {
        let x = variable("X");
        let t = variable("T");
        let variables = vec![x.clone(), t.clone()];
        let equations = vec![
            polynomial(&variables, &[(1, vec![2, 0]), (-2, vec![0, 0])]),
            polynomial(&variables, &[(-1, vec![1, 0]), (1, vec![0, 1])]),
        ];

        let result = route_result(equations, variables, t, CandidateOrigin::ResidualCyclic);

        assert_route_only_cover(result, CandidateOrigin::ResidualCyclic);
    }

    #[test]
    fn tower_route_forcing_selects_only_tower_candidates() {
        let y = variable("Y");
        let x = variable("X");
        let t = variable("T");
        let variables = vec![y.clone(), x.clone(), t.clone()];
        let equations = vec![
            polynomial(&variables, &[(1, vec![2, 0, 0]), (-2, vec![0, 0, 0])]),
            polynomial(&variables, &[(1, vec![0, 2, 0]), (-1, vec![1, 0, 0])]),
            polynomial(&variables, &[(1, vec![0, 0, 1]), (-1, vec![0, 1, 0])]),
        ];

        let candidates = collect_candidates_for_test(
            problem(equations, variables, t),
            &limits(),
            &TestRouteForcing::only(CandidateOrigin::NormTraceTower),
        );

        assert!(!candidates.is_empty());
        assert!(candidates
            .iter()
            .all(|candidate| candidate.origin == CandidateOrigin::NormTraceTower));
    }

    #[test]
    fn tower_route_forcing_solves_without_other_routes_or_complete_fallback() {
        let y = variable("Y");
        let x = variable("X");
        let t = variable("T");
        let variables = vec![y.clone(), x.clone(), t.clone()];
        let equations = vec![
            polynomial(&variables, &[(1, vec![2, 0, 0]), (-2, vec![0, 0, 0])]),
            polynomial(&variables, &[(1, vec![0, 2, 0]), (-1, vec![1, 0, 0])]),
            polynomial(&variables, &[(1, vec![0, 0, 1]), (-1, vec![0, 1, 0])]),
        ];

        let result = route_result(equations, variables, t, CandidateOrigin::NormTraceTower);

        assert_route_only_cover(result, CandidateOrigin::NormTraceTower);
    }

    #[test]
    fn krylov_route_forcing_selects_only_krylov_candidates() {
        let x = variable("X");
        let t = variable("T");
        let variables = vec![x.clone(), t.clone()];
        let equations = vec![
            polynomial(&variables, &[(1, vec![2, 0]), (-2, vec![0, 0])]),
            polynomial(&variables, &[(-1, vec![1, 0]), (1, vec![0, 1])]),
        ];

        let candidates = collect_candidates_for_test(
            problem(equations, variables, t),
            &limits(),
            &TestRouteForcing::only(CandidateOrigin::TargetCyclicKrylov),
        );

        assert!(!candidates.is_empty());
        assert!(candidates
            .iter()
            .all(|candidate| candidate.origin == CandidateOrigin::TargetCyclicKrylov));
    }

    #[test]
    fn krylov_route_forcing_solves_without_other_routes_or_complete_fallback() {
        let x = variable("X");
        let t = variable("T");
        let variables = vec![x.clone(), t.clone()];
        let equations = vec![
            polynomial(&variables, &[(1, vec![2, 0]), (-2, vec![0, 0])]),
            polynomial(&variables, &[(-1, vec![1, 0]), (1, vec![0, 1])]),
        ];

        let result = route_result(equations, variables, t, CandidateOrigin::TargetCyclicKrylov);

        assert_route_only_cover(result, CandidateOrigin::TargetCyclicKrylov);
    }

    #[test]
    fn resultant_route_forcing_selects_only_resultant_candidates() {
        let x = variable("X");
        let y = variable("Y");
        let t = variable("T");
        let variables = vec![x.clone(), y.clone(), t.clone()];
        let equations = vec![
            polynomial(&variables, &[(1, vec![1, 0, 0]), (-1, vec![0, 1, 0])]),
            polynomial(&variables, &[(1, vec![0, 1, 0]), (-1, vec![0, 0, 1])]),
            polynomial(&variables, &[(1, vec![1, 0, 0]), (-2, vec![0, 0, 0])]),
        ];

        let candidates = collect_candidates_for_test(
            problem(equations, variables, t),
            &limits(),
            &TestRouteForcing::only(CandidateOrigin::HiddenVariableSparseResultant),
        );

        assert!(!candidates.is_empty());
        assert!(candidates
            .iter()
            .all(|candidate| candidate.origin == CandidateOrigin::HiddenVariableSparseResultant));
    }

    #[test]
    fn resultant_route_forcing_solves_without_other_routes_or_complete_fallback() {
        let x = variable("X");
        let y = variable("Y");
        let t = variable("T");
        let variables = vec![x.clone(), y.clone(), t.clone()];
        let equations = vec![
            polynomial(&variables, &[(1, vec![1, 0, 0]), (-1, vec![0, 1, 0])]),
            polynomial(&variables, &[(1, vec![0, 1, 0]), (-1, vec![0, 0, 1])]),
            polynomial(&variables, &[(1, vec![1, 0, 0]), (-2, vec![0, 0, 0])]),
        ];

        let result = route_result(
            equations,
            variables,
            t,
            CandidateOrigin::HiddenVariableSparseResultant,
        );

        assert_route_only_cover(result, CandidateOrigin::HiddenVariableSparseResultant);
    }

    #[test]
    fn slice_route_forcing_selects_only_slice_candidates() {
        let x = variable("X");
        let t = variable("T");
        let variables = vec![x.clone(), t.clone()];
        let equations = vec![polynomial(
            &variables,
            &[(1, vec![0, 2]), (1, vec![1, 0]), (-2, vec![0, 0])],
        )];

        let candidates = collect_candidates_for_test(
            problem(equations, variables, t),
            &limits(),
            &TestRouteForcing::only(CandidateOrigin::SliceSpecialization),
        );

        assert!(!candidates.is_empty());
        assert!(candidates
            .iter()
            .all(|candidate| candidate.origin == CandidateOrigin::SliceSpecialization));
    }

    #[test]
    fn slice_route_forcing_solves_finite_target_family_without_complete_fallback() {
        let x = variable("X");
        let t = variable("T");
        let variables = vec![x.clone(), t.clone()];
        let equations = vec![polynomial(&variables, &[(1, vec![0, 2]), (-2, vec![0, 0])])];

        let result = route_result(
            equations,
            variables,
            t,
            CandidateOrigin::SliceSpecialization,
        );

        assert_route_only_cover(result, CandidateOrigin::SliceSpecialization);
    }

    #[test]
    fn complete_fallback_disabled_route_control_fails_on_reach() {
        let x = variable("X");
        let t = variable("T");
        let variables = vec![x.clone(), t.clone()];
        let equations = vec![polynomial(&variables, &[(1, vec![1, 0])])];
        let forcing = TestRouteForcing {
            enabled_origins: std::collections::BTreeSet::new(),
            allow_complete_fallback: false,
        };

        let result = std::panic::catch_unwind(|| {
            solve_target_for_test(problem(equations, variables, t), solver_options(), &forcing);
        });

        assert!(result.is_err());
    }
}
