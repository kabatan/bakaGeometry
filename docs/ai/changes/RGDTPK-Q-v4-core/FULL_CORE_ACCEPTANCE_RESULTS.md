# Full Core Acceptance Results

Status: FCR-P12 closure evidence prepared; reviewer approval required for final claim.

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
