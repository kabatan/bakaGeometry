# Spec Registry

Purpose: active spec index.
Status: maintained navigation record.
Authority: non-authoritative; Base Specs control implementation correctness.

This registry is an index only. It is not an authority over any Base Spec.

## Active Change Specs

| Spec ID | Status | Scope | Base Spec | Plan | Source Map |
| --- | --- | --- | --- | --- | --- |
| `CW-ARC-DTP-Q-CANDIDATE-COVER` | Active for Guardian P12 solver integration; P11 local review passed | `geosolver-core` target-value solver | `docs/ai/changes/cw-arc-dtp-q/BASE_SPEC.md` | `docs/ai/changes/cw-arc-dtp-q/PLAN.md` | `docs/ai/changes/cw-arc-dtp-q/source_map.md` |

## Import Record

- Package: `C:\Users\bakat\Downloads\CW_ARC_DTP_Q_GUARDIAN_PACKAGE.zip`
- Package SHA-256: `D4B235115AE245417014C7B5A50141402DEBFA7C24CF64AC20A4F83A3A12DC6E`
- Imported files:
  - `docs/ai/changes/cw-arc-dtp-q/BASE_SPEC.md`
  - `docs/ai/changes/cw-arc-dtp-q/PLAN.md`
  - `docs/ai/changes/cw-arc-dtp-q/REVIEWER_PROMPTS.md`
  - `docs/ai/changes/cw-arc-dtp-q/PACKAGE_README.md`
- Added original source:
  - `docs/ai/changes/cw-arc-dtp-q/sources/cw_arc_dtp_q_revised_spec_v2.md`
  - Source SHA-256: `F761604CE25C01994108A802D7262EDC6D17185873AAFA76719F3CBA3013F653`

## Authority Rules

- `BASE_SPEC.md` is the implementation correctness authority once admitted for a phase.
- `PLAN.md` maps phase order, R-IDs, MECHs, acceptance gates, and reviewer prompts.
- `source_map.md` names source classes and source-fidelity boundaries; it does not replace source text.
- `ACTIVE_CONTEXT.md`, registry rows, evidence files, reviews, summaries, and agent reports are navigation or evidence only.
- No R-ID is verified merely because it appears in this registry.
