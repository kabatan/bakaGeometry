# P5 Target Certificate Evidence

Purpose: phase evidence record.
Status: P5 local evidence after RP-P5 re-review PASS.
Authority: non-authoritative; tests and command output are evidence only.

## Scope

P5 implemented target certificate verification for:

- `IdealMembership`
- `RadicalMembership`
- `GuardedRadicalMembership`
- `CompositeCover` with `SameIdealGcd`
- `CompositeCover` with `ComponentUnionLcm` and an explicit component-union source marker

Verifier logic recomputes sparse polynomial identities over `BigRational`; it does not use solver trace, candidate origin, or identity labels as proof.

## Test-First Record

`tests/verifier_tests.rs` was added before implementation changes. The first run failed because the component-union source marker and target certificate verifier were not implemented. After implementation, the P5 verifier tests passed.

The first RP-P5 review returned `FAIL_FIXABLE`: target certificate support was not required to be nonzero, and support was not required to use the problem target variable. Recovery added verifier checks and regression tests for both cases:

- `target_certificate_rejects_zero_support`
- `target_certificate_rejects_non_target_support_variable`

## Commands

Run from `C:\Users\bakat\OneDrive\ドキュメント\bakaGeometry`.

```text
cargo test --test verifier_tests
initial result: failed before implementation, missing component-union source marker and target certificate verifier
```

```text
cargo test --test verifier_tests
final result after review recovery: pass, 7 tests passed
```

```text
cargo test
result: pass
unit tests: 12 passed
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

## P5 Claim Boundary

RP-P5 re-review returned `PASS`. The bounded P5 claim is target certificate verification accepts only nonzero supports whose support variable is the problem target, including composite parent and recursively verified composite children, and recomputes exact rational polynomial identities for ideal, radical, guarded radical, same-ideal gcd, and component-union lcm certificates.

This does not verify candidate generation, residual windows, fixed proof construction, solver route integration, root isolation, or exact-image behavior.
