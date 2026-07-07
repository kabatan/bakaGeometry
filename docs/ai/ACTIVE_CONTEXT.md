# Active Context - GeoSolver R-GDTPK-Q-v4-core

Active change: `docs/ai/changes/RGDTPK-Q-v4-core/`

Current instruction pack: `alg_cost_completion_repair_pack_v1`.

Active claim ceiling for this repair:

```text
CANDIDATE_COVER_PIPELINE_PRESENT_BUT_ALGEBRAIC_COST_INCOMPLETE
```

Candidate-cover means every true finite target value is contained in `roots(S)`. It does not mean
`roots(S)` equals the exact target image. Extra real roots are allowed and must not be treated as a
candidate-cover failure.

This repair is algebraic-cost completion work. It must not use the prior timeout input as a fixture,
gate, benchmark, expected-answer target, or implementation branch. The prior
`CANDIDATE_COVER_CORE_READY` and `SOURCE_FAITHFUL_TO_V4_CANDIDATE_COVER_LAYER` claims are suspended
until all `ALG_COST_COMPLETION_REPAIR_PLAN.md` phases pass.

Read first:

1. `ALG_COST_COMPLETION_REPAIR_BASE_SPEC.md`
2. `ALG_COST_COMPLETION_REPAIR_PLAN.md`
3. `ALG_COST_COMPLETION_REVIEWER_PROMPTS.md`
4. `ALG_COST_ACCEPTANCE_MATRIX.yaml`
5. `ALG_COST_CURRENT_DEFECT_AUDIT.md`
6. `ALG_COST_REPAIR_STATUS.md`
7. `ALG_COST_AGENT_RESET.md`
8. `ALG_COST_QUICK_GUARDIAN_PROMPT.md`
9. `BASE_SPEC.md`
10. `PLAN.md`
11. `SOURCE_MAP.md`

Current implementation evidence:

- `ALG_COST_P8_MECH_EVIDENCE.md` records sparse/lazy TargetRelationSearch implementation evidence.
- `evidence/ACR-P9/MECH_EVIDENCE.md` records the generic large-footprint stress-suite evidence
  pending reviewer closure.

Related diagnostic reports:

- A prior timeout investigation report exists in this folder as non-authoritative background only.
  Its concrete input is not an acceptance item for this repair.
- `MIXTILINEAR_POST_GPSR_TIMEOUT_ROOT_CAUSE_REPORT.md` records the post-GPSR timeout diagnosis for
  the user-supplied external diagnostic input. It is not an acceptance item or permanent fixture.
- `MIXTILINEAR_POST_GSR_CORE_REPAIR_TIMEOUT_REPORT.md` records the post-GSR core repair timeout
  diagnosis for the same user-supplied external diagnostic input. It is not an acceptance item or
  permanent fixture.
- Earlier candidate-cover and generic-success closures are historical evidence only during this
  repair. They are not current readiness authority until algebraic-cost completion closes.

Authority note: this file is navigation only. It does not add, remove, or weaken requirements.
