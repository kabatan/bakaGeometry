You are guardian_boundary_reviewer for RGDTPK-Q-v4-core P3b after a FAIL_FIXABLE review. Read-only review only; do not edit files. Workspace root is C:\Users\bakat\OneDrive\ドキュメント\bakaGeometry.

Review target: Plan P3b — modular arithmetic, CRT, rational reconstruction, sparse/dense matrices, and modular linear solving.

Prior failed review:
- docs/ai/changes/RGDTPK-Q-v4-core/reviews/P3b/20260704-235154Z/response.md

The required fixes from that review were:
1. Make every modular solve prime avoid all relevant matrix and RHS denominators and forbidden coefficient reductions, not just the first matrix-derived prime.
2. Implement Appendix A §10.7 CRT + rational reconstruction in modular solving, while still marking output as candidate-only pending exact Q verification by the caller.
3. Add tests for multi-prime avoidance, RHS denominator avoidance, and reconstruction/handoff behavior in modular solve.

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
- Modular reduction uses exact rational-to-Fp arithmetic and no floating arithmetic.
- CRT and rational reconstruction use exact integer arithmetic, and unstable/nonunique reconstruction fails.
- Sparse/dense rank and nullspace are exact over Fp.
- Modular linear solving exposes rank/nullspace/solve traces and rationally reconstructed candidates, but does not certify a relation over Q.
- Tests include deterministic prime selection, CRT round trip and incompatible failure, rational reconstruction success and nonunique failure, matrix rank/nullspace, multi-prime avoidance, RHS denominator avoidance, and candidate-only reconstruction/handoff behavior.
- Static scans do not show floating exact paths, geometry/fixture/answer dispatch, ordinary Unsupported, or modular proof overclaim.

Return exactly one leading status line: PASS, FAIL_FIXABLE, FAIL_BLOCKING, or USER_DECISION_REQUIRED.

Then include reviewed R-IDs and MECHs, files inspected, evidence and commands inspected, algorithmic sufficiency judgment, phase-specific checks, forbidden/fail-condition scan judgment, raw response consistency implications for review_summary.yaml, unresolved risks, and required fixes if any.
