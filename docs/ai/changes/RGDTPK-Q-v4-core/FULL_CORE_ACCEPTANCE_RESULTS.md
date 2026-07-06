# Full Core Acceptance Results

Status: P16 final acceptance evidence after reviewer pass.

## Suite Results

| Suite | Evidence | Result |
| --- | --- | --- |
| Candidate-cover support-producing | FCR-P10 12/12, FCR-P11 10/10, P14 10/10, P15 Suite A | PASS |
| Exact-image semantics | P13 7/7, P15 Suite B | PASS |
| Failure and nonfinite semantics | FCR-P11, P13, P14, P15 Suite C | PASS |

Support-producing cases return finite candidate-cover or exact-image success with nonzero support,
exact squarefree support, exact root isolation, decoded candidates when real roots exist, projection
messages, core certificate, replay acceptance, and cost fields.

Exact-image cases execute real fiber/guard/slack/branch semantics before exact-image statuses.
P16 binds `exact_image_certificate_hash` into `CoreRunCertificate` and replay rejects classification
hash tamper.

Failure/nonfinite cases stay separate from support-producing acceptance. Bounded search, resource
failure, certificate gaps, and relation-search exhaustion do not become `CertifiedNonFiniteTargetImage`.
Certified nonfinite status remains gated by positive proof evidence, carries a structured
`NonFiniteCertificate`, binds nonfinite replay hash to `hash_nonfinite_certificate`, accepts replay
for the positive baseline, rejects nonfinite certificate hash tamper, and rejects ordinary replay
when a nonfinite certificate is injected into an incompatible finite result.

## Verification

Fresh P16 verification:

```text
cargo fmt --manifest-path geosolver-core/Cargo.toml -- --check: PASS
cargo clippy --manifest-path geosolver-core/Cargo.toml --all-targets --all-features -- -D warnings: PASS
cargo test --manifest-path geosolver-core/Cargo.toml --all-targets --all-features: PASS
git diff --check: PASS
```

All-target test summary:

```text
lib tests: 212 passed
fcr_final_nonfinite_semantics: 2 passed
fcr_p10_acceptance_suite: 12 passed
fcr_p11_red_team_suite: 10 passed
fcr_p4_pure_planning: 7 passed
p12_roots_decode_integration: 1 passed
p12g_generality_stress: 1 passed
p13_exact_image_semantics: 7 passed
p14_full_pipeline_integration: 10 passed
p15_acceptance_stress: 6 passed
p3_public_pipeline_integration: 2 passed
```

## Claim Boundary

Allowed after P16 reviewer pass:

```text
RGDTPK_Q_V4_ACCEPTANCE_COMPLETE
```

Not claimed:

```text
SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC
benchmark superiority
universal finite-system completeness
geometry DSL support
natural-language or diagram support
any R-ID VERIFIED status
```
