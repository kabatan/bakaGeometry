# Generic Success Route Closure

Status: P7 closure complete; final reviewer reruns passed
Change: RGDTPK-Q-v4-generic-success-route-core-repair-v2
Date: 2026-07-06

## Claim Boundary

Allowed claim after this repair and evidence: `GENERIC_SUCCESS_ROUTE_PLANNER_READY`.

Not claimed from this repair alone:

- exact-image readiness
- supplied full v4 source fidelity
- full RGDTPK-Q-v4 acceptance
- benchmark superiority

## Implemented Scope

P1 dense TRS preflight/lazy schedule:

- Added public saturating monomial count API: `monomial_count_total_degree_leq_saturating`.
- Extended `RelationSearchStageEstimate` with machine-readable estimated export/multiplier/row counts and `RouteCostClass`.
- Added `cost_prohibited_reason` and preflight hash binding for dense schedule decline.
- Rechecked stage feasibility before schedule materialization and again before execution materializes supports/matrices.

P2 admission isolation and route records:

- Extended `KernelAdmissionStatus` with `CostProhibited` and `PlanProbeFailed`.
- Dense TRS cap failures now produce `CostProhibited` with preflight hash.
- Route-local planning failures are recorded as `PlanProbeFailed` with constructed-object hashes.
- `KernelRouteTrace` diagnostics are emitted for every kernel admission.

P3 declared ladder/execution isolation:

- Added `RouteCostClass` to kernel cost estimates and cost ordering.
- Cost-prohibited routes have no execution plan and therefore cannot enter the declared ladder.
- `execute_block_with_declared_ladder` records `BlockProjectionFailureTrace` diagnostics for route failures and continues on allowed failures.
- Continuation is constrained by each execution plan's declared `failure_behavior.allowed_statuses`; `ImplementationBug` and undeclared statuses abort instead of being hidden as route-local failures.
- `UniversalTargetElimination` is forced to the final declared ladder position even when listed in `kernel_priority`.

P4 UniversalTargetElimination generic route:

- Universal stage sequence remains fixed and deterministic.
- Internal dense TRS admission uses preflight and returns a continuable hard case when cost-prohibited.
- Internal sequence includes dense TRS, sparse resultant, target-action Krylov, specialization/interpolation, regular-chain, norm/trace, and bounded local Groebner stages.
- Universal certificates now record attempted strategy sequence, chosen strategy, and failed strategy hashes.
- Replay verification recomputes the fixed Universal stage hashes from certificate/context-bound source data and requires `failed_strategy_hashes` to equal the exact attempted-stage prefix before the chosen strategy.
- Universal local nonfinite remains disabled.

P5 graph/decomposition:

- Added `AlgebraicBlockMetrics` for arity, degree, monomial count, coefficient height, target-distance hint, affine/definitional counts, dense-TRS cost class/hash, quotient/action rank estimate, and sparse-template size.
- Extended separator scoring with relation-heavy, dense-TRS, and quotient-rank penalties.

P6 generic success-route stress suite:

- Added `geosolver-core/tests/generic_success_route_planner.rs`.
- Public `api::solve_target` cases cover action, sparse, separator-rich composition, and no-useful-separator large-footprint routes.
- Four large-footprint cases assert dense TRS cost-prohibition while another route succeeds.
- Replay acceptance is checked for each public result.
- Universal internal later-strategy trace is covered through public `api::solve_target`: earlier declared routes fail route-locally, Universal remains declared last, then Universal succeeds through a later internal strategy.
- A Universal failed-strategy-prefix tamper is rehashed and rejected both by direct projection-message verification and by run replay.

P7 static scan/closure:

- Added `GENERIC_SUCCESS_ROUTE_STATIC_SCAN.md`.
- This file records the bounded final claim and verification.

## Verification

Commands run:

```powershell
cargo fmt --manifest-path geosolver-core/Cargo.toml --check
cargo test --manifest-path geosolver-core/Cargo.toml --test gpsr_generic_planner_success_route
cargo test --manifest-path geosolver-core/Cargo.toml --test generic_success_route_planner
cargo test --manifest-path geosolver-core/Cargo.toml --test fcr_p11_red_team_suite
cargo test --manifest-path geosolver-core/Cargo.toml --test p15_acceptance_stress
cargo test --manifest-path geosolver-core/Cargo.toml
```

Observed result:

- `gpsr_generic_planner_success_route`: 5 passed.
- `generic_success_route_planner`: 3 passed.
- `fcr_p11_red_team_suite`: 10 passed.
- `p15_acceptance_stress`: 6 passed.
- Full crate/integration/doc test run: all tests passed, including 217 lib tests and all integration suites.

## Review Status

Spec verifier, quality reviewer, and boundary reviewer final reruns passed after the allowed-status, Universal-prefix, and plan/context-bound source-binding remediations. The bounded final claim is `GENERIC_SUCCESS_ROUTE_PLANNER_READY` only.
