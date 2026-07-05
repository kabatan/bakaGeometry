You are guardian_boundary_reviewer for RGDTPK-Q-v4-core P3d after one FAIL_FIXABLE review. Read-only review only; do not edit files. Workspace root is C:\Users\bakat\OneDrive\ドキュメント\bakaGeometry.

Review target: Plan P3d — sparse resultant and specialization/interpolation primitives.

Prior failed review:
- docs/ai/changes/RGDTPK-Q-v4-core/reviews/P3d/20260705-005637Z/response.md

Current required fixes from that review:
1. Add eliminate/keep boundary validation, including reject overlap and duplicate/noncanonical keep variables.
2. Make modular reduction in resultant certificate checking fallible, returning false rather than panicking on invalid/tampered trace primes.
3. Add negative tests for both issues.

Implementation notes to inspect, not trust:
- `resultant.rs` now has `validate_keep_variables` and `trace_prime_is_valid_for_poly`.
- Added regressions:
  - `algebra::resultant::tests::resultant_template_rejects_overlapping_or_duplicate_keep_variables`
  - `algebra::resultant::tests::resultant_certificate_rejects_tampered_trace_prime_without_panic`
- Fresh evidence reports `cargo test ... algebra::resultant -- --nocapture`: 5 passed.
- Fresh full evidence reports `cargo test ... -- --nocapture`: 64 passed.

Scope and claim boundary:
- P3d supports `RGQ-020`, `RGQ-025`, and `RGQ-043`.
- P3d continues `MECH-007` by adding algebra-level primitives only.
- P3d does not implement SparseResultantProjectionKernel or SpecializationInterpolationKernel admission/execution; P8b owns kernel integration.
- Candidate outputs from these primitives must remain untrusted until exact Q membership/elimination verification by later phases.
- Claim ceiling after P3d remains `SCAFFOLD_READY`.

Required files to read:
- docs/ai/changes/RGDTPK-Q-v4-core/BASE_SPEC.md, at least RGQ-020, RGQ-025, RGQ-043, RGQ-058, RGQ-059, Appendix A sections 10.12 and 10.13.
- docs/ai/changes/RGDTPK-Q-v4-core/PLAN.md, P3d section and general execution/review rules.
- docs/ai/changes/RGDTPK-Q-v4-core/REVIEW_ARCHIVE_SCHEMA.md.
- docs/ai/changes/RGDTPK-Q-v4-core/REVIEW_SUMMARY_SCHEMA.yaml.
- docs/ai/changes/RGDTPK-Q-v4-core/schemas/evidence_manifest.schema.yaml.
- docs/ai/changes/RGDTPK-Q-v4-core/REVIEWER_PROMPTS.md, P3d section and general instructions.
- geosolver-core/src/algebra/resultant.rs.
- geosolver-core/src/algebra/interpolation.rs.
- geosolver-core/src/algebra/modular.rs.
- geosolver-core/src/types/polynomial.rs.
- geosolver-core/src/result/status.rs.
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P3d/commands.txt.
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P3d/command_outputs.txt.
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P3d/static_scans.txt.
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P3d/function_implementation_table.yaml.
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P3d/notes.md.
- docs/ai/changes/RGDTPK-Q-v4-core/reviews/P3d/20260705-005637Z/review_summary.yaml.
- docs/ai/changes/RGDTPK-Q-v4-core/reviews/P3d/20260705-005637Z/evidence_manifest.yaml.

Check:
- The two prior blockers are actually fixed in code and tests.
- `support_sets` computes deterministic sparse monomial supports and hashes.
- `build_sparse_resultant_template` validates eliminate/keep boundaries and resource caps and does not allow non-keep variables into local resultant templates.
- `compute_resultant_relation` computes exact resultant candidates and records modular trace hashes without treating modular traces as proof of a Q relation.
- `verify_resultant_certificate` rejects tampered relation/template/modular trace evidence, recomputes exact certificate data, and returns false instead of panicking for invalid trace primes.
- `choose_specialization_points` and `specialize_polynomials` are deterministic and exact.
- `interpolate_sparse_coefficients` performs exact sparse coefficient interpolation for the implemented P3d primitive scope and rejects invalid samples.
- `verify_interpolated_relation` verifies samples by exact re-specialization and bad interpolation samples fail final verification.
- Resultant/interpolation outputs are candidate-only and do not produce `CertifiedCandidateCover`, `CertifiedExactTargetImage`, or `CertifiedNonFiniteTargetImage`.
- Static scans do not show coordinate roots, full coordinate RUR, global coordinate lex parametrization, solve-all-coordinates then target, global CAS/QE/CAD/homotopy, ordinary Unsupported, or placeholder/stub markers in P3d-owned files.

Return exactly one leading status line: PASS, FAIL_FIXABLE, FAIL_BLOCKING, or USER_DECISION_REQUIRED.

Then include reviewed R-IDs and MECHs, files inspected, evidence and commands inspected, algorithmic sufficiency judgment, phase-specific checks, forbidden/fail-condition scan judgment, raw response consistency implications for review_summary.yaml, unresolved risks, and required fixes if any.
