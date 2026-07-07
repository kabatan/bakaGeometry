# P12 Solver Integration Evidence

Scope: Phase P12, `BS-DAG-001`, `BS-SOLVER-001`, `BS-SOLVER-002`, `BS-SCOPE-002`, `BS-SCOPE-004`, `MECH-DAG`, `MECH-SOLVER`.

## Files inspected or changed

- `src/dependency_dag.rs`
- `src/solver.rs`
- `src/test_support.rs`
- `tests/candidate_route_forcing_tests.rs`
- `tests/fallback_elimination_solver_tests.rs`

## Implementation evidence

- `build_target_dependency_dag` constructs relation-variable incidence, target-level reachability, incident relation lists, and a target cone only from polynomial supports and variable indices.
- `plan_certificate_windows` emits bounded finite row-closed windows through `make_row_closed_certificate_window`.
- `certificate_window_schedule` / `CertificateWindowSchedule` supports unbounded degree advancement as a lazy iterator; tests take a finite prefix and verify degree growth.
- `solve_target` now builds the DAG, plans windows, loops over planned windows, runs all candidate routes against each planned window, then tries fixed proof, obstruction expansion, low-degree repair, localized Schur inspection, and only then complete fallback.
- After an initial RP-P12 review returned `FAIL_FIXABLE`, unbounded `max_window_degree = None` execution was changed to use the lazy schedule directly. It no longer exhausts a fixed finite prefix and then calls complete fallback. If a finite matrix/candidate resource boundary is supplied and reached during unbounded scheduling, solver returns `FiniteResourceFailure` without fallback.
- Obstruction expansion no longer substitutes a fixed degree cap when `max_window_degree = None`; bounded execution still honors the supplied degree cap.
- Candidate success now flows through `return_verified_cover`, `refine_and_finalize`, and `maybe_classify_exact_target_image`.
- `refine_and_finalize` uses gcd for multiple same-ideal verified target certificates and produces a `CompositeCover` with `CompositeRule::SameIdealGcd`.
- `ExactImageMode::RequireExactImage` fails closed until a complete exact real-fiber classifier exists; `TryExactImage` keeps a candidate cover and records incomplete exact-image classification in trace.
- Empty and no-target statuses remain separate from target cover.

## Tests and scans

Commands run from `C:\Users\bakat\OneDrive\ドキュメント\bakaGeometry`:

```text
cargo fmt --check
cargo test --lib dependency_dag
cargo test --lib solver
cargo test --lib test_support
cargo test --test candidate_route_forcing_tests
cargo test --test fallback_elimination_solver_tests
cargo test
```

Observed result after the unbounded-schedule fix: all listed commands exited 0. Full `cargo test` passed 53 lib tests, 2 anti-simplification tests, 7 candidate route integration tests, 6 exact algebra tests, 2 fallback solver tests, 7 verifier integration tests, and 0 doctests.

Additional scans:

```text
Select-String -Path src\lib.rs,src\options.rs -Pattern <internal route/proof/fallback/DAG API names> -SimpleMatch
```

Observed result: no matches.

```text
forbidden production scan over src\*.rs, Cargo.toml, README.md for simplification/name-sentinel substrings
```

Observed result: no matches.

## Bounded claim

RP-P12 boundary review returned `PASS` after the unbounded-schedule fix. This closes the local P12 review gate only.

This evidence supports local P12 review only. It does not claim final source fidelity, root isolation, exact-image completion, acceptance completion, production readiness, or R-ID verification.
