# FCR-P10 Reviewer Failure Remediation

Status: remediated and locally re-verified.

Initial reviewer result: FAIL_FIXABLE.

Findings addressed:
- A2 was labelled TargetActionKrylov but executed SparseResultantProjection.
- A4 was labelled sparse resultant but executed TargetRelationSearch.
- A5 was labelled specialization/interpolation but executed TargetUnivariate.
- A8 was labelled Universal but executed SparseResultantProjection.
- A9/A10 were labelled RegularChain/NormTrace but reduced to TargetUnivariate.
- A13 returned a bounded failure status without preserving cost trace evidence.

Fixes:
- Added generic `SolverOptions.kernel_priority` so public `api::solve_target` tests can force an admitted production kernel without fixture/problem-id dispatch.
- Added direct `ProjectionMessage.kernel_kind` assertions for every named-kernel P10 case.
- Kept Universal last by default, but allowed explicit priority to put it first for coverage tests.
- Changed P10 A5/A10 fixtures to avoid pre-kernel target-only collapse while remaining public algebraic inputs.
- Made DAG child exports include target plus separators and skipped relationless structural DAG nodes during planning/execution/replay.
- Tightened SpecializationInterpolation admission to require a target-bearing local relation.
- Added public failure-result cost trace retention for FiniteResourceFailure and asserted A13 matrix-dimension trace content.

Boundary reviewer result after the first remediation: FAIL_FIXABLE.

Additional findings addressed:
- P10 did not yet prove the inputs were insensitive to simple literal/coefficient matching.
- Multiseparator and projection-message composition evidence did not yet prove child message removal fails or changes support.

Additional fixes:
- Scaled every P10 public problem relation by deterministic nonzero rational factors in the shared `problem()` helper, and asserted each scaled relation differs from its unscaled source.
- Added public DAG projection-message composition/removal assertions for A3 and A5.
- Added a near-public production message-only separator-composition exercise in A3 with two separators; removing either child message must fail composition or change target support.

Quality reviewer result after boundary pass: FAIL_FIXABLE.

Quality finding addressed:
- Public finite-resource failures could lose the requested target when an internal `SolverError` had `target: None`.
- Public finite-resource cost traces hard-coded `TargetRelationSearch`, which could misidentify another failing kernel.

Quality fixes:
- `api::solve_target` now passes the original problem target into failure finalization as fallback metadata.
- `TargetSolveResult` failure finalization now prefers `err.target`, then the public fallback target.
- TargetRelationSearch finite-resource errors now carry `Some(ctx.system.target)`.
- Universal execution-path finite-resource errors now carry `Some(ctx.system.target)` when the execution context is available.
- Finite-resource cost trace kernel identity is derived from `FailureKind::FiniteResourceFailure.stage`.
- A13 now asserts `result.target == t` and that the forced TargetRelationSearch failure retains TargetRelationSearch trace identity.

Re-verification:
- `cargo test --manifest-path geosolver-core/Cargo.toml --test fcr_p10_acceptance_suite -- --nocapture`: 13 passed.
- `cargo test --manifest-path geosolver-core/Cargo.toml`: all tests passed.
