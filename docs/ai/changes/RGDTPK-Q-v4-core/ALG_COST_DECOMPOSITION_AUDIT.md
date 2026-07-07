# Algebraic-Cost Decomposition Audit

Scope: ACR-P10 audit for graph decomposition and projection DAG behavior.

## Implementation Anchors

- `geosolver-core/src/graph/separators.rs` scores separators using relation degree, arity,
  monomial count, coefficient height, separator width, fill cost, and estimated component cost.
- `geosolver-core/src/graph/tree_decomposition.rs` evaluates min-fill, bounded min-cut,
  algebraic-intermediate, and low-degree definitional-affine candidates.
- `geosolver-core/src/graph/tree_decomposition.rs` records diagnostics for selected separators,
  no-improvement cases, retained high-cost small blocks, and one-large-block outcomes.
- `geosolver-core/src/graph/projection_dag.rs` carries child exports into parent exported sets and
  validates relation authorization.

## Evidence

P7/P9 tests passed in the full suite:

- `acr_p7_algebraic_separator_reduces_large_block_width`
- `acr_p7_cost_aware_split_when_variable_count_only_would_keep`
- `acr_p7_large_block_records_no_improving_separator_reason`
- `acr_p7_small_high_cost_block_records_retention_reason`
- `acr_p7_graph_build_cost_aware_split_variable_count_only_would_keep`
- `acr_p7_graph_build_reduces_width_for_generic_algebraic_separator`
- `acr_p9_s7_graph_decomposition_separator`
- `acr_p9_s8_one_large_block_universal`

ACR-P9 S7 proves a cost-improving algebraic separator split. ACR-P9 S8 proves the no-useful-separator
one-large-block case still succeeds through bounded Universal instead of hiding an unbounded global
fallback.

## Closure Result

Graph decomposition is cost-aware for the candidate-cover layer. When decomposition does not split a
large block, that fact is diagnostic evidence and the solver must still use bounded declared routes.
This audit does not claim exact-image completeness.

