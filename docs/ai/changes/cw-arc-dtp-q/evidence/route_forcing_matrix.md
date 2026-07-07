# Route-Forcing Matrix

Status: P14 evidence; RP-P14 boundary review passed.

Authority: evidence only. The code and tests named here remain the executable source of truth.

## Controls

- Test-only route control is compiled only through `#[cfg(test)] mod test_support` in `src/lib.rs`.
- Public `SolverOptions` has no route forcing or complete-fallback switch.
- `solve_target_for_test` passes a specific `enabled_origins` set and `allow_complete_fallback` to the private solver entry point.
- `assert_route_only_cover` checks `CertifiedCandidateCover`, candidate/proof trace for the selected origin, and absence of `target_elimination:` trace events.

## Matrix

| Route | Test-only route control | Fallback disabled | Closure checked | Test evidence |
| --- | --- | --- | --- | --- |
| DirectTargetEquation | yes, only `DirectTargetEquation` | yes | candidate cover and proof trace from direct origin only | `direct_route_forcing_solves_without_other_routes_or_complete_fallback`; candidate isolation in `direct_route_forcing_selects_only_direct_candidates` |
| ResidualCyclic | yes, only `ResidualCyclic` | yes | candidate cover and proof trace from residual origin only | `residual_route_forcing_solves_without_other_routes_or_complete_fallback`; candidate isolation in `residual_route_forcing_selects_only_residual_candidates` |
| NormTraceTower | yes, only `NormTraceTower` | yes | candidate cover and proof trace from tower origin only | `tower_route_forcing_solves_without_other_routes_or_complete_fallback`; candidate isolation in `tower_route_forcing_selects_only_tower_candidates` |
| TargetCyclicKrylov | yes, only `TargetCyclicKrylov` | yes | candidate cover and proof trace from Krylov origin only | `krylov_route_forcing_solves_without_other_routes_or_complete_fallback`; candidate isolation in `krylov_route_forcing_selects_only_krylov_candidates` |
| HiddenVariableSparseResultant | yes, only `HiddenVariableSparseResultant` | yes | candidate cover and proof trace from resultant origin only | `resultant_route_forcing_solves_without_other_routes_or_complete_fallback`; candidate isolation in `resultant_route_forcing_selects_only_resultant_candidates` |
| SliceSpecialization | yes, only `SliceSpecialization` | yes for the finite-target success family | candidate cover and proof trace from slice origin only for a positive-dimensional finite-target family; separate test keeps unproved slice candidates from being accepted | `slice_route_forcing_solves_finite_target_family_without_complete_fallback`; `slice_route_forcing_selects_only_slice_candidates`; `slice_candidate_route_does_not_adopt_without_fixed_proof` |
| LocalizedSchur | not a candidate-origin success route in current implementation | not applicable to candidate route forcing | uncertified Schur returns support information only | `obstruction_scope_uses_incidence_subset`; `schur_repair_builds_local_membership_only`; `uncertified_schur_relation_is_support_info_only` |
| CompleteTargetEliminationFallback | explicitly disabled in route-control guard; enabled only for fallback-target tests | disabled test panics if reached | fallback certificates must verify independently | `complete_fallback_disabled_route_control_fails_on_reach`; `fallback_certifies_simple_target_eliminant`; `fallback_certifies_empty_admissible_set`; `no_target_eliminant_is_algebraic_certificate_only` |

## Production Call Chain Anchors

- Solver route loop: `src/solver.rs:529` collects enabled candidate routes.
- Test route control entry: `src/solver.rs:270` and `src/test_support.rs:113`.
- Candidate adoption gate: `src/solver.rs:283` and `src/solver.rs:327`.
- Verified-cover return path: `src/solver.rs:372` and `src/solver.rs:469`.
- Complete fallback boundary: `src/fallback_elimination.rs:30`.

## Command Evidence

Latest targeted route-control command:

```text
cargo test --lib test_support
result after SliceSpecialization success test: pass, 13 tests passed
```
