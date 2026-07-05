# P3c Evidence Notes

P3c implements local Groebner/F4 elimination APIs and the local elimination dispatcher. It is a support phase for local block target/separator elimination only.

Important proof boundary:

- `groebner_elimination_basis` is bounded by `GroebnerOptions` pair and basis caps and returns `FiniteResourceFailure` on cap exhaustion.
- `eliminate_to_keep_variables` rejects overlapping eliminate/keep variable sets before dispatch.
- Exported generators are filtered to `Q[keep]`; `validate_local_elimination_result` rejects any non-keep-variable export.
- Exported generators carry exact membership certificates; validation uses `verify_membership_by_certificate`.
- P3c does not enumerate coordinate roots, produce coordinate parametrizations or full coordinate RURs, call global QE/CAD/CAS/homotopy paths, or certify nonfinite target images.
- P3c does not close UniversalTargetEliminationKernel end-to-end. It only supplies local Groebner/F4 APIs used by later authorized Universal execution.

Negative coverage includes disjoint keep/eliminate validation, non-keep export rejection, exact certificate validation on exported generators, direct F4 batch reduction trace coverage, and LocalF4 keep-only/certificate export coverage.
