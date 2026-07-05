# Patch Notes — v2.1 nonfinite routing correction

This patch fixes a normative wording error in `RGQ-051` and removes related ambiguity in `RGQ-022` and `RGQ-000`.

## Corrected RGQ-051 wording

The previous draft used the wrong polarity for relation-search exhaustion and similar failures. The corrected normative sentence is:

```text
No relation found within RGQ-042 bounds, sparse heuristic failure, Universal stage failure, or composition failure to produce target-only support must not route to CertifiedNonFiniteTargetImage.
```

The corrected normative rule also says these failures must route to `AlgorithmicHardCase`, `FiniteResourceFailure`, or `CertificateDesignGap` according to the available evidence.

## Related RGQ-022 correction

`RGQ-022` no longer permits local `UniversalTargetEliminationKernel` execution to return `CertifiedNonFiniteTargetImage` when no exported relation exists. Local Universal execution returns a `ProjectionMessage` only when at least one exported generator is certified. Otherwise it returns `AlgorithmicHardCase`, `FiniteResourceFailure`, or `CertificateDesignGap`.

## Related RGQ-000 clarification

`RGQ-041` through `RGQ-056` are explicit hardening amendments. They override weaker or unsafe wording only in the stricter direction, and never authorize behavior Appendix A forbids.

## Normative rule after this patch

`CertifiedNonFiniteTargetImage` is allowed only through `RGQ-045` positive nonfiniteness certification, with `RGQ-050` additionally required in exact-image mode. Relation-search exhaustion, sparse heuristic failure, Universal stage failure, and composition failure are not nonfiniteness certificates.
