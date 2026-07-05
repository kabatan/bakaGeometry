# Quick Guardian Prompt for the Implementation Agent

You are implementing a mandatory remediation phase called P5R for `RGDTPK-Q-v4-core`.

Read in order:

```text
docs/ai/changes/RGDTPK-Q-v4-core/BASE_SPEC.md
docs/ai/changes/RGDTPK-Q-v4-core/PLAN.md
docs/ai/changes/RGDTPK-Q-v4-core/P5R_BASE_SPEC_AMENDMENT.md
docs/ai/changes/RGDTPK-Q-v4-core/P5R_PLAN.md
docs/ai/changes/RGDTPK-Q-v4-core/P5R_REVIEWER_PROMPTS.md
docs/ai/changes/RGDTPK-Q-v4-core/P5R_ACCEPTANCE_CHECKLIST.yaml
```

Do not begin P6. Insert and complete P5R first.

Your job is not to make the existing P5 implementation look good. Your job is to remove the unsafe paths that would let later phases overclaim a partial or heavy-fallback solver.

Implement the P5R subphases exactly:

```text
P5R-a evidence rebinding and claim consistency
P5R-b no fake F4
P5R-c guarded rational affine semantics
P5R-d quotient/action provenance
P5R-e primitive scope ledger and anti-overclaim wiring
P5R-f P6 readiness audit
```

For every subphase, produce:

```text
docs/ai/changes/RGDTPK-Q-v4-core/evidence/<phase>/commands.txt
docs/ai/changes/RGDTPK-Q-v4-core/evidence/<phase>/command_outputs.txt
docs/ai/changes/RGDTPK-Q-v4-core/evidence/<phase>/static_scans.txt
docs/ai/changes/RGDTPK-Q-v4-core/evidence/<phase>/function_implementation_table.yaml
docs/ai/changes/RGDTPK-Q-v4-core/evidence/<phase>/notes.md

docs/ai/changes/RGDTPK-Q-v4-core/reviews/<phase>/<timestamp>/prompt.md
docs/ai/changes/RGDTPK-Q-v4-core/reviews/<phase>/<timestamp>/response.md
docs/ai/changes/RGDTPK-Q-v4-core/reviews/<phase>/<timestamp>/review_summary.yaml
docs/ai/changes/RGDTPK-Q-v4-core/reviews/<phase>/<timestamp>/evidence_manifest.yaml
```

Do not close P5R if any of these remains true:

```text
- F4 is still a Groebner wrapper but production can call it F4.
- guarded affine substitution only works for polynomial quotients.
- TargetActionKrylov can use self-certifying injected action columns.
- primitive limitations are documented but not enforced against later overclaim.
- evidence is not bound to current Git commit.
- CLOSURE.md or ACTIVE_CONTEXT.md suggests candidate-cover or acceptance readiness.
```

P5R final claim ceiling is only:

```text
PARTIAL_MECHANISM_READY:MECH-004
```
