Guardian boundary reviewer re-review request for FCR-P2 after remediation.

Workspace: C:\Users\bakat\OneDrive\ドキュメント\bakaGeometry
Branch: codex/full-core-repair
Task: FCR-P2 pre-P13 Full Core Repair. Please perform read-only review. Do not edit files.

Prior result: you returned FAIL_FIXABLE because partial high-risk kernel modules remained public production modules in geosolver-core/src/kernels/mod.rs, so registry/admission quarantine alone was too narrow.

Remediation implemented:
- In geosolver-core/src/kernels/mod.rs, these modules are now test-only with #[cfg(test)]: action_krylov, norm_trace_projection, regular_chain_projection, sparse_resultant, specialization_interpolation, target_relation_search, universal_elimination. Production all_kernels() returns only TargetUnivariateKernel and LinearAffineKernel. kernel_by_kind is #[cfg(test)].
- In geosolver-core/src/compose/mod.rs, separator_elimination is #[cfg(test)]. In geosolver-core/src/compose/compose.rs, the separator fallback call/import are #[cfg(test)].
- In geosolver-core/src/algebra/mod.rs, f4 is #[cfg(test)]. NonProductionGroebnerBatchForTests and its dispatch arm are #[cfg(test)] in algebra/elimination.rs.
- In geosolver-core/src/verify/mod.rs, replay is #[cfg(test)], and replay re-export is #[cfg(test)]. KernelCertificate::synthetic_for_tests is #[cfg(test)].
- Added geosolver-core/src/planner/relation_schedule.rs as production-safe schedule metadata moved out of the test-only target_relation_search kernel module. kernel_plan/admission/planner imports were updated accordingly.
- Production planner admission explicitly declines advanced/non-production kernels under cfg(not(test)); all_planner_kernel_kinds() production list is TargetUnivariate + LinearAffine only, test list includes advanced kinds for quarantined tests.

Evidence available:
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/FCR-P2/deletion_and_quarantine_log.yaml
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/FCR-P2/production_reachability_scan.txt
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/FCR-P2/forbidden_path_scan.txt
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/FCR-P2/commands.txt
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/FCR-P2/command_outputs.txt
- prior fail archived at docs/ai/changes/RGDTPK-Q-v4-core/evidence/FCR-P2/guardian_boundary_reviewer_fail.md

Verification run:
- cargo check in geosolver-core: exit_code 0
- cargo test in geosolver-core: exit_code 0, 198 lib tests + 2 integration tests passed
- static cfg/reachability scan: all PASS
- forbidden path checks: all PASS/PASS_BY_QUARANTINE/PASS_FOR_FCR_P2_SCOPE
- git diff --check: exit_code 0, only LF->CRLF warnings

Please answer with:
RESULT: PASS or RESULT: FAIL_FIXABLE or RESULT: FAIL_BLOCKED
Then concise findings. Specifically determine whether the prior blocker is fixed and whether FCR-P2 can be admitted as complete under the claim ceiling PARTIAL_MECHANISM_READY:MECH-011, without claiming full production readiness.
