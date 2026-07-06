# Generic Success Route Acceptance Results

Status: final reviewer reruns passed
Change: RGDTPK-Q-v4-generic-success-route-core-repair-v2
Date: 2026-07-06

## Claim Boundary

This repair supports only the bounded claim `GENERIC_SUCCESS_ROUTE_PLANNER_READY`.

It does not claim exact-image readiness, full supplied-v4 source fidelity, full RGDTPK-Q-v4 acceptance, or benchmark superiority.

## Phase Results

| Phase | Result | Evidence |
| --- | --- | --- |
| GSR-P1 dense TRS preflight/lazy schedule | implemented | `RouteCostClass`, cost-prohibited preflight hash, saturating monomial count API, materialization rechecks |
| GSR-P2 admission isolation | implemented | `CostProhibited`, `PlanProbeFailed`, per-kernel `KernelRouteTrace`, route-local panic/error isolation |
| GSR-P3 declared ladder execution | implemented | `UniversalTargetElimination` declared last, cost-prohibited routes excluded, route failures recorded as `BlockProjectionFailureTrace`; continuation is limited to the execution plan's declared `allowed_statuses` and never swallows `ImplementationBug` |
| GSR-P4 Universal route | implemented | fixed internal strategy sequence, certificate-bound strategy trace, continuable dense decline, no local nonfinite; failed strategy hashes replay as the exact prefix of the fixed stage plan before the chosen strategy, using plan-bound certificate source hashes rather than payload-controlled counts |
| GSR-P5 decomposition repair | implemented | `AlgebraicBlockMetrics`, algebraic separator classes, relation-heavy/dense-TRS/quotient-rank scoring penalties |
| GSR-P6 generic stress tests | implemented | public support-producing route cases, including public Universal internal later-strategy trace after earlier route failures |
| GSR-P7 closure/static scan | evidence ready | static scan document, acceptance results, reviewer archive recorded |

## Acceptance Matrix Mapping

| Family | Evidence |
| --- | --- |
| G1 large footprint compact quotient/action | `generic_success_route_planner::gsr_p6_public_success_routes_remain_available_after_dense_decline` action case returns `CertifiedCandidateCover`, replay accepts, dense TRS is cost-prohibited |
| G2 large footprint sparse/specialization | same test sparse case returns `CertifiedCandidateCover`, replay accepts, dense TRS is cost-prohibited |
| G3 separator-rich composition | same test separator-rich case returns multiple projection messages and replay accepts |
| G4 Universal later-strategy trace | `generic_success_route_planner::gsr_p4_universal_internal_later_strategy_records_trace` verifies public `api::solve_target` success through `UniversalTargetElimination` after earlier declared route failures, and rejects a rehashed failed-strategy-prefix tamper through direct `verify_projection_message` plus run replay |
| G5 no useful separator one-large-block | generic/P10/P11/P15 tests verify candidate-cover success and Universal admission/last-ladder evidence under the public planner |

## Verification Commands

```powershell
cargo fmt --manifest-path geosolver-core/Cargo.toml --check
cargo test --manifest-path geosolver-core/Cargo.toml --test gpsr_generic_planner_success_route
cargo test --manifest-path geosolver-core/Cargo.toml --test generic_success_route_planner
cargo test --manifest-path geosolver-core/Cargo.toml --test fcr_p11_red_team_suite
cargo test --manifest-path geosolver-core/Cargo.toml --test p15_acceptance_stress
cargo test --manifest-path geosolver-core/Cargo.toml
```

Observed latest results:

- `cargo fmt --check`: pass.
- `gpsr_generic_planner_success_route`: 5 passed.
- `generic_success_route_planner`: 3 passed.
- `fcr_p11_red_team_suite`: 10 passed.
- `p15_acceptance_stress`: 6 passed.
- Full `geosolver-core` test run: pass, including 217 library tests and all integration/doc-test suites.

## Universal Public-Path Evidence

The public declared ladder intentionally keeps `UniversalTargetElimination` last for relation-bearing blocks. The G4 public stress case uses a generic algebraic input where earlier declared routes record continuable `BlockProjectionFailureTrace` entries and the final Universal route succeeds through a later internal strategy. Replay accepts the public result.
