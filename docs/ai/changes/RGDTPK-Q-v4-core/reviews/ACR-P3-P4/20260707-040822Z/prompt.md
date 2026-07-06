# Guardian Boundary Review Prompt

Review target: revised ACR-P3/P4 only.

Scope:

- `docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_COMPLETION_REPAIR_PLAN.md`, phases ACR-P3 and ACR-P4
- `docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_ACCEPTANCE_MATRIX.yaml`
- `docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_P3_P4_MECH_EVIDENCE.md`
- Prior failed review archive: `docs/ai/changes/RGDTPK-Q-v4-core/reviews/ACR-P3-P4/20260707-040440Z/`
- `geosolver-core/src/algebra/resultant.rs`
- `geosolver-core/src/kernels/sparse_resultant.rs`
- `geosolver-core/src/planner/cost_model.rs`
- `geosolver-core/src/solver/pipeline.rs`

Specific fixes to check:

1. `probe_sparse_resultant_plan` simulates the selected elimination chain without computing
   resultants by replacing selected pairs with cost-surrogate relations.
2. `SparseResultantSwellPreflight` binds per-selected-pair records for left/right term counts,
   eliminated-variable degrees, keep count, matrix dimensions, determinant entry product, output
   bound, coefficient-height growth, predicted intermediate terms, and route work units.
3. Runtime guard failures include a route trace hash in `FiniteResourceFailure.stage`, and pipeline
   diagnostics record allowed failure/continuation.
4. Tests cover all declared P4 guard classes, an intermediate-stop-before-next-step case, later
   declared route continuation after SparseResultant guard failure, determinant cap failure, and
   exact linear subresultant backend verification.

Questions:

- Does revised implementation satisfy ACR-P3 and ACR-P4 only?
- Are there remaining blocking boundary/source-fidelity issues?
- Are tests/evidence adequate for P3/P4 PASS only, without raising final readiness claims?
