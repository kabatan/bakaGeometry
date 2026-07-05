You are guardian_boundary_reviewer for RGDTPK-Q-v4-core P3e. Read-only review only; do not edit files. Workspace root is C:\Users\bakat\OneDrive\ドキュメント\bakaGeometry.

Review target: Plan P3e — quotient/action handles and verified characteristic support coverage primitives.

Scope and claim boundary:
- P3e supports `RGQ-021`, `RGQ-044`, and `RGQ-054`.
- P3e implements primitive layer for `MECH-014`.
- P3e does not implement TargetActionKrylovKernel admission/execution or ProjectionMessage production; later P8c owns kernel integration.
- Claim ceiling after P3e remains `SCAFFOLD_READY`.

Required files to read:
- docs/ai/changes/RGDTPK-Q-v4-core/BASE_SPEC.md, at least RGQ-021, RGQ-044, RGQ-054, Appendix A sections 10.14, 10.15, and section 19.4 coverage hardening.
- docs/ai/changes/RGDTPK-Q-v4-core/PLAN.md, P3e section and general execution/review rules.
- docs/ai/changes/RGDTPK-Q-v4-core/REVIEW_ARCHIVE_SCHEMA.md.
- docs/ai/changes/RGDTPK-Q-v4-core/REVIEW_SUMMARY_SCHEMA.yaml.
- docs/ai/changes/RGDTPK-Q-v4-core/schemas/evidence_manifest.schema.yaml.
- docs/ai/changes/RGDTPK-Q-v4-core/REVIEWER_PROMPTS.md, P3e section and general instructions.
- geosolver-core/src/algebra/quotient.rs.
- geosolver-core/src/algebra/krylov.rs.
- geosolver-core/src/types/univariate.rs.
- geosolver-core/src/types/matrix.rs.
- geosolver-core/src/result/status.rs.
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P3e/commands.txt.
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P3e/command_outputs.txt.
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P3e/static_scans.txt.
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P3e/function_implementation_table.yaml.
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P3e/notes.md.

Check:
- `TargetQuotientHandle` exposes no coordinate roots or full coordinate RUR APIs.
- `build_target_relevant_quotient_handle` rejects coordinate-exporting handles and inconsistent basis/action dimensions.
- Normal form and variable multiplication are exact over Q.
- `block_krylov_sequence` cannot create accepted relations by itself.
- `recover_recurrence` may recover a recurrence, but `certify_krylov_coverage` accepts only `VerifiedCharacteristicSupportCoverage`.
- Target action matrix columns are materialized from `handle.multiply_by_variable` and checked against exact normal form.
- Characteristic polynomial computation is exact over Q.
- Cayley-Hamilton verification is exact as a matrix identity.
- `verify_annihilator` accepts only the verified characteristic support polynomial.
- The undercoverage regression exists: a single-vector Krylov sequence misses an eigenvalue and is rejected with `CertificateDesignGap`.
- Static scans do not show coordinate roots/RUR APIs, single-vector returned relation path, block Wiedemann/trace-power acceptance, unverified `S(M_T)=0`, certified solver statuses, floating paths, ordinary Unsupported, or placeholder/stub markers.

Return exactly one leading status line: PASS, FAIL_FIXABLE, FAIL_BLOCKING, or USER_DECISION_REQUIRED.

Then include reviewed R-IDs and MECHs, files inspected, evidence and commands inspected, algorithmic sufficiency judgment, phase-specific checks, forbidden/fail-condition scan judgment, raw response consistency implications for review_summary.yaml, unresolved risks, and required fixes if any.
