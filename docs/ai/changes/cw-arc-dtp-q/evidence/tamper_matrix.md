# Tamper Matrix

Status: current tamper evidence for P7-P13 plus P1-P13 spec-gap fixes; ComponentUnionLcm and no-target entries remain design-gap behavior after P3 re-review blocker fix.

Authority: evidence only. Verifier code and tests are the executable source of truth.

## Replay Oracles

- Target-cover replay recomputes exact polynomial identities in `src/verifier.rs:35` and `src/verifier.rs:344`.
- Empty-admissible-set replay recomputes `1 = sum q_i F_i` in `src/verifier.rs:135`.
- No-target-eliminant replay is a P15 design-gap shell in `src/verifier.rs`; top-level solver does not return a no-target success status before P15 replay.
- Composite cover replay recomputes same-ideal gcd support from verified child certificates. Component-union lcm checks support but returns `CertificateDesignGap` without replay-verifiable source data.
- Guard replay recomputes input guard identity, Nullstellensatz nonvanishing, and guard products in `src/verifier.rs:276`.
- Exact target image construction is guarded by all-root classification checks in `src/exact_image.rs:61`.

## Matrix

| Certificate family | Tamper or rejection test | Failure mode exercised |
| --- | --- | --- |
| Ideal membership target cover | `ideal_membership_target_certificate_recomputes_identity` | altered support coefficient fails exact identity replay |
| Radical membership | `radical_membership_requires_positive_power_and_exact_identity`; `radical_proof_rejects_zero_support_power` | zero support power rejected |
| Guarded radical | `guarded_radical_refuses_missing_guard_certificate`; `guarded_radical_refuses_unverified_guards` | missing or unverifiable guard certificate rejected |
| Same-ideal composite cover | `same_ideal_composite_uses_gcd_not_product` | product support rejected when gcd support is required |
| Component-union composite cover | `component_union_lcm_without_replay_source_is_design_gap` | description-only component-union source returns design gap; missing source rejected |
| Target support validity | `target_certificate_rejects_zero_support`; `target_certificate_rejects_non_target_support_variable` | zero support and non-target variable support rejected |
| Input semantic guard | `input_semantic_nonzero_requires_identical_record` | altered guard record rejected |
| Derived guard product | `derived_product_recomputes_factor_product` | altered guard product rejected |
| Algebraic guard nonvanishing | `algebraic_nonvanishing_recomputes_nullstellensatz_identity` | altered Nullstellensatz multiplier rejected |
| Empty admissible set | `empty_algebraic_certificate_verifies_and_tampering_fails` | altered multiplier rejected |
| No-target eliminant | `solver_no_target_eliminant_is_design_gap_until_p15_replay`; `no_target_eliminant_is_p15_design_gap_not_monomial_acceptance` | top-level solver returns design gap with no success certificate; verifier design-gap shell remains until P15 |
| Exact target image | `exact_image_requires_every_root_classified`; solver status tests for Try/Require exact image | missing root classification cannot produce `CertifiedExactTargetImage` |
| Direct route cover | `direct_route_tampered_certificate_is_rejected` | route-forced certificate support tamper rejected by verifier |
| Residual route cover | `residual_route_tampered_certificate_is_rejected` | route-forced certificate support tamper rejected by verifier |
| Krylov route cover | `krylov_route_tampered_certificate_is_rejected` | route-forced certificate support tamper rejected by verifier |
| Resultant route cover | `resultant_route_tampered_certificate_is_rejected` | route-forced certificate support tamper rejected by verifier |
| Slice route cover | `slice_route_tampered_certificate_is_rejected` | route-forced certificate support tamper rejected by verifier |
| Tower route cover | `tower_route_tampered_certificate_is_rejected`; `tower_route_forcing_solves_guarded_nonmonic_tower_without_fallback` | route-forced certificate support tamper rejected by verifier; guarded nonmonic tower still adopts only after exact proof |

## Command Evidence

Latest command evidence is recorded in `p1_p13_spec_gap_fix_evidence.md`.

This matrix is evidence for the current P1-P13 scope only. It does not claim P14/P15/P16 or final V3 completion.
