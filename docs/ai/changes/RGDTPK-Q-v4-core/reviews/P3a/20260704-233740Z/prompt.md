You are guardian_boundary_reviewer for RGDTPK-Q-v4-core P3a. Read-only review only; do not edit files.

Review target: Plan P3a — Monomial orders, polynomial ops, reductions, exact membership verification.

Required files to read:
- docs/ai/changes/RGDTPK-Q-v4-core/BASE_SPEC.md, at least RGQ-010 and RGQ-027 plus Appendix A sections 10.1, 10.2, and 10.8.
- docs/ai/changes/RGDTPK-Q-v4-core/PLAN.md, P3a section and general execution/review rules.
- docs/ai/changes/RGDTPK-Q-v4-core/REVIEW_ARCHIVE_SCHEMA.md.
- docs/ai/changes/RGDTPK-Q-v4-core/REVIEW_SUMMARY_SCHEMA.yaml.
- docs/ai/changes/RGDTPK-Q-v4-core/schemas/evidence_manifest.schema.yaml.
- docs/ai/changes/RGDTPK-Q-v4-core/REVIEWER_PROMPTS.md, P3a section and general instructions.
- geosolver-core/src/algebra/monomial_order.rs.
- geosolver-core/src/algebra/polynomial_ops.rs.
- geosolver-core/src/algebra/normal_form.rs.
- geosolver-core/src/types/polynomial.rs.
- geosolver-core/src/types/monomial.rs.
- geosolver-core/src/types/rational.rs.
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P3a/commands.txt.
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P3a/command_outputs.txt.
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P3a/static_scans.txt.
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P3a/function_implementation_table.yaml.
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P3a/notes.md.

Check:
- `leading_term`, `s_polynomial`, `reduce_by_set`, `content_primitive_part`, `normal_form`, and `verify_membership_by_certificate` are actually implemented for P3a, not only scaffolding.
- Reductions and S-polynomials account for leading coefficients over Q.
- `verify_membership_by_certificate` reconstructs `Σ multiplier_i * relations[relation_id_i] - g` exactly and does not accept by hash equality or unconditional success.
- Tests include both correct and incorrect membership certificates, and a coefficient-sensitive reduction case.
- P3a makes no candidate-cover, exact-image, root-isolation, global support, or full solver completion claim.
- Evidence is fresh after the P3a code changes.
- Static scans do not show P3a fail-condition patterns or geometry/fixture/answer dispatch.

Return exactly one leading status line: PASS, FAIL_FIXABLE, FAIL_BLOCKING, or USER_DECISION_REQUIRED.

Then include:
- reviewed R-IDs and MECHs;
- files inspected;
- evidence and commands inspected;
- algorithmic sufficiency judgment;
- phase-specific checks;
- forbidden/fail-condition scan judgment;
- raw response consistency implications for a review_summary.yaml;
- unresolved risks and required fixes, if any.
