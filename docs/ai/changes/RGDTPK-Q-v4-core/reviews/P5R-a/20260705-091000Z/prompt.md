P5R-a re-review for RGDTPK-Q-v4-core. Scope: P5R-a / P5R-RGQ-065, P5R-RGQ-066, P5R-RGQ-071 evidence rebind and claim consistency only. Inspect files/evidence, not summaries as authority.

Required read set:
- docs/ai/changes/RGDTPK-Q-v4-core/P5R_BASE_SPEC_AMENDMENT.md (P5R-RGQ-065, 066, 071, 072)
- docs/ai/changes/RGDTPK-Q-v4-core/P5R_PLAN.md (P5R-a)
- docs/ai/changes/RGDTPK-Q-v4-core/P5R_REVIEWER_PROMPTS.md#P5R-a
- docs/ai/ACTIVE_CONTEXT.md
- docs/ai/changes/RGDTPK-Q-v4-core/CLOSURE.md
- docs/ai/changes/RGDTPK-Q-v4-core/P6_READINESS.md
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P5R-a/*
- existing P5 evidence/review archive only to classify as historical/superseded if needed

Previous P5R-a review failed for missing claim_consistency_matrix, missing rerun/rebinding of P5 static scans, and dirty/untracked packet not commit-bound. Fixes since then:
- evidence/P5R-a/claim_consistency_matrix.yaml now exists and states claim ceiling PARTIAL_MECHANISM_READY:MECH-004 plus negative claims.
- P5 graph-focused tests rerun on current code: 9 passed, appended in command_outputs.
- Full crate tests rerun on current code after P5R remediation: 106 passed, appended in command_outputs.
- cargo fmt --check rerun and passed.
- P5 static scans rerun, including a corrected forbidden-marker regex; active closure/readiness forbidden-string scan classifies historical pre-first-commit archives separately.
- Active CLOSURE/ACTIVE_CONTEXT/P6_READINESS state no planner/kernel/candidate-cover/exact-image/public/performance/acceptance completion.

Important: the P5R packet is still intentionally uncommitted because P5R-a..f review archives and final P5R-f updates must be included together in the final remediation commit. Judge whether P5R-a is closable as a subphase with final commit binding deferred to P5R-f, or identify any blocker that must be fixed before archive/commit. Do not grant P6 admission. Do not mark R-IDs VERIFIED.

Return YAML-like PASS/FAIL with blockers, files inspected, evidence/commands inspected or run, and forbidden claims.
