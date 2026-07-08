# Spec Registry

Purpose: active spec index.
Status: maintained navigation record.
Authority: non-authoritative; Base Specs control implementation correctness.

This registry is an index only. It is not an authority over any Base Spec.

## Active Change Specs

| Spec ID | Status | Scope | Base Spec | Plan | Source Map |
| --- | --- | --- | --- | --- | --- |
| `CW-ARC-DTP-Q-FULL-V3` | Active; P0 authority admission passed; P1-P3 checkpoint closed; P4 not started | `geosolver-core` target-value solver | `docs/ai/changes/cw-arc-dtp-q/BASE_SPEC.md` | `docs/ai/changes/cw-arc-dtp-q/PLAN.md` | `docs/ai/changes/cw-arc-dtp-q/source_map.md` |

## Current Import Record

- Package: `C:\Users\bakat\Downloads\CW_ARC_DTP_Q_FULL_GUARDIAN_PACKAGE_V3.zip`
- Package SHA-256: `2ADEB4370BA81496A9BE9023B67734559C773F86E35405ABA90DA8627156676B`
- Imported files:
  - `docs/ai/changes/cw-arc-dtp-q/BASE_SPEC.md`
  - `docs/ai/changes/cw-arc-dtp-q/PLAN.md`
  - `docs/ai/changes/cw-arc-dtp-q/REVIEWER_PROMPTS.md`
  - `docs/ai/changes/cw-arc-dtp-q/PACKAGE_README.md`
  - `docs/ai/changes/cw-arc-dtp-q/sources/failure_analysis_and_fix_principles_v3.md`
  - `docs/ai/changes/cw-arc-dtp-q/sources/route_checklists_and_test_matrix_v3.md`

## Superseded Import Record

- Prior package: `C:\Users\bakat\Downloads\CW_ARC_DTP_Q_GUARDIAN_PACKAGE.zip`
- Prior package SHA-256: `D4B235115AE245417014C7B5A50141402DEBFA7C24CF64AC20A4F83A3A12DC6E`
- Prior bounded closure remains in git history and in older evidence, but it is superseded by the V3 authority for future implementation work.
- Original source retained:
  - `docs/ai/changes/cw-arc-dtp-q/sources/cw_arc_dtp_q_revised_spec_v2.md`
  - Source SHA-256: `F761604CE25C01994108A802D7262EDC6D17185873AAFA76719F3CBA3013F653`

## Authority Rules

- `BASE_SPEC.md` is the current implementation correctness authority once admitted for a phase.
- `PLAN.md` maps phase order, acceptance gates, and reviewer prompts.
- `source_map.md` names source classes and source-fidelity boundaries; it does not replace source text.
- `ACTIVE_CONTEXT.md`, registry rows, evidence files, reviews, summaries, and agent reports are navigation or evidence only.
- No requirement is satisfied merely because it appears in this registry.
