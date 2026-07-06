# ACR-P1 Guardian Boundary Review Prompt

Review phase: `ACR-P1`

Change: `RGDTPK-Q-v4-algebraic-cost-completion-repair-v1`

Use the ACR-P1 reviewer prompt from
`docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_COMPLETION_REVIEWER_PROMPTS.md`.

## Review Scope

Review `ALG_COST_SOURCE_SPEC_GAP_MAP.md` for completeness against v4 sections 1, 3, 4, 12, 13,
17, 18, 19, 20, 23, 24, 25, 30, 32, and 33.

## Files to Review

```text
docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_SOURCE_SPEC_GAP_MAP.md
docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_COMPLETION_REPAIR_BASE_SPEC.md
docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_COMPLETION_REPAIR_PLAN.md
docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_CURRENT_DEFECT_AUDIT.md
docs/ai/changes/RGDTPK-Q-v4-core/BASE_SPEC.md
geosolver-core/src/planner/cost_model.rs
geosolver-core/src/planner/ladder.rs
geosolver-core/src/planner/admission.rs
geosolver-core/src/planner/relation_schedule.rs
geosolver-core/src/kernels/target_relation_search.rs
geosolver-core/src/kernels/sparse_resultant.rs
geosolver-core/src/kernels/action_krylov.rs
geosolver-core/src/kernels/specialization_interpolation.rs
geosolver-core/src/kernels/universal_elimination.rs
geosolver-core/src/graph/separators.rs
geosolver-core/src/graph/tree_decomposition.rs
geosolver-core/src/graph/projection_dag.rs
geosolver-core/src/algebra/resultant.rs
geosolver-core/src/types/polynomial.rs
geosolver-core/src/solver/pipeline.rs
```

## Required Checks

Fail if the gap map misses any of:

```text
- SparseResultant expression swell;
- serial route monopolization;
- dense TRS large-block materialization risk;
- route-level budget absence;
- graph decomposition leaving high-cost blocks without evidence;
- Universal not guaranteeing a bounded success route;
- lack of sparse/lazy TargetRelationSearch for large blocks;
- planner confusing matrix dimensions with actual symbolic work.
```

## Required Output

Return the required `alg-cost-review-v1` YAML-like summary and concise findings. If FAIL, identify
exact missing gap-map entries or required edits.
