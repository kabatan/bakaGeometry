# P4 Evidence Notes

P4 implements Appendix A section 11 preprocessing in the required order:

1. definitional elimination;
2. linear-affine elimination;
3. binomial/monomial simplification;
4. safe saturation for explicitly nonzero encodings;
5. target-independent component marking;
6. coefficient-height and monomial-count trace recording.

Safety boundaries:

- Definitional elimination only accepts exact constant nonzero pivots and never eliminates the target variable.
- Linear-affine elimination only applies substitutions when the expression is an exact Q-polynomial quotient. Nonconstant pivots require an explicit `A*s - 1 = 0` nonzero witness and record an `AffineDenominator` guard.
- Unguarded nonconstant affine pivots are left in the system.
- Binomial simplification primitive-normalizes and deduplicates local monomial/binomial relations but does not split factors or create union semantics.
- Explicit saturation records only `A*s - 1 = 0` witnesses. The semantic encoding type does not bind arbitrary factors, so nonconstant semantic-only factor proofs are not accepted by `is_explicit_nonzero_factor`.
- Target-independent components are removed from candidate-cover relations only after a relation-variable component computation and are retained as hash-bound feasibility obligations.

Failure loop:

- The initial preprocess focused test expected one substitution in a system where two safe substitutions are valid.
- The implementation performed `y = x`, then safely eliminated `x = t`; the target is not eliminated.
- The test oracle was corrected to expect two substitutions and no remaining relation in that free-target case.
- Focused preprocess tests and the full crate test suite passed after the correction.

Claim boundary:

- P4 remains `SCAFFOLD_READY` until Guardian review admits any stronger statement.
- P4 may close `MECH-003` only if the Guardian reviewer confirms the guard-provenance tests and implementation satisfy the Plan.
- P4 does not claim graph/DAG/planner/kernel integration, candidate-cover readiness, exact-image readiness, or acceptance completion.
