# P12G Quick Guardian Prompt

You are implementing the mandatory P12G remediation in `kabatan/bakaGeometry`.

Read first:

```text
docs/ai/changes/RGDTPK-Q-v4-core/BASE_SPEC.md
docs/ai/changes/RGDTPK-Q-v4-core/P5R_BASE_SPEC_AMENDMENT.md
docs/ai/changes/RGDTPK-Q-v4-core/P12G_DIRECT_ALGORITHM_AUDIT_FINDINGS.md
docs/ai/changes/RGDTPK-Q-v4-core/P12G_BASE_SPEC_AMENDMENT.md
docs/ai/changes/RGDTPK-Q-v4-core/P12G_PLAN.md
docs/ai/changes/RGDTPK-Q-v4-core/P12G_REVIEWER_PROMPTS.md
docs/ai/changes/RGDTPK-Q-v4-core/P12G_ACCEPTANCE_CHECKLIST.yaml
```

Do not begin P13 or P14. P12G is a mandatory barrier.

Your goal is to determine and repair whether the current P6–P12 implementation is truly a general R-GDTPK target-direct algebraic algorithm, not merely a set of narrow kernels with good evidence.

Implement P12G-a through P12G-h in order.

The most important blockers are:

```text
1. TargetActionKrylov currently appears to require an already target-only univariate relation.
   Either implement generic quotient/action construction from authorized local relations, or reopen/demote P8c/MECH-014.

2. Several kernels appear to compute final output relations during planning.
   Refactor to pure plans or introduce typed CertifiedProbePlan objects with evidence and replay.

3. Candidate-cover finalizer must not treat nonzero support with zero real roots as AlgorithmicHardCase.

4. CoreRunCertificate invariant flags must become truthful/evidence-bound or hard-block P14/P16.

5. Replay must eventually use actual TargetProjectionDAG/block authorization, not synthetic all-relations replay.

6. Add the P12G general algebraic stress battery G1–G8.
```

Reviewer PASS alone is not enough. Each subphase requires code/evidence/review archive. P12G-h must produce `P13_P14_READINESS_AFTER_P12G.md`.

Do not claim:

```text
CANDIDATE_COVER_CORE_READY
EXACT_IMAGE_CORE_READY
RGDTPK_Q_V4_ACCEPTANCE_COMPLETE
SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC
```

P12G can only preserve or downgrade the current partial mechanism claim. If generic TargetActionKrylov is not fixed, reopen the relevant claim instead of hiding it.
