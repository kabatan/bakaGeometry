# P16 Review Prompt - Final Closure And Claim Ladder

Review target: Plan P16.

Changed code:

- `geosolver-core/src/verify/run_certificate.rs`
- `geosolver-core/src/verify/replay.rs`
- `geosolver-core/src/solver/pipeline.rs`
- `geosolver-core/src/solver/orchestrator.rs`
- `geosolver-core/src/result/output.rs`
- `geosolver-core/src/compose/final_support.rs`
- `geosolver-core/tests/fcr_final_nonfinite_semantics.rs`
- `geosolver-core/tests/p13_exact_image_semantics.rs`
- `geosolver-core/tests/p14_full_pipeline_integration.rs`

Changed closure/evidence/archive files:

- `docs/ai/changes/RGDTPK-Q-v4-core/CLOSURE.md`
- `docs/ai/changes/RGDTPK-Q-v4-core/CONSISTENCY_AUDIT.md`
- `docs/ai/changes/RGDTPK-Q-v4-core/FULL_CORE_ACCEPTANCE_RESULTS.md`
- `docs/ai/changes/RGDTPK-Q-v4-core/evidence/P16/*`
- schema-formalized review archives under `docs/ai/changes/RGDTPK-Q-v4-core/reviews/*/*`

Source anchors:

- `BASE_SPEC.md` RGQ-048 through RGQ-064, plus acceptance criteria section 10.
- `PLAN.md` P16.
- `SOURCE_MAP.md`.
- `REVIEWER_PROMPTS.md#P16`.
- Guardian Runtime Contract.

Fresh verification:

- `cargo fmt --manifest-path geosolver-core/Cargo.toml -- --check`: PASS.
- `cargo clippy --manifest-path geosolver-core/Cargo.toml --all-targets --all-features -- -D warnings`: PASS.
- `cargo test --manifest-path geosolver-core/Cargo.toml --all-targets --all-features`: PASS.
- `git diff --check`: PASS with CRLF warnings only.

Static scans:

- Unsupported/unsupported in `geosolver-core/src`: 0.
- geometry/fixture/expected-answer scan: 142 classified matches, no production dispatch.
- todo/unimplemented/placeholder/dummy/fake/stub in `geosolver-core/src`: 0.
- CAD/QE/RCF/full-coordinate/RUR scan: 3 classified rejection-text matches.
- old candidate-cover acceptance-complete phrase: 0.

Archive audit:

- formal archives: 56.
- schema invalid: 0.
- prompt/response/manifest hash mismatches: 0.
- PASS-with-blocker or PASS-with-required-fix: 0.

Replay/tamper updates after initial P16 spec feedback:

- `CoreRunCertificate` includes `exact_image_certificate_hash`; replay recomputes
  `hash_fiber_classification_result` and rejects mismatched exact-image classification hashes.
- `TargetSolveResult` exposes structured `nonfinite_certificate` for
  `CertifiedNonFiniteTargetImage`.
- Nonfinite replay reconstructs canonicalization, compression, DAG, and composed projection, then
  verifies `hash_nonfinite_certificate` and `verify_nonfinite_certificate`.
- `fcr_final_nonfinite_public_certified_nonfinite_requires_positive_proof` accepts baseline
  nonfinite replay and rejects nonfinite certificate hash tamper.

Requested review:

Check all R-IDs and MECHs required by the claimed label, especially RGQ-057 through RGQ-064. Check
fresh commands, static scans, schema mirror hash, review archive schema validity, prompt/response
hashes, PASS-with-blocker rejection, suite tables, `CONSISTENCY_AUDIT.md`, replay/tamper evidence,
and exact final claim.

Proposed final claim:

```text
RGDTPK_Q_V4_ACCEPTANCE_COMPLETE
```

Forbidden claims remain:

```text
SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC
benchmark superiority
universal finite-system completeness
geometry DSL support
natural-language or diagram support
any R-ID VERIFIED status
```

Return PASS or FAIL_FIXABLE with concrete file/line citations.
