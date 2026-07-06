# ACR-P2 Guardian Boundary Review Prompt

Review phase: `ACR-P2`

Change: `RGDTPK-Q-v4-algebraic-cost-completion-repair-v1`

Use the ACR-P2 reviewer prompt from
`docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_COMPLETION_REVIEWER_PROMPTS.md`.

## Review Scope

Review the route budget architecture and dominant-cost binding added for ACR-P2.

## Files to Review

```text
docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_COMPLETION_REPAIR_BASE_SPEC.md
docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_COMPLETION_REPAIR_PLAN.md
docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_ACCEPTANCE_MATRIX.yaml
docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_SOURCE_SPEC_GAP_MAP.md
geosolver-core/src/planner/algebraic_cost.rs
geosolver-core/src/planner/cost_model.rs
geosolver-core/src/planner/kernel_plan.rs
geosolver-core/src/planner/admission.rs
geosolver-core/src/planner/planner.rs
geosolver-core/src/result/cost_trace.rs
geosolver-core/src/compose/message.rs
geosolver-core/src/verify/certificates.rs
geosolver-core/src/kernels/target_univariate.rs
geosolver-core/src/kernels/linear_affine.rs
geosolver-core/src/kernels/target_relation_search.rs
geosolver-core/src/kernels/sparse_resultant.rs
geosolver-core/src/kernels/action_krylov.rs
geosolver-core/src/kernels/norm_trace_projection.rs
geosolver-core/src/kernels/regular_chain_projection.rs
geosolver-core/src/kernels/specialization_interpolation.rs
geosolver-core/src/kernels/universal_elimination.rs
```

## Evidence Commands Already Run

```text
cargo fmt --check
cargo check
cargo test acr_p2 -- --nocapture
cargo test p6_declared_ladder -- --nocapture
cargo test p6_planning_is_deterministic -- --nocapture
git diff --check
rg -n "mixtilinear|mixt|expected answer|problem hash|geometry name" geosolver-core/src -g "*.rs"
```

Observed result: all commands passed; the anti-overfit scan returned no matches.

## Required Checks

Fail if:

```text
- route budget exists only in docs;
- budget does not include expression-growth fields;
- cost estimates still rank by kernel name and matrix rows/cols only;
- plan hash is unchanged when dominant-cost estimates change;
- route budget is not replay/certificate-bound.
```

Also inspect that generated code does not reference the external diagnostic problem, geometry names,
problem hashes, or expected answers.

## Required Output

Return the required `alg-cost-review-v1` YAML-like summary and concise findings. If FAIL, identify
exact files and required edits.
