# ACR-P7 MECH Evidence

Status: implementation evidence, not readiness authority.

Claim ceiling remains:

```text
CANDIDATE_COVER_PIPELINE_PRESENT_BUT_ALGEBRAIC_COST_INCOMPLETE
```

## Scope

Implemented the ACR-P7 algebraic-cost-aware graph decomposition slice:

- `WeightedPrimalGraph` variable weights now include relation arity, relation degree, monomial
  contribution, coefficient-height estimate, target distance, linear/definitional relation counts,
  and predicted local projection cost.
- `SeparatorScore` now explicitly records all required P7 scoring terms:
  relation arity, relation degree, monomial count, coefficient-height estimate, predicted local
  projection cost, linear/definitional eliminability, target distance, separator width, and relation
  duplication certificate cost.
- Separator ordering uses the algebraic estimated total cost, not variable count alone.
- `DecompositionTree` records structured diagnostics for selected separators and for large blocks
  that remain unsplit.
- Large-block diagnostics state whether no candidate existed, no candidate reduced block width, or
  no width-reducing candidate improved estimated cost.
- `step_build_graphs` propagates unsplit large-block decomposition diagnostics into
  `SolverContext`, so final solver diagnostics can explain why one large block remains.
- `ProjectionDAG` now exposes a replayable relation-duplication certificate cost helper and rejects
  zero-cost duplication certificates.
- `AlgebraicBlockMetrics` now records predicted local projection cost, separator width, and
  relation-duplication certificate cost.

## Changed Files

- `geosolver-core/src/graph/weighted_primal.rs`
- `geosolver-core/src/graph/separators.rs`
- `geosolver-core/src/graph/tree_decomposition.rs`
- `geosolver-core/src/graph/projection_dag.rs`
- `geosolver-core/src/graph/metrics.rs`
- `geosolver-core/src/solver/pipeline.rs`

`solver/pipeline.rs` is included only to surface graph decomposition diagnostics from the graph
bundle into the existing solver diagnostic channel.

## Verification Run

All cargo commands below were run from `geosolver-core`.

```text
cargo fmt --check
cargo check
cargo test --lib acr_p7 -- --nocapture
cargo test --lib graph::tree_decomposition::tests -- --nocapture
cargo test --lib graph::separators::tests -- --nocapture
cargo test --lib graph::projection_dag::tests -- --nocapture
cargo test --lib graph::metrics::tests -- --nocapture
cargo test --lib
rg -n -i "<forbidden diagnostic marker expression>" src -g "*.rs"
git diff --check
```

Observed result:

```text
cargo fmt --check: passed
cargo check: passed
acr_p7: 9 passed
graph::tree_decomposition::tests: 5 passed
graph::separators::tests: 3 passed
graph::projection_dag::tests: 4 passed
graph::metrics::tests: 1 passed
cargo test --lib: 247 passed
forbidden-marker scan over implementation Rust sources: no matches
git diff --check: exit 0, CRLF conversion warnings only
```

## Acceptance Evidence

Generic algebraic separator stress:

```text
graph::tree_decomposition::tests::acr_p7_algebraic_separator_reduces_large_block_width
solver::pipeline::tests::acr_p7_graph_build_reduces_width_for_generic_algebraic_separator
```

These tests verify that a generic algebraic separator reduces maximum block width and records a
selected cost-improving separator diagnostic.

Variable-count-only counterexample stress:

```text
graph::tree_decomposition::tests::acr_p7_cost_aware_split_when_variable_count_only_would_keep
solver::pipeline::tests::acr_p7_graph_build_cost_aware_split_variable_count_only_would_keep
```

These tests build a generic high-footprint block whose useful separator has width two. A
variable-count-only baseline using only component width plus separator-width cost would keep the
block, while the algebraic-cost score splits it because the split lowers the predicted local
projection cost.

Large-block diagnostic stress:

```text
graph::tree_decomposition::tests::acr_p7_large_block_records_no_improving_separator_reason
solver::pipeline::tests::acr_p7_large_block_diagnostic_reaches_solver_context
graph::tree_decomposition::tests::acr_p7_small_high_cost_block_records_retention_reason
solver::pipeline::tests::acr_p7_small_high_cost_block_diagnostic_reaches_solver_context
```

These tests verify that a generic large block with no improving separator remains one block and that
the diagnostic channel records the reason and the best candidate/cost summary. They also verify that
a small-variable but high-cost block is not silently retained without a decomposition diagnostic.

Score-term stress:

```text
graph::separators::tests::acr_p7_separator_score_records_required_algebraic_terms
```

This test verifies the required score fields are populated for a generic algebraic separator.

## Anti-Overfit Boundary

No prior diagnostic input, expected-answer hook, domain-name dispatch, variable-role dispatch, or
fixed external benchmark fixture was used in this implementation slice.

Reviews are required before ACR-P7 may be treated as closed.
