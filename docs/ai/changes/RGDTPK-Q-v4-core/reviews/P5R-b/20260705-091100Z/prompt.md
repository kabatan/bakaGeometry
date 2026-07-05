P5R-b review for RGDTPK-Q-v4-core. Scope: P5R-RGQ-067 only, fake F4/F4-by-name remediation. Inspect source and evidence, not summaries as authority.

Required read set:
- docs/ai/changes/RGDTPK-Q-v4-core/P5R_BASE_SPEC_AMENDMENT.md (P5R-RGQ-067, P5R-RGQ-072)
- docs/ai/changes/RGDTPK-Q-v4-core/P5R_PLAN.md (P5R-b)
- docs/ai/changes/RGDTPK-Q-v4-core/P5R_REVIEWER_PROMPTS.md#P5R-b
- geosolver-core/src/algebra/f4.rs
- geosolver-core/src/algebra/elimination.rs
- geosolver-core/src/algebra/mod.rs if relevant
- docs/ai/changes/RGDTPK-Q-v4-core/PRIMITIVE_SCOPE_LEDGER.md
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P5R-b/*

Check that the current Groebner-backed batch helper is not production F4 by name or selection path, that production dispatch rejects the non-production strategy, and that tests/evidence support Route B demotion. Return YAML-like PASS/FAIL, blockers, commands inspected/run, files inspected, and forbidden claims. Do not mark R-IDs VERIFIED and do not broaden to P5R closure.
