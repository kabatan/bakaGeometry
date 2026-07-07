# P6 Algorithm Evidence

Status: P6 implementation evidence before spec reviewer.

Implemented/verified behavior:

- Hypergraph coverage includes every compressed variable even when the variable is isolated after
  compression. Regression `hypergraph_represents_every_polynomial_occurrence` asserts that the
  complete compressed variable set is represented.
- Influence analysis uses target-start BFS through the bipartite hypergraph, not arbitrary connected
  component ordering. Regression
  `target_influence_bfs_starts_from_target_and_separates_isolated_component` covers a target
  component and an isolated non-target component.
- Separator/decomposition behavior remains deterministic and source-family based. Existing tests
  cover articulation candidates, generic algebraic separator classes, score contents, no-useful
  separator retention, and cost-aware target-rooted splitting.
- Remediation after spec review: bounded min-cut is now evaluated for sub-6 non-small blocks before
  fallback. Regression `bounded_min_cut_is_tried_for_sub_six_variable_blocks` covers a 5-variable
  block selected via a bounded min-cut candidate.
- Projection DAG validation rejects omitted relations, unauthorized authorization hashes, duplicate
  relations without certificates, relation access outside block-local variables, parentless non-root
  blocks, parent-child listing mismatches, and duplicate root reachability.
- Structural metrics and cost model paths include monomial, coefficient-height, rank/template, and
  coefficient-growth signals used by planner estimates.

Static audit guard added:

- `audit_v4_conformance.py --phase P6 --strict` checks for all-variable hypergraph construction,
  target BFS implementation markers, required separator family calls, sub-6 bounded min-cut
  placement, and DAG validation/root-topology markers.

Claim boundary:

- This evidence supports only Phase 6 graph/decomposition/DAG conformance.
- It does not claim finite candidate-cover completion or full source-faithful completion.
