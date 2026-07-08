# Current Gap Inventory

Status: P0 evidence plus P1-P3 checkpoint tracking; P1-P3 scoped reviews passed.
Authority: evidence only. The V3 Base Spec and production source control correctness.

Purpose: quarantine the current implementation against CW-ARC-DTP-Q Full Implementation v3. Every listed production gap is a `replace` target unless a later reviewer confirms full conformance from production data-flow.

## Search Record

P0-required source searches were run over `src`, `tests`, and current closure docs for:

```text
guard_certificates: Vec::new()
semantic_guards: Vec::new()
classify_real_fibers / Incomplete
complete_target_elimination_fallback
max_window_degree.unwrap_or
NoTargetEliminant monomial ideal special case
reconstruct_from_modular_support / first prime
factor_schedule returning clone only
localized_schur_repair returning SupportInformation only
ExactTargetImage unhandled
TODO / unimplemented / not available / Unsupported / normal-path ImplementationBug
```

The search found the following production gaps.

## Gaps

### GAP-001 — certified system construction drops guards and replay

Finding:
- `src/solver.rs` constructs `CertifiedSystemQ` with `guard_certificates: Vec::new()` and empty replay.
- `src/proof.rs` constructs a dummy `TargetProblemQ` with `semantic_guards: Vec::new()` for proof verification.
- Candidate and derived-system production paths also construct guard-empty systems:
  `src/candidate_direct.rs`, `src/candidate_residual.rs`, `src/candidate_krylov.rs`,
  `src/candidate_resultant.rs`, `src/candidate_slice.rs`, `src/candidate_tower.rs`,
  `src/dependency_dag.rs`, `src/proof_learning.rs`, and `src/window.rs`.

Impact:
- Violates V3 G5/G6 and P2 acceptance. Input semantic nonzero guards do not reliably reach guarded proof modes through production compression.

Disposition:
- P2 checkpoint implementation added validation, canonicalization, guard-certificate construction, replay verification, guarded proof propagation, and solver-returned certificate multiplier lifting back to the original input equations. Scoped P1-P3 spec, quality, and boundary reviews passed on 2026-07-08.

### GAP-002 — compression replay is placeholder-level

Finding:
- Existing `CompressionReplayCertificate` exists as a type, but production system construction does not populate replay steps from original input transformations.

Impact:
- Violates V3 certified compression requirements.

Disposition:
- P2 checkpoint implementation added deterministic primitive-normalization, zero-removal replay, `verify_compression_replay`, and original-problem multiplier lifting for returned success certificates. Scoped P1-P3 spec, quality, and boundary reviews passed on 2026-07-08.

### GAP-003 — exact image is conservative incomplete, not full exact image

Finding:
- `src/exact_image.rs` has `ExactImageClassification::Incomplete` as the conservative classifier result.
- `src/verifier.rs` rejects `SolverCertificate::ExactTargetImage` as not handled.

Impact:
- Acceptable only as prior bounded closure behavior. V3 final completion requires real-fiber classification and exact image verification.

Disposition:
- P3 checkpoint implementation replaces verifier "not handled" behavior with a structural shell returning P16 design gap. Full exact image remains P16.

### GAP-004 — complete fallback is bounded relation search

Finding:
- `src/fallback_elimination.rs` defines `complete_target_elimination_fallback`.
- It uses `limits.max_window_degree.unwrap_or(2).max(1)` and searches bounded multiplier/target degrees.
- Other production repair/search paths also default to bounded windows with `max_window_degree.unwrap_or`, including
  `src/repair_multiple.rs` and `src/repair_schur.rs`.

Impact:
- Violates V3 G9 and P15 for the fallback path, and leaves route/repair phases dependent on bounded-window defaults until their V3 exact replacements land. A bounded search must not be named or treated as complete target elimination.

Disposition:
- `replace` fallback behavior through P14 exact elimination substrate and P15 complete fallback closure. Replace non-fallback bounded repair/search defaults in their owning phases, including P7/P13, rather than using the default degree as proof of route completeness.

### GAP-005 — no-target-eliminant verifier is monomial-only / design-gap behavior

Finding:
- `src/verifier.rs` verifies no-target-eliminant only for a monomial non-target ideal shape and otherwise rejects as replay unavailable.
- `src/fallback_elimination.rs` creates no-target-eliminant certificate for that narrow shape only.

Impact:
- Violates V3 G10 and P15.

Disposition:
- P3 checkpoint implementation removes monomial-only acceptance and returns a P15 design gap after guard verification. Full exact elimination-zero certificate remains P15.

### GAP-006 — modular reconstruction is first-prime-only

Finding:
- `src/normalize.rs` `reconstruct_from_modular_support` reads `support_mod_primes.first()` and lifts residues directly into a small representative range.

