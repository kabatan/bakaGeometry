# CW-ARC-DTP-Q Closure

Status: RP-CLOSURE reviewed; bounded closure claim accepted.

Authority: closure evidence summary only. `BASE_SPEC.md`, production code, tests, and reviewer results control correctness.

## Supported Claim

This implementation completes the CW-ARC-DTP-Q certified candidate-cover core as covered by the admitted Base Spec, implementation evidence, tests, and the reviewed production-code paths through P14. The solver returns `CertifiedCandidateCover` only through exact `TargetCertificate` construction and replayable exact polynomial-identity verification. It includes exact rational root isolation and exact-image fail-closed control flow. It does not claim general `CertifiedExactTargetImage` completion because the real-fiber classifier is conservative incomplete.

This closure does not claim `SOURCE_FAITHFUL`, `PRODUCTION_SAFE`, R-ID verification, or broader production readiness. It also does not claim `ACCEPTANCE_COMPLETE` until RP-CLOSURE accepts this exact closure wording.

RP-CLOSURE accepted this bounded wording. The accepted claim remains limited to the candidate-cover core, exact certificate replay, exact rational root isolation, and exact-image fail-closed solver-path behavior.

## Working Tree State

No git commit hash is available for this closure because the repository contents are currently untracked in this workspace.

Latest `git status --short`:

```text
?? .gitignore
?? Cargo.lock
?? Cargo.toml
?? README.md
?? docs/
?? src/
?? tests/
```

No staging or commit was requested.

## Commands Run

Latest post-recovery commands:

```text
cargo fmt --check
result: pass
```

```text
cargo test
result: pass
unit tests: 61 passed
integration tests: 27 passed total
doc tests: 0 passed
```

```text
public API boundary scan over src/lib.rs and src/options.rs
result: pass, no hits for internal route/proof/fallback/DAG/window/root/classifier names
```

```text
RP-P14 forbidden-pattern scan over src/*.rs, Cargo.toml, and README.md
result: pass, no hits
```

```text
git diff --check
result: pass
```

QuestionDebt search found no open QuestionDebt artifacts; hits were only normative requirements in `BASE_SPEC.md` and `PLAN.md`.

## Route-Forcing Matrix

The full route-forcing matrix is in `docs/ai/changes/cw-arc-dtp-q/evidence/route_forcing_matrix.md`.

Summary:

| Route | Evidence |
| --- | --- |
| DirectTargetEquation | route-only candidate cover with fallback disabled |
| ResidualCyclic | route-only candidate cover with fallback disabled |
| NormTraceTower | route-only candidate cover with fallback disabled |
| TargetCyclicKrylov | route-only candidate cover with fallback disabled |
| HiddenVariableSparseResultant | route-only candidate cover with fallback disabled |
| SliceSpecialization | route-only candidate cover for a positive-dimensional finite-target family with fallback disabled; separate non-adoption test for unproved slice candidates |
| LocalizedSchur | obstruction-local support information only unless an exact certificate exists; uncertified support is not solver success |
| CompleteTargetEliminationFallback | explicit fallback only after bounded route exhaustion; test-only route control panics if fallback is reached while disabled |

## Tamper Matrix

The full tamper matrix is in `docs/ai/changes/cw-arc-dtp-q/evidence/tamper_matrix.md`.

Tamper/replay coverage includes:

- ideal membership support tamper
- radical zero-power rejection
- missing or unverifiable guard certificate rejection
- same-ideal gcd versus product rejection
- component-union source requirement
- zero and non-target support rejection
- altered input semantic guard record rejection
- altered derived guard product rejection
- altered Nullstellensatz multiplier rejection
- empty-certificate multiplier tamper
- missing exact-image root classification rejection

## Reviewer Summaries

- P5: RP-P5 re-review PASS after target support was required to be nonzero and use the problem target variable.
- P6: RP-P6 re-review PASS after internal route/window/residual exports were removed from the public crate root.
- P7: RP-P7 PASS for primary candidate routing as candidate/trace only, with no route-local solver success.
- P8: RP-P8 PASS for fixed proof, obstruction learning, and fair certificate-mode scheduling.
- P9: RP-P9 PASS for low-degree multiple repair and localized Schur support-information boundary.
- P10: RP-P10 PASS for non-primary candidate routes as candidate producers only.
- P11: RP-P11 PASS after complete fallback was made explicitly disabled by the test-only route-control guard.
- P12: RP-P12 PASS after unbounded scheduling was corrected to avoid finite-prefix exhaustion followed by fallback.
- P13: RP-P13 PASS for exact rational root isolation and exact-image fail-closed behavior.
- P14: initial RP-P14 review returned FAIL_FIXABLE for missing SliceSpecialization route-only success; recovery RP-P14 review PASS after adding the positive-dimensional finite-target slice test.
- P15: initial RP-CLOSURE review returned FAIL_FIXABLE for overly broad exact-image construction wording and review-artifact wording; recovery RP-CLOSURE review PASS after narrowing the closure claim.

P1-P4 evidence files are present but retain early local-evidence status text. The `reviews/` directory is currently empty; review outcomes are referenced in phase evidence summaries and subagent outputs in this thread. This closure treats those review outcomes as boundary-review pointers, not as executable proof, and relies on RP-CLOSURE to inspect the implementation and evidence set directly.

## Residual Limitations

- Resource limits can still produce `FiniteResourceFailure`; unbounded search does not call complete fallback after an artificial finite prefix.
- Exact target-image classification is not generally implemented. The classifier is conservative incomplete, so `TryExactImage` returns a candidate cover on incomplete classification and `RequireExactImage` fails closed.
- Localized Schur currently returns support information in the reviewed path; uncertified Schur information is not solver success.
- Complete no-target-eliminant replay is restricted to the implemented algebraic monomial-ideal boundary.
- Source-fidelity remains bounded by `source_map.md`: the original revised spec v2 is stored, but failure-analysis artifacts named by the imported package are not present in this repo.

## Exact-Image Boundary

`CertifiedCandidateCover` contains support, squarefree support, exact rational real-root records, and the target certificate. The current solver path and private exact-image helper do not produce `CertifiedExactTargetImage` unless every root has a classification; this is not a claim that public exact-image structs are unconstructible by external code. Current public solver behavior is:

- `CoverOnly`: returns a candidate cover when a target certificate is found.
- `TryExactImage`: returns a candidate cover and trace if the classifier is incomplete.
- `RequireExactImage`: returns `NoVerifiedTargetCertificate` or a resource failure if the classifier is incomplete.

No general real-fiber classification completion is claimed.
