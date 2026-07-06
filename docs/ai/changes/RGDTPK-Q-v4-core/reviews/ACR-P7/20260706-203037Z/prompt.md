# ACR-P7 Guardian Re-Review Prompt

Scope: ACR-P7 only, after remediation of the first FAIL_FIXABLE review.

Previous FAIL archive:

- `docs/ai/changes/RGDTPK-Q-v4-core/reviews/ACR-P7/20260706-202254Z/`

Review algebraic-cost-aware graph decomposition against:

- `docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_COMPLETION_REPAIR_BASE_SPEC.md`
- `docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_COMPLETION_REPAIR_PLAN.md` / ACR-P7
- `docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_COMPLETION_REVIEWER_PROMPTS.md` / ACR-P7
- `docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_P7_MECH_EVIDENCE.md`

Changed implementation files:

- `geosolver-core/src/graph/weighted_primal.rs`
- `geosolver-core/src/graph/separators.rs`
- `geosolver-core/src/graph/tree_decomposition.rs`
- `geosolver-core/src/graph/projection_dag.rs`
- `geosolver-core/src/graph/metrics.rs`
- `geosolver-core/src/solver/pipeline.rs`

Specific re-review blockers to close:

1. Confirm there is now a generic high-footprint variable-count-only counterexample where
   algebraic-cost scoring splits a block that a variable-count-only baseline would keep.
2. Confirm high-cost retained blocks now receive decomposition diagnostics even when the variable
   count is small.

Original P7 fail conditions still apply:

- Fail if decomposition ignores relation degree, relation arity, monomial count, or coefficient
  height.
- Fail if separator improvement is measured only by variable count.
- Fail if predicted local projection cost, target distance, linear/definitional eliminability, or
  relation-duplication certificate cost is missing from scoring.
- Fail if a high-cost block can remain without diagnostic explanation.
- Fail if relation duplication lacks certificate-cost handling.
- Fail if domain names, external diagnostic fixtures, or variable-role dispatch drive decomposition.

Use fresh evidence only from the listed files and local commands if needed. Return PASS,
FAIL_FIXABLE, or FAIL_BLOCKING. Keep the claim ceiling at
`CANDIDATE_COVER_PIPELINE_PRESENT_BUT_ALGEBRAIC_COST_INCOMPLETE`; do not authorize final
candidate-cover readiness, source fidelity, full acceptance, or any later ACR phase.
