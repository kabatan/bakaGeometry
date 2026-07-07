# Source Map

Purpose: source-class and source-location record.
Status: active for `CW-ARC-DTP-Q-CANDIDATE-COVER`.
Authority: non-authoritative; source text and admitted Base Spec control.

This file names source classes for the CW-ARC-DTP-Q change. It is not a summary and does not replace `BASE_SPEC.md`.

## Imported Package

- Package path at import time: `C:\Users\bakat\Downloads\CW_ARC_DTP_Q_GUARDIAN_PACKAGE.zip`
- Package SHA-256: `D4B235115AE245417014C7B5A50141402DEBFA7C24CF64AC20A4F83A3A12DC6E`
- Imported package files:
  - `BASE_SPEC.md`
  - `PLAN.md`
  - `REVIEWER_PROMPTS.md`
  - `PACKAGE_README.md`
- Original CW-ARC-DTP-Q v2 source:
  - `sources/cw_arc_dtp_q_revised_spec_v2.md`
  - SHA-256: `F761604CE25C01994108A802D7262EDC6D17185873AAFA76719F3CBA3013F653`

## Source Classes

| Source | Class | Scope | Current Location |
| --- | --- | --- | --- |
| CW-ARC-DTP-Q revised specification v2 | `EXACT` | Algorithm semantics and mathematical solver contract | `sources/cw_arc_dtp_q_revised_spec_v2.md` |
| Guardian failure analysis | `EXACT` | Generalized anti-simplification and anti-shell rules | Not present in repo at P0 |
| GeoSolver failure causes | `EXACT` | Avoiding heavy-fallback, narrow-slice, and name-only failures | Not present in repo at P0 |
| Rust API Guidelines | `REFERENCE_ONLY` | Naming and API convention guidance | External reference |
| Parnas / information-hiding references | `REFERENCE_ONLY` | Module boundary and representation-hiding guidance | External reference |

## Authority Boundary

- `BASE_SPEC.md` is the active implementation authority for this change after phase admission.
- `PLAN.md` controls phase sequencing and required review gates.
- This `source_map.md` records source classes only.
- `PACKAGE_README.md`, `REVIEWER_PROMPTS.md`, `ACTIVE_CONTEXT.md`, this file, reviews, evidence logs, and agent summaries do not override `BASE_SPEC.md`.

## Source-Fidelity Limitation

The original CW-ARC-DTP-Q revised specification v2 is available in this repo. The failure-analysis artifacts named by the imported Base Spec are not available in this repo at this checkpoint. Until source admission and closure reviews pass, no final `SOURCE_FAITHFUL` claim may be made.
