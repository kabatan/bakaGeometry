# P10 Non-Primary Candidate Routes Evidence

Purpose: phase evidence record.
Status: P10 local evidence after RP-P10 review PASS.
Authority: non-authoritative; tests and command output are evidence only.

## Scope

P10 implemented:

- `NormTraceTower` candidate route with structural monic triangular tower detection, target-expression relation detection, multiplication-by-target-expression matrix construction, Q[T]-coefficient characteristic/norm calculation, and `Z=T` substitution
- `TargetCyclicKrylov` candidate route using target-power recurrence from the row-closed membership quotient handle
- `HiddenVariableSparseResultant` candidate route using all equations, Newton support collection, Macaulay-style degree expansion, modular matrix null relations, and target-only modular candidate extraction
- `SliceSpecialization` candidate route using deterministic finite-field affine assignments for non-target variables, sliced target-only relations, and slice trace records
- solver planning call chains for all four non-primary routes
- cfg(test) route forcing for each route with all other candidate origins disabled and complete fallback disabled
- public trace tests showing each route can produce a candidate/proof_try event while solver success remains absent

All P10 route outputs are `TargetCandidate` values and traces only. They do not return `TargetCertificate`, `CertifiedCandidateCover`, or solver success. Normalization may reconstruct modular support over Q, but adoption still requires later fixed proof.

## Test Evidence Note

P10 unit/integration tests and implementation were introduced in the same editing segment after reading RP-P10; a separate failing red run was not captured. The P10 claim is therefore limited to reviewed implementation behavior and passing tests listed below, not a broader test-first process claim.

## Commands

Run from `C:\Users\bakat\OneDrive\ドキュメント\bakaGeometry`.

```text
cargo test --lib candidate
result: pass, 16 tests passed, 27 filtered out
```

```text
cargo test --lib test_support
result: pass, 6 tests passed, 37 filtered out
```

```text
cargo test --test candidate_route_forcing_tests
result: pass, 7 tests passed
```

```text
cargo test
result: pass
unit tests: 43 passed
integration tests: 22 passed total
doc tests: 0 passed
```

```text
cargo fmt --check
result: pass
```

```text
production forbidden-pattern scan over src/*.rs, Cargo.toml, README.md
result: pass, no hits
```

```text
crate-root and SolverOptions internal exposure scan
result: pass, no hits for route, repair, proof, candidate, residual, or window internals
```

## P10 Claim Boundary

RP-P10 review returned `PASS`. This supports local P10 non-primary route readiness only. It does not claim final source-fidelity, fixed-proof integration success, complete fallback, dependency DAG/window planning, root isolation, exact-image behavior, or acceptance completion.

Generated route candidates remain candidates. Slice agreement, modular null relations, tower characteristic output, and Krylov recurrence are not solver success evidence.
