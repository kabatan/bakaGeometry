# P7 Source-To-Code Map

Status: implementation evidence for Phase 7.

Relevant R-IDs:

- BS-R080 -> `planner/cost_model.rs`, `planner/probes.rs`
- BS-R081 -> `planner/admission.rs`, `kernels/mod.rs`, `kernels/traits.rs`,
  `planner/planner.rs`
- BS-R082 -> `planner/kernel_plan.rs`, `planner/ladder.rs`, `solver/options.rs`

Mapping:

- `KernelKind`, `all_kernels()`, and `all_planner_kernel_kinds()` now use the same nine-kernel
  source order: TargetUnivariate, LinearAffine, TargetRelationSearch, SparseResultantProjection,
  TargetActionKrylov, NormTraceProjection, RegularChainProjection, SpecializationInterpolation,
  UniversalTargetElimination.
- `run_cost_probes` records structural metrics, modular rank, local Macaulay size, mixed support,
  coefficient growth, and an aggregate probe hash. Probes feed planning estimates and diagnostics;
  they are not final proof.
- `collect_kernel_admissions` iterates the runtime `all_kernels()` registry and calls every
  `TargetProjectionKernel::admit` entrypoint. If an admitted runtime entry has no plan, or if the
  route must be replanned against current solver options, it uses the same trait `plan` entrypoint.
  The resulting admission records carry runtime admission hash, source relation data, initial
  bounds, matrix/template estimates, status, and admission hash.
- `ensure_universal_admitted_for_relation_block` makes a relation block without admitted
  UniversalTargetElimination an `ImplementationBug`.
- `record_structural_empty_block` records an explicit structural no-projection diagnostic for empty
  relation blocks before skipping route planning.
- `KernelExecutionPlan` binds plan id, block id, kernel kind, authorization hash, source relation
  IDs/hashes, child IDs/message hashes, exported/eliminated variables, support plan, resource
  bounds, certificate route, failure behavior, algebraic work estimate, route budget, and plan hash.
- `build_declared_ladder` includes only admitted plans with enforceable route budgets, honors
  explicit user priority, and otherwise keeps UniversalTargetElimination as the last generic route.
- `require_declared_kernel_plan` rejects execution if the declared ladder hash changed or the
  requested kernel is absent from the declared ladder.
