# Active Context

Purpose: operational hot context.
Status: active for current Guardian phase.
Authority: non-authoritative; use `BASE_SPEC.md` and `PLAN.md` for correctness.

Status: P1-P3 checkpoint closed narrowly; P4 is not started.

Current spec: `CW-ARC-DTP-Q-FULL-V3`
Base Spec: `docs/ai/changes/cw-arc-dtp-q/BASE_SPEC.md`
Plan: `docs/ai/changes/cw-arc-dtp-q/PLAN.md`
Reviewer prompts: `docs/ai/changes/cw-arc-dtp-q/REVIEWER_PROMPTS.md`
Source map: `docs/ai/changes/cw-arc-dtp-q/source_map.md`
Original algorithm source: `docs/ai/changes/cw-arc-dtp-q/sources/cw_arc_dtp_q_revised_spec_v2.md`
V3 failure analysis: `docs/ai/changes/cw-arc-dtp-q/sources/failure_analysis_and_fix_principles_v3.md`
V3 route checklist: `docs/ai/changes/cw-arc-dtp-q/sources/route_checklists_and_test_matrix_v3.md`

Current phase: P1-P3 checkpoint closed; awaiting scoped permission before P4.

P0 result:
- V3 Base Spec / Plan / Reviewer Prompt admitted as current authority
- `current_gap_inventory.md` admitted as current replacement-target inventory
- existing implementation gaps quarantined as replace-targets, not acceptable shortcuts

P0 review:
- `docs/ai/changes/cw-arc-dtp-q/reviews/p0_authority_admission_2026-07-08.md`

P1-P3 evidence and review:
- `docs/ai/changes/cw-arc-dtp-q/evidence/p1_p3_implementation.md`
- `docs/ai/changes/cw-arc-dtp-q/reviews/p1_p3_checkpoint_2026-07-08.md`

Current ReadSet:
- `docs/ai/changes/cw-arc-dtp-q/BASE_SPEC.md`
- `docs/ai/changes/cw-arc-dtp-q/PLAN.md`
- `docs/ai/changes/cw-arc-dtp-q/REVIEWER_PROMPTS.md`
- `docs/ai/changes/cw-arc-dtp-q/source_map.md`
- `docs/ai/changes/cw-arc-dtp-q/evidence/current_gap_inventory.md`
- current production files named by the P0 search

Current edit scope:
- `docs/ai/SPEC_REGISTRY.md`
- `docs/ai/ACTIVE_CONTEXT.md`
- `docs/ai/changes/cw-arc-dtp-q/`
- P1-P3 production and test files changed under scoped user permission

Claim ceiling:
- Allowed: P1-P3 checkpoint was implemented, reviewed, and local tests pass.
- Forbidden: claiming the current implementation satisfies V3 final completion.
- Forbidden: `SOURCE_FAITHFUL`, `VERIFIED`, `ACCEPTANCE_COMPLETE`, `PRODUCTION_SAFE`, or any requirement verified claim.

Known boundary:
- Existing `main` contains the prior bounded candidate-cover implementation and closure.
- V3 explicitly treats that bounded implementation as incomplete for the full algorithm.
- The repo is clean before this V3 import work; current changes are local and not yet committed.

Next action:
- Request scoped implementation permission before starting P4.
