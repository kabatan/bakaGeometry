# ACR-P3/P4 MECH Evidence

Status: implementation evidence, not readiness authority.

Claim ceiling remains:

```text
CANDIDATE_COVER_PIPELINE_PRESENT_BUT_ALGEBRAIC_COST_INCOMPLETE
```

## Scope

Implemented the ACR-P3/P4 SparseResultant algebraic-cost repair slice:

- `SparseResultantSwellPreflight` records pair input terms, coefficient-product term growth,
  estimated template support, keep-variable count, coefficient-height growth, predicted
  intermediate/output terms, and route work units.
- `SparseResultantSwellPreflight` includes per-selected-pair records for left/right term count,
  eliminated-variable degrees, keep count, matrix dimensions, determinant entry product, output
  term bound, coefficient-height growth, intermediate terms, and route work units.
- `probe_sparse_resultant_plan` simulates the selected elimination chain without computing
  resultants by replacing each selected pair with a deterministic cost-surrogate relation.
- SparseResultant plan hashes bind the swell preflight hash through the template trace hash.
- SparseResultant direct planning now creates an `AlgebraicWorkEstimate` and `RouteBudget` from
  the swell preflight, so direct kernel execution and planner admission share the same cost basis.
- Pair ranking and rejection include expression-swell risk, not only matrix dimensions.
- Runtime guards enforce route budgets before and after each resultant step.
- Guard failures carry a route trace hash in the route-local `FiniteResourceFailure` stage string,
  and pipeline diagnostics record the allowed failure before continuing to later declared routes.
- Resultant certificates now bind backend choice and exact verification hash.
- Recursive symbolic determinant execution is capped to small matrix/entry footprints.
- A linear subresultant backend handles large linear-entry resultants and remains exact-Q
  certificate verified.
- The planner cost model now includes SparseResultant pair route work units.

## Changed Files

- `geosolver-core/src/algebra/resultant.rs`
- `geosolver-core/src/kernels/sparse_resultant.rs`
- `geosolver-core/src/planner/cost_model.rs`
- `geosolver-core/src/solver/pipeline.rs`

## Verification Run

All commands below were run from `geosolver-core`.

```text
cargo fmt --check
cargo test --lib acr_p3 -- --nocapture
cargo test --lib acr_p4 -- --nocapture
cargo test --lib acr_p2 -- --nocapture
cargo test --lib p8b_sparse_resultant_kernel_produces_exact_exported_relation -- --nocapture
cargo check
cargo test --lib
rg forbidden-pattern geosolver-core/src -g "*.rs"
```

Observed result:

```text
acr_p3: 3 passed
acr_p4: 6 passed
acr_p2: 8 passed
p8b SparseResultant exact exported relation: 1 passed
cargo check: passed
cargo test --lib: 234 passed
forbidden-pattern scan over implementation Rust sources: no matches
```

## Anti-Overfit Boundary

No diagnostic problem file, expected-answer hook, geometry-name dispatch, or special-case fixture
was used in this implementation slice.
