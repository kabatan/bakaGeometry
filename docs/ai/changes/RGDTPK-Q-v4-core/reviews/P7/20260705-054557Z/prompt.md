Please re-review P7 after remediation. Do not edit files.

Workspace: C:\Users\bakat\OneDrive\ドキュメント\bakaGeometry

The prior FAIL_FIXABLE findings were addressed with these changes:
- TargetUnivariate admission/execution now includes child projection message target-only relation generators via `admit_target_univariate_with_messages`, binds child message package hashes in the plan, and validates child message hashes/source hashes at execution.
- Planner TargetUnivariate admission now requires variables subset `{T}` only, not all exported variables.
- TargetUnivariate and LinearAffine now check block authorization hashes and planned source relation hashes before execution.
- Replay for both P7 kernels now rejects stale package identity and wrong block/export context.
- Added regression tests:
  - `p7_target_univariate_uses_child_message_target_relation`
  - `p7_target_univariate_rejects_separator_only_planner_admission`
  - `p7_target_univariate_rejects_auth_source_and_child_message_tamper`
  - `p7_linear_affine_rejects_auth_and_source_hash_tamper`

Fresh evidence after remediation:
- cargo fmt --manifest-path geosolver-core/Cargo.toml --check: pass
- cargo test --manifest-path geosolver-core/Cargo.toml p6_ -- --nocapture: pass, 3 passed
- cargo test --manifest-path geosolver-core/Cargo.toml p7_ -- --nocapture: pass, 9 passed
- cargo test --manifest-path geosolver-core/Cargo.toml: pass, 121 passed
- git diff --check: pass except CRLF warnings

Please inspect the changed code and evidence, especially:
- geosolver-core/src/kernels/target_univariate.rs
- geosolver-core/src/kernels/linear_affine.rs
- geosolver-core/src/planner/admission.rs
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P7/*

Return RESULT: PASS or FAIL_FIXABLE/FAIL_BLOCKING with file/line findings. Scope remains PLAN P7 only and no claims beyond MECH-007 started.
