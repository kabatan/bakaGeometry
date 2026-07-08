# P4-P6 Implementation Evidence

Purpose: scoped evidence for `CW-ARC-DTP-Q-FULL-V3` P4 through P6.
Status: scoped P4-P6 implementation evidence with P5/P6 blocker-fix addendum; latest blocker-fix review pending.
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
- `src/proof_schedule.rs`
- `src/proof_learning.rs`
- `src/solver.rs`
- `src/repair_multiple.rs`

Implemented/confirmed:
- Fixed proof builds `H`, solves the exact Q system, restores multipliers, and gates certificate construction on exact recomputation of `H - sum q_i F_i`.
- Guarded radical mode builds the guard product only from verifier-accepted system guard certificates.
- `FairProofSchedule::unbounded()` is a lazy deterministic iterator over all finite `(multiplier_degree, support_power, guard_power)` tuples.
- Bounded proof search uses the explicitly named bounded-prefix API; no production path uses `max_proof_weight.unwrap_or(6)` or `max_window_degree.unwrap_or(6)`.
- `solve_target` returns `FiniteResourceFailure` with trace `resource:unbounded_proof_requires_bound` when no explicit proof bound is available after early empty-set certification.
- Solver expands proof windows by scheduled multiplier degree before fixed proof trials.
- Initial proof windows include modular active supports derived from trace witnesses; obstruction expansion adds predecessor supports.

Executable tests:
- `proof::tests::fixed_proof_builds_exact_ideal_certificate`
- `proof::tests::semantic_nonzero_guard_reaches_guarded_radical_proof_mode`
- `proof_schedule::tests::fair_schedule_reaches_every_tuple_up_to_weight_4`
- `proof_schedule::tests::fair_schedule_is_lazy_and_not_a_fixed_default_prefix`
- `proof_schedule::tests::bounded_prefix_is_explicitly_bounded_by_argument`
- `proof_learning::tests::modular_witness_support_seeds_initial_proof_window`
- `proof_learning::tests::obstruction_expansion_adds_predecessor_support`
- `solver::tests::solve_target_without_proof_bound_does_not_silently_use_default_six`
- `solver::tests::max_window_degree_none_does_not_insert_hidden_default_window_bound`
- `fallback_elimination::tests::fallback_without_window_bound_is_resource_failure_not_hidden_capped_search`
- `fallback_elimination::tests::early_empty_without_window_bound_does_not_use_hidden_capped_search`
- `repair_multiple::tests::low_degree_multiple_without_window_bound_does_not_use_hidden_capped_search`
- `repair_schur::tests::schur_repair_without_window_bound_does_not_use_hidden_capped_search`

## P6 Candidate Normalization and Factor Schedule

Production source:
- `src/normalize.rs`
- `src/univariate.rs`
- `src/candidates.rs`
- `src/crt.rs`
- `src/rational_reconstruction.rs`

Implemented/confirmed:
- Modular supports are normalized to monic representatives over valid prime fields.
- Separate single-prime modular candidates are merged before CRT/rational reconstruction only when they also carry a matching route-family witness key; candidates without correspondence evidence remain separate.
- Duplicate-prime alternatives are preserved rather than silently skipped, while distinct-prime combinations from the same correspondence group still reach CRT/rational reconstruction.
- Multi-prime reconstruction uses CRT plus rational reconstruction; no first-prime-only lift is used in the multi-prime path.
- Single-prime modular candidates remain modular-only and do not populate `reconstructed`; they are not ranked as certified or multi-prime reconstruction evidence.
- `factor_squarefree_over_q` returns `FactorizationResult` with `Complete`, `Partial`, or `ResourceFailure` status and trace data.
- Exact Q factorization includes Kronecker-style factor search and splits reducible squarefree polynomials without rational roots, including `(T^2 + 1)(T^2 + 2)`.
- Complete factorization verifies exact division and product reconstruction; resource-limited factorization reports `ResourceFailure` instead of silently completing as `[original]`.
- `factor_schedule` surfaces factorization status through solver trace and schedules verified factors plus the original only when the factorization status permits factor trials.
- `TargetCandidate` carries `origin_evidence`; candidates with the same primitive reconstructed support are merged across origins.
- Ranking order uses exact/reconstructed status, degree, prime count, origin count, coefficient height, active support size, then deterministic tie-breakers. Origin count affects trial order only and never certifies or adopts a candidate.
- Squarefree/factor trials and multi-origin aggregation are only proof-trial ordering inputs; fixed exact proof remains required for adoption.

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
- `exact_algebra_tests::factorization_splits_product_of_irreducible_quadratics_without_rational_roots`
- `exact_algebra_tests::factorization_reports_resource_failure_instead_of_false_complete_when_bounds_exceeded`
- `exact_algebra_tests::factorization_product_reconstructs_original_squarefree_part`
- `normalize::tests::same_reconstructed_support_from_two_origins_is_merged_and_ranked_by_origin_count`
- `normalize::tests::different_supports_from_different_origins_are_not_merged`
- `solver::tests::origin_count_does_not_certify_candidate_without_exact_proof`

## P3 Regression Preservation

Executable tests:
- `verifier_tests::component_union_lcm_without_replay_source_is_design_gap`
- `fallback_elimination_solver_tests::solver_no_target_eliminant_is_design_gap_until_p15_replay`

## Local Evidence

Commands:

```text
cargo fmt --check
cargo test
git diff --check
```

Latest blocker-fix result:

```text
cargo test passed. The current run covered 132 tests.
```

Review record:
- `docs/ai/changes/cw-arc-dtp-q/reviews/p4_p6_checkpoint_2026-07-08.md`

## Claim Ceiling

This file supports only the scoped P4-P6 implementation checkpoint plus the P5/P6 blocker-fix evidence listed above. It does not mark any R-ID verified and does not support P7+, final V3 completion, or source-faithfulness claims.

Blocker-fix claim boundaries:
- P4 is closed only for the scoped row-closed window, residual oracle, and active-support witness behavior cited here.
- P5 is closed only if `FairProofSchedule` remains a true lazy fair iterator.
- Bounded proof prefix is not unbounded fairness.
- P6 is closed only if factorization distinguishes `Complete`, `Partial`, and `ResourceFailure`.
- P6 is not closed by rational-root-only factorization.
