# Current Gap Inventory

Status: P0 evidence plus P1-P13 checkpoint tracking; P1-P13 spec-gap blocker-fix local evidence added.
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
max_proof_weight.unwrap_or(6)
NoTargetEliminant monomial ideal special case
reconstruct_from_modular_support / first prime
factor_schedule returning clone only
factor_squarefree_over_q returns Vec only with no status
rank uses candidate.origin enum order as origin-count substitute
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
- The original V3 import audit found hidden bounded-window defaults in fallback and repair paths.
- P5/P6 blocker work removed those hidden defaults; fallback and repair now require explicit caller limits or fail closed.

Impact:
- Violates V3 G9 and P15 for the fallback path, and leaves route/repair phases dependent on bounded-window defaults until their V3 exact replacements land. A bounded search must not be named or treated as complete target elimination.

Disposition:
- P1-P13 spec-gap fix connects `solve_target` to `GlobalSolveSchedule` when `max_window_degree=None` or `max_proof_weight=None`. The old `resource:unbounded_proof_requires_bound` production sentinel is removed.
- `replace` complete fallback behavior through P14 exact elimination substrate and P15 complete fallback closure. Bounded fallback/repair behavior remains bounded by explicit caller limits and must not be used as proof of route completeness.

### GAP-004A — P5 top-level unbounded ideal execution blocker

Finding:
- Historical P5/P6 evidence had `FairProofSchedule::unbounded()` without top-level integration and `solve_target` returned `FiniteResourceFailure` with `resource:unbounded_proof_requires_bound` when `max_proof_weight = None`.
- Current F1 code connects `max_window_degree = None` or `max_proof_weight = None` to top-level unbounded fair candidate/proof execution.

Impact:
- This was a P1-P13 spec-gap blocker. F1 local implementation evidence exists; F6 review remains before any stronger claim.

Disposition:
- F1 implementation added `src/solve_schedule.rs`, connected it to `solve_target`, and added high radical power, bounded small-prefix non-success, arbitrary tuple reachability, and anti-hidden-default scans. This closes the P1-P13 blocker for top-level unbounded ideal execution, without closing P14/P15/P16 or final V3.

### GAP-005 — no-target-eliminant verifier is monomial-only / design-gap behavior

Finding:
- `src/verifier.rs` has only a P15 design-gap shell for no-target-eliminant replay.
- `src/fallback_elimination.rs` creates no-target-eliminant certificate for that narrow shape only.

Impact:
- Violates V3 G10 and P15.

Disposition:
- P3 checkpoint implementation removes monomial-only verifier acceptance and returns a P15 design gap after guard verification. Top-level `solve_target` returns `CertificateDesignGap` with no success certificate for this path until exact elimination-zero replay exists. Full exact elimination-zero certificate remains P15.

### GAP-006 — modular reconstruction first-prime-only behavior

Finding:
- The original V3 import audit found first-prime-only modular lifting.
- P4-P6 work replaced the multi-prime path with CRT plus rational reconstruction and keeps single-prime candidates modular-only.

Impact:
- Any reintroduction of first-prime-only multi-prime reconstruction would violate V3 P6 and route requirements.

Disposition:
- P6 checkpoint implementation keeps single-prime modular candidates modular-only, uses CRT plus rational reconstruction for multi-prime candidates, and preserves duplicate-prime alternatives while still forming distinct-prime reconstruction combinations. Scoped P4-P6 spec, quality, and boundary reviews passed on 2026-07-08.

### GAP-007 — factor schedule was clone-only

Finding:
- Earlier P6 checkpoint code still allowed too-weak factor scheduling/factorization semantics relative to the blocker source.

Impact:
- Violated the P5/P6 blocker requirement for status-bearing factorization, exact Q factor search beyond rational roots, and no silent `[original]` completion under partial/resource-limited factorization.

Disposition:
- P5/P6 blocker fix replaced `factor_squarefree_over_q` with `FactorizationResult` carrying `Complete`, `Partial`, or `ResourceFailure`, exact division/product reconstruction checks, and Kronecker-style Q factor search. New tests cover splitting `(T^2+1)(T^2+2)`, product reconstruction, and resource failure instead of false completion.

### GAP-007A — factor adoption requires fixed exact proof

Finding:
- Factorization and factor schedule are candidate-trial data-flow only.

Impact:
- Any factor adopted without its own fixed exact certificate would violate the P5/P6 blocker source.

Disposition:
- Current solver sends every scheduled factor candidate through `try_candidate_certificate` and fixed exact proof. `origin_count_does_not_certify_candidate_without_exact_proof` covers the non-adoption boundary for evidence-only ranking inputs.

### GAP-008 — localized Schur exact-certificate path required

Finding:
- P7-P13 route closure adds a `SchurRepairOutput::Certified` path when local membership produces a target-only relation that replays through fixed proof on the original system.
- Support-information-only behavior remains for noncertifying local scopes.

Impact:
- P13 scoped route-closure inspection has passed, but this does not close P14+, P15, P16, or final V3 behavior.

