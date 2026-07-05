You are guardian_boundary_reviewer for RGDTPK-Q-v4-core P3c. Read-only review only; do not edit files. Workspace root is C:\Users\bakat\OneDrive\ドキュメント\bakaGeometry.

Review target: Plan P3c — local Groebner/F4 elimination APIs with exported-variable restriction.

Scope and claim boundary:
- P3c supports `RGQ-005`, `RGQ-010`, `RGQ-022`, and `RGQ-056`.
- P3c starts the local part of `MECH-008`; it does not close full UniversalTargetEliminationKernel execution.
- P3c must not claim coordinate root enumeration, full coordinate RUR, global coordinate lex parametrization, final target support acceptance, or nonfinite target-image certification.
- Claim ceiling after P3c remains `SCAFFOLD_READY`.

Required files to read:
- docs/ai/changes/RGDTPK-Q-v4-core/BASE_SPEC.md, at least RGQ-005, RGQ-010, RGQ-022, RGQ-056, Appendix A sections 10.9 through 10.11, and MECH-008.
- docs/ai/changes/RGDTPK-Q-v4-core/PLAN.md, P3c section and general execution/review rules.
- docs/ai/changes/RGDTPK-Q-v4-core/REVIEW_ARCHIVE_SCHEMA.md.
- docs/ai/changes/RGDTPK-Q-v4-core/REVIEW_SUMMARY_SCHEMA.yaml.
- docs/ai/changes/RGDTPK-Q-v4-core/schemas/evidence_manifest.schema.yaml.
- docs/ai/changes/RGDTPK-Q-v4-core/REVIEWER_PROMPTS.md, P3c section and general instructions.
- geosolver-core/src/algebra/groebner.rs.
- geosolver-core/src/algebra/f4.rs.
- geosolver-core/src/algebra/elimination.rs.
- geosolver-core/src/algebra/normal_form.rs.
- geosolver-core/src/algebra/monomial_order.rs.
- geosolver-core/src/algebra/polynomial_ops.rs.
- geosolver-core/src/result/status.rs.
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P3c/commands.txt.
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P3c/command_outputs.txt.
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P3c/static_scans.txt.
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P3c/function_implementation_table.yaml.
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P3c/notes.md.

Check:
- `groebner_elimination_basis` is local/bounded and does not become a global coordinate solver.
- S-pair/reduction certificate propagation is exact enough for P3c and every new Groebner remainder certificate is checked with exact membership reconstruction.
- `extract_elimination_generators` and certified extraction restrict outputs to `Q[keep]`.
- `f4_reduce_batch` performs exact reductions and records matrix trace dimensions.
- `f4_elimination_local` runs only a local elimination path and returns certified local elimination output.
- `eliminate_to_keep_variables` rejects overlapping keep/eliminate sets, dispatches only local strategies admitted by P3c, validates every exported generator is in `Q[keep]`, and validates every certificate exactly.
- Tests include Groebner certificate export, LocalF4 export, disjoint set validation, non-keep export rejection, and F4 batch-reduction trace coverage.
- Static scans do not show coordinate roots, full coordinate RUR, global coordinate lex parametrization, solve-all-coordinates then target, global CAS/QE/CAD/homotopy, nonfinite certification, ordinary Unsupported, or placeholder/stub markers.

Return exactly one leading status line: PASS, FAIL_FIXABLE, FAIL_BLOCKING, or USER_DECISION_REQUIRED.

Then include reviewed R-IDs and MECHs, files inspected, evidence and commands inspected, algorithmic sufficiency judgment, phase-specific checks, forbidden/fail-condition scan judgment, raw response consistency implications for review_summary.yaml, unresolved risks, and required fixes if any.
