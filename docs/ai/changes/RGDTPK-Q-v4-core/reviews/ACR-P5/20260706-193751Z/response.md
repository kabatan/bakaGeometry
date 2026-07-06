# ACR-P5 Guardian Boundary Re-Review Response

Status: FAIL_FIXABLE

Blocker:

- Prior F3 is not fully closed. In `geosolver-core/src/solver/pipeline.rs`, a route whose kernel is
  not found is skipped with `continue` after `BlockProjectionRouteStart` is recorded, but no route
  failure or route summary is added. The no-failure aggregate path then returns a hardcase without
  attempted route summaries. This still violates ACR-P5 requirement 5.

Closed prior findings:

- F1, F2, and F4 appear closed for the reviewed P5 claim. `ActiveRouteBudget` consumes elapsed
  steps/work units, pipeline activates it before `kernel.execute`, `SparseResultant` has in-flight
  checkpoints, and the near-public in-flight test asserts cooperative budget stop followed by later
  route success.

Required next action:

- Make the missing-kernel/no-failure aggregate path preserve attempted route summaries and include
  `all_attempts` in every aggregate ladder failure.
- Add or adjust a focused test for that path.

Forbidden claims:

- This response does not close ACR-P5 and does not authorize P6+, exact-image readiness,
  candidate-cover readiness, source fidelity, final acceptance, or any R-ID VERIFIED claim.
