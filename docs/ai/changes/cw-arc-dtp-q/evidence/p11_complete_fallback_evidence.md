# P11 Complete Fallback Evidence

Scope: Phase P11, `BS-FALLBACK-001`, `BS-CERT-004`, `MECH-COMPLETE-FALLBACK`.

## Files inspected or changed

- `src/fallback_elimination.rs`
- `src/solver.rs`
- `src/verifier.rs`
- `tests/candidate_route_forcing_tests.rs`
- `tests/fallback_elimination_solver_tests.rs`

## Implementation evidence

- `complete_target_elimination_fallback` returns only:
  - `CertifiedSupport(TargetCertificate)`
  - `CertifiedEmpty(EmptyAdmissibleSetCertificate)`
  - `CertifiedNoTargetEliminant(NoTargetEliminantCertificate)`
  - `ResourceFailure(CostTrace)`
- Target-support fallback searches exact target-power relations against the exact rational membership matrix, then calls `prove_fixed_target` before returning a support certificate.
- Empty fallback solves an exact rational membership identity for `1 = sum q_i F_i`.
- No-target fallback is conservative: it emits a certificate only for the exact monomial non-target ideal class implemented in both fallback and verifier, and top-level solver maps it to `CertifiedNoNonzeroTargetEliminant`, not an exact real image.
- `solve_target` now performs early exact empty certification, then candidate proof/repair paths, localized Schur inspection, and only then complete target elimination fallback.
- Nonzero constant support is not returned as a candidate cover; empty cases go through an empty certificate path.
- Fallback success paths emit `target_elimination:*` trace events; candidate proof success tests assert these events are absent.
- After an initial RP-P11 review returned `FAIL_FIXABLE`, the test-only top-level route-control harness was extended so `allow_complete_fallback = false` fails if complete fallback is reached. The new unit test `complete_fallback_disabled_route_control_fails_on_reach` covers that boundary.

## Tests and scans

Commands run from `C:\Users\bakat\OneDrive\ドキュメント\bakaGeometry`:

```text
cargo fmt --check
cargo test --test fallback_elimination_solver_tests
cargo test --test candidate_route_forcing_tests
cargo test --lib fallback_elimination
cargo test --lib test_support
cargo test
```

Observed result after the route-control fix: all listed commands exited 0. Full `cargo test` passed 47 lib tests, 2 anti-simplification tests, 7 candidate route integration tests, 6 exact algebra tests, 2 fallback solver tests, 7 verifier integration tests, and 0 doctests.

Additional scans:

```text
Select-String -Path src\lib.rs,src\options.rs -Pattern <internal route/proof/fallback API names> -SimpleMatch
```

Observed result: no matches.

```text
forbidden production scan over src\*.rs, Cargo.toml, README.md for simplification/name-sentinel substrings
```

Observed result: no matches.

## Bounded claim

RP-P11 boundary review returned `PASS` after the route-control fix. This closes the local P11 review gate only.

This evidence supports local P11 review only. It does not claim final source fidelity, P12 solver orchestration completion, dependency DAG/window planning completion, root isolation, exact-image classification, acceptance completion, or R-ID verification.
