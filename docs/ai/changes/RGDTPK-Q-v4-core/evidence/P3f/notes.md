# P3f Evidence Notes

P3f implements the Appendix A section 10.16 through 10.19 algebra-level primitives:

- regular-chain decomposition/projection/combination primitives with component and guard semantics preserved in records and hashes;
- explicit one-step algebraic tower detection by polynomial form and exported-variable boundary, not by geometry names;
- norm relation computation as an exact Sylvester resultant and verification by exact recomputation up to primitive nonzero scalar normalization;
- exact Sturm root isolation with RationalQ arithmetic, rational Cauchy bounds, and rational isolating intervals;
- exact sign/Thom helpers that return `RefinementRequired` instead of claiming a sign when the current interval is not enough to certify it.

The implementation does not use floating arithmetic, coordinate solution enumeration, full coordinate RURs, generic QE/CAD, homotopy, or geometry-name dispatch.

Failure loop:

- The initial `algebra::norm_trace` focused run failed because the expected test polynomial used the opposite resultant sign convention.
- The root-cause hypothesis was that resultants are defined up to nonzero scalar in this primitive certificate context.
- The targeted fix changed the test oracle to compare both sides after `clear_denominators_primitive`.
- Focused tests and the full crate test suite then passed.

Claim boundary:

- P3f remains `SCAFFOLD_READY`.
- P3f starts executable primitive support for `MECH-007`, `MECH-011`, and `MECH-012`; it does not close those MECHs.
- P3f does not claim `RegularChainProjectionKernel` or `NormTraceProjectionKernel` execution, projection-message production, candidate-cover readiness, exact-image readiness, root decode completion, or acceptance completion.
