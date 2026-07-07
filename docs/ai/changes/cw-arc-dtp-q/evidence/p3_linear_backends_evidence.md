# P3 Linear Backends Evidence

Purpose: phase evidence record.
Status: P3 local evidence, pending RP-P3 review.
Authority: non-authoritative; tests and command output are evidence only.

## Scope

P3 implemented:

- Prime modulus validation and finite-field arithmetic.
- Exact rational Gaussian elimination with a left-null obstruction for inconsistent systems.
- Finite-field row reduction, nullspace computation, and column-relation search.

No finite-field result is promoted to a target certificate or solver success path.

## Test-First Record

Unit tests were added before implementation. The first valid `cargo test` run failed because the P3 APIs and solve routines did not exist. After implementation, one finite-field nullity assertion failed because the test matrix had rank 1 over `Fp`, so nullity was 2. The test oracle was corrected to match the matrix rank, then the full suite passed.

## Commands

Run from `C:\Users\bakat\OneDrive\ドキュメント\bakaGeometry`.

```text
cargo test
initial result: failed before implementation, missing P3 functions and types
```

```text
cargo test
intermediate result: one Fp nullity assertion failed; root cause was test oracle, not production arithmetic
```

```text
cargo test
final result: pass
unit tests: 8 passed
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

## P3 Claim Boundary

This supports local finite-field and linear-algebra backend readiness for review. It does not verify residual-window construction, candidate generation, fixed proof adoption, verifier certificates, root isolation, or exact-image behavior.

## Review Recovery

Initial RP-P3 review returned `FAIL_FIXABLE`: finite-field `add` and `sub` used `u64` intermediate addition, which could overflow for large admitted primes.

Fix:

- `PrimeModulus::{add,sub}` now use `u128` intermediate arithmetic.
- Prime validation now uses deterministic Miller-Rabin bases for the `u64` domain instead of trial division.
- Added `finite_field_operations_avoid_large_prime_overflow`.

Post-fix evidence:

```text
cargo fmt --check
result: pass
```

```text
cargo test
result: pass
unit tests: 8 passed
integration tests: 8 passed total
```

```text
production forbidden-pattern scan over src/*.rs, Cargo.toml, README.md
result: pass, no hits
```
