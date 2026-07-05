# P8a Notes

P8a implements TargetRelationSearch execution for the dense deterministic schedule planned in P6. Execution recomputes the dense schedule from `J,Y,Z,options`, rejects mismatch as `ImplementationBug`, builds the membership matrix for `g(Z) - sum_i q_i f_i = 0`, runs modular homogeneous nullspace only to generate candidate vectors, reconstructs candidate Q vectors, and accepts a relation only after exact Q identity verification.

After the first P8a Guardian review, the RGQ-042 required public surface was added:

- `MembershipMatrixBuilder`
- `VerifiedRelationSearchCandidate`
- `build_membership_matrix_builder`
- `reconstruct_and_verify_relation`

The execute path and public wrappers share the same matrix-building and exact-verification helpers.

The returned relation is primitive under the repository-wide `clear_denominators_primitive` convention. Multipliers are scaled by the same rational factor before exact verification.

Status boundaries:

- declared schedule exhaustion returns `AlgorithmicHardCase`;
- matrix row/column resource limits return `FiniteResourceFailure`;
- plan hash, block authorization, source relation hash, schedule, or stage mismatch returns `ImplementationBug`;
- no TargetRelationSearch path returns `CertifiedNonFiniteTargetImage`.

Residual risks:

- P8a does not close SparseResultant, SpecializationInterpolation, ActionKrylov, Universal, RegularChain, NormTrace, final composition, real-root decoding, exact-image semantics, public orchestration, or acceptance.
- Replay/certificate closure remains bounded by later P11/P16 phases.
