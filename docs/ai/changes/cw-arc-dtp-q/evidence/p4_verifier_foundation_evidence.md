# P4 Verifier Foundation Evidence

Purpose: phase evidence record.
Status: P4 local evidence, pending RP-P4 review.
Authority: non-authoritative; tests and command output are evidence only.

## Scope

P4 implemented:

- Input problem well-formedness checks.
- Guard certificate verification for input nonzero records, algebraic nonvanishing, real certificates with exact algebraic replay, and derived guard products.
- Empty admissible set algebraic and guarded algebraic certificate verification.
- Conservative rejection for no-target-eliminant and real infeasibility paths not yet implemented.
- Top-level `verify_certificate` dispatch for the P4-supported solver certificate kinds.

Verifier logic recomputes sparse polynomial identities over `BigRational`; it does not trust stored identity labels, strings, traces, or hash-like evidence.

## Test-First Record

P4 verifier unit tests were added before implementation. The first run failed because `verify_guard_certificate` and related verifier routines did not exist. After implementation, the P4 verifier tests passed.

## Commands

Run from `C:\Users\bakat\OneDrive\ドキュメント\bakaGeometry`.

```text
cargo test verifier::tests
initial result: failed before implementation, missing guard verifier routines
```

```text
cargo test verifier::tests
final result: pass, 4 verifier tests passed
```

```text
cargo test
result: pass
unit tests: 12 passed
integration tests: 8 passed total
```

```text
cargo fmt --check
result: pass
```

```text
production forbidden-pattern scan over src/*.rs, Cargo.toml, README.md
result: pass, no hits
```

## P4 Claim Boundary

This supports local problem/guard/empty-certificate verifier foundation readiness for review. It does not verify target-cover certificates, fixed proof, candidate routes, root isolation, or exact-image behavior.
