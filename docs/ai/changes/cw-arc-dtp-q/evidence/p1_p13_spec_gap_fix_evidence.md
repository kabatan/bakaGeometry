# P1-P13 Spec-Gap Fix Evidence

Status: current implementation evidence for the P1-P13 blocker-fix source imported on 2026-07-08; F6 spec, quality, and boundary reviews passed for this narrow scope.
Authority: evidence only. `P1_P13_SPEC_GAP_FIX_BASE_SPEC_DELTA.md` and production source define correctness.

## Implemented Fixes

- F1 top-level unbounded execution: `solve_target` now routes `max_window_degree=None` or `max_proof_weight=None` through `GlobalSolveSchedule`, which fairly interleaves candidate-window degree, proof tuple weight, and fallback budgets. The old `resource:unbounded_proof_requires_bound` production sentinel is removed.
- F2 HiddenVariableSparseResultant: the route now builds sparse support-sum eliminant templates, records `SparseResultantWitnessTrace`, derives active multiplier supports from the relation, and includes non-chain 3-equation conformance.
- F3 SliceSpecialization: the route now constructs deterministic generic affine equations over all non-target variables, appends them to the full sliced system, records affine trace data, and checks denominator-admissibility per prime.
- F4 NormTraceTower: the route now supports nonconstant guarded leading coefficients over lower tower variables, verifies guard certificates via `verify_guard_certificate`, supports `DerivedProduct` replay, and computes leading inverses in the prior quotient basis.
- F5 regression gates: static tests now reject reintroduction of the old unbounded fail-close sentinel, total-degree Macaulay helper in the resultant route, and empty-semantic-guard tower replay.

## Key Tests

Latest command:

```text
cargo test -- --nocapture
result: pass
```

Observed result:

```text
119 lib tests
5 anti-simplification tests
7 candidate route integration tests
10 exact algebra tests
3 fallback solver tests
5 guard/compression tests
1 root isolation test
2 solver status tests
16 verifier tests
0 doctests
```

Specific regression anchors:

- F1: `unbounded_high_radical_power_finds_exact_cover`; `bounded_small_prefix_does_not_use_unbounded_radical_power`; `unbounded_global_schedule_reaches_arbitrary_tuple`; `unbounded_spurious_route_candidate_does_not_starve_complete_fallback_budget`; `solver_production_does_not_fail_close_unbounded_proof_bounds`.
- F2: `resultant_route_handles_two_polynomial_hidden_resultant`; `resultant_route_handles_non_chain_three_equation_eliminant`; `sparse_template_support_sums_do_not_fill_total_degree_macaulay_shape`; `multi_prime_monic_modular_candidate_reconstructs_rational_then_primitive_integer`; `resultant_route_forcing_solves_sr_f1_two_polynomial_hidden_resultant_without_fallback`; `resultant_route_forcing_solves_non_chain_sparse_eliminant_without_fallback`; `resultant_route_does_not_use_total_degree_macaulay_support_helper`.
- F3: `slice_route_records_affine_slice_candidate_only`; `affine_slice_admissibility_rejects_singular_or_bad_denominator_trace`; `slice_route_rejects_prime_with_input_denominator_obstruction`; `slice_route_forcing_selects_only_slice_candidates`; `slice_route_forcing_solves_finite_target_family_without_complete_fallback`; `slice_route_forcing_solves_affine_coupled_family_without_complete_fallback`; `slice_route_forcing_rejects_spurious_candidate_without_fallback`; `slice_route_tampered_certificate_is_rejected`.
- F4: `tower_route_uses_verified_guarded_nonconstant_leading_coefficient`; `tower_route_forcing_solves_guarded_nonmonic_tower_without_fallback`; `tower_guard_replay_does_not_verify_against_empty_semantic_guards`; `compression_replay_rejects_guard_and_replay_tampering`.
- F5 proof-gate continuity: route spurious-rejection tests for Direct, Residual, Krylov, Resultant, Slice, and Tower; tamper tests for each route; `origin_count_does_not_certify_candidate_without_exact_proof`; `factor_schedule_resource_failure_produces_no_trials`; `unbounded_solver_no_target_eliminant_is_design_gap_until_p15_replay`.

## F6 Review

- `spec_verifier`: initial `FAIL_FIXABLE`; passed after finite per-work-item unbounded proof attempts and SR-F1 route-forcing evidence were added.
- `quality_reviewer`: initial `FAIL_FIXABLE`; passed after unbounded no-target-eliminant fallback returned `CertificateDesignGap` with no success certificate.
- `guardian_boundary_reviewer`: passed the narrow boundary claim that F1-F5 local implementation has passing spec/quality review and local tests.

## Claim Boundary

This evidence supports proceeding to later phases after P1-P13 blocker-fix review. It does not claim final V3 completion, P14/P15/P16 completion, production safety, source-faithfulness, acceptance completeness, or any R-ID verified status.
