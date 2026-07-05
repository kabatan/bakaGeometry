P5R-c re-review for RGDTPK-Q-v4-core. Scope: only P5R-RGQ-068 guarded rational affine semantics after remediation. Inspect source and evidence, not summaries as authority.

Required source/read set:
- docs/ai/changes/RGDTPK-Q-v4-core/P5R_BASE_SPEC_AMENDMENT.md (P5R-RGQ-068, P5R-RGQ-072)
- docs/ai/changes/RGDTPK-Q-v4-core/P5R_PLAN.md (P5R-c)
- docs/ai/changes/RGDTPK-Q-v4-core/P5R_REVIEWER_PROMPTS.md#P5R-c
- geosolver-core/src/preprocess/linear_affine.rs
- geosolver-core/src/preprocess/compression.rs
- geosolver-core/src/preprocess/saturation.rs
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P5R-c/*

Context: previous P5R-c review found algorithmic path sufficient but failed because the safe stress test did not assert exact provenance fields. Fixes since then:
- `guarded_rational_affine_non_polynomial_case_clears_denominator` now asserts exact original/transformed relation ids, pivot relation id, source witness relation ids, guard source ids, original/transformed relation hashes, recomputed transformation hash equality, and tamper hash inequality.
- `apply_rational_affine_substitution` now records the final transformed canonical relation hash, not the intermediate polynomial hash.
- Targeted tests were rerun and appended to evidence/P5R-c/command_outputs.txt; latest linear_affine and compression runs pass.

Return YAML-like result with PASS/FAIL, blocking findings, files inspected, commands run/inspected, and explicit forbidden claims. Do not mark R-IDs VERIFIED and do not broaden to full P5R closure.
