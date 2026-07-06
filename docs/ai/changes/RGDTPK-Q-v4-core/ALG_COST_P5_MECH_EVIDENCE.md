# ACR-P5 MECH Evidence

Status: implementation evidence, not readiness authority.

Claim ceiling remains:

```text
CANDIDATE_COVER_PIPELINE_PRESENT_BUT_ALGEBRAIC_COST_INCOMPLETE
```

## Scope

Implemented the ACR-P5 declared-ladder bounded execution slice:

- `planner/ladder.rs` only places routes with hash-current, nonzero route budgets in the declared
  ladder.
- `problem/context.rs` carries an active route budget and cooperative checkpoint meter for route
  execution.
- Production projection kernels call cooperative checkpoints at route entry and dominant internal
  work boundaries, so `max_elapsed_steps` and `max_work_units` are enforceable during execution,
  not only before/after `kernel.execute`.
- `execute_block_with_declared_ladder` records route start, route success, route allowed failure,
  route budget/resource stop, elapsed microseconds, predicted work, budget work cap, budget hash,
  estimate hash, and route-attempt hash.
- Route-local budget stops continue to later routes when the declared failure behavior allows the
  failure status.
- Aggregate ladder failure includes all attempted route summaries and attempt hashes.
- Near-public pipeline stresses run validate/canonicalize/compress/graph/dag/plan/execute/verify/
  compose/support/root/certificate/cost-trace steps, mutate the first declared SparseResultant
  route to exceed either preflight work budget or in-flight elapsed budget, and verify a later route
  still produces a certified candidate cover result.

The first ACR-P5 guardian review failed because the route budget was only checked before/after
`kernel.execute`. The FAIL archive is:

```text
docs/ai/changes/RGDTPK-Q-v4-core/reviews/ACR-P5/20260706-193037Z/
```

The second ACR-P5 guardian review closed the in-flight findings but failed the no-failure aggregate
path. The FAIL archive is:

```text
docs/ai/changes/RGDTPK-Q-v4-core/reviews/ACR-P5/20260706-193751Z/
```

That path now includes `all_attempts` even when no route failure object exists, and records an
attempt summary for the defensive missing-kernel branch.

## Changed Files

- `geosolver-core/src/planner/ladder.rs`
- `geosolver-core/src/planner/algebraic_cost.rs`
- `geosolver-core/src/problem/context.rs`
- `geosolver-core/src/solver/pipeline.rs`
- `geosolver-core/src/kernels/action_krylov.rs`
- `geosolver-core/src/kernels/linear_affine.rs`
- `geosolver-core/src/kernels/norm_trace_projection.rs`
- `geosolver-core/src/kernels/regular_chain_projection.rs`
- `geosolver-core/src/kernels/sparse_resultant.rs`
- `geosolver-core/src/kernels/specialization_interpolation.rs`
- `geosolver-core/src/kernels/target_relation_search.rs`
- `geosolver-core/src/kernels/target_univariate.rs`
- `geosolver-core/src/kernels/universal_elimination.rs`

## Verification Run

All cargo commands below were run from `geosolver-core`.

```text
cargo fmt --check
cargo test --lib acr_p5 -- --nocapture
cargo test --lib acr_p3 -- --nocapture
cargo test --lib acr_p4 -- --nocapture
cargo check
cargo test --lib
rg -n -i "mixtilinear|mixt|circumcircle|incircle|tangent_solver|expected_cos|expected answer|problem hash|diagnostic_problem|known_support_polynomial|geometry name" src -g "*.rs"
```

Observed result:

```text
acr_p5: 3 passed
acr_p3: 3 passed
acr_p4: 6 passed
cargo check: passed
cargo test --lib: 237 passed
forbidden-pattern scan over implementation Rust sources: no matches
```

Reviews:

```text
guardian_boundary_reviewer: PASS (ACR-P5 only)
spec_verifier: PASS (ACR-P5 only)
quality_reviewer: PASS (ACR-P5 only)
```

## Acceptance Evidence

The P5 stress test is:

```text
solver::pipeline::tests::acr_p5_near_public_pipeline_budget_stop_yields_to_second_route_candidate_cover
solver::pipeline::tests::acr_p5_near_public_pipeline_inflight_budget_stop_yields_to_second_route_candidate_cover
solver::pipeline::tests::acr_p5_aggregate_no_failure_path_preserves_attempt_summaries
```

They verify:

- first declared SparseResultant route yields with `route_budget_stop`;
- one case yields by preflight work budget;
- one case passes preflight and yields inside `SparseResultant::execute_start` through cooperative
  in-flight `max_elapsed_steps` metering;
- `BlockProjectionRouteStart`, `BlockProjectionFailureTrace`, and `BlockProjectionRouteSuccess`
  diagnostics are emitted with route-attempt work summaries;
- a later route succeeds after the allowed budget stop;
- the constructed pipeline result status is `CertifiedCandidateCover`.
- the no-failure aggregate path preserves `all_attempts`, route event, status, and attempt hash.

## Anti-Overfit Boundary

No diagnostic problem file, expected-answer hook, geometry-name dispatch, or special-case fixture
was used in this implementation slice.
