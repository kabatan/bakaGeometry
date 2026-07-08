# Non-Simplification Manifests

Status: P7-P13 route closure evidence reviewed; spec, quality, and boundary reviews passed for the scoped delta.

Authority: evidence only. Manifest entries must be checked against production code.

Shared forbidden-pattern search set:

```text
Unsupported
expected
fixture
circle
distance
area
incircle
mixtilinear
orthic
RUR
coordinate_solution
solve_all
lex_param
len() != 2
polynomials.len() != 2
f64
f32
TODO
panic!("unsupported")
equations.len() != 2
v2_impl
new_algo
hack
legacy
temp
fallback_solver
toy
phase
```

## P5/P6 Blocker-Fix Shared Manifest

- Production call chain: `solve_target` -> bounded proof-prefix scheduling from `src/proof_schedule.rs` -> `try_candidate_certificate` -> `prove_fixed_target`.
- Controlling data-flow: `FairProofSchedule::unbounded()` is lazy; production solver uses only the explicitly bounded prefix when `max_proof_weight` is provided, and returns `FiniteResourceFailure` with `resource:unbounded_proof_requires_bound` when no proof bound is available after early empty-set certification.
- Fallback/repair data-flow: complete fallback, early empty certification, low-degree multiple repair, and localized Schur repair require explicit `max_window_degree`; without it they fail closed instead of running a hidden capped search.
- Factorization data-flow: `factor_squarefree_over_q` returns a status-bearing `FactorizationResult`; `factor_schedule` records the status through solver trace and schedules factor/original proof trials only when status is `Complete`.
- Exact replay oracle: factor trials and multi-origin ranking never adopt candidates directly; every scheduled support must pass fixed exact proof and verifier replay.
- Multi-origin data-flow: `TargetCandidate.origin_evidence` is merged only for equal primitive reconstructed supports; different supports remain distinct.
- Route-forcing tests: `solve_target_without_proof_bound_does_not_silently_use_default_six`; `fallback_without_window_bound_is_resource_failure_not_hidden_capped_search`; `early_empty_without_window_bound_does_not_use_hidden_capped_search`; `low_degree_multiple_without_window_bound_does_not_use_hidden_capped_search`; `schur_repair_without_window_bound_does_not_use_hidden_capped_search`; `factorization_splits_product_of_irreducible_quadratics_without_rational_roots`; `factorization_reports_resource_failure_instead_of_false_complete_when_bounds_exceeded`; `factor_schedule_resource_failure_produces_no_trials`; `same_reconstructed_support_from_two_origins_is_merged_and_ranked_by_origin_count`; `different_supports_from_different_origins_are_not_merged`; `origin_count_does_not_certify_candidate_without_exact_proof`.
- Non-simplification notes: no hidden proof-weight default, no rational-root-only factorization, no false complete factorization status, and no origin-count certificate authority.

## DirectTargetEquation

- Production call chain: `solve_target` -> `collect_candidate_routes` -> `DirectTargetEquationOracle::generate` -> `direct_target_equation_candidates` -> `try_candidate_certificate` -> `prove_fixed_target` -> `return_verified_cover`.
- Controlling data-flow: scans all input equations, accepts only nonzero equations depending only on the target variable, converts to primitive integer univariate support, and records direct-equation trace.
- Exact replay oracle: `prove_fixed_target` constructs multipliers over Q; `verify_target_certificate` recomputes the exact linear combination against the input equations.
- Route-forcing tests: `direct_route_forcing_selects_only_direct_candidates`; `direct_route_forcing_solves_without_other_routes_or_complete_fallback`; `direct_route_forcing_rejects_spurious_candidate_without_fallback`.
- Tamper tests: `direct_route_tampered_certificate_is_rejected`; ideal membership support tamper and non-target support rejection in `tests/verifier_tests.rs`.
- Non-simplification notes: no geometry names, no finite-field/numeric adoption, no fallback call in route module.

## ResidualCyclic

