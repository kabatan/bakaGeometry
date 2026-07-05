RESULT: PASS

Findings: no blocking P8b findings after remediation.

The prior blockers are addressed in the inspected code:

- `SparseResultantProjectionKernel` now collects local relations plus child message relation generators, binds child package hashes into `KernelExecutionPlan.child_message_hashes`, checks them at execution, and verifies resultant certificates before returning a `ProjectionMessage`.
- `SpecializationInterpolationKernel` now generates samples through `execute_inner_target_only_kernel`, which executes admitted `TargetRelationSearch` per specialization sample. Local Groebner/elimination is used only for final exact Q verification.

Evidence inspected:

- Requested source files under `geosolver-core/src/kernels`, `algebra`, and `planner/admission.rs`
- `PRIMITIVE_SCOPE_LEDGER.md`
- `evidence/P8b/*`
- Fresh evidence: fmt pass, P8b 7 passed, interpolation 9 passed, resultant 11 passed, P6/P7/P8a regressions pass, full suite 136 passed, `git diff --check` exit 0 with CRLF warnings only

Exact PASS scope: Plan P8b only, supporting `RGQ-020`, `RGQ-025`, and `RGQ-043` for the two remediated kernels with exact verification before projection-message output. This PASS only continues `MECH-007`.

Forbidden claims remain: do not claim P8 umbrella closure, `MECH-007` closure, P8c/P8d/P9, replay closure, final composition, exact-image semantics, public orchestration, acceptance readiness, or any R-ID as VERIFIED.
