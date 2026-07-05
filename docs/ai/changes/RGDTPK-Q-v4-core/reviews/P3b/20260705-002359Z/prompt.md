You are guardian_boundary_reviewer for RGDTPK-Q-v4-core P3b after three FAIL_FIXABLE reviews. Read-only review only; do not edit files. Workspace root is C:\Users\bakat\OneDrive\ドキュメント\bakaGeometry.

Review target: Plan P3b — modular arithmetic, CRT, rational reconstruction, sparse/dense matrices, and modular linear solving.

Prior failed reviews:
- docs/ai/changes/RGDTPK-Q-v4-core/reviews/P3b/20260704-235154Z/response.md
- docs/ai/changes/RGDTPK-Q-v4-core/reviews/P3b/20260705-000419Z/response.md
- docs/ai/changes/RGDTPK-Q-v4-core/reviews/P3b/20260705-001213Z/response.md

Current required fix from the third review:
1. Reset or filter `basis_samples` and `solution_samples` whenever `pivot_columns` changes.
2. Reconstruct only from the final stable pivot-profile suffix.
3. Extend regression coverage to assert stable-suffix reconstruction, including inhomogeneous coverage.

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

Check:
- Every selected modular solve prime is screened against all relevant matrix coefficients, and inhomogeneous solve screens RHS coefficients too.
- Modular solve traces include pivot-column rank profile.
- Homogeneous and inhomogeneous modular solving stabilize on rank profile, not rank alone, before CRT/reconstruction handoff.
- Homogeneous and inhomogeneous reconstruction use only the final stable rank-profile suffix, not stale samples before a profile change.
- There are regressions for same-rank/different-pivot-profile and inhomogeneous stable-suffix reconstruction.
- Modular reduction uses exact rational-to-Fp arithmetic and no floating arithmetic.
- CRT and rational reconstruction use exact integer arithmetic, and unstable/nonunique reconstruction fails.
- Sparse/dense rank and nullspace are exact over Fp.
- Modular linear solving exposes rank/nullspace/solve traces and rationally reconstructed candidates, but does not certify a relation over Q.
- Static scans do not show floating exact paths, geometry/fixture/answer dispatch, ordinary Unsupported, or modular proof overclaim.

Return exactly one leading status line: PASS, FAIL_FIXABLE, FAIL_BLOCKING, or USER_DECISION_REQUIRED.

Then include reviewed R-IDs and MECHs, files inspected, evidence and commands inspected, algorithmic sufficiency judgment, phase-specific checks, forbidden/fail-condition scan judgment, raw response consistency implications for review_summary.yaml, unresolved risks, and required fixes if any.
