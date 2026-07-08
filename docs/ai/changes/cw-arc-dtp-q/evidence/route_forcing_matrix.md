# Route-Forcing Matrix

Status: current route-forcing evidence for P7-P13 plus P1-P13 spec-gap fixes.

Authority: evidence only. The code and tests named here remain the executable source of truth.

## Controls

- Test-only route control is compiled only through `#[cfg(test)] mod test_support` in `src/lib.rs`.
- Public `SolverOptions` has no route forcing or complete-fallback switch.
- `RouteForcing` drives `solve_target_with_route_forcing` with explicit `enabled_origins`, `allow_complete_fallback`, `allow_other_heavy_routes`, and test-only spurious-candidate injection.
- `assert_route_only_cover` checks `CertifiedCandidateCover`, candidate/proof trace for the selected origin, and absence of `target_elimination:` trace events.
- `assert_route_spurious_rejected` checks a route-specific spurious candidate is rejected with `NoVerifiedTargetCertificate`, no certificate, no cover, proof trace for the selected origin, and no complete-fallback trace.

## Matrix

| Route | Test-only route control | Fallback disabled | Closure checked | Test evidence |
| --- | --- | --- | --- | --- |
| DirectTargetEquation | yes, only `DirectTargetEquation` | yes | candidate cover and proof trace from direct origin only; spurious candidate rejected before fallback; certificate tamper rejects | `direct_route_forcing_solves_without_other_routes_or_complete_fallback`; `direct_route_forcing_selects_only_direct_candidates`; `direct_route_forcing_rejects_spurious_candidate_without_fallback`; `direct_route_tampered_certificate_is_rejected` |
| ResidualCyclic | yes, only `ResidualCyclic` | yes | candidate cover and proof trace from residual origin only; active support and prime filters tested; spurious candidate rejected before fallback; certificate tamper rejects | `residual_route_forcing_solves_without_other_routes_or_complete_fallback`; `residual_route_forcing_selects_only_residual_candidates`; `residual_route_forcing_rejects_spurious_candidate_without_fallback`; `residual_route_tampered_certificate_is_rejected` |
| NormTraceTower | yes, only `NormTraceTower` | yes | candidate cover and proof trace from tower origin only; nonconstant guarded-nonmonic and non-unit target coefficient tested; spurious candidate rejected before fallback; certificate tamper rejects | `tower_route_forcing_solves_without_other_routes_or_complete_fallback`; `tower_route_forcing_solves_guarded_nonmonic_tower_without_fallback`; `tower_route_forcing_selects_only_tower_candidates`; `tower_route_forcing_rejects_spurious_candidate_without_fallback`; `tower_route_tampered_certificate_is_rejected` |
| TargetCyclicKrylov | yes, only `TargetCyclicKrylov` | yes | candidate cover and proof trace from Krylov origin only; quotient/residual recurrence tested; spurious candidate rejected before fallback; certificate tamper rejects | `krylov_route_forcing_solves_without_other_routes_or_complete_fallback`; `krylov_route_forcing_selects_only_krylov_candidates`; `krylov_route_forcing_rejects_spurious_candidate_without_fallback`; `krylov_route_tampered_certificate_is_rejected` |
| HiddenVariableSparseResultant | yes, only `HiddenVariableSparseResultant` | yes | sparse support-sum template route produces candidates only; route cover requires exact proof; SR-F1 and non-chain sparse eliminant route-forced | `resultant_route_forcing_solves_without_other_routes_or_complete_fallback`; `resultant_route_forcing_solves_sr_f1_two_polynomial_hidden_resultant_without_fallback`; `resultant_route_forcing_solves_non_chain_sparse_eliminant_without_fallback`; `resultant_route_forcing_selects_only_resultant_candidates`; `resultant_route_forcing_rejects_spurious_candidate_without_fallback`; `resultant_route_tampered_certificate_is_rejected` |
| SliceSpecialization | yes, only `SliceSpecialization` | yes for the finite-target success, affine-coupled success, and spurious-rejection families | generic affine full sliced-system candidate evidence; route cover requires exact proof; affine trace and denominator-admissibility tested | `slice_route_forcing_solves_finite_target_family_without_complete_fallback`; `slice_route_forcing_solves_affine_coupled_family_without_complete_fallback`; `slice_route_forcing_selects_only_slice_candidates`; `slice_route_forcing_rejects_spurious_candidate_without_fallback`; `slice_candidate_route_does_not_adopt_without_fixed_proof`; `slice_route_tampered_certificate_is_rejected` |
| LocalizedSchur | yes, via forced obstruction seed rather than candidate-origin enumeration | yes | support-information path, exact target-certificate path, and solver-level no-fallback certified adoption tested | `schur_repair_builds_local_membership_only`; `uncertified_schur_relation_is_support_info_only`; `schur_repair_returns_exact_certificate_for_target_only_local_relation`; `localized_schur_certifies_after_spurious_seed_without_complete_fallback` |
| CompleteTargetEliminationFallback | explicitly disabled in route-control guard; enabled only for fallback-target tests | disabled test panics if reached | fallback certificates must verify independently | `complete_fallback_disabled_route_control_fails_on_reach`; `fallback_certifies_simple_target_eliminant`; `fallback_certifies_empty_admissible_set`; `no_target_eliminant_is_algebraic_certificate_only` |

## Production Call Chain Anchors

- Solver route loop: `src/solver.rs` `collect_candidate_routes` collects enabled candidate routes.
- Test route control entry: `src/solver.rs` `solve_target_with_route_forcing` and `src/test_support.rs` `RouteForcing`.
- Candidate adoption gate: `src/solver.rs` `try_candidate_certificate`.
- Verified-cover return path: `src/solver.rs` `return_verified_cover`.
- Complete fallback boundary: `src/fallback_elimination.rs:30`.

## Command Evidence

Latest targeted route-control command:

```text
cargo test --lib test_support -- --nocapture
result: pass; 31 route-control, exact-proof-gate, no-fallback, and tamper tests passed
```

This matrix is evidence for the current P1-P13 route and blocker-fix scope only. It does not claim P14/P15/P16 or final V3 completion.
