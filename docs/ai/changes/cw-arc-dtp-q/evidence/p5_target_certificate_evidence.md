# P5 Target Certificate Evidence

Purpose: phase evidence record.
Status: superseded historical P5 local evidence; ComponentUnionLcm claim downgraded by P3 re-review blocker fix on 2026-07-08.
Authority: non-authoritative; tests and command output are evidence only.

## Scope

Historical P5 evidence attempted target certificate verification coverage for:

- `IdealMembership`
- `RadicalMembership`
- `GuardedRadicalMembership`
- `CompositeCover` with `SameIdealGcd`
- `CompositeCover` with `ComponentUnionLcm` and an explicit component-union source marker; this item is downgraded below because a marker string is not replay-verifiable source data

Current downgrade:
- A nonempty `ComponentUnionSource.description` is not replay-verifiable evidence.
- Current production verifier code in `src/verifier.rs` computes/checks the lcm support but returns `CertificateDesignGap` for description-only component-union sources.
- Current regression test: `tests/verifier_tests.rs::component_union_lcm_without_replay_source_is_design_gap`.

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

RP-P5 re-review returned `PASS` for the historical bounded implementation. The current claim is narrower: target certificate verification accepts only nonzero supports whose support variable is the problem target, including composite parent and recursively verified composite children, and recomputes exact rational polynomial identities for ideal, radical, guarded radical, and same-ideal gcd certificates. `ComponentUnionLcm` is not accepted from a description-only source; it returns a design-gap result until replay-verifiable source data exists.

This does not verify candidate generation, residual windows, fixed proof construction, solver route integration, root isolation, or exact-image behavior.
