# Generic Success-Route Current Audit

Status: P0 audit for `RGDTPK-Q-v4-generic-success-route-core-repair-v2`.

```yaml
- file: geosolver-core/src/planner/relation_schedule.rs
  relevant_functions:
    - estimate_dense_relation_search_schedule
    - build_dense_relation_search_schedule
    - dense_relation_search_decline_reason
  current_problem: Dense TRS preflight exists from GPSR, but v2 requires public saturating count API names, route cost class, and execution-stage cap rechecks.
  required_repair: Expose the v2 preflight shape, cost class, deterministic hash, default caps, and no materialization before preflight admission.
  overfit_risk: Low if estimators depend only on variable counts, degrees, and relation terms.

- file: geosolver-core/src/planner/admission.rs
  relevant_functions:
    - all_planner_kernel_kinds
    - collect_kernel_admissions
    - build_kernel_admission
  current_problem: Admissions are collected for every kernel, but status variants and machine-readable route traces are too weak for v2.
  required_repair: Add route-local cost-prohibited/plan-probe-failed status or equivalent diagnostics without solve-level failure.
  overfit_risk: Low if diagnostics are generic and kernel-kind based.

- file: geosolver-core/src/planner/ladder.rs
  relevant_functions:
    - build_declared_ladder
  current_problem: Declared ladder excludes declined routes and places Universal last, but does not yet use an explicit route cost class.
  required_repair: Exclude cost-prohibited routes, preserve compact executable routes, and keep Universal last for relation-bearing blocks.
  overfit_risk: Low if ordering uses kernel/cost metadata only.

- file: geosolver-core/src/planner/planner.rs
  relevant_functions:
    - plan_all_blocks
    - record_dense_relation_search_admission_diagnostics
  current_problem: Dense TRS diagnostic exists, but all route admissions need machine-readable trace coverage.
  required_repair: Emit route diagnostics for every kernel admission and preserve block-level failure context.
  overfit_risk: Low if diagnostics are generated from generic admissions.

- file: geosolver-core/src/solver/pipeline.rs
  relevant_functions:
    - step_execute
    - execute_block_with_declared_ladder
  current_problem: The executor iterates declared routes, but route failure trace is not retained in diagnostics/cost evidence.
  required_repair: Record every failed route and aggregate only after the declared ladder is exhausted.
  overfit_risk: Low if failures are generic `SolverError` public statuses and kernel kinds.

- file: geosolver-core/src/kernels/target_relation_search.rs
  relevant_functions:
    - admit_target_relation_search
    - execute_target_relation_search
  current_problem: Dense preflight guards admission and execution entry, but v2 asks for per-stage recheck before materialization.
  required_repair: Recheck materialization caps before each stage support build and keep dense route failure route-local.
  overfit_risk: Low if checks use schedule/preflight bounds only.

- file: geosolver-core/src/kernels/universal_elimination.rs
  relevant_functions:
    - plan_universal_elimination
    - execute_universal_elimination
    - execute_strategy
  current_problem: Universal has strategy sequence and guarded dense escalation, but certificate/trace evidence for chosen and failed internal strategies needs strengthening.
  required_repair: Record attempted, failed, and chosen internal strategies and ensure output remains exported-only.
  overfit_risk: Low if strategies are algebraic kernels only.

- file: geosolver-core/src/graph/tree_decomposition.rs
  relevant_functions:
    - build_target_rooted_decomposition
  current_problem: Decomposition is graph-cost driven and may leave large relation-heavy blocks.
  required_repair: Add algebraic footprint penalties or audit metrics for relation-heavy blocks and separator candidates.
  overfit_risk: Medium if using concrete shape constants; avoid by generic thresholds and metrics.

- file: geosolver-core/src/graph/separators.rs
  relevant_functions:
    - CostModel
    - separator scoring helpers
  current_problem: Separator scoring does not explicitly expose all v2 algebraic metrics.
  required_repair: Add/record generic algebraic separator metrics without geometry names.
  overfit_risk: Low if based on arity, degree, monomials, target distance, and definitional form.

- file: geosolver-core/src/graph/weighted_primal.rs
  relevant_functions:
    - build_weighted_primal_graph
  current_problem: Weighted graph construction records variable coupling but not all algebraic projection-cost metrics.
  required_repair: Add or preserve access to relation arity/degree/monomial/coefficient features for decomposition scoring evidence.
  overfit_risk: Low if no semantic geometry roles are used.
```
