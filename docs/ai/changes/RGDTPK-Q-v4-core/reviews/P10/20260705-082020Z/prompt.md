You are guardian_boundary_reviewer for RGDTPK-Q-v4-core P10. Read-only review only; do not edit files.

Workspace root: C:\Users\bakat\OneDrive\ドキュメント\bakaGeometry
Review target: Plan P10 only.

Scope and source anchors:
- docs/ai/changes/RGDTPK-Q-v4-core/BASE_SPEC.md: RGQ-026, RGQ-032, RGQ-038, RGQ-045, RGQ-051, RGQ-057, RGQ-058; MECH-009 and MECH-015.
- BASE_SPEC Appendix A section 24.3 separator elimination and 24.4 final support.
- docs/ai/changes/RGDTPK-Q-v4-core/PLAN.md: P10 only.
- docs/ai/changes/RGDTPK-Q-v4-core/REVIEWER_PROMPTS.md: P10 prompt.

Review prompt requirements:
Check separator elimination uses only message relations, final support is target-only and verified, and nonfinite status requires RGQ-045 certificate. Check RGQ-057 Appendix overrides and RGQ-058 status/error mapping. Fail if no target relation, relation-search exhaustion, or local Universal failure can become CertifiedNonFiniteTargetImage without the required proof, or if status enum values are mixed into internal errors without an explicit reviewed mapping.

Changed/evidence files to inspect:
- geosolver-core/src/compose/message.rs
- geosolver-core/src/compose/compose.rs
- geosolver-core/src/compose/separator_elimination.rs
- geosolver-core/src/compose/final_support.rs
- geosolver-core/src/compose/mod.rs
- geosolver-core/src/result/status.rs and result/output.rs only as needed for RGQ-058 mapping.
- geosolver-core/src/kernels/target_relation_search.rs and universal_elimination.rs only as needed for nonfinite polarity.
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P10/commands.txt
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P10/command_outputs.txt
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P10/function_implementation_table.yaml
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P10/static_scans.txt
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P10/notes.md
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P10/test_first_failure.txt
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P10/spec_verifier_pass.md

Fresh verification reported by main agent:
- cargo fmt --manifest-path geosolver-core/Cargo.toml --check: pass.
- cargo test --manifest-path geosolver-core/Cargo.toml p10_ -- --nocapture: 4 passed, 0 failed.
- cargo test --manifest-path geosolver-core/Cargo.toml: 158 passed, 0 failed.
- git diff --check: exit 0 with CRLF conversion warnings only.
- Forbidden P10 shortcut scan: no matches for geometry names, fixture/expected-answer strings, coordinate-root/full-coordinate/RUR paths, QE/CAD, Unsupported/NotYetImplemented, local Universal nonfinite wording, or relation-search-failure-to-nonfinite wording.
- Spec verifier result: PASS. It found composition consumes only message relation generators after package/export checks, separator elimination builds a synthetic message-only system, final support is target-only and no-support is AlgorithmicHardCase, nonfinite status requires positive certificate and re-verification, and local TargetRelationSearch/Universal paths do not emit CertifiedNonFiniteTargetImage.

Review questions:
1. Does P10 satisfy RGQ-026 composition/final support within scope, using only message relations and target-direct separator elimination?
2. Does build_global_support_polynomial build target-only support and avoid turning no-support into nonfinite?
3. Does P10 satisfy RGQ-045/RGQ-051 by requiring a positive NonFiniteCertificate for CertifiedNonFiniteTargetImage, separate from relation-search exhaustion/local Universal failure?
4. Does RGQ-058 status mapping remain explicit, with CertifiedNonFiniteTargetImage produced through TargetSolveResult finalization after certificate verification rather than as a local SolverError?
5. If PASS, may P10 close MECH-009 and MECH-015 only? Explicitly exclude P11 replay/certificate closure beyond P10, P12 roots/decode, exact-image completion beyond the P10 real-nonfinite certificate function, public orchestration, performance readiness, final acceptance, source-faithful/acceptance-complete claims, and any R-ID VERIFIED claim.

Return exactly one leading status line: PASS, FAIL_FIXABLE, FAIL_BLOCKING, or USER_DECISION_REQUIRED. Include files/line references inspected, findings, residual risks, reviewed R-IDs/MECHs, and exact claim ceiling. Do not mark R-IDs VERIFIED.
