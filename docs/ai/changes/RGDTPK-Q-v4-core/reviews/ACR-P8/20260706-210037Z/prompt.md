# ACR-P8 Reviewer Prompt

Scope: ACR-P8 sparse/lazy `TargetRelationSearch` only.

Use the ACR-P8 reviewer prompt in `ALG_COST_COMPLETION_REVIEWER_PROMPTS.md`.

Review goal: determine whether `DenseTotalDegree` is no longer the only production relation-search strategy and whether sparse/lazy target relation search can produce verified projection messages when dense TRS is cost-prohibited.

Required checks:

1. Sparse/lazy strategy must avoid dense total-degree support enumeration for large blocks.
2. Planner admission and cost estimates must be bound to the actual sparse schedule, not probe-only dimensions.
3. Sparse/lazy output must require exact Q membership verification before message construction.
4. Near-public pipeline stress must reach `CertifiedCandidateCover` when dense TRS is prohibited and sparse/lazy TRS is feasible.
5. Claim ceiling remains `CANDIDATE_COVER_PIPELINE_PRESENT_BUT_ALGEBRAIC_COST_INCOMPLETE`; no final readiness or exact-image claim is allowed.

Files to inspect:

- `geosolver-core/src/kernels/target_relation_search.rs`
- `geosolver-core/src/planner/admission.rs`
- `geosolver-core/src/planner/cost_model.rs`
- `geosolver-core/src/planner/kernel_plan.rs`
- `geosolver-core/src/solver/pipeline.rs`
- `docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_P8_MECH_EVIDENCE.md`

Required reviewer output:

- PASS, FAIL_FIXABLE, or FAIL_BLOCKING.
- Record whether sparse descriptors avoid dense total-degree enumeration.
- Record whether exact membership verification is enforced.
- Record whether public or near-public pipeline evidence exists.
- Do not grant ACR-P9/P10 closure, exact-image acceptance, source-fidelity, or final readiness.

