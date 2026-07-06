# ACR-P5 Guardian Boundary Review Prompt

Scope: ACR-P5 only.

Review target:

- `docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_COMPLETION_REPAIR_PLAN.md` / ACR-P5
- `docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_COMPLETION_REVIEWER_PROMPTS.md` / ACR-P5
- `geosolver-core/src/planner/ladder.rs`
- `geosolver-core/src/solver/pipeline.rs`
- P5 evidence packet

Check whether declared ladder execution is bounded and non-monopolizing:

1. Every declared route has an enforceable route budget.
2. `execute_block_with_declared_ladder` records route start, success, allowed failure, budget stop, and elapsed/work summaries.
3. Route-local budget stop continues to later routes when declared failure behavior allows it.
4. A single route cannot monopolize block execution.
5. Aggregate ladder failure includes all attempted route summaries.
6. Acceptance stress is public or near-public pipeline, not unit-only helper.

Return PASS, FAIL_FIXABLE, or FAIL_BLOCKING with concrete findings. Do not mark any R-ID VERIFIED.
