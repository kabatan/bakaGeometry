# P5R Patch Notes

This remediation pack was created after reviewing the P5 implementation state.

The P5 graph/DAG layer was not rejected outright. Its hypergraph, target influence graph, weighted primal graph, target-rooted decomposition, authorization-bound TargetProjectionDAG, and tamper/deletion checks are useful and should be preserved.

However, P6 must not start until the following unsafe continuation paths are removed:

```text
1. current F4 naming can overclaim a Groebner wrapper as F4;
2. guarded affine preprocessing is narrower than Appendix A section 11.3 if it only accepts polynomial quotient substitutions;
3. TargetActionKrylov quotient/action handles can become self-certifying if action columns are injected externally;
4. resultant, interpolation, regular-chain, norm-trace, F4, affine, quotient, and Krylov primitives can be overclaimed as generic kernels;
5. P5 evidence was not commit-bound in the reviewed state;
6. closure text did not match the actual P5 claim ceiling.
```

P5R is therefore a barrier phase. It should be treated as part of the implementation plan, not as optional reviewer commentary.
