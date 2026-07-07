# Active Context

Purpose: operational hot context.
Status: active for current Guardian phase.
Authority: non-authoritative; use `BASE_SPEC.md` and `PLAN.md` for correctness.

Status: Base Spec and Plan admitted for implementation; P5 through P14 passed Guardian review; P15 closure review passed with bounded closure claim.

Current spec: `CW-ARC-DTP-Q-CANDIDATE-COVER`
Base Spec: `docs/ai/changes/cw-arc-dtp-q/BASE_SPEC.md`
Plan: `docs/ai/changes/cw-arc-dtp-q/PLAN.md`
Reviewer prompts: `docs/ai/changes/cw-arc-dtp-q/REVIEWER_PROMPTS.md`
Source map: `docs/ai/changes/cw-arc-dtp-q/source_map.md`
Original algorithm source: `docs/ai/changes/cw-arc-dtp-q/sources/cw_arc_dtp_q_revised_spec_v2.md`

Current phase: P15 - Closure complete.

Open P15 R-IDs:
- all active R-IDs

Required reviewer before closing P15:
- completed: `guardian_boundary_reviewer` using `RP-CLOSURE` / final closure review

Current ReadSet:
- `docs/ai/changes/cw-arc-dtp-q/BASE_SPEC.md`
- `docs/ai/changes/cw-arc-dtp-q/PLAN.md`
- `docs/ai/changes/cw-arc-dtp-q/source_map.md`
- `docs/ai/changes/cw-arc-dtp-q/sources/cw_arc_dtp_q_revised_spec_v2.md`
- `docs/ai/changes/cw-arc-dtp-q/REVIEWER_PROMPTS.md`
- all P1-P14 evidence files
- changed production and test files

Current edit scope:
- `docs/ai/ACTIVE_CONTEXT.md`
- `docs/ai/changes/cw-arc-dtp-q/CLOSURE.md`
- P14/P15 evidence updates only if closure review requires correction

Claim ceiling:
- Allowed: closure packet may state a bounded candidate-cover core and exact-image fail-closed boundary if supported by fresh evidence and RP-CLOSURE.
- Forbidden: `SOURCE_FAITHFUL`, `VERIFIED`, `ACCEPTANCE_COMPLETE`, `PRODUCTION_SAFE`, or any R-ID verified claim unless the final closure reviewer explicitly supports that exact wording and the evidence supports it.
- Forbidden: claiming general `CertifiedExactTargetImage` completion; the exact-image classifier is conservative incomplete.

Known boundary:
- The original CW-ARC-DTP-Q revised spec v2 is stored in this repo.
- Failure-analysis artifacts named by the imported package are still not stored in this repo.
- Repository files are currently untracked in this workspace; no commit has been requested.

Next action:
- Hand back summary to user.
