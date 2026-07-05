# Full Core Acceptance Results

Status: FCR-P12 closure passed for candidate-cover; P13 exact-image evidence prepared and awaiting
P13 reviewer pass.

Timestamp: 20260706-060906+09:00

## Candidate-Cover Acceptance

| Suite | Evidence | Result |
| --- | --- | --- |
| FCR-P10 support-producing acceptance | `cargo test --manifest-path geosolver-core/Cargo.toml --test fcr_p10_acceptance_suite -- --nocapture` via all-target run | PASS, 12/12 |
| FCR-P11 red-team fresh inputs | `cargo test --manifest-path geosolver-core/Cargo.toml --test fcr_p11_red_team_suite -- --nocapture` via all-target run | PASS, 10/10 |
| root isolation/decode integration | `p12_roots_decode_integration` via all-target run | PASS, 1/1 |
| public pipeline integration | `p3_public_pipeline_integration` via all-target run | PASS, 2/2 |

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
