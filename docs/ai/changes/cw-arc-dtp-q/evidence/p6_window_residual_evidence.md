# P6 Window And Residual Evidence

Purpose: phase evidence record.
Status: P6 local evidence after RP-P6 re-review PASS.
Authority: non-authoritative; tests and command output are evidence only.

## Scope

P6 implemented:

- `CertificateWindow` and `ProofWindow`
- row-closed certificate-window construction from target powers and multiplier products
- membership and target-power matrices over rational coefficients
- modular reduction of rational matrix columns for admissible primes
- `ResidualOracleFp` and `DenseEchelonResidualOracleFp` with private echelon storage
- residual-cyclic candidate generation returning `TargetCandidate`, never `TargetCertificate`
- candidate model types needed by the residual route

## Test-First Record

`tests/residual_window_tests.rs` was added before implementation changes. The first run failed with unresolved imports for the P6 API. After implementation, two test-oracle issues were corrected before claiming progress:

- the residual-oracle in-space vector was corrected to a true finite-field column combination
- the residual-cyclic window for `F = {X^2 - 2, T - X}` was corrected to include the `T` multiplier support needed to express `(T + X)(T - X)`

The first RP-P6 review returned `FAIL_FIXABLE`: P6 route/backend internals were re-exported from the crate root. Recovery removed the crate-root re-exports for residual route, candidate internals, window matrices, residual oracle, compression internals, and dense residual backend. P6 tests were moved to crate-local module tests so the production public API can remain minimal while the internal data flow stays covered. A static crate-root test now checks these internal re-exports do not reappear.

The final P6 target tests pass as crate-local tests in `src/window.rs`, `src/residual.rs`, and `src/candidate_residual.rs`.

## Commands

Run from `C:\Users\bakat\OneDrive\ドキュメント\bakaGeometry`.

```text
cargo test --test residual_window_tests
initial result: failed before implementation with unresolved P6 imports
```

```text
cargo test
final P6-related result after review recovery: pass
crate-local P6 tests: 3 passed
```

```text
cargo test
result: pass
unit tests: 15 passed
integration tests: 15 passed total
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
crate-root internal re-export scan
result: pass, no hits for residual/window/candidate/compression internals
```

## P6 Claim Boundary

RP-P6 re-review returned `PASS`. The bounded P6 claim is local certificate-window construction, modular residual oracle behavior, and residual-cyclic candidate generation satisfy RP-P6 criteria.

This does not verify fixed proof construction, proof-window learning, candidate ranking, direct/tower/Krylov/resultant/slice routes, solver route integration, root isolation, or exact-image behavior.
