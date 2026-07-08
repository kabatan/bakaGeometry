# Active Context

Purpose: operational hot context.
Status: active for current Guardian phase.
Authority: non-authoritative; use `BASE_SPEC.md` and `PLAN.md` for correctness.

Status: P7-P13 route closure implemented, locally tested, and reviewed under the admitted P7-P13 delta.

Current spec: `CW-ARC-DTP-Q-FULL-V3`
Base Spec: `docs/ai/changes/cw-arc-dtp-q/BASE_SPEC.md`
Plan: `docs/ai/changes/cw-arc-dtp-q/PLAN.md`
Reviewer prompts: `docs/ai/changes/cw-arc-dtp-q/REVIEWER_PROMPTS.md`
Source map: `docs/ai/changes/cw-arc-dtp-q/source_map.md`
Original algorithm source: `docs/ai/changes/cw-arc-dtp-q/sources/cw_arc_dtp_q_revised_spec_v2.md`
V3 failure analysis: `docs/ai/changes/cw-arc-dtp-q/sources/failure_analysis_and_fix_principles_v3.md`
V3 route checklist: `docs/ai/changes/cw-arc-dtp-q/sources/route_checklists_and_test_matrix_v3.md`

Current phase: P7-P13 route closure reviewed; P14+ requires separate scoped permission and review.

Current P7-P13 delta:
- `docs/ai/changes/cw-arc-dtp-q/P7_P13_ROUTE_CLOSURE_BASE_SPEC_DELTA.md`
- P7-P12 cannot pass from P4-P6 foundations alone.
- Each route needs route-forcing, no-fallback, exact-proof-gate, and tamper evidence.
- `FairProofSchedule::unbounded()` is not the same claim as top-level unbounded ideal execution.
- `FactorizationResult::ResourceFailure` and `Partial` must not be treated as `Complete`.
- `origin_evidence` remains ranking evidence only.

P7-P13 implementation evidence:
- `docs/ai/changes/cw-arc-dtp-q/evidence/p7_p13_route_closure_evidence.md`
- Latest local `cargo test` passed before reviewer handoff.
- Latest local `cargo test --lib test_support` passed 26 route-control/no-fallback/tamper tests.
- `spec_verifier`, `quality_reviewer`, and `guardian_boundary_reviewer` passed for scoped P7-P13.

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
- `docs/ai/changes/cw-arc-dtp-q/source_map.md`
- `docs/ai/changes/cw-arc-dtp-q/evidence/current_gap_inventory.md`
- current production files named by the P0 search

Current edit scope:
- `docs/ai/SPEC_REGISTRY.md`
- `docs/ai/ACTIVE_CONTEXT.md`
- `docs/ai/changes/cw-arc-dtp-q/`
- P1-P13 production and test files changed under scoped user permission

Claim ceiling:
- Allowed: P1-P3 checkpoint, scoped P3 blocker fixes, scoped P4-P6 checkpoint, and scoped P7-P13 route closure were implemented, locally tested, and reviewed.
- Forbidden: claiming the current implementation satisfies V3 final completion.
- Forbidden: `SOURCE_FAITHFUL`, `VERIFIED`, `ACCEPTANCE_COMPLETE`, `PRODUCTION_SAFE`, or any requirement verified claim.

Known boundary:
- Existing `main` contains the prior bounded candidate-cover implementation and closure.
- V3 explicitly treats that bounded implementation as incomplete for the full algorithm.
- The repo is clean before this V3 import work; current changes are local and not yet committed.

Next action:
- Ask for scoped permission before starting P14 or any later phase.
