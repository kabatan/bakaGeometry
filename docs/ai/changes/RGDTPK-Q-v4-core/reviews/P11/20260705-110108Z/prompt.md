Guardian boundary re-review request for Plan P11 in C:\Users\bakat\OneDrive\ドキュメント\bakaGeometry after remediation. Read-only. Do not edit files. Do not mark any R-ID VERIFIED.

Review target:
- Approved Base Spec: docs/ai/changes/RGDTPK-Q-v4-core/BASE_SPEC.md
- Plan: docs/ai/changes/RGDTPK-Q-v4-core/PLAN.md P11
- Reviewer prompt: docs/ai/changes/RGDTPK-Q-v4-core/REVIEWER_PROMPTS.md#P11
- Active context: docs/ai/ACTIVE_CONTEXT.md
- P11 evidence: docs/ai/changes/RGDTPK-Q-v4-core/evidence/P11/

P11 Plan authority:
- Supports R-IDs: RGQ-027, RGQ-038, RGQ-040, RGQ-047, RGQ-052, RGQ-053.
- MECHs: closes MECH-010 and advances MECH-016.
- Tasks: KernelCertificate variants, exact verify_projection_message, exact verify_global_support, CoreRunCertificate/replay all hashes and invariant flags, tamper/deletion tests, no unconditional verifier success.

Previous boundary reviewer result was FAIL_FIXABLE for:
1. `compression_hash` and `hypergraph_hash` were carried and included in `run_hash`, but not recomputed/checked by replay.
2. `derive_core_invariant_flags` set final anti-dispatch/QE-CAD flags to literal `true` without P11 enforcement evidence.

Remediation to verify:
- `CoreRunCertificateInput` now carries concrete `compression_hash` and `hypergraph_hash`; `build_core_run_certificate` records those values instead of `p11-*-not-supplied` placeholders.
- `replay_run_certificate` recomputes `compressed.compressed_hash` and `build_relation_variable_hypergraph(&compressed).hypergraph_hash` and rejects mismatches.
- `p11_replay_fails_on_input_canonical_dag_plan_and_squarefree_tamper` now includes self-consistently rehashed compression/hash and hypergraph/hash tamper rejections.
- `derive_core_invariant_flags` no longer sets `no_geometry_dispatch`, `no_problem_id_dispatch`, `no_expected_answer_dispatch`, or `no_qe_cad` to bare true. These final-claim flags remain false at P11; replay requires only P11-supported subset through `p11_replay_enforced`.
- Static scan has no `CoreInvariantFlags::enforced`, `all_enforced`, bare true final invariant literals, or P11 compression/hypergraph placeholder sentinels under `geosolver-core/src/verify`.

Latest review/evidence:
- Spec verifier after boundary remediation: `RESULT: PASS` in docs/ai/changes/RGDTPK-Q-v4-core/evidence/P11/spec_verifier_pass_after_boundary_remediation.md.
- Quality reviewer after boundary remediation: `RESULT: PASS` in docs/ai/changes/RGDTPK-Q-v4-core/evidence/P11/quality_reviewer_pass_after_boundary_remediation.md.
- Focused `p11_` tests: 13 passed.
- Full `geosolver-core` tests: 171 passed plus doc-tests.
- `cargo fmt --check`: exit 0.
- `git diff --check`: exit 0 with only CRLF warnings.

Changed files to inspect for latest remediation:
- geosolver-core/src/verify/run_certificate.rs
- geosolver-core/src/verify/replay.rs
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P11/*

Please return exactly one of:
- `RESULT: PASS` if P11 may be closed, specifying claim ceiling: P11 closes MECH-010 and advances MECH-016 only, P12 may start, no R-ID VERIFIED/final readiness/source-faithful/acceptance-complete claim.
- `RESULT: FAIL_FIXABLE` with blockers tied to file/line/evidence.
- `RESULT: FAIL_BLOCKED` only if external/user input is required.
