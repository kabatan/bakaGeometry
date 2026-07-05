# Full Core Acceptance Results

Status: FCR-P12 closure passed for candidate-cover; P13 exact-image review passed for MECH-012;
P14 full-pipeline integration review passed for Appendix 29-30 stage-trace evidence only.

Timestamp: 20260706-060906+09:00

## Candidate-Cover Acceptance

| Suite | Evidence | Result |
| --- | --- | --- |
| FCR-P10 support-producing acceptance | `cargo test --manifest-path geosolver-core/Cargo.toml --test fcr_p10_acceptance_suite -- --nocapture` via all-target run | PASS, 12/12 |
| FCR-P11 red-team fresh inputs | `cargo test --manifest-path geosolver-core/Cargo.toml --test fcr_p11_red_team_suite -- --nocapture` via all-target run | PASS, 10/10 |
| root isolation/decode integration | `p12_roots_decode_integration` via all-target run | PASS, 1/1 |
| public pipeline integration | `p3_public_pipeline_integration` via all-target run | PASS, 2/2 |
| P14 full pipeline integration | `p14_full_pipeline_integration` via all-target run | PASS, 10/10 |

Support-producing cases return `CertifiedCandidateCover`, retain nonzero support, produce exact
squarefree support, isolate target roots, decode target candidates, carry projection messages and a
`CoreRunCertificate`, and accept replay.

## Failure And Nonfinite Semantics

| Case | Evidence | Result |
| --- | --- | --- |
| bounded resource/hard case | FCR-P10 B1 and FCR-P11 case 08 | hard/resource/certificate status, never nonfinite |
| positive nonfinite | FCR-P11 case 07 and `fcr_final_nonfinite_semantics` | `CertifiedNonFiniteTargetImage`, but not final nonfinite readiness |
| no-positive-proof boundary | FCR-P11 case 08 | no `CertifiedNonFiniteTargetImage`, cost trace retained |

Public nonfinite results still do not carry a replay-bound nonfinite certificate. Therefore
`CANDIDATE_COVER_CORE_READY` excludes final nonfinite readiness.

## P13 Exact-Image Semantics

| Case | Evidence | Result |
| --- | --- | --- |
| candidate-cover mode does not claim exact image | `p13_candidate_cover_mode_does_not_claim_exact_image_for_semantic_problem` | `CertifiedCandidateCover`, no exact-image certificate |
| nonempty exact image after slack semantics | `p13_exact_image_filters_spurious_slack_root_with_certificates` | `CertifiedExactTargetImage`, spurious root rejected |
| empty real target image | `p13_exact_image_distinguishes_empty_real_target_image` | `CertifiedEmptyRealTargetImage` |
| branch/slack semantics affect feasibility | `p13_branch_choice_semantics_affect_exact_classification` | one exact candidate retained |
| exact-image nonfinite real certificate gate | `p13_exact_image_nonfinite_requires_real_nonfinite_certificate` | `CertifiedNonFiniteTargetImage` with real-certificate proof-kind diagnostic |
| exact-image nonfinite with unproved semantics | `p13_exact_image_nonfinite_with_semantics_returns_gap_without_real_semantic_proof` | `CertificateDesignGap`, not nonfinite success |
| exact-image nonfinite with unproved guard/saturation | `p13_exact_image_nonfinite_with_guard_or_saturation_returns_gap_without_real_proof` | `CertificateDesignGap`, not nonfinite success |

This P13 section does not by itself claim final full acceptance, source fidelity, benchmark
readiness, or public replay-bound nonfinite readiness.

## P14 Full Pipeline Integration

| Case | Evidence | Result |
| --- | --- | --- |
| Appendix 29 stage trace | `p14_stage_trace_executes_appendix_29_pipeline_in_order` | manual stage run matches public `solve_target` output |
| empty-relation nonfinite stage trace | `p14_empty_relation_nonfinite_still_runs_appendix_29_stages` | plan/execute/verify/compose run before nonfinite support finalization |
| Groebner resource trace | `p14_groebner_resource_error_carries_coefficient_height` | finite-resource pair-limit error carries observed coefficient height |
| sparse resultant resource trace | `p14_sparse_resultant_resource_error_carries_coefficient_height` | template resource error carries observed coefficient height |
| candidate-cover public success | `p14_public_candidate_cover_success_has_all_result_fields_and_trace` | `CertifiedCandidateCover`, full finite result fields, replay accepted, cost trace populated |
| exact-image public success | `p14_public_exact_image_success_is_not_candidate_cover` | `CertifiedExactTargetImage`, exact-image certificate present |
| exact-image public empty | `p14_public_exact_image_empty_keeps_support_but_no_candidates` | `CertifiedEmptyRealTargetImage`, finite support retained, no roots/candidates |
| public certified nonfinite | `p14_public_certified_nonfinite_is_finalized_without_panic` | `CertifiedNonFiniteTargetImage` finalized without panic |
| bounded hard/resource | `p14_public_bounded_hard_case_has_status_and_resource_trace` | allowed failure status, global and block resource trace retained without synthetic verification count |
| invalid input | `p14_public_invalid_input_maps_to_result_not_panic` | `InvalidInput`, empty result payload, no panic |

Temporary pipeline scaffold identifiers are absent from `geosolver-core/src` and
`geosolver-core/tests` after P14. P14 remediation also binds production kernel
coefficient-height cost trace fields to input/output polynomial coefficients, and resource failure
traces carry observed coefficient-height evidence. Post-compression failure finalization preserves
known global cost trace context instead of falling back to default error-only traces. Finite success
traces record final support degree `delta` and certificate size `kappa`; nonfinite/failure traces
leave those fields absent.
This P14 section does not close P15 stress acceptance, P16 final closure, source fidelity,
benchmark readiness, final public replay-bound nonfinite readiness, or any R-ID as `VERIFIED`.

## Claim Boundary

Allowed after FCR-P12 reviewer pass:

```text
CANDIDATE_COVER_CORE_READY
```

Still forbidden until later phases:

```text
EXACT_IMAGE_CORE_READY
SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC
RGDTPK_Q_V4_ACCEPTANCE_COMPLETE
```
