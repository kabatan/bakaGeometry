You are guardian_boundary_reviewer for RGDTPK-Q-v4-core P3b after four FAIL_FIXABLE reviews and a targeted fourth-fix patch. Read-only review only; do not edit files. Workspace root is C:\Users\bakat\OneDrive\ドキュメント\bakaGeometry.

Review target: Plan P3b — modular arithmetic, CRT, rational reconstruction, sparse/dense matrices, and modular linear solving.

Prior failed reviews:
- docs/ai/changes/RGDTPK-Q-v4-core/reviews/P3b/20260704-235154Z/response.md
- docs/ai/changes/RGDTPK-Q-v4-core/reviews/P3b/20260705-000419Z/response.md
- docs/ai/changes/RGDTPK-Q-v4-core/reviews/P3b/20260705-001213Z/response.md
- docs/ai/changes/RGDTPK-Q-v4-core/reviews/P3b/20260705-002359Z/response.md

Current required fix from the fourth review:
1. Track whether `stable_rank >= stable_rank_after.max(1)` was achieved.
2. Return no reconstructed basis/solution candidates, or an explicit non-handoff result, when `max_primes` is exhausted before stability.
3. Add homogeneous and inhomogeneous regressions where profiles do not stabilize before `max_primes`, asserting no CRT/reconstruction candidate is produced.

Implementation note to inspect, not trust:
- `solve_homogeneous_modular` and `solve_inhomogeneous_modular` now use `stability_achieved` before producing `reconstructed_*_candidate` values.
- Added regressions:
  - `algebra::linear_solve::tests::homogeneous_does_not_reconstruct_without_stability`
  - `algebra::linear_solve::tests::inhomogeneous_does_not_reconstruct_without_stability`
- Fresh targeted evidence reports `cargo test ... algebra::linear_solve -- --nocapture`: 8 passed.
- Fresh full evidence reports `cargo test ... -- --nocapture`: 54 passed.

Required files to read:
- docs/ai/changes/RGDTPK-Q-v4-core/BASE_SPEC.md, at least RGQ-010, RGQ-019, RGQ-020, RGQ-025, plus Appendix A sections 10.3 through 10.7.
- docs/ai/changes/RGDTPK-Q-v4-core/PLAN.md, P3b section and general execution/review rules.
- docs/ai/changes/RGDTPK-Q-v4-core/REVIEW_ARCHIVE_SCHEMA.md.
- docs/ai/changes/RGDTPK-Q-v4-core/REVIEW_SUMMARY_SCHEMA.yaml.
- docs/ai/changes/RGDTPK-Q-v4-core/schemas/evidence_manifest.schema.yaml.
- docs/ai/changes/RGDTPK-Q-v4-core/REVIEWER_PROMPTS.md, P3b section and general instructions.
- geosolver-core/src/algebra/modular.rs.
- geosolver-core/src/algebra/crt.rs.
- geosolver-core/src/algebra/rational_reconstruction.rs.
- geosolver-core/src/algebra/sparse_matrix.rs.
- geosolver-core/src/algebra/dense_matrix.rs.
- geosolver-core/src/algebra/linear_solve.rs.
- geosolver-core/src/types/matrix.rs.
- geosolver-core/src/types/rational.rs.
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P3b/commands.txt.
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P3b/command_outputs.txt.
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P3b/static_scans.txt.
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P3b/function_implementation_table.yaml.
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P3b/notes.md.
- docs/ai/changes/RGDTPK-Q-v4-core/reviews/P3b/20260705-002359Z/review_summary.yaml.
- docs/ai/changes/RGDTPK-Q-v4-core/reviews/P3b/20260705-002359Z/evidence_manifest.yaml.

Check:
- Every selected modular solve prime is screened against all relevant matrix coefficients, and inhomogeneous solve screens RHS coefficients too.
- Modular solve traces include pivot-column rank profile.
- Homogeneous and inhomogeneous modular solving stabilize on rank profile, not rank alone, before CRT/reconstruction handoff.
- Homogeneous and inhomogeneous reconstruction use only the final stable rank-profile suffix, not stale samples before a profile change.
- Homogeneous and inhomogeneous reconstruction is suppressed when `max_primes` is exhausted before the configured stability threshold.
- There are regressions for same-rank/different-pivot-profile, inhomogeneous stable-suffix reconstruction, and no reconstruction before configured stability.
- Modular reduction uses exact rational-to-Fp arithmetic and no floating arithmetic.
- CRT and rational reconstruction use exact integer arithmetic, and unstable/nonunique reconstruction fails.
- Sparse/dense rank and nullspace are exact over Fp.
- Modular linear solving exposes rank/nullspace/solve traces and rationally reconstructed candidates, but does not certify a relation over Q.
- Static scans do not show floating exact paths, geometry/fixture/answer dispatch, ordinary Unsupported, or modular proof overclaim.

Return exactly one leading status line: PASS, FAIL_FIXABLE, FAIL_BLOCKING, or USER_DECISION_REQUIRED.

Then include reviewed R-IDs and MECHs, files inspected, evidence and commands inspected, algorithmic sufficiency judgment, phase-specific checks, forbidden/fail-condition scan judgment, raw response consistency implications for review_summary.yaml, unresolved risks, and required fixes if any.
