# Guardian Boundary Review Prompt

Review target: ACR-P3/P4 only.

Scope:

- `docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_COMPLETION_REPAIR_PLAN.md`, phases ACR-P3 and ACR-P4
- `docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_ACCEPTANCE_MATRIX.yaml`
- `docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_P3_P4_MECH_EVIDENCE.md`
- `geosolver-core/src/algebra/resultant.rs`
- `geosolver-core/src/kernels/sparse_resultant.rs`
- `geosolver-core/src/planner/cost_model.rs`

Questions:

1. Does the implementation satisfy ACR-P3 requirements for SparseResultant swell-aware planning?
2. Does it satisfy ACR-P4 requirements for bounded execution and backend repair?
3. Are there boundary or claim violations?
4. Are tests and evidence adequate for P3/P4, or is this FAIL with blocking gaps?