- Production call chain: `solve_target` -> `collect_candidate_routes` -> `ResidualCyclicOracle::generate` -> `residual_cyclic_candidates` -> `normalize_candidates` modular reconstruction -> `try_candidate_certificate` -> `prove_fixed_target`.
- Controlling data-flow: builds row-closed membership and target-power matrices, reduces target powers through `DenseEchelonResidualOracleFp`, extracts nullspace relations modulo configured primes, and only produces candidates/traces.
- Exact replay oracle: modular support is not accepted directly; after reconstruction, fixed proof over Q and verifier identity replay are required.
- Route-forcing tests: `residual_route_forcing_selects_only_residual_candidates`; `residual_route_forcing_solves_without_other_routes_or_complete_fallback`; `residual_route_forcing_rejects_spurious_candidate_without_fallback`.
- Tamper tests: `residual_route_tampered_certificate_is_rejected`; target certificate identity tamper and empty-certificate multiplier tamper.
- Non-simplification notes: modular traces are computational witnesses, not certificates; no route-local solver success.

## NormTraceTower

- Production call chain: `solve_target` -> `collect_candidate_routes` -> `NormTraceTowerOracle::generate` -> `norm_trace_tower_candidates` -> `try_candidate_certificate` -> `prove_fixed_target`.
- Controlling data-flow: detects triangular towers structurally, requires a guard certificate for nonmonic leading coefficients, permits non-unit target equation coefficients, reduces multiplication by the target expression on the tower basis, computes a characteristic polynomial over Q, and returns a candidate.
- Exact replay oracle: the characteristic output is still only a candidate; fixed proof and verifier replay are required for cover success.
- Route-forcing tests: `tower_route_forcing_selects_only_tower_candidates`; `tower_route_forcing_solves_without_other_routes_or_complete_fallback`; `tower_route_forcing_rejects_spurious_candidate_without_fallback`; `tower_route_requires_guard_for_nonmonic_leading_coefficient`; `tower_route_allows_non_unit_target_coefficient`.
- Tamper tests: `tower_route_tampered_certificate_is_rejected`; target certificate support tamper and same-ideal composite tamper.
- Non-simplification notes: selection depends on guarded tower structure and variable incidence, not problem names or geometry terms.

## TargetCyclicKrylov

- Production call chain: `solve_target` -> `collect_candidate_routes` -> `TargetCyclicKrylovOracle::generate` -> `target_cyclic_krylov_candidates` -> `try_candidate_certificate` -> `prove_fixed_target`.
- Controlling data-flow: builds membership and target-power matrices over Q, constructs a quotient residual handle from the membership matrix left nullspace, computes residual classes of target powers, solves a recurrence with exact rational nullspace linear algebra, and emits a reconstructed target support candidate.
- Exact replay oracle: recurrence is not final authority; fixed proof and verifier replay are required.
- Route-forcing tests: `krylov_route_forcing_selects_only_krylov_candidates`; `krylov_route_forcing_solves_without_other_routes_or_complete_fallback`; `krylov_route_forcing_rejects_spurious_candidate_without_fallback`.
- Tamper tests: `krylov_route_tampered_certificate_is_rejected`; target certificate support tamper and non-target support variable rejection.
- Non-simplification notes: no numeric Krylov adoption and no fallback call in the route module.

## HiddenVariableSparseResultant

- Production call chain: `solve_target` -> `collect_candidate_routes` -> `HiddenVariableSparseResultantOracle::generate` -> `hidden_variable_sparse_resultant_candidates` -> modular reconstruction -> `try_candidate_certificate` -> `prove_fixed_target`.
- Controlling data-flow: builds Newton-support multiplier arrays, resultant rows, exact polynomial columns, target-power columns, modular null relations, and target-only modular supports.
- Exact replay oracle: modular resultant relation remains a candidate; cover success requires Q reconstruction, fixed proof, and verifier replay.
- Route-forcing tests: `resultant_route_forcing_selects_only_resultant_candidates`; `resultant_route_forcing_solves_without_other_routes_or_complete_fallback`; `resultant_route_forcing_rejects_spurious_candidate_without_fallback`.
- Tamper tests: `resultant_route_tampered_certificate_is_rejected`; target identity tamper and composite support tamper.
- Non-simplification notes: the route uses all supplied equations in the expansion path and does not delegate to complete fallback.

