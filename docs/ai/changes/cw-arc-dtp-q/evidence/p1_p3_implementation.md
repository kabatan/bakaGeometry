# P1-P3 Implementation Evidence

Status: P1-P3 checkpoint evidence; scoped spec, quality, and boundary reviews passed.
Authority: evidence only. `BASE_SPEC.md` and `PLAN.md` remain controlling.

## Scope

Implemented P1 through P3 under `CW-ARC-DTP-Q-FULL-V3`.

No claim is made for P4 or later phases.

## Review Results

Date: 2026-07-08.

- `spec_verifier`: PASS for the scoped P1-P3 checkpoint after compression certificate lifting and latest verifier hardening.
- `quality_reviewer`: PASS after two fixable public-verifier findings were corrected.
- `guardian_boundary_reviewer`: PASS to close the P1-P3 checkpoint narrowly.

No R-ID is marked verified by this evidence document.

## P1 Algebra Primitives

Code changes:
- Added `src/matrix_q.rs` and `src/matrix_fp.rs`.
- Kept `src/linear_q.rs` and `src/linear_fp.rs` as compatibility re-exports.
- Added `src/crt.rs`.
- Added `src/rational_reconstruction.rs`.
- Replaced multi-prime modular candidate reconstruction in `src/normalize.rs` with CRT plus rational reconstruction.
- Replaced `UniPolynomialQ::factor_squarefree_over_q` clone-only behavior with exact rational-root factorization for the conformance family.

Evidence:
- `factor_squarefree_over_q` splits `(T-1)(T-2)(T-3)`.
- CRT reconstruction test reconstructs coefficient `42`, larger than any single prime in the test set.
- Q linear solver keeps left-null obstruction tests.
- Fp solver exposes rank, a solution, nullspace behavior, and active nonzero support.

Remaining later-phase boundary:
- `factor_schedule` is still the P6 replacement target.
- Single-prime modular lifting remains only a non-certified proof-search convenience; multi-prime reconstruction no longer uses the first prime only.

## P2 Problem, Compression, Guards

Code changes:
- Implemented `validate_target_problem`.
- Implemented `certified_system_from_problem`.
- Implemented `verify_compression_replay`.
- Added direct `InputSemanticNonzero` guard construction for `GuardKind::NonZero` semantic guards.
- Added primitive normalization and zero equation removal replay.
- Changed `solve_target` to use `certified_system_from_problem`.
- Changed proof verification problem construction so `InputSemanticNonzero` records from `CertifiedSystemQ` reach guarded proof verification instead of using an empty semantic guard list.
- Added compression multiplier lifting so solver-returned target and empty certificates are translated back from compressed equations to the original input equation list before being returned.
- Added malformed public polynomial arity validation before compression/normalization paths that accept public `PolynomialQ` values.

Evidence:
- `tests/guard_and_compression_tests.rs` covers nonzero guard transfer, replay verification, guard tamper rejection, replay tamper rejection, and invalid target validation.
- `tests/guard_and_compression_tests.rs` also covers a solver success certificate from a primitive-normalized system with zero equation removal, verifying the returned certificate against the original input problem.
- `proof::tests::semantic_nonzero_guard_reaches_guarded_radical_proof_mode` covers a semantic nonzero guard reaching `GuardedRadicalMembership`.

Search boundary:
- Remaining `guard_certificates: Vec::new()` and `semantic_guards: Vec::new()` hits in route and repair modules are later-phase replacement targets tracked in `current_gap_inventory.md`; they are not claimed closed by P2.
- Remaining no-target-eliminant empty guard vectors are part of the P15 design-gap shell or later-phase replacement targets, not P2 system compression closure.

## P3 Certificate Verifier

Code changes:
- Added `VerificationResult::CertificateDesignGap`.
- Guard certificates are compared structurally and verified against original `GuardRecord` provenance for `InputSemanticNonzero`.
- Target certificate verification recomputes ideal, radical, guarded radical, and same-ideal gcd identities. `ComponentUnionLcm` computes and checks the lcm support but returns `CertificateDesignGap` unless the source is replay-verifiable.
- Target, empty, guard-product, and guard Nullstellensatz verifier paths reject mismatched `ExactIdentityKind`.
- Public certificate polynomials are checked for variable list and monomial arity before verifier arithmetic.
- `GuardedRadicalMembership` recomputes `guard_product` from verified guard certificates.
- Empty algebraic and guarded algebraic certificates are checked by exact polynomial identity.
- `ExactTargetImage` now has a structural shell: cover certificate, squarefree support, and root classification coverage are checked before returning a P16 design gap.
- `NoNonzeroTargetEliminant` no longer accepts a monomial-only special case; after guard verification it returns a P15 design gap.
- `solve_target` does not return `CertifiedNoNonzeroTargetEliminant` while the verifier has only a P15 design-gap shell; it returns `CertificateDesignGap` with no success certificate until P15 replay exists.

Evidence:
- `tests/verifier_tests.rs` covers multiplier tamper, malformed multiplier arity, support tamper, wrong identity kind, guard Nullstellensatz kind tamper, guard product tamper, guard certificate tamper, composite rule tamper, description-only component-union design gap, exact-image shell checks, and no-target P15 design gap.
- `tests/fallback_elimination_solver_tests.rs` covers solver fail-closed behavior for no-target eliminant while empty algebraic certificates remain verified.

Remaining later-phase boundary:
- `ExactTargetImage` full real-fiber replay remains P16.
- `NoNonzeroTargetEliminant` exact elimination-zero replay remains P15.
- `ComponentUnionLcm` replay-verifiable source remains a later certificate replay gap.
- Real infeasibility replay remains P16.

## Test Evidence

Command:

```text
cargo fmt
cargo fmt --check
cargo test
```

Result:

```text
cargo fmt --check passed.
cargo test passed: 109 tests.
```

The run included library unit tests, anti-simplification static tests, route-forcing tests, exact algebra tests, fallback solver tests, guard/compression tests, root isolation tests, solver status tests, verifier tests, and doctests.

## Claim Ceiling

This evidence supports only that the scoped P1-P3 checkpoint was implemented and reviewed. It does not mark any R-ID verified and does not support final V3 completion claims.
