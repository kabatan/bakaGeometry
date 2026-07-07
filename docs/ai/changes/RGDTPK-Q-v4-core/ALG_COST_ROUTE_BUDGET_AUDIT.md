# Algebraic-Cost Route Budget Audit

Scope: ACR-P10 audit for bounded production routes.

## Budget-Binding Anchors

- `geosolver-core/src/planner/algebraic_cost.rs` defines `RouteBudget` with work, elapsed-step,
  pair-term, intermediate-term, output-term, keep-variable, degree, and coefficient-height caps.
- `geosolver-core/src/planner/kernel_plan.rs` binds `route_budget_hash` into plan/support hashes.
- `geosolver-core/src/planner/ladder.rs` rejects execution plans without hash-current, nonzero
  route budgets.
- `geosolver-core/src/solver/pipeline.rs` enforces route budgets before execution, during execution
  through active route metering, and after message construction.
- `geosolver-core/src/kernels/sparse_resultant.rs` checks pair cost, intermediate growth, output
  terms, degree, coefficient height, elapsed steps, and route work units.
- `geosolver-core/src/kernels/universal_elimination.rs` gives every internal Universal stage its
  own route budget and binds that budget into subplans.

## Non-Monopolizing Ladder Evidence

The declared ladder records `KernelRouteTrace` diagnostics with route budget hashes, route events,
and allowed failures. Route-local budget stops and finite resource failures can yield to later
declared routes when `planned_failure_behavior` permits continuation.

Required tests passed in the full suite:

- `acr_p2_route_budget_preflight_stops_over_budget_estimate`
- `acr_p2_route_budget_postflight_stops_over_budget_output`
- `acr_p4_declared_ladder_continues_after_sparse_resultant_guard_failure`
- `acr_p5_near_public_pipeline_budget_stop_yields_to_second_route_candidate_cover`
- `acr_p5_near_public_pipeline_inflight_budget_stop_yields_to_second_route_candidate_cover`
- `acr_p6_universal_skips_cost_prohibited_dense_sparse_and_returns_candidate_cover`
- ACR-P9 S4/S5/S6 stress families

## Dominant Cost Coverage

Dense TRS:

- Dense materialization is guarded by closed-form admission/resource bounds.
- Large-block cases use sparse/lazy descriptors or later routes.

SparseResultant:

- Admission uses expression-swell estimates, not only Sylvester dimensions.
- Runtime guards stop growth before unbounded symbolic determinant chains.
- Quadratic subresultant backend is exact and replayable for the P9 feasible sparse-resultant case.

Universal:

- Internal stages have route budgets and cost classes.
- `executed_failed_strategy_hashes` proves actual enabled failures separately from skipped
  `CostProhibited` stages.

## Closure Result

No production route is justified by planning alone. Projection messages require execution,
certificate payloads, exact Q verification, and replay. The route budget audit supports only the
candidate-cover algebraic-cost closure, not exact-image or full-spec acceptance.

