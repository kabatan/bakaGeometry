Read-only Guardian boundary review for P12 MECH completion. Do not edit files.

Workspace: C:\Users\bakat\OneDrive\ドキュメント\bakaGeometry

Use AGENTS.md Guardian Runtime Contract and `using-spec-guardian` semantics. Review P12 for phase closure only after spec_verifier PASS and quality_reviewer PASS. Do not mark any R-ID VERIFIED. Do not accept claims for P13+, final readiness, source-faithful status, or acceptance-complete status.

Closure claim requested:
- Phase: P12 roots/decode/algebraic-number layer.
- MECH closure: close MECH-011 only.
- Claim ceiling after phase: PARTIAL_MECHANISM_READY:MECH-011.
- Relevant R-IDs/source anchors: RGQ-028, RGQ-034, RGQ-037, RGQ-048; MECH-011; PLAN.md P12.

Primary artifacts/evidence to inspect:
- docs/ai/changes/RGDTPK-Q-v4-core/BASE_SPEC.md (MECH-011 and relevant RGQ sections)
- docs/ai/changes/RGDTPK-Q-v4-core/PLAN.md (P12)
- docs/ai/changes/RGDTPK-Q-v4-core/REVIEWER_PROMPTS.md (P12 if present)
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P12/test_first_failure.txt
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P12/commands.txt
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P12/command_outputs.txt
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P12/static_scans.txt
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P12/function_implementation_table.yaml
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P12/notes.md
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P12/spec_verifier_fail_fixable.md
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P12/spec_verifier_pass_after_integration_remediation.md
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P12/quality_reviewer_pass_after_integration_remediation.md

Primary code files to inspect:
- geosolver-core/src/roots/{squarefree.rs,isolate.rs,decode.rs,algebraic_number.rs,mod.rs}
- geosolver-core/src/algebra/real_root.rs
- geosolver-core/src/types/rational.rs
- geosolver-core/src/compose/final_support.rs
- geosolver-core/src/verify/{replay.rs,run_certificate.rs}
- geosolver-core/tests/p12_roots_decode_integration.rs

Fresh local evidence rerun before this boundary review:
- cargo fmt --manifest-path geosolver-core\Cargo.toml; cargo test --manifest-path geosolver-core\Cargo.toml p12_ -- --nocapture => pass: 6 lib p12 tests + 1 integration p12 test.
- cargo test --manifest-path geosolver-core\Cargo.toml -- --nocapture => pass: 177 lib tests + 1 integration test + doc tests.
- cargo fmt --manifest-path geosolver-core\Cargo.toml -- --check => pass.
- git diff --check => exit 0; only CRLF warnings.
- no-stub/floating scan over roots/real_root/rational => no matches (rg exit 1).
- decode_candidates scan shows production call in geosolver-core/src/compose/final_support.rs plus definition and unit test.
- anchor scan includes roots APIs, algebraic root APIs, finalize_candidate_cover_result, verify_roots_and_candidates, P12 replay tamper test, and P12 integration test.

Spec/quality review results already obtained:
- spec_verifier PASS after remediation. It specifically confirmed support-producing integration, candidate omission/duplicate replay rejection, and production decode wiring.
- quality_reviewer PASS. It noted `Descartes` routes to exact Sturm, acceptable for P12 but not a distinct Descartes implementation.

Boundary review questions:
1. Is P12 phase closable with claim ceiling PARTIAL_MECHANISM_READY:MECH-011?
2. Are the prior FAIL_FIXABLE issues adequately remediated?
3. Does the implementation avoid overclaiming P13 exact-image semantics, P14 public orchestration, P15 acceptance, P16 final closure, or R-ID VERIFIED status?
4. Are required Guardian evidence and fresh command outputs sufficient to archive a PASS review?

Return exactly PASS / FAIL_FIXABLE / FAIL_BLOCKING with concise evidence and file/line references. If PASS, explicitly state residual limits and that no R-ID is verified.
