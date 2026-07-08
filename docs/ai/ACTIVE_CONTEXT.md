# Active Context

Purpose: operational hot context.
Status: active for current Guardian phase.
Authority: non-authoritative; use `BASE_SPEC.md` and `PLAN.md` for correctness.

Status: P1-P13 spec-gap fix F1-F5 implemented locally; F6 spec, quality, and Guardian boundary review passed for the narrow scope.

Current spec: `CW-ARC-DTP-Q-FULL-V3`
Base Spec: `docs/ai/changes/cw-arc-dtp-q/BASE_SPEC.md`
Plan: `docs/ai/changes/cw-arc-dtp-q/PLAN.md`
Reviewer prompts: `docs/ai/changes/cw-arc-dtp-q/REVIEWER_PROMPTS.md`
Source map: `docs/ai/changes/cw-arc-dtp-q/source_map.md`
Original algorithm source: `docs/ai/changes/cw-arc-dtp-q/sources/cw_arc_dtp_q_revised_spec_v2.md`
V3 failure analysis: `docs/ai/changes/cw-arc-dtp-q/sources/failure_analysis_and_fix_principles_v3.md`
V3 route checklist: `docs/ai/changes/cw-arc-dtp-q/sources/route_checklists_and_test_matrix_v3.md`

Current phase: P1-P13 spec-gap blocker fix before P14.

Current P7-P13 delta:
- `docs/ai/changes/cw-arc-dtp-q/P7_P13_ROUTE_CLOSURE_BASE_SPEC_DELTA.md`
- P7-P12 cannot pass from P4-P6 foundations alone.
- Each route needs route-forcing, no-fallback, exact-proof-gate, and tamper evidence.
- `FairProofSchedule::unbounded()` is not the same claim as top-level unbounded ideal execution.
- `FactorizationResult::ResourceFailure` and `Partial` must not be treated as `Complete`.
- `origin_evidence` remains ranking evidence only.

P7-P13 and P1-P13 implementation evidence:
- `docs/ai/changes/cw-arc-dtp-q/evidence/p7_p13_route_closure_evidence.md`
- `docs/ai/changes/cw-arc-dtp-q/evidence/p1_p13_spec_gap_fix_evidence.md`
- `docs/ai/changes/cw-arc-dtp-q/reviews/p1_p13_spec_gap_fix_2026-07-08.md`
- Latest local `cargo test -- --nocapture` passed after F1-F5 implementation and F6 blocker fixes.
- Latest local route-control evidence includes 31 route-control/no-fallback/tamper tests.
- `spec_verifier`, `quality_reviewer`, and `guardian_boundary_reviewer` passed for scoped P7-P13 and scoped P1-P13 spec-gap F1-F5 closure.

Current P1-P13 spec-gap fix source:
- `C:\Users\bakat\Downloads\CW_ARC_DTP_Q_P1_P13_SPEC_GAP_FIX_INSTRUCTIONS.md`
- SHA-256 `2D646EFA570B45365618B7506FEA925B8412D8686651231ED12810B111C5FE59`
- Local delta: `docs/ai/changes/cw-arc-dtp-q/P1_P13_SPEC_GAP_FIX_BASE_SPEC_DELTA.md`
- F1-F5 local implementation status: top-level unbounded execution, true sparse resultant data-flow, generic affine slicing, guarded-nonmonic tower, and regression/static gates implemented and locally tested.
- F6 status: spec verifier, quality reviewer, and Guardian boundary reviewer passed after blocker fixes.

P0 result:
- V3 Base Spec / Plan / Reviewer Prompt admitted as current authority
- `current_gap_inventory.md` admitted as current replacement-target inventory
- existing implementation gaps quarantined as replace-targets, not acceptable shortcuts

P0 review:
- `docs/ai/changes/cw-arc-dtp-q/reviews/p0_authority_admission_2026-07-08.md`

P1-P3 evidence and review:
- `docs/ai/changes/cw-arc-dtp-q/evidence/p1_p3_implementation.md`
- `docs/ai/changes/cw-arc-dtp-q/reviews/p1_p3_checkpoint_2026-07-08.md`
- `docs/ai/changes/cw-arc-dtp-q/evidence/p3_re_review_blocker_fixes.md`
- `docs/ai/changes/cw-arc-dtp-q/reviews/p3_re_review_blocker_2026-07-08.md`

P4-P6 evidence and review:
- `docs/ai/changes/cw-arc-dtp-q/evidence/p4_p6_implementation.md`
- `docs/ai/changes/cw-arc-dtp-q/reviews/p4_p6_checkpoint_2026-07-08.md`

Current P3 re-review blocker checklist:
- `ComponentUnionLcm` must not verify from a description-only source.
- `solve_target` must not return `CertifiedNoNonzeroTargetEliminant` while verifier returns only `CertificateDesignGap`.
- Evidence claims must be backed by production tests or source citations and downgraded otherwise.

Current ReadSet:
- `docs/ai/changes/cw-arc-dtp-q/BASE_SPEC.md`
- `docs/ai/changes/cw-arc-dtp-q/PLAN.md`
- `docs/ai/changes/cw-arc-dtp-q/REVIEWER_PROMPTS.md`
- `docs/ai/changes/cw-arc-dtp-q/P7_P13_ROUTE_CLOSURE_BASE_SPEC_DELTA.md`
- `docs/ai/changes/cw-arc-dtp-q/P1_P13_SPEC_GAP_FIX_BASE_SPEC_DELTA.md`
- `docs/ai/changes/cw-arc-dtp-q/source_map.md`
- `docs/ai/changes/cw-arc-dtp-q/evidence/current_gap_inventory.md`
- current production files named by the P0 search

Current edit scope:
- `docs/ai/SPEC_REGISTRY.md`
- `docs/ai/ACTIVE_CONTEXT.md`
- `docs/ai/changes/cw-arc-dtp-q/`
- P1-P13 production and test files changed under scoped user permission

Claim ceiling:
- Allowed: P1-P3 checkpoint, scoped P3 blocker fixes, scoped P4-P6 checkpoint, scoped P7-P13 route closure, and P1-P13 spec-gap F1-F5 local implementation were implemented, locally tested, and passed F6 spec/quality/boundary review.
- Forbidden: claiming the current implementation satisfies V3 final completion.
- Forbidden: claiming P14/P15/P16 are complete without a separate admitted scope and review.
- Forbidden: `SOURCE_FAITHFUL`, `VERIFIED`, `ACCEPTANCE_COMPLETE`, `PRODUCTION_SAFE`, or any requirement verified claim.

Known boundary:
- Existing `main` contains the prior bounded candidate-cover implementation and closure.
- V3 explicitly treats that bounded implementation as incomplete for the full algorithm.
- The repo is clean before this V3 import work; current changes are local and not yet committed.

Next action:
- Handle any P14+ work only under a separate admitted scope.
