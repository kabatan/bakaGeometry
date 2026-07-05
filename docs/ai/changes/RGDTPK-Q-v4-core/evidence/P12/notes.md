# P12 Notes

P12 implements the public roots layer required for exact squarefree support processing, exact real root isolation, decoded target candidates, and mandatory algebraic root records.

`squarefree_support` rejects zero support and computes the exact squarefree support using the existing derivative/gcd-backed univariate arithmetic. It does not use floating root finding or approximate factorization.

`isolate_real_roots` squarefrees first, then delegates to exact Sturm isolation or the existing Descartes-selected exact route. `algebra/real_root.rs` now also exposes `isolate_real_roots_sturm_with_max_width`, which keeps the same exact Sturm counting logic but refines intervals until a positive rational width bound is satisfied.

P12 fixed a foundational exactness issue in `RationalQ`: the derived `Ord` compared numerator and denominator lexicographically, which made rational interval containment wrong for values such as `3/2`. `RationalQ` now compares by exact cross-products, and the full suite still passes.

`TargetCandidate` now binds target, support hash, root index, isolating interval, and candidate hash. `decode_candidates` derives those candidates deterministically from exact root records. Run replay now includes interval and candidate hash in `hash_decoded_candidates` and recomputes candidate hashes in `verify_roots_and_candidates`.

`finalize_candidate_cover_result` in `compose/final_support.rs` wires P12 into the production support-producing result construction path: it computes exact squarefree support, exact real root isolation, and decoded candidates before returning `CertifiedCandidateCover`. The integration test `geosolver-core/tests/p12_roots_decode_integration.rs` verifies that a nonzero target support returns exact roots and non-placeholder decoded candidates.

Run replay now requires candidate completeness: when roots are present, decoded candidates must have the same root-index set, no duplicates, matching intervals, and matching recomputed candidate hashes. `p12_replay_rejects_candidate_omission_and_duplicates_even_when_hashes_match` proves omitted or duplicate candidates are rejected even if run-certificate hashes are rebuilt around the tampered result.

`AlgebraicRootRecord` stores the support polynomial, support hash, root index, isolating interval, and root hash. It rejects support-hash mismatches when converting from a root record. The comparison helper returns equality for the same support/root index, orders disjoint rational intervals exactly, and returns `None` for unresolved overlap. The refinement helper re-isolates with an exact rational width bound and errors if the requested root index disappears.

P12 focused tests cover the required cases from the Plan:

- no real roots;
- one rational root;
- multiple rational roots;
- irrational roots;
- repeated roots via squarefree support;
- high-coefficient roots;
- non-placeholder decoded candidate binding;
- algebraic-root interval/hash binding, comparison, and refinement.

The first P12 `spec_verifier` checkpoint returned `FAIL_FIXABLE` because P12 initially lacked support-producing integration evidence, replay did not reject omitted candidates, and `decode_candidates` was not wired outside unit tests. The remediation added the production candidate-cover finalizer, an integration test for non-placeholder roots/candidates, replay candidate completeness checks, and omitted/duplicate-candidate replay tamper tests.

P12 closes only `MECH-011` if review passes. It does not claim exact-image semantics, public orchestration, acceptance stress, final readiness, source-faithful status, or any R-ID as VERIFIED.
