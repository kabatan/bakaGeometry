# P13 Evidence — Exact roots and exact-image fail-closed behavior

Status: implementation evidence collected; RP-P13 boundary review passed.

Phase: P13 — Exact root isolation and exact-image fail-closed behavior.

R-IDs read for this phase:
- BS-ROOT-001
- BS-IMAGE-001
- BS-IMAGE-002
- BS-SCOPE-003

Plan anchors:
- PLAN.md P13
- REVIEWER_PROMPTS.md RP-P13

Implementation files:
- `src/roots.rs`
- `src/exact_image.rs`
- `src/solver.rs`
- `tests/root_isolation_tests.rs`
- `tests/solver_status_tests.rs`

Implemented behavior:
- `isolate_real_roots_squarefree` builds rational isolating intervals using a Sturm sequence over exact rational arithmetic.
- `exact_root_count_in_interval` recomputes the exact Sturm count for a rational interval.
- `CertifiedCandidateCover` construction now includes support, squarefree support, exact real root records, and the `TargetCertificate`.
- The conservative exact-image classifier returns incomplete classification for all roots until a complete classifier exists.
- `build_certified_exact_target_image` returns an exact image only when every root has a matching classification.
- `TryExactImage` records incomplete classification in trace and returns a candidate cover.
- `RequireExactImage` records incomplete classification in trace and fails closed with `NoVerifiedTargetCertificate`.

Verification commands:
- `cargo fmt --check` — passed.
- `cargo test --lib roots` — passed.
- `cargo test --lib exact_image` — passed.
- `cargo test --test root_isolation_tests` — passed.
- `cargo test --test solver_status_tests` — passed.
- `cargo test` — passed: 55 unit tests, 27 integration tests, and doc tests.
- Public API boundary scan of `src/lib.rs` and `src/options.rs` for internal route/proof/fallback/root/classifier names — no matches.
- Static forbidden-substring scan over `src/*.rs`, `Cargo.toml`, and `README.md` — no matches.

Targeted tests added or updated:
- `roots::tests::squarefree_quadratic_has_two_exact_isolating_intervals`
- `exact_image::tests::exact_image_requires_every_root_classified`
- `root_isolation_tests::quadratic_support_returns_two_rational_root_intervals`
- `solver_status_tests::try_exact_image_keeps_candidate_cover_when_classifier_incomplete`
- `solver_status_tests::require_exact_image_fails_closed_when_classifier_incomplete`

Claim ceiling:
- Local P13 evidence supports exact rational root-isolation implementation and exact-image fail-closed behavior for reviewed code paths.
- This evidence does not mark any R-ID VERIFIED and does not claim full source fidelity before RP-P13 review.

Reviewer result:
- `guardian_boundary_reviewer` RP-P13 result: PASS.
- Reviewer noted that root records use rational intervals, Sturm-style counting pushes intervals only at exact count 1, candidate covers carry support/squarefree support/root records/certificate, exact image construction requires all root classifications, and Try/Require exact-image modes fail closed when incomplete.
- Reviewer also prohibited stronger claims: no R-ID verified claim, no general exact target image classifier completion claim, and no production-safe/source-faithful/acceptance-complete claim.
