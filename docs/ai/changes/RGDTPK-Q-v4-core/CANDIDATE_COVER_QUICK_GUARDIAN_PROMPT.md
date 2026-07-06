# Quick Guardian Prompt — Candidate-Cover Source-Faithful Completion

You are implementing the candidate-cover layer of R-GDTPK-Q / ACCTP-Q.

Do not continue from the previous source-faithful exact-image repair pack as-is. That pack over-constrained the current work by making exact-image mixed real fiber classification a blocker.

Use these files as controlling instructions:

```text
CANDIDATE_COVER_COMPLETION_BASE_SPEC.md
CANDIDATE_COVER_COMPLETION_PLAN.md
CANDIDATE_COVER_COMPLETION_REVIEWER_PROMPTS.md
CANDIDATE_COVER_ACCEPTANCE_MATRIX.yaml
CANDIDATE_COVER_AGENT_RESET_TEMPLATE.md
```

Core semantic rule:

```text
CertifiedCandidateCover means:
    true target values ⊆ roots(S)

It does not mean:
    roots(S) = true target values
```

Spurious roots are allowed.

Your final candidate-cover repair may claim only:

```text
CANDIDATE_COVER_CORE_READY
SOURCE_FAITHFUL_TO_V4_CANDIDATE_COVER_LAYER
```

Do not claim:

```text
EXACT_IMAGE_CORE_READY
SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC
RGDTPK_Q_V4_ACCEPTANCE_COMPLETE
```

unless exact-image source-faithful completion is separately implemented and reviewed.

Start with:

```text
CCC-P0
CCC-P1
```

Do not edit code before completing the reset and source map.