## SliceSpecialization

- Production call chain: `solve_target` -> `collect_candidate_routes` -> `SliceSpecializationOracle::generate` -> `slice_specialization_candidates` -> modular reconstruction if possible -> `try_candidate_certificate`.
- Controlling data-flow: assigns deterministic finite-field values to non-target variables, builds a full sliced system with all original equations plus slice equations, runs ResidualCyclic or HiddenVariableSparseResultant inside the sliced system, and records slice equations plus internal route in witness traces.
- Exact replay oracle: slice output is a candidate only; without fixed proof, solver does not return a cover.
- Route-forcing tests: `slice_route_forcing_solves_finite_target_family_without_complete_fallback`; `slice_route_forcing_selects_only_slice_candidates`; `slice_route_forcing_rejects_spurious_candidate_without_fallback`; `slice_candidate_route_does_not_adopt_without_fixed_proof`.
- Tamper tests: `slice_route_tampered_certificate_is_rejected`; target identity tamper and exact-image missing-classification tests cover the fail-closed adoption boundary.
- Non-simplification notes: specialization never becomes certificate authority by itself and no longer extracts single-equation substituted target polynomials.

## LocalizedSchur

- Production call chain: `solve_target` -> proof obstruction collection -> `localized_schur_repair` -> `SchurRepairOutput`.
- Controlling data-flow: derives obstruction-local equation scope from incidence, builds boundary variables and local membership equations, extracts target-only local null relations when available, and replays them through fixed exact proof on the original system before returning a certificate.
- Exact replay oracle: support information is not adopted as solver success; certified output goes through `return_verified_cover` and target-certificate verifier replay.
- Route-forcing tests: `obstruction_scope_uses_incidence_subset`; `schur_repair_builds_local_membership_only`; `uncertified_schur_relation_is_support_info_only`; `schur_repair_returns_exact_certificate_for_target_only_local_relation`; `localized_schur_certifies_after_spurious_seed_without_complete_fallback`.
- Tamper tests: target certificate and composite verifier tamper tests cover accepted certificate forms; support-information output has no certificate authority.
- Non-simplification notes: no full-system Schur outside complete fallback and no complete-fallback call labelled as localized Schur.

## CompleteTargetEliminationFallback

- Production call chain: bounded `solve_target` route exhaustion -> explicit fallback gate -> `complete_target_elimination_fallback` -> certified support, empty certificate, no-target-eliminant certificate, or resource failure.
- Controlling data-flow: enumerates exact multiplier windows, solves exact rational linear systems for empty certificates or target eliminants, and sends target support through `prove_fixed_target`. The no-target-eliminant branch is not replay-complete until P15.
- Exact replay oracle: fallback support uses fixed proof; empty certificates are independently replayed by `verify_certificate`. No-target-eliminant verifier behavior is a P15 `CertificateDesignGap`, and top-level `solve_target` returns `SolverStatus::CertificateDesignGap` with no success certificate on that path.
- Route-forcing tests: `complete_fallback_disabled_route_control_fails_on_reach`; fallback-specific tests `fallback_certifies_simple_target_eliminant`, `fallback_certifies_empty_admissible_set`, `no_target_eliminant_is_algebraic_certificate_only`, and `solver_no_target_eliminant_is_design_gap_until_p15_replay`.
- Tamper tests: empty-certificate multiplier tamper; no-target replay is tracked as a design-gap shell rather than accepted replay.
- Non-simplification notes: fallback is explicit in top-level solver and disabled during route-only closure tests.
