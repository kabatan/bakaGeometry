# ACR-P7 Guardian Review Prompt

Scope: ACR-P7 only.

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

Review requirements:

- Fail if decomposition ignores relation degree, relation arity, monomial count, or coefficient
  height.
- Fail if separator improvement is measured only by variable count.
- Fail if predicted local projection cost, target distance, linear/definitional eliminability, or
  relation-duplication certificate cost is missing from scoring.
- Fail if a high-cost block can remain without diagnostic explanation.
- Fail if relation duplication lacks certificate-cost handling.
- Fail if domain names, external diagnostic fixtures, or variable-role dispatch drive decomposition.
- Construct or inspect at least one generic hypergraph where cost-aware decomposition must split a
  block that variable-count-only scoring would not justify.

Use fresh evidence only from the listed files and local commands if needed. Return PASS,
FAIL_FIXABLE, or FAIL_BLOCKING. Keep the claim ceiling at
`CANDIDATE_COVER_PIPELINE_PRESENT_BUT_ALGEBRAIC_COST_INCOMPLETE`; do not authorize final
candidate-cover readiness, source fidelity, full acceptance, or any later ACR phase.
