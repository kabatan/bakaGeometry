# P13 Review Packet - Exact Image Mode

Review target: Plan P13 only.

## Scope

Source anchors:

- `BASE_SPEC.md` RGQ-029, RGQ-049, RGQ-050, MECH-012, Appendix A section 27.
- `PLAN.md` P13 implementation tasks.
- `REVIEWER_PROMPTS.md#P13`.
- Guardian Runtime Contract from `AGENTS.md`.

Changed implementation files:

- `geosolver-core/src/algebra/sign.rs`
- `geosolver-core/src/compose/final_support.rs`
- `geosolver-core/src/fiber/exact_image.rs`
- `geosolver-core/src/fiber/hermite.rs`
- `geosolver-core/src/fiber/slack_semantics.rs`
- `geosolver-core/src/fiber/thom.rs`
- `geosolver-core/src/result/output.rs`
- `geosolver-core/src/solver/orchestrator.rs`
- `geosolver-core/src/solver/pipeline.rs`
- `geosolver-core/src/verify/replay.rs`
- `geosolver-core/tests/p13_exact_image_semantics.rs`

Changed evidence/docs:

- `docs/ai/ACTIVE_CONTEXT.md`
- `docs/ai/changes/RGDTPK-Q-v4-core/FULL_CORE_ACCEPTANCE_RESULTS.md`
- `docs/ai/changes/RGDTPK-Q-v4-core/FULL_CORE_SOURCE_SPEC_COMPLIANCE_MAP.md`
- `docs/ai/changes/RGDTPK-Q-v4-core/CLOSURE.md`
- `docs/ai/changes/RGDTPK-Q-v4-core/evidence/P13/*`

## Implementation Summary

P13 implements `classify_real_target_image`, Hermite-style real fiber count certificates, Thom/sign
classification certificates, and slack/guard/branch semantic binding. The public orchestrator now
returns `CertifiedExactTargetImage` or `CertifiedEmptyRealTargetImage` only after finite exact-image
classification executes. Candidate-cover mode still returns `CertifiedCandidateCover` and has no
exact-image certificate. Exact-image nonfinite diagnostics expose proof-kind evidence showing that
the real nonfinite certificate path ran.

The finite classifier is conservative. Mixed target/coordinate fibers not covered by target-only or
recognized slack-semantic certificate patterns return `CertificateDesignGap`, not exact-image
success.

## Verification

Commands run and passing:

```text
cargo fmt --manifest-path geosolver-core/Cargo.toml -- --check
cargo clippy --manifest-path geosolver-core/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path geosolver-core/Cargo.toml --test p13_exact_image_semantics -- --nocapture
cargo test --manifest-path geosolver-core/Cargo.toml --all-targets --all-features -- --nocapture
git diff --check
```

P13 suite cases:

- candidate-cover mode does not claim exact image;
- slack semantics removes a spurious support root and leaves a nonempty exact image;
- exact image is empty under real slack semantics;
- branch/slack semantics changes target feasibility;
- exact-image-mode nonfinite uses the real nonfinite certificate path.
- exact-image-mode nonfinite with semantic encodings but no real semantic proof returns
  `CertificateDesignGap`.
- exact-image-mode nonfinite with guard/saturation obligations but no real proof returns
  `CertificateDesignGap`.

Initial P13 reviewers found fixable blockers for hard-coded Hermite counts, semantic/guard/saturation
nonfinite overclaim, and an omitted material changed file (`solver/pipeline.rs`). This packet now
includes remediation: Hermite counts exact support-root/root-index and recognized semantic real-root
factors, `solver/pipeline.rs` passes compressed-system semantic/guard/saturation obligations into
final support, and exact-image nonfinite returns certificate gap unless the relevant real proof is
available.

## Claim Boundary

Before reviewer pass, the approved claim remains:

```text
CANDIDATE_COVER_CORE_READY
```

If P13 passes, it may close MECH-012 and support the exact-image layer evidence. This review must
not approve P14/P15/P16, final public replay-bound nonfinite readiness, benchmark readiness,
`RGDTPK_Q_V4_ACCEPTANCE_COMPLETE`, `SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC`, or any R-ID as
`VERIFIED`.
