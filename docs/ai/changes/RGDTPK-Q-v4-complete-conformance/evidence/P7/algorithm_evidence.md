# P7 Algorithm Evidence

Status: P7 implementation evidence before spec reviewer.

Implemented/verified behavior:

- Kernel source order is fixed across enum discriminants, runtime registry, and planner admission
  list. Regressions:
  `p7_kernel_kind_discriminants_match_source_order_registry` and
  `p7_planner_kernel_list_matches_runtime_registry_order`.
- `plan_all_blocks` calls `collect_kernel_admissions` for each non-empty relation block and records
  diagnostics for every admission and cost estimate.
- Remediation after spec review: `collect_kernel_admissions` now calls the runtime registry
  `kernel.admit(block, &kernel_context)` for every kernel and records the runtime admission hash in
  `KernelAdmissionEvidence`.
- Remediation after spec review: `KernelAdmission` now contains `KernelAdmissionEvidence` with
  source relation IDs/hashes, initial resource bounds, matrix/template estimates, runtime admission
  hash, and evidence hash. The evidence hash is bound into the admission hash and route diagnostic.
- TargetRelationSearch plans are regenerated through the trait `plan` path with the active
  `SolverContext`, preventing stale default-option schedules from entering the declared ladder.
- Relation blocks without admitted UniversalTargetElimination now fail with `ImplementationBug`.
  Regression: `p7_relation_block_without_universal_admission_is_implementation_bug`.
- Empty relation blocks are not silently ignored; they receive a structural no-projection
  diagnostic. Regression:
  `p7_empty_relation_block_records_structural_no_projection_diagnostic`.
- Declared ladder execution remains guarded by `require_declared_kernel_plan`, which rejects hash
  tampering and absent kernels. Existing regressions cover hidden fallback and plan-field tampering.
- Removed unused `kernel_not_ready_error` production placeholder from `kernels/traits.rs`.

Static audit guard added:

- `audit_v4_conformance.py --phase P7 --strict` checks required P7 files/symbols, exact nine-kernel
  order, runtime all-kernel admission path, admission evidence fields, ladder budget guards,
  structural empty-block record, Universal missing invariant, and declared-ladder execution guards.

Claim boundary:

- This evidence supports only Phase 7 planner/admission/ladder conformance.
- It does not claim finite candidate-cover completion or full source-faithful completion.
