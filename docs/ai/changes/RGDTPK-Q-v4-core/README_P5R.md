# P5R Remediation Instruction Pack

This pack is a mandatory remediation layer to insert **after P5 and before P6** of `RGDTPK-Q-v4-core`.

It is not a replacement for the existing v2.2 Base Spec and Plan. It is a tightening amendment whose purpose is to prevent P6 and later phases from treating narrow or partially verified P3/P4 primitives as a completed generic target-direct solver foundation.

The implementation Agent must copy these files into:

```text
docs/ai/changes/RGDTPK-Q-v4-core/
```

Then the Agent must patch the active Guardian documents as specified in `P5R_PLAN.md`.

## Files

```text
P5R_BASE_SPEC_AMENDMENT.md
P5R_PLAN.md
P5R_REVIEWER_PROMPTS.md
P5R_ACCEPTANCE_CHECKLIST.yaml
P5R_QUICK_GUARDIAN_PROMPT.md
P5R_PATCH_NOTES.md
PACK_MANIFEST.sha256
```

## Mandatory rule

P6 must not start until P5R is fully implemented, reviewed, and evidence-bound to the current Git commit.

P5R does not claim candidate-cover readiness, exact-image readiness, kernel readiness, or acceptance completion. It only makes the pre-planner foundation safe enough to continue.
