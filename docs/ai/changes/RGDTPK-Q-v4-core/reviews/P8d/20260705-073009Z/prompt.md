Read-only Guardian boundary review for GeoSolver P8d. Workspace: C:\Users\bakat\OneDrive\ドキュメント\bakaGeometry. Review target: Plan P8d only. Do not modify files.

You must inspect actual code and evidence, not summaries alone. Required source/spec files:
- docs/ai/changes/RGDTPK-Q-v4-core/BASE_SPEC.md, especially RGQ-022, RGQ-036, RGQ-041, RGQ-051, RGQ-056, MECH-008, and RGQ-057/RGQ-058 if relevant to nonfinite/status polarity.
- docs/ai/changes/RGDTPK-Q-v4-core/PLAN.md P8d.
- docs/ai/changes/RGDTPK-Q-v4-core/REVIEWER_PROMPTS.md P8d and general reviewer instructions.
- docs/ai/changes/RGDTPK-Q-v4-core/PRIMITIVE_SCOPE_LEDGER.md, especially non-production F4/Groebner limitations.

Required implementation files:
- geosolver-core/src/kernels/universal_elimination.rs
- geosolver-core/src/planner/admission.rs
- geosolver-core/src/planner/kernel_plan.rs
- geosolver-core/src/algebra/elimination.rs
- geosolver-core/src/algebra/f4.rs as needed for the non-production F4 boundary.

Required evidence:
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P8d/commands.txt
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P8d/command_outputs.txt
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P8d/static_scans.txt
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P8d/test_first_failure.txt
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P8d/function_implementation_table.yaml
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P8d/notes.md

Fresh verification already run after the last code changes:
- cargo fmt --manifest-path geosolver-core/Cargo.toml --check: pass
- cargo test --manifest-path geosolver-core/Cargo.toml p8d_ -- --nocapture: 6 passed
- cargo test --manifest-path geosolver-core/Cargo.toml p6_/p7_/p8a_/p8b_/p8c_: pass
- cargo test --manifest-path geosolver-core/Cargo.toml: 147 passed
- forbidden Universal fallback rg scan: no matches
- git diff --check: exit 0, only CRLF warnings

Review questions:
1. Does P8d satisfy RGQ-041/RGQ-056 as a bounded local target/separator projection, not hidden heavy fallback?
2. Is the fixed strategy sequence exact and hash-bound before execution?
3. Does Universal avoid local CertifiedNonFiniteTargetImage and route exhaustion only to AlgorithmicHardCase, FiniteResourceFailure, or CertificateDesignGap?
4. Are authorization hash, source hash, child message hash, and plan hash tamper/deletion challenges operational?
5. Does the implementation avoid overclaiming non-production F4, narrow helpers, or debug quotient/action handles?
6. Can P8d close MECH-008, without closing P8 umbrella, P9/P10, final composition, roots, exact-image, public orchestration, performance readiness, or final acceptance?

Return exactly one status: PASS, FAIL_FIXABLE, FAIL_BLOCKING, or USER_DECISION_REQUIRED. Include reviewed R-IDs/MECHs, files inspected, commands/evidence inspected, findings, residual risks, and exact claim ceiling. Do not mark R-IDs VERIFIED.
