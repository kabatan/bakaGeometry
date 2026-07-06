# Candidate-Cover Agent Reset

## Current Goal

The current goal is to implement and close the v4 candidate-cover layer. The solver must return a
finite support polynomial `S(T)` whose real roots contain every true finite target value. It is
allowed to return extra candidates.

## What Must Not Be Done

- Do not treat spurious roots as a candidate-cover failure.
- Do not use exact-image filtering to make candidate-cover pass.
- Do not shrink the implementation to target-univariate, affine, alias, bivariate, or fixture-only
  slices.
- Do not use a full coordinate solution, full coordinate RUR, or hidden QE/CAD fallback.
- Do not route failure to nonfinite without positive proof.

## Prior Failure Modes To Avoid

- Gate/review evidence treated as algorithmic proof.
- Documented limitation treated as acceptable completion.
- No-fallback evidence confused with narrow implementation scope.
- Expected-answer or fixture-like stress substituted for algebraic tests.
- Exact-image requirements confused with candidate-cover requirements.
- Static-scan claims made from runtime flags without scan evidence.
- Nonfinite success claimed without replayable positive proof.

## Phase Handoff Answers

1. Did this phase prove true values subset roots(S)?
   - Yes, by exact target support verification via target-only support or composed ideal membership.
2. Did this phase accidentally require roots(S) subset true values?
   - No. Candidate-cover mode preserves extra roots and emits explicit diagnostics.
3. Did any support-producing case depend on exact-image filtering?
   - No. The public candidate-cover tests use default `exact_image_mode=false`.
4. Did any partial slice become a completion claim?
   - No. The active claim is limited to candidate-cover readiness.
5. Did any hidden fallback or unsupported path appear?
   - No hidden coordinate/RUR/QE/CAD fallback is admitted by the current closure.
