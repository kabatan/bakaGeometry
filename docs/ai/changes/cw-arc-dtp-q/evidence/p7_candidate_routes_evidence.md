# P7 Candidate Routes Evidence

Purpose: phase evidence record.
Status: P7 local evidence after RP-P7 review PASS.
Authority: non-authoritative; tests and command output are evidence only.

## Scope

P7 implemented:

- `TargetCandidate`, `CandidateOrigin`, `CandidateTrace`, and `CandidateOracle`
- direct target-equation route using structural target-only checks
- residual-cyclic route integration through the candidate oracle interface
- candidate normalization, modular integer reconstruction, ranking, and proof-target scheduling
- top-level candidate planning in `solve_target` without producing solver success
- `#[cfg(test)]` route forcing harness with no production `SolverOptions` route controls

Candidate origin, modular reconstruction, and ranking are trace/order data only. They do not produce `TargetCertificate`, `CertifiedCandidateCover`, or solver success.

## Test-First Record

`tests/candidate_route_forcing_tests.rs` was added before implementation changes. The first run failed because candidate routes were not connected to the solver trace. After implementation, route traces appeared while solver status remained `NoVerifiedTargetCertificate`.

During local recovery, a forbidden-pattern scan caught `temp` as a substring of an internal trace name. The trace name was changed to avoid the forbidden production pattern.

## Commands

Run from `C:\Users\bakat\OneDrive\ドキュメント\bakaGeometry`.

```text
cargo test --test candidate_route_forcing_tests
initial result: failed before implementation because direct/residual candidate trace events were absent
```

```text
cargo test --test candidate_route_forcing_tests
final result: pass, 3 tests passed
```

```text
cargo test
result: pass
unit tests: 22 passed
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
result: pass, no hits for route forcing or candidate/residual/window internals
```

## P7 Claim Boundary

RP-P7 review returned `PASS`. The bounded P7 claim is primary candidate routing is locally supported: direct and residual routes produce `TargetCandidate` plus traces only, are reachable from solver candidate planning, normalize/rank candidates without adoption, and `solve_target` only records candidate/proof_try traces before returning `NoVerifiedTargetCertificate` with no cover or certificate.

This does not verify fixed proof construction, proof-window learning, non-primary candidate routes, repair routes, root isolation, exact-image behavior, or final solver certification.
