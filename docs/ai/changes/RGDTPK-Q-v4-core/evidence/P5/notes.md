# P5 Evidence Notes

P5 implements Appendix A section 12 graph construction and operational DAG authorization:

1. relation-variable hypergraph;
2. target influence graph;
3. weighted primal graph with algebraic-only weights;
4. separator candidates and scoring;
5. target-rooted decomposition with no-separator one-large-block behavior;
6. TargetProjectionDAG construction, relation authorization hashes, and validation;
7. local structural metrics and estimates.

Safety boundaries:

- Hypergraph incidence is derived from `poly_variables` on every compressed relation polynomial.
- Weighted graph and separator scoring use algebraic incidence, degree, monomial count, coefficient height, occurrence count, graph distance, and deterministic cost-model values only.
- The DAG builder assigns each compressed relation to the smallest block containing all of its polynomial variables, then validates before returning.
- Authorization hashes bind a block to its parent, local variables, exported variables, child ids, relation ids, and relation hashes.
- Validation rejects authorization tampering, omitted relations, duplicate relation use without a duplication certificate, relation ids outside the compressed system, and relations whose variables are outside the block.
- No useful separator produces a single large target block rather than an empty or decorative DAG.

Failure loop:

- The initial separator test used `t*x - y` plus `x*y`, which creates a triangle in the primal graph and therefore does not make `x` an articulation variable.
- The test oracle was corrected to use the algebraic incidence chain `t*x`, `x*y` and to bypass P4 compression for that graph-specific test.
- Focused graph tests and the full crate test suite passed after the correction.

Claim boundary:

- P5 remains `SCAFFOLD_READY` until Guardian review admits any stronger statement.
- P5 may close `MECH-004` only if the Guardian reviewer confirms the deletion/tamper tests and operational authorization satisfy the Plan.
- P5 does not claim planner/kernels/candidate-cover/exact-image/replay/orchestrator readiness.
