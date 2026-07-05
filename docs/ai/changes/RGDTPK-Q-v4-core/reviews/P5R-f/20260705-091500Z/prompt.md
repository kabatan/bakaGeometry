P5R-f final audit review for RGDTPK-Q-v4-core. Scope: P5R-f / P5R-RGQ-065 through P5R-RGQ-072 closure gate before P6. Inspect source/docs/evidence, not summaries as authority.

Required read set:
- docs/ai/changes/RGDTPK-Q-v4-core/P5R_BASE_SPEC_AMENDMENT.md
- docs/ai/changes/RGDTPK-Q-v4-core/P5R_PLAN.md (P5R-f)
- docs/ai/changes/RGDTPK-Q-v4-core/P5R_REVIEWER_PROMPTS.md#P5R-f
- docs/ai/ACTIVE_CONTEXT.md
- docs/ai/changes/RGDTPK-Q-v4-core/CLOSURE.md
- docs/ai/changes/RGDTPK-Q-v4-core/P6_READINESS.md
- docs/ai/changes/RGDTPK-Q-v4-core/PLAN.md P5R/P6/P8/P9 portions as needed
- docs/ai/changes/RGDTPK-Q-v4-core/PRIMITIVE_SCOPE_LEDGER.md
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P5R-f/*
- docs/ai/changes/RGDTPK-Q-v4-core/reviews/P5R-a/20260705-091000Z/*
- docs/ai/changes/RGDTPK-Q-v4-core/reviews/P5R-b/20260705-091100Z/*
- docs/ai/changes/RGDTPK-Q-v4-core/reviews/P5R-c/20260705-091200Z/*
- docs/ai/changes/RGDTPK-Q-v4-core/reviews/P5R-d/20260705-091300Z/*
- docs/ai/changes/RGDTPK-Q-v4-core/reviews/P5R-e/20260705-091400Z/*
- geosolver-core/src/algebra/f4.rs
- geosolver-core/src/algebra/elimination.rs
- geosolver-core/src/preprocess/linear_affine.rs
- geosolver-core/src/preprocess/compression.rs
- geosolver-core/src/algebra/quotient.rs
- geosolver-core/src/algebra/krylov.rs

Check all P5R-f requirements:
- all P5R-a through P5R-e PASS archives exist and their review/evidence schemas validate;
- P5R-f fresh command evidence includes fmt check, full tests, P5R-specific tests, and static scans;
- fake F4 path cannot be production LocalF4 and cannot satisfy later Universal F4 claim;
- guarded rational affine no longer narrows to polynomial quotient only, and unsafe no-witness case is not rejected as Unsupported/InvalidInput;
- production TargetActionKrylov cannot use injected self-certifying debug handles;
- primitive scope ledger blocks P6/P8/P9 overclaims;
- active CLOSURE, ACTIVE_CONTEXT, and P6_READINESS agree on claim ceiling PARTIAL_MECHANISM_READY:MECH-004 and negative claims;
- active closing evidence/review summaries do not contain the forbidden P5R-RGQ-066 precommit/documentation-only strings, while historical P0-P5 archives are classified as historical;
- P6_READINESS explicit answers are sufficient for P6 to begin after this review archive and final remediation commit, but do not claim P6 has started.

Return YAML-like PASS/FAIL with blockers, files inspected, commands/evidence inspected, and forbidden claims. Do not mark R-IDs VERIFIED. If PASS, scope it only to P5R remediation closure readiness before P6; do not claim candidate-cover, exact-image, public orchestration, performance readiness, or final acceptance.