Disposition:
- P7-P13 spec, quality, and boundary reviews passed for `schur_repair_returns_exact_certificate_for_target_only_local_relation`, `schur_repair_builds_local_membership_only`, solver-level `localized_schur:certified`, and production `localized_schur_repair`. Full final behavior remains bounded by later phases.

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
- P7-P13 route closure has implementation changes for Residual route tests/evidence, Krylov quotient residual handle, Slice full sliced-system internal routes, NormTraceTower guarded-nonmonic/non-unit target coefficient handling, and LocalizedSchur exact certificate output.
- P1-P13 spec-gap fix replaced the previous weak P10/P11/P12 evidence with sparse resultant support-sum templates, generic affine slices, and guarded nonconstant tower leading coefficients.

Impact:
- P7-P13 scoped route closure has been reviewed. This does not imply final V3 route completeness outside the admitted P7-P13 delta or P14+ closure.

Disposition:
- P7-P13 historical reviews remain scoped evidence. P1-P13 spec-gap fix evidence is in `p1_p13_spec_gap_fix_evidence.md`. Neither evidence file is final V3 proof.

### GAP-013A — P10 HiddenVariableSparseResultant sparse resultant data-flow blocker

Finding:
- The previous route was Macaulay-style and insufficient. The current route now builds support-sum sparse eliminant templates and records sparse resultant witness data, but still remains candidate-generation evidence gated by exact proof.

Impact:
- This was a P1-P13 spec-gap blocker. F2 local implementation evidence exists; F6 review remains before any stronger claim.

Disposition:
- F2 implementation added `SparseResultantWitnessTrace`, support-sum template data-flow, non-chain conformance, no-total-degree static scan, and route-forced exact-proof-gated tests. This closes the P1-P13 blocker for P10 before P14 review.

### GAP-013B — P11 SliceSpecialization generic affine slicing blocker

Finding:
- The previous slice route used deterministic coordinate/constant slices. The current route constructs generic affine equations over all non-target variables and appends them to the full sliced system.

Impact:
- This was a P1-P13 spec-gap blocker. F3 local implementation evidence exists; F6 review remains before any stronger claim.

Disposition:
- F3 implementation added affine slice trace data, denominator-admissible per-prime checks, multi-variable affine route-forcing/no-fallback/tamper tests, and exact-proof-gated adoption. This closes the P1-P13 blocker for P11 before P14 review.

### GAP-013C — P12 NormTraceTower nonconstant guarded-nonmonic leading coefficient blocker

Finding:
- The previous tower route handled only constant nonmonic leading coefficients by equality check. The current route supports nonconstant leading polynomials over lower tower variables only after replay-verified guard certificates.

Impact:
- This was a P1-P13 spec-gap blocker. F4 local implementation evidence exists; F6 review remains before any stronger claim.

Disposition:
- F4 implementation added polynomial leading coefficients, quotient-basis inverse computation, `verify_guard_certificate` replay, `DerivedProduct`/`InputSemanticNonzero` provenance checks, and the required `x^2-2, x*y^2-1, T-y` conformance family. This closes the P1-P13 blocker for P12 before P14 review.

### GAP-014 — public closure from prior implementation is superseded

Finding:
- `docs/ai/changes/cw-arc-dtp-q/CLOSURE.md` documents only bounded candidate-cover core and exact-image fail-closed solver-path behavior.

Impact:
- V3 explicitly says the full implementation must not call this simplified subset complete.

Disposition:
- Treat prior closure as historical evidence only. V3 P22 must produce a new final claim if and only if all V3 phases close.

### GAP-015 — ComponentUnionLcm source replay is not implemented

Finding:
- `ComponentUnionSource` currently carries a description string, not replay-verifiable component-union source data.
- `src/verifier.rs` can compute/check lcm support from verified child certificates, but description-only source data cannot justify component-union semantics.

Impact:
- Violates V3 T-CERT-6 if accepted as a verified certificate from description alone.

Disposition:
- Current verifier returns `CertificateDesignGap` for description-only `ComponentUnionLcm` after support checking. Replace with replay-verifiable source data and replay verification in the owning later certificate phase.

### GAP-016 — multi-origin evidence was not aggregated

Finding:
- Earlier P6 checkpoint ranking used `candidate.origin` enum order and did not carry a merged origin-evidence set for same-support candidates.

Impact:
- Violated the P5/P6 blocker requirement that same primitive reconstructed support from multiple origins be aggregated, and that origin count affect only trial order.

Disposition:
- P5/P6 blocker fix adds `TargetCandidate.origin_evidence`, merges same primitive reconstructed support across origins, ranks by origin count only after exact/reconstructed status, degree, and prime count, and keeps origin count non-certifying.

## Test-only / documentation hits

- `Unsupported` and `TODO` hits found in `tests/anti_simplification_static_tests.rs` are anti-pattern test fixtures, not production behavior.
- `guard_certificates: Vec::new()` appears in many unit-test system constructors; these do not close or excuse the production compression gap.

## Current Claim Ceiling

This inventory supports only the claim that V3 authority has been imported, P0 admission passed, earlier checkpoint work exists as evidence, and the P1-P13 spec-gap blocker fixes F1-F5 have local test evidence. It does not claim final V3 completion, P14/P15/P16 completion, source-faithfulness, production-safety, readiness, acceptance-complete, or any R-ID verified status.
