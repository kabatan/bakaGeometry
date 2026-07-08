# P5/P6 Blocker Fix Base Spec

Purpose: controlling delta for the P5/P6 blocker fix before P7 may start.
Status: active blocker-fix authority after user instruction on 2026-07-08.
Authority: this file narrows and strengthens the active V3 Base Spec for the listed blockers only. `BASE_SPEC.md` remains controlling outside this delta.

Source:
- `C:\Users\bakat\Downloads\CW_ARC_DTP_Q_P5_P6_BLOCKER_FIX_INSTRUCTIONS.md`
- SHA-256: `460EACD1C644E32E62094ECBF13E7D111346F23FF430BA151EDE8A6314557C32`

## Scope

This fix must be completed before P7 route closure starts. Existing P4
behavior for row-closed windows, residual-oracle checking, and active-support
witnesses must be maintained while applying this blocker fix.

Required production replacements:

1. P5 proof scheduling:
   - Implement a lazy deterministic `FairProofSchedule::unbounded()` iterator over all finite tuples `(multiplier_degree, support_power, guard_power)`.
   - Implement an explicitly named bounded prefix API.
   - Do not use hidden defaults such as `max_proof_weight.unwrap_or(6)` or `max_window_degree.unwrap_or(6)` to claim unbounded fairness.
   - If production `solve_target` lacks an explicit proof bound and does not certify before proof search, return fail-closed resource/design-gap behavior with trace `resource:unbounded_proof_requires_bound`; do not silently cap search.

2. P6 factorization and factor schedule:
   - Replace `Vec<UniPolynomialQ>`-only squarefree factorization with status-bearing factorization: `Complete`, `Partial`, or `ResourceFailure`.
   - Implement exact Q factor search that splits reducible squarefree polynomials without rational roots, including `(T^2+1)(T^2+2)`.
   - Verify exact division for every factor and verify product reconstruction for `Complete`.
   - Do not treat partial or resource-limited factorization as complete.
   - `factor_schedule` must surface factorization status/trace and must not silently convert factorization failure to completed `[original]`.
   - Every factor trial must pass through the fixed exact proof pipeline separately.
   - A factor may be adopted only when that factor's own exact certificate verifies.

3. P6 multi-origin ranking:
   - Add production data-flow for origin evidence.
   - Merge candidates with the same primitive reconstructed support across origins.
   - Ranking order is: exact/reconstructed before modular-only, lower degree,
     higher prime count, higher origin count, lower coefficient height, lower
     active support size, and deterministic tie-breakers.
   - Origin count affects only attempt order.
   - Origin count must never certify or adopt a candidate without exact proof.

4. P3 regressions:
   - `ComponentUnionLcm` remains `CertificateDesignGap` without replay-verifiable component source.
   - When replay-verifiable component source exists, verification must verify child
     certificates, recompute the lcm support exactly, reject support mismatch, and
     return `CertificateDesignGap` instead of `Verified` if replay evidence is absent.
   - Solver must not return `CertifiedNoNonzeroTargetEliminant` while verifier support is only `CertificateDesignGap`.

## Sequencing

Production semantics and tests must be implemented before updating evidence or
review files. Evidence/review documents may describe only behavior supported by
production tests or cited source anchors.

## Required Tests

Required new or updated tests:

- `fair_schedule_reaches_every_tuple_up_to_weight_4`
- `fair_schedule_is_lazy_and_not_a_fixed_default_prefix`
- `bounded_prefix_is_explicitly_bounded_by_argument`
- `solve_target_without_proof_bound_does_not_silently_use_default_six`
- `factorization_splits_product_of_irreducible_quadratics_without_rational_roots`
- `factorization_reports_resource_failure_instead_of_false_complete_when_bounds_exceeded`
- `factorization_product_reconstructs_original_squarefree_part`
- `same_reconstructed_support_from_two_origins_is_merged_and_ranked_by_origin_count`
- `different_supports_from_different_origins_are_not_merged`
- `origin_count_does_not_certify_candidate_without_exact_proof`

## Forbidden Production Patterns

The following are forbidden in production code for this fix:

```text
max_proof_weight.unwrap_or(6)
max_window_degree.unwrap_or(6)
effective_window_limits silently replacing None with Some(...)
factor_squarefree_over_q returns Vec only with no status
rational_root loop as the only nontrivial factorization method
ComponentUnionLcm returns VerificationResult::Verified without replay-verifiable source
CertifiedNoNonzeroTargetEliminant returned with certificate=None or unverifiable certificate
rank uses candidate.origin enum order as origin-count substitute
```

## Claim Ceiling

This delta can close only the P5/P6 blocker fix. It does not close P7+, P15, P16, final V3 completion, source-faithfulness, production-safety, readiness, acceptance-complete, or any R-ID verified claim.
