# ACR-P6 Guardian Review Prompt

Scope: ACR-P6 only.

Review `UniversalTargetElimination` against:

- `docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_COMPLETION_REPAIR_BASE_SPEC.md`
- `docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_COMPLETION_REPAIR_PLAN.md` / ACR-P6
- `docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_COMPLETION_REVIEWER_PROMPTS.md` / ACR-P6
- `docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_P6_MECH_EVIDENCE.md`

Changed implementation files:

- `geosolver-core/src/kernels/universal_elimination.rs`
- `geosolver-core/src/planner/cost_model.rs`
- `geosolver-core/src/planner/kernel_plan.rs`
- `geosolver-core/src/solver/pipeline.rs`
- `geosolver-core/src/verify/certificates.rs`
- `geosolver-core/src/verify/replay.rs`
- `geosolver-core/src/verify/verify_message.rs`

Review requirements:

- Fail if Universal is merely last in the ladder and not a real bounded projector.
- Fail if internal stages do not carry route budgets and cost estimates.
- Fail if cost-prohibited dense or sparse stages can execute anyway.
- Fail if no generic stress shows Universal producing a verified projection message after internal
  stage failures/skips.
- Fail if Universal uses full coordinate roots, full coordinate RUR, or hidden global fallback.
- Fail if Universal returns hardcase while a feasible bounded internal route can produce a relation.
- Confirm the certificate/replay path includes attempted strategies, skipped cost-prohibited
  strategies, failed strategy hashes, chosen strategy, and exact relation verification.

Use fresh evidence only from the listed files and local commands if needed. Return PASS,
FAIL_FIXABLE, or FAIL_BLOCKING. Keep the claim ceiling at
`CANDIDATE_COVER_PIPELINE_PRESENT_BUT_ALGEBRAIC_COST_INCOMPLETE`; do not authorize final
candidate-cover readiness, source fidelity, full acceptance, or any later ACR phase.
