Guardian boundary review for RGDTPK-Q-v4-core P8c only.

Workspace: C:\Users\bakat\OneDrive\ドキュメント\bakaGeometry

Scope:
- Review Plan P8c only: TargetActionKrylov kernel with VerifiedCharacteristicSupportCoverage only.
- Required R-IDs: RGQ-021, RGQ-044, RGQ-054. Mechanism: MECH-014 only.
- Do not evaluate or close P8 umbrella, P8d/P9/P10, final composition, root isolation, exact-image semantics, nonfinite certification, or acceptance readiness.

Read these source/plan files as authority:
- docs/ai/changes/RGDTPK-Q-v4-core/BASE_SPEC.md, at least RGQ-021, RGQ-044, RGQ-054, MECH-014, Appendix A section 19, and the Appendix override table entry for section 19.4.
- docs/ai/changes/RGDTPK-Q-v4-core/PLAN.md, P8c.
- docs/ai/changes/RGDTPK-Q-v4-core/REVIEWER_PROMPTS.md#P8c.
- docs/ai/changes/RGDTPK-Q-v4-core/PRIMITIVE_SCOPE_LEDGER.md, especially algebra/quotient.rs and algebra/krylov.rs.

Changed implementation files to inspect:
- geosolver-core/src/kernels/action_krylov.rs
- geosolver-core/src/planner/admission.rs
Relevant primitives:
- geosolver-core/src/algebra/krylov.rs
- geosolver-core/src/algebra/quotient.rs

Evidence files to inspect:
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P8c/test_first_failure.txt
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P8c/commands.txt
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P8c/command_outputs.txt
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P8c/static_scans.txt
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P8c/function_implementation_table.yaml
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P8c/notes.md

Verification already run by main agent:
- cargo fmt --manifest-path geosolver-core/Cargo.toml --check: pass after rustfmt
- cargo test --manifest-path geosolver-core/Cargo.toml p8c_ -- --nocapture: 5 passed
- cargo test --manifest-path geosolver-core/Cargo.toml krylov -- --nocapture: 8 passed
- cargo test --manifest-path geosolver-core/Cargo.toml p6_ -- --nocapture: 3 passed
- cargo test --manifest-path geosolver-core/Cargo.toml p7_ -- --nocapture: 9 passed
- cargo test --manifest-path geosolver-core/Cargo.toml p8a_ -- --nocapture: 6 passed
- cargo test --manifest-path geosolver-core/Cargo.toml p8b_ -- --nocapture: 7 passed
- cargo test --manifest-path geosolver-core/Cargo.toml: 141 passed
- git diff --check: exit 0, CRLF warnings only

Review questions:
1. Does P8c implement TargetActionKrylov admission/planning/execution/replay without relying on debug explicit handles or externally injected self-certifying action columns?
2. Is the only support-producing coverage proof VerifiedCharacteristicSupportCoverage?
3. Is the emitted relation exactly the characteristic support polynomial of the verified target action matrix, with exact Cayley-Hamilton verification in the primitive path?
4. Is the RGQ-054 undercoverage regression present and meaningful?
5. Are no-coordinate-root/RUR export tests present?
6. Are plan hash, authorization hash, source hash, and child message hash boundaries materially checked?

Return:
- RESULT: PASS or FAIL_FIXABLE or FAIL_BLOCKING.
- Findings with file/line references.
- Exact claim ceiling if PASS.
- Any forbidden claims that must still be avoided.

Do not mark R-IDs VERIFIED. Reviewers do not grant implementation/deletion/shell approval.
