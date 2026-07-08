# Source Map

Purpose: source-class and source-location record.
Status: active for `CW-ARC-DTP-Q-FULL-V3`.
Authority: non-authoritative; source text and admitted Base Spec control.

This file names source classes for the CW-ARC-DTP-Q full implementation change. It is not a summary and does not replace `BASE_SPEC.md`.

## Current Imported Package

- Package path at import time: `C:\Users\bakat\Downloads\CW_ARC_DTP_Q_FULL_GUARDIAN_PACKAGE_V3.zip`
- Package SHA-256: `2ADEB4370BA81496A9BE9023B67734559C773F86E35405ABA90DA8627156676B`
- Imported package files:
  - `01_BASE_SPEC_CW_ARC_DTP_Q_FULL_V3.md` -> `BASE_SPEC.md`
  - `02_PLAN_CW_ARC_DTP_Q_FULL_V3.md` -> `PLAN.md`
  - `03_REVIEWER_PROMPTS_CW_ARC_DTP_Q_FULL_V3.md` -> `REVIEWER_PROMPTS.md`
  - `README.md` -> `PACKAGE_README.md`
  - `00_FAILURE_ANALYSIS_AND_FIX_PRINCIPLES.md` -> `sources/failure_analysis_and_fix_principles_v3.md`
  - `04_ROUTE_CHECKLISTS_AND_TEST_MATRIX.md` -> `sources/route_checklists_and_test_matrix_v3.md`

## Retained Source

- Original CW-ARC-DTP-Q v2 source:
  - `sources/cw_arc_dtp_q_revised_spec_v2.md`
  - SHA-256: `F761604CE25C01994108A802D7262EDC6D17185873AAFA76719F3CBA3013F653`

## Source Classes

| Source | Class | Scope | Current Location |
| --- | --- | --- | --- |
| CW-ARC-DTP-Q revised specification v2 | `EXACT` | Mathematical algorithm semantics and solver contract | `sources/cw_arc_dtp_q_revised_spec_v2.md` |
| Full Guardian Package v3 Base Spec | `EXACT` | Current repository implementation authority and completion definition | `BASE_SPEC.md` |
| Full Guardian Package v3 Plan | `EXACT` | Required phase order and phase gates | `PLAN.md` |
| Full Guardian Package v3 Reviewer Prompts | `EXACT` | Required review prompts and review failure criteria | `REVIEWER_PROMPTS.md` |
| V3 failure analysis and fix principles | `EXACT` | Generalized anti-simplification and anti-shell rules | `sources/failure_analysis_and_fix_principles_v3.md` |
| V3 route checklist and test matrix | `EXACT` | Required route-forcing, tamper, and non-simplification matrix content | `sources/route_checklists_and_test_matrix_v3.md` |

## Authority Boundary

- `BASE_SPEC.md` is the active implementation authority for this V3 change after P0 admission.
- `PLAN.md` controls phase sequencing and required review gates.
- `REVIEWER_PROMPTS.md` controls reviewer packets.
- This `source_map.md` records source classes only.
- `PACKAGE_README.md`, `ACTIVE_CONTEXT.md`, this file, evidence logs, reviews, and agent summaries do not override `BASE_SPEC.md`.

## Source-Fidelity Limitation

The original CW-ARC-DTP-Q revised specification v2 and the V3 failure-analysis package sources are stored in this repo. If V3 Base Spec and original source appear to conflict, the agent must stop and create QuestionDebt instead of choosing the easier interpretation.
