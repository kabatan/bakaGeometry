# ACR-P2 Guardian Boundary Re-Review Prompt

Review phase: `ACR-P2`

Change: `RGDTPK-Q-v4-algebraic-cost-completion-repair-v1`

Use the ACR-P2 reviewer prompt from
`docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_COMPLETION_REVIEWER_PROMPTS.md`.

## Prior Failed Review

The previous ACR-P2 review at
`docs/ai/changes/RGDTPK-Q-v4-core/reviews/ACR-P2/20260706-182041Z/response.md`
failed for:

```text
1. production cost estimates ignored actual polynomial term counts;
2. SparseResultant pair scoring still used matrix size/hash ordering only;
3. RouteBudget was hash/certificate/message-bound but not runtime-enforced.
```

## Files to Review

```text
docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_COMPLETION_REPAIR_BASE_SPEC.md
docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_COMPLETION_REPAIR_PLAN.md
docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_ACCEPTANCE_MATRIX.yaml
geosolver-core/src/planner/algebraic_cost.rs
geosolver-core/src/planner/cost_model.rs
geosolver-core/src/planner/kernel_plan.rs
geosolver-core/src/planner/admission.rs
geosolver-core/src/planner/planner.rs
geosolver-core/src/solver/pipeline.rs
geosolver-core/src/kernels/sparse_resultant.rs
geosolver-core/src/result/cost_trace.rs
geosolver-core/src/compose/message.rs
geosolver-core/src/verify/certificates.rs
```

## Expected Fix Evidence

Inspect specifically:

```text
- estimate_kernel_cost now receives CompressedSystemQ and its AlgebraicWorkEstimate uses actual
  authorized relation polynomial monomial counts, degree, coefficient height, and sparse-pair
  keep-variable/product-growth estimates.
- collect_kernel_admissions maps CostProhibited estimates to KernelAdmissionStatus::CostProhibited
  and removes their execution_plan from the declared ladder.
- SparseResultant selectable_resultant_pair now scores pairs by predicted intermediate terms,
  input term count, keep-variable count, and matrix size, and rejects prohibited pair footprints.
- execute_block_with_declared_ladder calls RouteBudget preflight and postflight enforcement; route
  budget failures are route-local FiniteResourceFailure when allowed.
- ProjectionMessage hashes delegate to the central hash_projection_message, which includes route
  cost hashes when present.
```

## Evidence Commands Already Run After Fixes

```text
cargo fmt --check
cargo check
cargo test --lib acr_p2 -- --nocapture
cargo test --lib p8b_sparse_resultant_kernel_produces_exact_exported_relation -- --nocapture
cargo test --lib p6_planning_is_deterministic -- --nocapture
cargo test --lib
git diff --check
rg -n "mixtilinear|mixt|expected answer|problem hash|geometry name" geosolver-core/src -g "*.rs"
```

Observed result: all verification commands passed; the anti-overfit scan returned no matches.

## Required Checks

Fail if:

```text
- route budget exists only in docs;
- budget does not include expression-growth fields;
- cost estimates still rank by kernel name and matrix rows/cols only;
- plan hash is unchanged when dominant-cost estimates change;
- route budget is not replay/certificate-bound;
- the three prior failed-review blockers remain.
```

Return the required `alg-cost-review-v1` YAML-like summary and concise findings.
