# P16 Review Result

Reviewer: spec_verifier

Decision: PASS

Scope: P16 only, BS-R003 / BS-R040 / BS-R122 / MECH-06.

Accepted remediation:

- Finite exact-image requests return `CertificateDesignGap` with `ExactImageOutOfScope` and preserve unfiltered roots/candidates.
- Exact-image nonfinite outcomes map to `CertificateDesignGap`, not `CertifiedNonFiniteTargetImage`, and bind `nonfinite_certificate_hash`.
- Regression expects `CertificateDesignGap`, `ExactImageOutOfScope`, and no replay acceptance for exact-image nonfinite guard result.
- `exact_image_mode` remains certificate-bound via solver options hash.
- Replay rejects exact-image certificate hashes and exact-image success statuses.
- P16 audit checks the nonfinite diagnostic and hash marker.

Fresh checks observed by reviewer:

- fmt
- P16 audit findings 0
- `p13_exact_image_semantics`
- `p14_full_pipeline_integration`
- `--lib replay`
- `--lib run_certificate`
- `--no-run`

Blockers: none for P16 scope.
