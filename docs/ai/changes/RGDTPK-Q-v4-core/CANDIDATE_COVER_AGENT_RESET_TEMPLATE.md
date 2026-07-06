# Candidate-Cover Agent Reset Template

The Agent must copy this file to `CANDIDATE_COVER_AGENT_RESET.md` and fill every item before editing code.

## 1. Current goal

Write in your own words:

```text
The current goal is to implement the v4 candidate-cover layer.
The solver must return finite candidates containing every true target value.
It is allowed to return extra candidates.
```

## 2. What must not be done

Acknowledge:

```text
- Do not treat spurious roots as a failure.
- Do not use exact-image filtering to make candidate-cover pass.
- Do not shrink to target-univariate / affine / alias / bivariate / fixture-specific slices.
- Do not use full coordinate solution, full RUR, or hidden QE/CAD fallback.
- Do not route failure to nonfinite without positive proof.
```

## 3. Prior failure modes to avoid

List at least five concrete prior failure modes from this project, generalized.

Required inclusions:

```text
- gate/review evidence treated as algorithmic proof;
- documented limitation treated as acceptable completion;
- no-fallback confused with narrow scope;
- expected-answer or fixture-like stress;
- exact-image requirements confused with candidate-cover requirements.
```

## 4. Phase handoff pledge

Every phase handoff must answer:

```text
1. Did this phase prove true values ⊆ roots(S)?
2. Did this phase accidentally require roots(S) ⊆ true values?
3. Did any support-producing case depend on exact-image filtering?
4. Did any partial slice become a completion claim?
5. Did any hidden fallback or unsupported path appear?
```
