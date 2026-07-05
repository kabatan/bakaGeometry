You are guardian_boundary_reviewer for RGDTPK-Q-v4-core FCR-P1 after two FAIL_FIXABLE reviews. Read-only review only; do not edit files.

Scope:
- `docs/ai/changes/RGDTPK-Q-v4-core/FULL_CORE_PRODUCTION_AUDIT.md`
- `docs/ai/changes/RGDTPK-Q-v4-core/evidence/FCR-P1/`

Prior FAIL blocker:
- Wildcard TargetProjectionKernel trait rows like `geosolver-core/src/kernels/*::...` were not acceptable.

Expected second-remediation state:
- All TargetProjectionKernel trait-dispatch rows use concrete exact file paths.
- Trait rows include source_line values.
- All 45 trait-dispatch rows are concrete.

Review checks:
- FCR-P1 full production audit is admissible for the repair plan only.
- Every production-reachable function/path is audited enough to drive later repairs.
- Actual input/output classes are semantic, not boilerplate.
- No high/fatal rows are assigned `keep`.
- Replay/final_support certificate downgrades remain intact.
- `KernelPlan::new` and `KernelExecutionPlan::new` are disambiguated.
- Evidence is adequate.

Return `RESULT: PASS` or `RESULT: FAIL_FIXABLE` with concrete blockers.
