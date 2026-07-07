# P8 Fixed Proof And Learning Evidence

Purpose: phase evidence record.
Status: P8 local evidence after RP-P8 review PASS.
Authority: non-authoritative; tests and command output are evidence only.

## Scope

P8 implemented:

- crate-internal `FixedProofInput`, `CertificateMode`, and `ProofFailure`
- fair certificate-mode schedule for ideal, radical, and guarded radical modes
- exact fixed-target proof construction over Q for ideal, radical, and guarded radical certificates
- final Q identity recomputation before returning a `TargetCertificate`
- guarded-radical guard product construction only from certificates accepted by `verify_guard_certificate`
- exact left-null obstruction emission for inconsistent proof windows
- proof-window expansion by obstruction predecessors
- initial proof-window learning from modular witness active multiplier support
- degree-order proof-window expansion schedule for multiplier support fairness

The fixed-proof API remains crate-internal to preserve the admitted minimal public API boundary. P8 proof tests are crate-local unit tests in `src/proof.rs` and `src/proof_learning.rs`, rather than public integration tests, because exposing fixed-proof internals through `src/lib.rs` would widen the public API.

## Test-First Record

P8 tests were added before implementation. The first targeted run failed before implementation because `FixedProofInput`, `CertificateMode`, `ProofFailure`, `prove_fixed_target`, and learning helpers did not exist. After implementation, the targeted and full suites pass.

## Commands

Run from `C:\Users\bakat\OneDrive\ドキュメント\bakaGeometry`.

```text
cargo test --lib proof
result: pass, 11 tests passed, 20 filtered out
```

```text
cargo test --lib proof_learning
result: pass, 3 tests passed, 28 filtered out
```

```text
cargo test
result: pass
unit tests: 31 passed
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
result: pass, no hits for proof internals or candidate/residual/window internals
```

## P8 Claim Boundary

RP-P8 review returned `PASS`. This supports local P8 fixed-proof/proof-learning/fairness readiness only. It does not claim final source-fidelity, completed solver integration, repairs, non-primary candidate routes, root isolation, exact-image behavior, or acceptance completion.

Failure to prove in one proof window remains scoped to that window and mode. The implementation returns `ProofFailure::Inconsistent` with obstruction data and does not treat it as a proof that the target support is globally false.
