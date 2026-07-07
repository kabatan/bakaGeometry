# P15 Algorithm Evidence

Implemented/verified behavior:

- Squarefree support rejects zero support and normalizes the exact squarefree part.
- Sturm isolation uses exact rational arithmetic, exact Cauchy bound, exact Sturm sequence, and rational interval subdivision.
- Descartes/Vincent isolation is a distinct exact implementation: each interval is transformed by a rational Mobius map, coefficient sign variations are counted exactly, and intervals with one variation are accepted as isolating intervals.
- Rational split search no longer has a fixed insufficient cap.
- Root records bind support hash, root index, and rational isolating interval.
- Candidate decode deterministically maps root records to candidates and hashes target/support/root-index/interval.
- Algebraic root records and sign/Thom helpers preserve exact rational interval/hash bindings.

Regression coverage:

- `descartes_vincent_isolates_same_roots_without_sturm_alias`
- `p12_squarefree_rejects_zero_and_removes_repeated_factors`
- `p12_exact_isolation_handles_no_real_one_rational_and_multiple_rational_roots`
- `p12_exact_isolation_handles_irrational_repeated_and_high_coefficient_roots`
- `p12_decode_candidates_bind_target_support_index_interval_and_hash`
- `p12_algebraic_records_bind_interval_hash_and_compare_disjoint_roots`
- `sign_at_isolated_root_is_exact_when_interval_has_no_guard_root`
- `thom_encoding_records_derivative_signs`
- `p12_candidate_cover_finalizer_returns_exact_roots_and_nonplaceholder_candidates`
- `p15_support_producing_candidate_cover_suite`
- `p15_anti_decoration_replay_rejects_tamper_and_deletion`

Static audit:

- `audit_v4_conformance.py --phase P15 --strict` checks P15 files, required symbols, Descartes-not-Sturm delegation, no `f32`/`f64`/float markers, no old `..=128` split cap, root-record binding, candidate hash binding, algebraic-root binding, and sign/Thom markers.

Residual performance note:

- `p15_acceptance_stress` passed but took about 215 seconds, with `p15_anti_decoration_replay_rejects_tamper_and_deletion` being the long case. This is not a P15 correctness failure, but it is relevant performance evidence for later closure.
