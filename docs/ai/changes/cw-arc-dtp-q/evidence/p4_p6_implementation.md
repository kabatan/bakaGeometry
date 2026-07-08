# P4-P6 Implementation Evidence

Purpose: scoped evidence for `CW-ARC-DTP-Q-FULL-V3` P4 through P6.
Status: scoped P4-P6 implementation evidence; spec, quality, and boundary reviews passed.
Authority: evidence only. `BASE_SPEC.md` and `PLAN.md` remain controlling.

Date: 2026-07-08.

## Scope

Implemented scoped P4-P6 work after the P3 re-review blocker fixes.

Out of scope:
- P7 and later candidate route closure.
- P15 exact no-target eliminant replay.
- P16 exact real image replay.
- Final V3 completion.

## P4 Window and Residual Oracle

Production source:
- `src/window.rs`
- `src/residual.rs`
- `src/candidate_residual.rs`

Implemented/confirmed:
- Row-closed certificate windows recompute rows from target powers and multiplier products.
- Membership and target-power matrices rebuild canonical row sets and ignore forged `row_monomials`.
- Q-to-Fp matrix conversion rejects denominator-bad primes.
- ResidualCyclic explicitly excludes primes that divide denominators in system equations, guard certificates, compression replay data, or target-power vectors before modular reduction.
- Dense residual oracle is tested against brute-force column-space membership over multiple primes and pseudo-random small matrices.
- ResidualCyclic solves the modular membership system `M_p,W u = N_p,d c` for each residual relation and records active multiplier support from nonzero solution entries.

Executable tests:
- `window::tests::row_closed_window_recomputes_target_powers_and_multiplier_products`
- `window::tests::modular_matrix_reduction_filters_denominator_bad_primes`
- `residual::tests::residual_oracle_matches_bruteforce_column_space_over_multiple_primes`
- `residual::tests::dense_residual_oracle_reduces_zero_exactly_for_column_space_vectors`
- `candidate_residual::tests::residual_witness_active_support_is_solved_not_full_window_copy`
- `candidate_residual::tests::residual_prime_filter_reads_guard_rationals`
- `candidate_residual::tests::residual_prime_filter_reads_replay_rationals`

## P5 Fixed Proof and Fair Schedule

Production source:
- `src/proof.rs`
- `src/proof_learning.rs`
- `src/solver.rs`

Implemented/confirmed:
- Fixed proof builds `H`, solves the exact Q system, restores multipliers, and gates certificate construction on exact recomputation of `H - sum q_i F_i`.
- Guarded radical mode builds the guard product only from verifier-accepted system guard certificates.
- Fair proof schedule now includes support degree with certificate mode, covering finite `(d_B, a, e)` tuples within the configured prefix.
- Solver expands proof windows by scheduled support degree before fixed proof attempts.
- `max_window_degree = None` uses a finite fair prefix from `max_proof_weight` instead of an infinite window stream that blocks fallback, and the same effective bound is passed into fixed proof obstruction expansion.
- Initial proof windows include modular active supports derived from trace witnesses; obstruction expansion adds predecessor supports.

Executable tests:
- `proof::tests::fixed_proof_builds_exact_ideal_certificate`
- `proof::tests::semantic_nonzero_guard_reaches_guarded_radical_proof_mode`
- `proof::tests::fair_proof_schedule_covers_support_degree_power_and_guard_tuples`
- `proof_learning::tests::modular_witness_support_seeds_initial_proof_window`
- `proof_learning::tests::obstruction_expansion_adds_predecessor_support`
- `solver::tests::max_window_degree_none_uses_fair_prefix_then_reaches_fallback`

## P6 Candidate Normalization and Factor Schedule

Production source:
- `src/normalize.rs`
- `src/univariate.rs`
- `src/crt.rs`
- `src/rational_reconstruction.rs`

Implemented/confirmed:
- Modular supports are normalized to monic representatives over valid prime fields.
- Separate single-prime modular candidates are merged before CRT/rational reconstruction only when they also carry a matching route-family witness key; candidates without correspondence evidence remain separate.
- Duplicate-prime alternatives are preserved rather than silently skipped, while distinct-prime combinations from the same correspondence group still reach CRT/rational reconstruction.
- Multi-prime reconstruction uses CRT plus rational reconstruction; no first-prime-only lift is used in the multi-prime path.
- Single-prime modular candidates remain modular-only and do not populate `reconstructed`; they are not ranked as certified or multi-prime reconstruction evidence.
- `factor_schedule` now schedules nontrivial exact Q factors and the original candidate, rather than returning only the original candidate.
- Squarefree/factor trials are only proof attempts; ranking remains attempt order and not adoption.

Executable tests:
- `normalize::tests::multi_prime_modular_candidate_uses_crt_not_first_prime`
- `normalize::tests::separate_modular_candidates_merge_before_crt_reconstruction`
- `normalize::tests::single_prime_modular_candidate_remains_modular_only`
- `normalize::tests::unrelated_same_degree_modular_candidates_are_not_merged_or_dropped`
- `normalize::tests::duplicate_prime_modular_alternatives_are_preserved`
- `normalize::tests::duplicate_prime_alternatives_still_form_distinct_prime_reconstructions`
- `normalize::tests::modular_support_is_normalized_to_monic_representative`
- `normalize::tests::factor_schedule_trials_factors_without_replacing_original_candidate`
- `exact_algebra_tests::squarefree_factorization_splits_conformance_family`

## Local Evidence

Commands:

```text
cargo fmt --check
cargo test
git diff --check
```

Result:

```text
All passed. cargo test covered 121 tests.
```

Review record:
- `docs/ai/changes/cw-arc-dtp-q/reviews/p4_p6_checkpoint_2026-07-08.md`

## Claim Ceiling

This file supports only the scoped P4-P6 implementation checkpoint. It does not mark any R-ID verified and does not support P7+, final V3 completion, or source-faithfulness claims.
