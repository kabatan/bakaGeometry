# P9 Repair Evidence

Purpose: phase evidence record.
Status: P9 local evidence after RP-P9 review PASS.
Authority: non-authoritative; tests and command output are evidence only.

## Scope

P9 implemented:

- low-degree multiple repair with unknown `A(T)` represented by an anchored exact linear system
- repair adoption only after constructing `P=A*S` and passing fixed-target proof on `P`
- exact primitive normalization of repaired support before proof
- fail-closed repair exhaustion through `ProofFailure::NoCertificateFound`
- obstruction-incidence scope detection for localized Schur repair
- boundary variable selection from target plus separator variables
- local membership equation builder for `M_Omega u + N_Omega f = 0`
- localized support information output for proof-window expansion
- no full-system Schur path outside final fallback
- no solver success from uncertified Schur support information

The repair APIs remain crate-internal and are not re-exported from `src/lib.rs`.

## Test Evidence Note

P9 unit tests and implementation were introduced in the same editing segment after reading RP-P9; a separate failing red run was not captured. The P9 claim is therefore limited to the reviewed implementation and passing local tests listed below, not a broader test-first process claim.

## Commands

Run from `C:\Users\bakat\OneDrive\ドキュメント\bakaGeometry`.

```text
cargo test --lib repair_multiple
result: pass, 1 test passed, 34 filtered out
```

```text
cargo test --lib repair_schur
result: pass, 3 tests passed, 32 filtered out
```

```text
cargo test
result: pass
unit tests: 35 passed
integration tests: 18 passed total
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
result: pass, no hits for repair, proof, candidate, residual, or window internals
```

## P9 Claim Boundary

RP-P9 review returned `PASS`. This supports local P9 repair readiness only. It does not claim final source-fidelity, completed top-level solver integration, non-primary candidate routes, complete fallback, root isolation, exact-image behavior, or acceptance completion.

Low-degree repair returns only fixed-proof certificates for the repaired support `P=A*S`; it does not return the original candidate unless that polynomial itself passes fixed proof. Localized Schur output without an exact certificate is support information only.
