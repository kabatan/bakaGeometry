# Tamper Matrix

Status: P14 evidence; RP-P14 boundary review passed.

Authority: evidence only. Verifier code and tests are the executable source of truth.

## Replay Oracles

- Target-cover replay recomputes exact polynomial identities in `src/verifier.rs:35` and `src/verifier.rs:344`.
- Empty-admissible-set replay recomputes `1 = sum q_i F_i` in `src/verifier.rs:135`.
- No-target-eliminant replay is restricted to the implemented algebraic monomial-ideal case in `src/verifier.rs:170`.
- Composite cover replay recomputes gcd/lcm support from verified child certificates in `src/verifier.rs:221`.
- Guard replay recomputes input guard identity, Nullstellensatz nonvanishing, and guard products in `src/verifier.rs:276`.
- Exact target image construction is guarded by all-root classification checks in `src/exact_image.rs:61`.

## Matrix

| Certificate family | Tamper or rejection test | Failure mode exercised |
| --- | --- | --- |
| Ideal membership target cover | `ideal_membership_target_certificate_recomputes_identity` | altered support coefficient fails exact identity replay |
| Radical membership | `radical_membership_requires_positive_power_and_exact_identity`; `radical_proof_rejects_zero_support_power` | zero support power rejected |
| Guarded radical | `guarded_radical_refuses_missing_guard_certificate`; `guarded_radical_refuses_unverified_guards` | missing or unverifiable guard certificate rejected |
| Same-ideal composite cover | `same_ideal_composite_uses_gcd_not_product` | product support rejected when gcd support is required |
| Component-union composite cover | `component_union_composite_requires_lcm_and_source_marker` | missing component-union source rejected |
| Target support validity | `target_certificate_rejects_zero_support`; `target_certificate_rejects_non_target_support_variable` | zero support and non-target variable support rejected |
| Input semantic guard | `input_semantic_nonzero_requires_identical_record` | altered guard record rejected |
| Derived guard product | `derived_product_recomputes_factor_product` | altered guard product rejected |
| Algebraic guard nonvanishing | `algebraic_nonvanishing_recomputes_nullstellensatz_identity` | altered Nullstellensatz multiplier rejected |
| Empty admissible set | `empty_algebraic_certificate_verifies_and_tampering_fails` | altered multiplier rejected |
| Exact target image | `exact_image_requires_every_root_classified`; solver status tests for Try/Require exact image | missing root classification cannot produce `CertifiedExactTargetImage` |

## Command Evidence

Full command evidence is recorded in the P14 audit evidence after the final P14 test run.
