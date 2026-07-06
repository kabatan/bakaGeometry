# Generic Success Route Static Scan

Status: P7 closure evidence
Change: RGDTPK-Q-v4-generic-success-route-core-repair-v2
Date: 2026-07-06

## Scope

Scanned production and test code for forbidden concrete-case routing, geometry-family dispatch, expected-answer fixtures, coordinate-enumeration shortcuts, QE/CAD claims, and placeholder implementation markers.

Primary changed code and tests:

- `geosolver-core/src/planner/relation_schedule.rs`
- `geosolver-core/src/planner/admission.rs`
- `geosolver-core/src/planner/planner.rs`
- `geosolver-core/src/planner/cost_model.rs`
- `geosolver-core/src/solver/pipeline.rs`
- `geosolver-core/src/kernels/target_relation_search.rs`
- `geosolver-core/src/kernels/universal_elimination.rs`
- `geosolver-core/src/verify/certificates.rs`
- `geosolver-core/src/verify/verify_message.rs`
- `geosolver-core/src/verify/replay.rs`
- `geosolver-core/src/graph/metrics.rs`
- `geosolver-core/src/graph/separators.rs`
- `geosolver-core/src/graph/tree_decomposition.rs`
- `geosolver-core/tests/generic_success_route_planner.rs`
- `geosolver-core/tests/gpsr_generic_planner_success_route.rs`
- `geosolver-core/tests/fcr_p10_acceptance_suite.rs`
- `geosolver-core/tests/fcr_p11_red_team_suite.rs`
- `geosolver-core/tests/p15_acceptance_stress.rs`

## Commands

```powershell
rg -n -i "mixtilinear|circle solver|triangle solver|tangent solver|expected_answer|golden|fixture|problem_id|coordinate solution enumeration|full coordinate RUR|\bQE\b|\bCAD\b|unimplemented!|todo!|placeholder" geosolver-core/src geosolver-core/tests
rg -n "Geometry|Mixtilinear|circle|triangle|tangent|problem_id|expected_answer|fixture|golden" geosolver-core/src/planner geosolver-core/src/kernels geosolver-core/src/graph geosolver-core/tests/generic_success_route_planner.rs geosolver-core/tests/gpsr_generic_planner_success_route.rs
rg -n "KernelRouteTrace|BlockProjectionFailureTrace|CostProhibitedDenseRoute|RouteCostClass|CostProhibited|PlanProbeFailed|monomial_count_total_degree_leq_saturating|attempted_strategies|failed_strategy_hashes|AlgebraicBlockMetrics" geosolver-core/src geosolver-core/tests
rg -n -i "mixtilinear|incircle|circumcircle|tangent|triangle|olympiad|fixture|official|expected_answer|problem_id" geosolver-core/src geosolver-core/tests
rg -n -i "full coordinate RUR|coordinate solution enumeration|\bQE\b|\bCAD\b|hidden fallback|unimplemented!|todo!|placeholder" geosolver-core/src geosolver-core/tests
```

## Classification

No hits were found in the focused changed planner/kernel/graph/test scan for concrete geometry names, geometry-family dispatch, expected-answer fixtures, problem-id routing, golden fixtures, QE/CAD shortcuts, or concrete investigated-problem identifiers.

The broad production/test scan returned only pre-existing or allowed generic hits:

- `geosolver-core/src/verify/run_certificate.rs`: invariant flag names such as `no_problem_id_dispatch` and `no_expected_answer_dispatch`; these are anti-dispatch invariant fields, not dispatch logic.
- `geosolver-core/tests/p14_full_pipeline_integration.rs`: tests asserting invariant flags remain bounded by explicit evidence; not expected-answer routing.
- `geosolver-core/src/algebra/quotient.rs`: guard strings rejecting full coordinate RUR exposure; this is a prohibition, not coordinate enumeration.
- `geosolver-core/tests/p12_roots_decode_integration.rs`: test name uses `nonplaceholder`; this is not a placeholder implementation marker.
- `geosolver-core/src/verify/run_certificate.rs`: `no_qe_cad` and dispatch scan hashes are evidence-binding fields; they do not implement QE/CAD or dispatch.

## Positive Implementation Markers

The implementation scan confirmed the following generic success-route markers:

- `RouteCostClass`, `CostProhibited`, and `PlanProbeFailed` are present in planner admission/cost paths.
- `KernelRouteTrace` and `BlockProjectionFailureTrace` diagnostics are emitted.
- `CostProhibitedDenseRoute` remains machine-readable and includes preflight details.
- `monomial_count_total_degree_leq_saturating` is public and tested.
- Universal certificates include `attempted_strategies`, `chosen_strategy`, and `failed_strategy_hashes`.
- Universal strategy trace verification is implemented in `verify_message.rs`, including exact replay of the failed-strategy hash prefix before the chosen strategy from certificate/context-bound source hashes.
- Declared ladder continuation is gated by each route plan's declared `failure_behavior.allowed_statuses` and rejects `ImplementationBug`.
- `AlgebraicBlockMetrics` records arity, degree, monomial count, height, dense-TRS cost class, quotient/action rank estimate, and sparse-template size.
- Separator candidate generation includes bounded min-cut, algebraic-intermediate, and low-degree definitional affine classes.

## Result

Static scan passes for this repair scope. The scan does not support exact-image readiness, supplied-v4 source fidelity, benchmark superiority, or full acceptance claims.
