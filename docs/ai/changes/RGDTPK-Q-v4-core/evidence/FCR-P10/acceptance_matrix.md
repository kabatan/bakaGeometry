# FCR-P10 Acceptance Matrix

Status: implemented and locally verified after spec and boundary-review remediation.

Pre-P11 correction: certified nonfinite target image is no longer counted as a P10 acceptance
category. It has been moved to the final nonfinite semantics gate because the public
`TargetSolveResult` does not yet carry a machine-readable nonfinite certificate through
`CoreRunCertificate` replay.

| Category | Test | Public/near-public path | Required result | Named kernel evidence |
| --- | --- | --- | --- | --- |
| A1 | fcr_p10_a1_public_no_initial_target_only_one_block | api::solve_target | CertifiedCandidateCover | support-producing public pipeline |
| A2 | fcr_p10_a2_public_multivariate_quotient_nonlinear_target | api::solve_target | CertifiedCandidateCover | TargetActionKrylov |
| A3 | fcr_p10_a3_public_multiple_eliminated_variables_and_separators | api::solve_target | CertifiedCandidateCover | multi-eliminated-variable projection message |
| A4 | fcr_p10_a4_public_sparse_resultant_eliminant_without_target_only_input | api::solve_target | CertifiedCandidateCover | SparseResultantProjection |
| A5 | fcr_p10_a5_public_specialization_interpolation_style_multiseparator | api::solve_target | CertifiedCandidateCover | SpecializationInterpolation |
| A6 | fcr_p10_a6_public_guarded_rational_affine_preprocessing_to_support | api::solve_target | CertifiedCandidateCover | guarded compression through final support |
| A7 | fcr_p10_a7_public_target_independent_component_with_feasibility_obligation | api::solve_target | CertifiedCandidateCover | feasibility-obligation path |
| A8 | fcr_p10_a8_public_one_large_block_no_useful_separator | api::solve_target | CertifiedCandidateCover | UniversalTargetElimination admitted in route trace |
| A9 | fcr_p10_a9_public_regular_chain_style_projection | api::solve_target | CertifiedCandidateCover | RegularChainProjection |
| A10 | fcr_p10_a10_public_norm_trace_two_step_tower | api::solve_target | CertifiedCandidateCover | NormTraceProjection |
| A11 | fcr_p10_a11_public_nonreal_support_empty_candidate_cover | api::solve_target | CertifiedCandidateCover with empty roots/candidates | support retained despite no real roots |
| B1 | fcr_p10_b1_public_resource_bounded_hard_case_has_spec_status | api::solve_target | AlgorithmicHardCase, FiniteResourceFailure, or CertificateDesignGap | bounded failure status with matrix cost trace |

Shared assertions for support-producing cases:
- every problem relation is multiplied by a deterministic nonzero rational factor in the shared `problem()` helper before the public solve call, and the test asserts the scaled relation differs from the unscaled relation;
- nonconstant support polynomial
- squarefree support polynomial present
- nonempty projection messages
- run certificate present
- root/candidate counts match
- candidate hashes are exact and nonzero
- replay_run_certificate accepts
- cost trace block count matches projection messages
- verification trace checks at least one relation

Additional remediated assertions:
- required named-kernel cases assert the executed ProjectionMessage.kernel_kind directly, except A8 which asserts UniversalTargetElimination remains admitted because the current declared ladder keeps Universal last and may succeed earlier through a compact exact route;
- public replay accepts every support-producing result;
- relationless structural DAG blocks do not produce projection messages, while their child messages remain hash-bound in final DAG replay evidence.
- resource-bounded B1 failure retains a nonempty cost trace with matrix rows or columns, the requested public target, and TargetRelationSearch kernel identity.
- A3 and A5 assert projection-message composition is essential by recomposing the public DAG result and verifying that removing each projection message either fails composition or changes target support.
- A3 also exercises near-public production message-only separator composition over two separators and asserts removing either separator message fails or changes target support.

Moved out of P10:
- certified nonfinite positive-proof behavior now lives in `fcr_final_nonfinite_semantics.rs`.
- final nonfinite readiness remains blocked until the final nonfinite gate either adds a public
  replay-bound nonfinite certificate or explicitly excludes nonfinite readiness from
  `CANDIDATE_COVER_CORE_READY`.
