RESULT: FAIL_FIXABLE (FCR-P1 FAIL)

Prior blockers are fixed:
- No old boilerplate actual input/output phrases found.
- `KernelPlan::new` and `KernelExecutionPlan::new` are disambiguated.
- `replay_run_certificate` is downgraded to `certificate_binding: missing`.
- `final_support` finalizers are downgraded to `missing`; nonfinite certification is only `decorative`.
- Evidence now includes semantic spot checks, not just row-count checks.

Blocking issue:
- Production-reachable trait-dispatch methods in mandatory kernel files are still not listed with exact file paths. The audit uses wildcard rows like `geosolver-core/src/kernels/*::TargetActionKrylovKernel(TargetProjectionKernel)::plan`, while the code has concrete production paths in `action_krylov.rs`, `sparse_resultant.rs`, `specialization_interpolation.rs`, `regular_chain_projection.rs`, and `norm_trace_projection.rs`. FCR-P1 requires `path: <file::function>` and every production-reachable function/path.

Required fix:
- Replace wildcard trait rows with exact rows such as `geosolver-core/src/kernels/action_krylov.rs::TargetActionKrylovKernel(TargetProjectionKernel)::plan`, including source line, actual input/output class, plan-time relation status, certificate strength, and repair action.

Residual risk:
- Exact trait method expansion may reveal additional per-kernel classification issues.