Impact:
- Violates V3 P6 and candidate route requirements for multi-prime CRT/rational reconstruction.

Disposition:
- P1 checkpoint implementation uses CRT plus rational reconstruction for multi-prime modular candidates. Single-prime lifting remains a non-certified proof-search convenience. P6 still owns factor schedule.

### GAP-007 — factor schedule is clone-only

Finding:
- `src/normalize.rs` `factor_schedule` returns `vec![candidate.clone()]`.

Impact:
- Violates V3 P6 final factor schedule requirements.

Disposition:
- `replace` in P6 with actual Q factor schedule and tests for reducible candidates.

### GAP-008 — localized Schur has support-information-only production path

Finding:
- `src/repair_schur.rs` `localized_schur_repair` returns `SchurRepairOutput::SupportInformation` in the implemented path.
- Current tests assert support info only and no certified Schur success.

Impact:
- Violates V3 P13, which requires a route-forced exact certificate family and support-info family.

Disposition:
- `replace` in P13 with exact certificate construction where a target-only relation appears, while preserving support-info-only behavior for noncertifying cases.

### GAP-009 — exact target image certificate verifier is unhandled

Finding:
- `src/verifier.rs` rejects `SolverCertificate::ExactTargetImage(_)` with an unhandled-checkpoint message.

Impact:
- Violates V3 P3 temporary gap closure and P16 final exact-image verification.

Disposition:
- P3 checkpoint tracks this as a design gap shell after cover, squarefree support, and root coverage checks. Replace by P16 final verifier support.

### GAP-010 — real infeasibility and guarded no-target replay unavailable

Finding:
- `src/verifier.rs` rejects real infeasibility replay and guarded no-target-eliminant replay as unavailable.

Impact:
- Violates V3 final closure if any required certificate variant remains unavailable.

Disposition:
- `replace` across P3, P15, and P16 depending on certificate family.

### GAP-011 — normal-path `ImplementationBug` status remains

Finding:
- `src/solver.rs` can return `SolverStatus::ImplementationBug` when finalization unexpectedly fails.

Impact:
- V3 reviewer prompts reject normal production paths that panic or expose implementation-gap statuses instead of specified result semantics.

Disposition:
- Review in P17 and replace with a specified fail-closed result or prove the branch unreachable from source-level invariants.

### GAP-012 — V3 mandatory module structure is not present

Finding:
- Current source uses flat modules such as `src/linear_q.rs`, `src/linear_fp.rs`, and `src/candidate_*.rs`.
- V3 requires modules such as `src/matrix_q.rs`, `src/matrix_fp.rs`, `src/crt.rs`, `src/rational_reconstruction.rs`, `src/candidates/*`, `src/proof/*`, `src/elimination/*`, and `src/real/*`.

Impact:
- The repository does not yet satisfy V3 mandatory structure.

Disposition:
- `replace`/reorganize over P1, P4-P6, P7-P16, and integration phases. Do not claim conformance from old flat module names.

### GAP-013 — candidate routes are narrow relative to V3 route data-flow

Finding:
- Residual route lacks multi-prime CRT/rational reconstruction and active multiplier solve as required by V3 P8.
- Krylov route is an exact linear dependence search over current matrices, not yet the full quotient/residual handle required by V3 P9.
- Sparse resultant route is a simplified Macaulay-style nullspace path, not the V3 sparse template/determinant/minor contract.
- Slice route substitutes values into equations directly rather than building full sliced systems with slice equations and internal subroutes.
- Norm/trace tower handles monic triangular structure but not guarded-nonmonic tower detection.

Impact:
- Violates V3 route closure phases P8-P12. Existing route-forcing tests from the prior bounded implementation are insufficient for full V3.

Disposition:
- `replace` each route in its assigned phase, not patch evidence around current narrow behavior.

### GAP-014 — public closure from prior implementation is superseded

Finding:
- `docs/ai/changes/cw-arc-dtp-q/CLOSURE.md` documents only bounded candidate-cover core and exact-image fail-closed solver-path behavior.

Impact:
- V3 explicitly says the full implementation must not call this simplified subset complete.

Disposition:
- Treat prior closure as historical evidence only. V3 P22 must produce a new final claim if and only if all V3 phases close.

## Test-only / documentation hits

- `Unsupported` and `TODO` hits found in `tests/anti_simplification_static_tests.rs` are anti-pattern test fixtures, not production behavior.
- `guard_certificates: Vec::new()` appears in many unit-test system constructors; these do not close or excuse the production compression gap.

## Current Claim Ceiling

This inventory supports only the claim that V3 authority has been imported, P0 admission passed, P1-P3 checkpoint work was implemented and reviewed, and the current implementation still has known replacement targets for later phases. It does not claim final V3 completion.
