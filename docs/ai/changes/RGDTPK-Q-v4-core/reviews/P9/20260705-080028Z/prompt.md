You are guardian_boundary_reviewer for RGDTPK-Q-v4-core P9 after a prior FAIL_FIXABLE review and remediation. Read-only review only; do not edit files.

Workspace root: C:\Users\bakat\OneDrive\ドキュメント\bakaGeometry
Review target: Plan P9 only.

Scope and source anchors:
- docs/ai/changes/RGDTPK-Q-v4-core/BASE_SPEC.md: RGQ-023, RGQ-024, RGQ-033; Appendix A sections 21 and 22; MECH-007.
- docs/ai/changes/RGDTPK-Q-v4-core/PLAN.md: P9 only.
- docs/ai/changes/RGDTPK-Q-v4-core/REVIEWER_PROMPTS.md: P9 prompt.
- docs/ai/changes/RGDTPK-Q-v4-core/PRIMITIVE_SCOPE_LEDGER.md: regular_chain and norm_trace entries. This ledger is mandatory. Fail if single-chain regular-chain helpers or single-variable tower norm/trace helpers are still overclaimed as generic P9 completion.

Prior FAIL_FIXABLE findings to verify fixed:
1. RegularChainProjection dropped CompressedSystemQ guards and still relied on a single-chain/component helper, so component/guard/projection semantics were overclaimed.
2. NormTraceProjection accepted only exactly one non-exported algebraic variable and overclaimed the single-variable norm/trace helper as P9-ready.

Changed/evidence files to inspect:
- geosolver-core/src/algebra/regular_chain.rs
- geosolver-core/src/kernels/regular_chain_projection.rs
- geosolver-core/src/algebra/norm_trace.rs
- geosolver-core/src/kernels/norm_trace_projection.rs
- geosolver-core/src/planner/admission.rs
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P9/commands.txt
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P9/command_outputs.txt
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P9/function_implementation_table.yaml
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P9/static_scans.txt
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P9/notes.md
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P9/test_first_failure.txt
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P9/reviewer_fail_fix.md

Verification evidence reported by main agent after remediation:
- cargo fmt --manifest-path geosolver-core/Cargo.toml --check: pass.
- cargo test --manifest-path geosolver-core/Cargo.toml p9_ -- --nocapture: 6 passed, 0 failed.
- cargo test --manifest-path geosolver-core/Cargo.toml: 154 passed, 0 failed.
- Forbidden P9 shortcut scan over regular_chain_projection.rs, norm_trace_projection.rs, planner/admission.rs: no matches for geometry names, fixture/expected-answer strings, coordinate-root/full-coordinate/RUR solve paths, kernel-not-ready, or P9 placeholder text.
- git diff --check: exit 0 with CRLF conversion warnings only.

Review questions:
1. Does P9 now preserve regular-chain component/guard/projection semantics sufficiently for RGQ-023 and Appendix A section 21 within the admitted algebraic scope?
2. Does P9 now implement algebraic tower detection and exact norm relation verification sufficiently for RGQ-024 and Appendix A section 22, including a multi-step tower path instead of a single-variable-only overclaim?
3. Does P9 avoid geometry-name/fixture/expected-answer dispatch and avoid coordinate-root/full-coordinate/RUR production?
4. Does planner admission route RegularChainProjection and NormTraceProjection through P9 plan builders without hidden fallback?
5. If PASS, can P9 close remaining MECH-007 only? Explicitly exclude P10 final support composition, replay/certificate closure beyond P9, root isolation/decode, exact-image semantics, public orchestration, performance readiness, final acceptance, and marking any R-ID VERIFIED.

Return exactly one leading status line: PASS, FAIL_FIXABLE, FAIL_BLOCKING, or USER_DECISION_REQUIRED. Include files/line references inspected, findings, residual risks, reviewed R-IDs/MECHs, and exact claim ceiling. Do not mark R-IDs VERIFIED.
