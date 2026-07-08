# P7-P13 Route Closure Evidence

Status: scoped P7-P13 implementation evidence reviewed; spec, quality, and boundary reviews passed.
Authority: evidence only. `BASE_SPEC.md` and `P7_P13_ROUTE_CLOSURE_BASE_SPEC_DELTA.md` control correctness.

## Scope

This evidence covers the P7-P13 route-closure implementation slice only:

- P7 DirectTargetEquation route closure.
- P8 ResidualCyclic route closure.
- P9 TargetCyclicKrylov quotient/residual recurrence closure.
- P10 HiddenVariableSparseResultant route closure.
- P11 SliceSpecialization full sliced-system route closure.
- P12 NormTraceTower guarded-nonmonic and non-unit target-coefficient closure.
- P13 LocalizedSchur exact-certificate path plus support-information path.

It does not close P14+, P15, P16, final V3 completion, source-faithfulness, production-safety, readiness, acceptance-complete, or any R-ID verified claim.

## Source Anchors

- `src/candidate_direct.rs`
- `src/candidate_residual.rs`
- `src/candidate_krylov.rs`
- `src/candidate_resultant.rs`
- `src/candidate_slice.rs`
- `src/candidate_tower.rs`
- `src/repair_schur.rs`
- `src/normalize.rs`
- `src/test_support.rs`
- `tests/verifier_tests.rs`

## Implementation Notes

- P9 now builds an explicit exact Q quotient residual handle by taking the left nullspace of the membership matrix and computing residual classes of target powers before recurrence detection.
- P11 now builds a full sliced system containing all original equations plus affine slice equations, then invokes ResidualCyclic first and HiddenVariableSparseResultant if needed inside the sliced system. Slice observations remain candidates only.
- P12 now permits non-unit target equation coefficients and requires a matching guard certificate before using nonmonic tower leading coefficients.
- P13 now searches local membership null relations for target-only boundary relations and replays any such relation through `prove_fixed_target` on the original system before returning `SchurRepairOutput::Certified`.
- `factor_schedule` schedules factor/original trials only when factorization status is `Complete`; `ResourceFailure` produces no proof trials.
- `origin_evidence` remains ranking evidence only. Candidate adoption still requires fixed proof and verifier replay.
- Test-only route control is exposed as `RouteForcing` plus `solve_target_with_route_forcing`; it can disable complete fallback, forbid other heavy routes, and inject spurious candidate supports to prove the exact-proof gate rejects them.
- `FairProofSchedule::unbounded()` remains distinct from top-level solver behavior. The top-level solver still requires an explicit finite `max_proof_weight` after early empty-set certification and otherwise returns `FiniteResourceFailure`.

## Test Evidence

Latest command:

```text
cargo test
result: pass
```

Observed test counts in the latest run:

```text
105 lib tests
2 anti-simplification tests
7 candidate route integration tests
10 exact algebra tests
2 fallback solver tests
5 guard/compression tests
1 root isolation test
2 solver status tests
16 verifier tests
0 doctests
```

Route-specific test pointers:

- Direct route forcing/no-fallback/exact-proof-gate: `direct_route_forcing_selects_only_direct_candidates`; `direct_route_forcing_solves_without_other_routes_or_complete_fallback`; `direct_route_forcing_rejects_spurious_candidate_without_fallback`.
- Direct tamper: `direct_route_tampered_certificate_is_rejected`.
- Residual route forcing/no-fallback/exact-proof-gate: `residual_route_forcing_selects_only_residual_candidates`; `residual_route_forcing_solves_without_other_routes_or_complete_fallback`; `residual_route_forcing_rejects_spurious_candidate_without_fallback`.
- Residual data-flow: `residual_witness_active_support_is_solved_not_full_window_copy`; `residual_prime_filter_reads_guard_rationals`; `residual_prime_filter_reads_replay_rationals`; `multi_prime_modular_candidate_uses_crt_not_first_prime`.
- Residual tamper: `residual_route_tampered_certificate_is_rejected`.
- Krylov route forcing/no-fallback/exact-proof-gate: `krylov_route_forcing_selects_only_krylov_candidates`; `krylov_route_forcing_solves_without_other_routes_or_complete_fallback`; `krylov_route_forcing_rejects_spurious_candidate_without_fallback`.
- Krylov quotient/residual recurrence: `krylov_route_uses_target_power_recurrence`.
- Krylov tamper: `krylov_route_tampered_certificate_is_rejected`.
- Resultant route forcing/no-fallback/exact-proof-gate: `resultant_route_forcing_selects_only_resultant_candidates`; `resultant_route_forcing_solves_without_other_routes_or_complete_fallback`; `resultant_route_forcing_rejects_spurious_candidate_without_fallback`.
- Resultant 3-polynomial route: `resultant_route_uses_three_polynomial_expansion`.
- Resultant tamper: `resultant_route_tampered_certificate_is_rejected`.
- Slice route forcing/no-fallback/exact-proof-gate: `slice_route_forcing_selects_only_slice_candidates`; `slice_route_forcing_solves_finite_target_family_without_complete_fallback`; `slice_route_forcing_rejects_spurious_candidate_without_fallback`.
- Slice exact-proof gate: `slice_candidate_route_does_not_adopt_without_fixed_proof`.
- Slice full-system witness: `slice_route_records_affine_slice_candidate_only`.
- Slice tamper: `slice_route_tampered_certificate_is_rejected`.
- Tower route forcing/no-fallback/exact-proof-gate: `tower_route_forcing_selects_only_tower_candidates`; `tower_route_forcing_solves_without_other_routes_or_complete_fallback`; `tower_route_forcing_rejects_spurious_candidate_without_fallback`.
- Tower guarded-nonmonic and non-unit target coefficient: `tower_route_requires_guard_for_nonmonic_leading_coefficient`; `tower_route_allows_non_unit_target_coefficient`.
- Tower tamper: `tower_route_tampered_certificate_is_rejected`.
- LocalizedSchur support-information path: `schur_repair_builds_local_membership_only`; `uncertified_schur_relation_is_support_info_only`.
- LocalizedSchur exact certificate path and solver-level no-fallback adoption: `schur_repair_returns_exact_certificate_for_target_only_local_relation`; `localized_schur_certifies_after_spurious_seed_without_complete_fallback`.
- Factorization false-complete guard: `factor_schedule_resource_failure_produces_no_trials`; `factorization_reports_resource_failure_instead_of_false_complete_when_bounds_exceeded`.
- Origin evidence non-adoption: `origin_count_does_not_certify_candidate_without_exact_proof`; `same_reconstructed_support_from_two_origins_is_merged_and_ranked_by_origin_count`; `different_supports_from_different_origins_are_not_merged`.

## Claim Boundary

This evidence supports only the scoped claim that P7-P13 route closure was implemented, locally tested, and reviewed under the admitted delta. It does not mark any requirement verified, does not permit passing from P4-P6 primitives alone, and does not support P14+, final V3 completion, source-faithfulness, production-safety, readiness, or acceptance-complete claims.
