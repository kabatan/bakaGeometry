# P7 Reviewer FAIL_FIXABLE Remediation

Reviewer result before remediation: `FAIL_FIXABLE`.

Findings addressed:

1. TargetUnivariate did not include child-message target-only relations.
   Fix: added `admit_target_univariate_with_messages`, child-message collection, child message hash binding in the execution plan, and execution-time child message hash validation.

2. Planner TargetUnivariate admission admitted separator-only relations.
   Fix: planner admission now filters TargetUnivariate candidates to variables subset `{T}` and sets exported variables to `[T]` for that plan.

3. P7 execute/replay tamper protection was too weak.
   Fix: TargetUnivariate and LinearAffine now check block authorization hashes and source relation hashes during execution. Replay now rejects messages whose package hash does not recompute for the message content or whose block/export context is inconsistent.

New regression tests:

- `p7_target_univariate_uses_child_message_target_relation`
- `p7_target_univariate_rejects_separator_only_planner_admission`
- `p7_target_univariate_rejects_auth_source_and_child_message_tamper`
- `p7_linear_affine_rejects_auth_and_source_hash_tamper`

Verification after remediation:

- `cargo fmt --manifest-path geosolver-core/Cargo.toml --check`: pass
- `cargo test --manifest-path geosolver-core/Cargo.toml p6_ -- --nocapture`: pass, 3 passed
- `cargo test --manifest-path geosolver-core/Cargo.toml p7_ -- --nocapture`: pass, 9 passed
- `cargo test --manifest-path geosolver-core/Cargo.toml`: pass, 121 passed
- `git diff --check`: pass, with CRLF warnings only
