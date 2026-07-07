# P6 Source-To-Code Map

Status: implementation evidence for Phase 6.

Relevant R-IDs:

- BS-R070 -> `graph/hypergraph.rs`, `graph/influence.rs`, `graph/weighted_primal.rs`,
  `graph/separators.rs`, `graph/tree_decomposition.rs`
- BS-R071 -> `graph/projection_dag.rs`, `graph/metrics.rs`, `planner/cost_model.rs`

Mapping:

- `build_relation_variable_hypergraph` inserts every compressed variable before relation scanning,
  then inserts every relation and every polynomial variable incidence.
- `connected_components`, `relation_variables`, and `variable_relations` expose the complete
  relation-variable bipartite structure for downstream BFS and component checks.
- `analyze_target_influence` builds the target component by BFS starting at the target variable,
  then treats only disjoint non-target components as independent feasibility components.
- `build_weighted_primal_graph` constructs variable and edge weights from algebraic source signals:
  relation arity, relation degree, degree participation, occurrence count, monomial contribution,
  coefficient height, target distance, and predicted local projection cost.
- `separators.rs` provides articulation, min-fill, bounded min-cut, and deterministic
  `score_separator` candidates. `tree_decomposition.rs` evaluates bounded min-cut for every
  non-small block before the source-approved large-block-only algebraic intermediate families.
- `tree_decomposition.rs` recursively tries the source-specified separator families from the
  target-rooted block, accepts only separators that improve estimated cost, and otherwise records a
  one-block leaf reason rather than failing.
- `projection_dag.rs` validates relation coverage exactly once unless a replayable duplication
  certificate is present, validates block authorization hashes, checks relation-variable
  authorization, and validates root topology: only the root is parentless, every non-root parent
  exists and lists the child, and every block is reachable exactly once from the root.
- `metrics.rs` and `planner/cost_model.rs` record structural metrics, quotient-rank estimates,
  sparse-template estimates, and coefficient-growth cost signals for planner decisions.
