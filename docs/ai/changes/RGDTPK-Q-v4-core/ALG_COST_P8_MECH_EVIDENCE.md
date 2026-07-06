# ACR-P8 MECH Evidence

Status: implementation evidence, not readiness authority.

Claim ceiling remains:

```text
CANDIDATE_COVER_PIPELINE_PRESENT_BUT_ALGEBRAIC_COST_INCOMPLETE
```

## Scope

Implemented the ACR-P8 sparse/lazy TargetRelationSearch slice:

- Added `SparseRelationSearchPreflight` and `SparseRelationSearchSchedule` with stable replay
  hashes and `SparseFootprint` support descriptors.
- Added sparse footprint support builders that use constants, single-variable powers, projected
  observed support, and small exact term footprints without dense total-degree monomial
  enumeration.
- TargetRelationSearch admission now records dense preflight first. If dense materialization is
  cost-prohibited but the sparse footprint is feasible, it admits a sparse schedule instead of
  dropping the route.
- TargetRelationSearch execution now supports both dense schedules and sparse schedules. Both
  paths share the same membership matrix construction, modular nullspace reconstruction, and exact
  Q membership verification before producing a `ProjectionMessage`.
- Planner admission and planner cost estimates bind sparse TargetRelationSearch to the sparse
  stage shape instead of prohibiting it by the dense-route estimate.
- Sparse TargetRelationSearch plans created by the public planner path bind template shape,
  algebraic work estimate, route budget, and cost estimate to the replayed sparse schedule stage.
- `KernelRouteTrace` diagnostics now record dense materialization status and sparse footprint
  feasibility for TargetRelationSearch.
- Added generic algebraic tests where dense TRS is cost-prohibited, sparse footprint TRS is
  feasible, the kernel produces a membership-certified projection message, and the public pipeline
  reaches `CertifiedCandidateCover`.

## Changed Files

- `geosolver-core/src/planner/relation_schedule.rs`
- `geosolver-core/src/planner/kernel_plan.rs`
- `geosolver-core/src/planner/admission.rs`
- `geosolver-core/src/planner/cost_model.rs`
- `geosolver-core/src/planner/planner.rs`
- `geosolver-core/src/kernels/target_relation_search.rs`
- `geosolver-core/src/solver/pipeline.rs`
- Kernel plan initializer touchpoints in other kernel modules to add the new optional sparse
  schedule field.

## Verification Run

All cargo commands below were run from `geosolver-core`.

```text
cargo fmt --check
cargo check
cargo test --lib acr_p8 -- --nocapture
cargo test --lib target_relation_search -- --nocapture
cargo test --lib
rg -n -i "<forbidden diagnostic marker expression>" geosolver-core/src
git diff --check
```

Observed result:

```text
cargo fmt --check: passed
cargo check: passed
acr_p8: 3 passed
target_relation_search: 12 passed
cargo test --lib: 250 passed
forbidden-marker scan over implementation Rust sources: no matches
git diff --check: exit 0, CRLF conversion warnings only
```

## Acceptance Evidence

Sparse descriptor / no dense enumeration stress:

```text
kernels::target_relation_search::tests::acr_p8_sparse_footprint_descriptor_is_not_dense_total_degree
```

This verifies that the sparse schedule uses `SparseFootprint` descriptors and that the sparse
export support is smaller than the corresponding dense total-degree export support.

Dense prohibited / sparse kernel execution stress:

```text
kernels::target_relation_search::tests::acr_p8_sparse_footprint_executes_when_dense_route_is_cost_prohibited
```

This verifies that dense TargetRelationSearch preflight is not materializable, sparse preflight is
feasible, the admitted plan has no dense schedule and has a sparse schedule, execution produces a
TargetRelationSearch message, and the certificate payload is exact membership.

Public pipeline candidate-cover stress:

```text
solver::pipeline::tests::acr_p8_sparse_footprint_target_relation_pipeline_returns_candidate_cover
```

This verifies that the planned TargetRelationSearch route carries a sparse schedule, its planner
cost estimate is not `CostProhibited`, the support template, algebraic estimate, route budget, and
cost estimate are bound to the sparse schedule stage rows/columns, the message verifies, and the
final result constructed by the near-public pipeline reaches `CertifiedCandidateCover`. The route
diagnostic records `materialization_allowed=false` and `sparse_footprint_feasible=true`.

## Review-Driven Fix

The first ACR-P8 quality review found that the generic `planner/admission.rs` path attached sparse
TargetRelationSearch schedules but left the support template and initial algebraic work estimate
bound to probe-derived matrix dimensions. The fix binds sparse TargetRelationSearch planner plans to
the sparse schedule stage shape and extends the pipeline stress test to assert:

```text
support_plan.template_plan.matrix_rows == sparse_schedule.stage.matrix_rows
support_plan.template_plan.matrix_cols == sparse_schedule.stage.matrix_cols
algebraic_work_estimate.matrix_rows == sparse_schedule.stage.matrix_rows
algebraic_work_estimate.matrix_cols == sparse_schedule.stage.matrix_cols
route_budget.is_hash_current()
planner cost estimate rows/cols and estimate hash match the plan-bound algebraic estimate
```

## Anti-Overfit Boundary

No prior diagnostic input, expected-answer hook, domain-name dispatch, variable-role dispatch, or
fixed external benchmark fixture was used in this implementation slice. The new tests are generic
rational polynomial systems over renamed algebraic variables.

Review archive:

```text
docs/ai/changes/RGDTPK-Q-v4-core/reviews/ACR-P8/20260706-210037Z/
```

Guardian boundary, spec verifier, and quality reviewer are PASS for ACR-P8 MECH. This does not
grant final readiness or source-fidelity closure beyond ACR-P8.
