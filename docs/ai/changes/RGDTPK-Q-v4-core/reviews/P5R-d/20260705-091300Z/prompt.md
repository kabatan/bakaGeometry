P5R-d review for RGDTPK-Q-v4-core. Scope: P5R-RGQ-069 only, TargetActionKrylov quotient/action provenance after remediation. Inspect source and evidence, not summaries as authority.

Required read set:
- docs/ai/changes/RGDTPK-Q-v4-core/P5R_BASE_SPEC_AMENDMENT.md (P5R-RGQ-069, P5R-RGQ-072)
- docs/ai/changes/RGDTPK-Q-v4-core/P5R_PLAN.md (P5R-d)
- docs/ai/changes/RGDTPK-Q-v4-core/P5R_REVIEWER_PROMPTS.md#P5R-d
- geosolver-core/src/algebra/quotient.rs
- geosolver-core/src/algebra/krylov.rs
- geosolver-core/src/algebra/normal_form.rs
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P5R-d/*

Check that production handles are provenance-bound to authorized relations, basis/action normal-form certificates are independently verified and hash-checked, debug explicit handles cannot be used by production Krylov boundary, malicious/tampered action columns are rejected, and undercoverage is still rejected. Also check no coordinate roots/full coordinate RUR API was introduced. Return YAML-like PASS/FAIL, blockers, files inspected, evidence/commands inspected or run, and forbidden claims. Do not mark R-IDs VERIFIED and do not broaden to P5R closure.
