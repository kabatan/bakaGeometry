# Patch Notes — From Source-Faithful Completion v1 to Candidate-Cover Completion v1

## What changed

The previous `SOURCE_FAITHFUL_COMPLETION_*` pack targeted full v4 source fidelity, including exact-image semantics. That was too broad for the current research layer.

This pack narrows the implementation target to the candidate-cover layer, as defined in v4:

```text
CertifiedCandidateCover:
    true target values ⊆ roots(S)
    spurious roots allowed
```

## Removed as candidate-cover blockers

```text
- mixed coordinate real fiber classifier;
- exact-image semantic filtering;
- removal of spurious roots;
- CertifiedExactTargetImage readiness;
- general nonfinite completeness for every nonfinite target image.
```

## Kept as blockers

```text
- missing exact proof that S contains all true target values;
- narrow slice completion;
- hidden full-coordinate fallback;
- expected answer / geometry dispatch;
- missing root isolation/decode;
- replay not binding DAG/messages/support/root/candidate;
- false nonfinite without positive proof.
```

## Important reviewer rule

Do not fail a candidate-cover implementation because it returns extra roots.

Fail only if the implementation cannot prove that all true target values are included in the finite candidate set.
