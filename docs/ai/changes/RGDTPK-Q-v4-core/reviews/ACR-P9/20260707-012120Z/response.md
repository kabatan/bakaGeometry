# ACR-P9 Review Response

## Guardian Boundary Reviewer

Result: PASS

ACR-P9 is MECH-closable for the generic large-footprint support-producing stress suite after the S6 executed-failure repair. No ACR-P10/final-readiness, exact-image, source-fidelity, or full-acceptance claim is supported by this review.

Key checks passed:

- Eight S1-S8 tests exist and run through `solve_target`, not only helper APIs.
- Anti-overfit variants cover baseline scaling, non-base variable-id renaming, relation-order permutation, and nonzero rational scaling.
- Support-producing checks assert `CertifiedCandidateCover`, nonzero/nonconstant support, projection messages, cost trace, route success trace, `verify_global_support`, `replay_run_certificate`, and message verification.
- S3 requires `ResultantBackendKind::QuadraticSubresultant` and exact resultant certificate verification exists.
- S5 includes SpecializationInterpolation success plus bounded SparseResultant probe failure/prohibition.
- S6 now uses `executed_failed_strategy_hashes`, not `failed_strategy_hashes` or skipped cost-prohibited hashes.

Per-family record:

```yaml
S1: {dense_trs_status: materialization_allowed_false, sparse_resultant_status: not_required, successful_route: TargetActionKrylov, projection_message_verified: true, support_verified: true, replay_accepted: true}
S2: {dense_trs_status: materialization_allowed_false, sparse_resultant_status: not_required, successful_route: TargetRelationSearch, projection_message_verified: true, support_verified: true, replay_accepted: true}
S3: {dense_trs_status: not_required, sparse_resultant_status: Admitted, successful_route: SparseResultantProjection, projection_message_verified: true, support_verified: true, replay_accepted: true}
S4: {dense_trs_status: materialization_allowed_false, sparse_resultant_status: CostProhibited, successful_route: TargetRelationSearch, projection_message_verified: true, support_verified: true, replay_accepted: true}
S5: {dense_trs_status: materialization_allowed_false, sparse_resultant_status: PlanProbeFailed_or_CostProhibited, successful_route: SpecializationInterpolation, projection_message_verified: true, support_verified: true, replay_accepted: true}
S6: {dense_trs_status: materialization_allowed_false, sparse_resultant_status: internal_bounded_or_not_selected, successful_route: UniversalTargetElimination, projection_message_verified: true, support_verified: true, replay_accepted: true}
S7: {dense_trs_status: not_required, sparse_resultant_status: not_required, successful_route: TargetRelationSearch, projection_message_verified: true, support_verified: true, replay_accepted: true}
S8: {dense_trs_status: not_required, sparse_resultant_status: internal_bounded_or_not_selected, successful_route: UniversalTargetElimination, projection_message_verified: true, support_verified: true, replay_accepted: true}
```

Adversarial counterexample recorded: two quadratic polynomials in eliminated variable `x`, with a 4x4 Sylvester template but coefficients containing hundreds/thousands of terms over many keep variables, plus a compact later target route. Expected behavior: SparseResultant must be `CostProhibited` before execution and must not monopolize; later compact route must produce verified support.

## Spec Verifier

Result: PASS

```yaml
phase_id: ACR-P9
status: PASS
scope: ACR-P9 only; no ACR-P10/final readiness review
blocking_findings: []
acr_p9_required_booleans:
  eight_stress_families_present: true
  support_producing_checks_asserted: true
  anti_overfit_variants_present: true
  no_diagnostic_fixture_or_expected_answer_dependency: true
  public_or_near_public_pipeline_used: true
  s3_quadratic_subresultant_exact_replay: true
  s5_specialization_after_dense_and_sparse_probe: true
  s6_executed_internal_failures_only: true
  evidence_files_match_current_code: true
```

Adversarial counterexample recorded: Universal ladder where dense TRS and sparse resultant are `CostProhibited` before a bounded local elimination success. Expected behavior: skipped or `CostProhibited` stages may appear in `failed_strategy_hashes` but must not contribute to `executed_failed_strategy_hashes`.

Forbidden claims remain: `CANDIDATE_COVER_CORE_READY`, `SOURCE_FAITHFUL_TO_V4_CANDIDATE_COVER_LAYER`, `SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC`, `RGDTPK_Q_V4_ACCEPTANCE_COMPLETE`, exact-image acceptance, and full acceptance.

