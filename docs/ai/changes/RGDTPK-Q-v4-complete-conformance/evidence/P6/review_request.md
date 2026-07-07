# P6 Review Request

Reviewer prompt: RP-P6 from `REVIEWER_PROMPTS.md`.

Relevant R-IDs:

- BS-R070
- BS-R071

Files to inspect:

- `geosolver-core/src/graph/hypergraph.rs`
- `geosolver-core/src/graph/influence.rs`
- `geosolver-core/src/graph/weighted_primal.rs`
- `geosolver-core/src/graph/separators.rs`
- `geosolver-core/src/graph/tree_decomposition.rs`
- `geosolver-core/src/graph/projection_dag.rs`
- `geosolver-core/src/graph/metrics.rs`
- `geosolver-core/src/planner/cost_model.rs`
- `geosolver-core/scripts/audit_v4_conformance.py`

Evidence:

- `source_to_code_map.md`
- `algorithm_evidence.md`
- `commands.log`
- `static_audit.log`
- `changed_files.txt`

Requested checks:

- Hypergraph contains every compressed relation, variable, and relation-variable incidence.
- Influence graph BFS starts from the target variable.
- Weighted graph uses all source cost signals required by the active Base Spec.
- Decomposition tries articulation, min-fill, and bounded min-cut separator families and keeps a
  one-block leaf when no useful separator improves cost.
- Projection DAG validates relation coverage, authorization hashes, duplication certificates, and
  root consistency.

Remediation focus after prior P6 FAIL_FIXABLE:

- `choose_useful_separator` now evaluates `bounded_min_cut_separator_candidates` for sub-6
  non-small blocks, before the large-block-only algebraic intermediate families.
- Added regression `bounded_min_cut_is_tried_for_sub_six_variable_blocks`.
- `validate_projection_dag_topology` now requires only the root to be parentless, every non-root
  parent to exist and list the child, and every block to be reachable exactly once from root.
- Added regressions for parentless non-root block, parent that does not list child, and duplicate
  root reachability.

Requested decision: PASS / FAIL_FIXABLE / FAIL_BLOCKING / NEEDS_MORE_EVIDENCE for P6 only.
