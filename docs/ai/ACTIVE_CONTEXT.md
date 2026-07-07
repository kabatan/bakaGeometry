# Active Context - GeoSolver R-GDTPK-Q-v4-core

Active change: `docs/ai/changes/RGDTPK-Q-v4-core/`

Current instruction pack: `alg_cost_completion_repair_pack_v1`.

Current algebraic-cost completion claim boundary:

```text
CANDIDATE_COVER_CORE_READY
SOURCE_FAITHFUL_TO_V4_CANDIDATE_COVER_LAYER
```

These labels are restored only for the candidate-cover algebraic-cost layer after ACR-P10 closure
review. They do not imply exact-image acceptance or full supplied-v4 acceptance.

Candidate-cover means every true finite target value is contained in `roots(S)`. It does not mean
`roots(S)` equals the exact target image. Extra real roots are allowed and must not be treated as a
candidate-cover failure.

This repair is algebraic-cost completion work. It must not use the prior timeout input as a fixture,
gate, benchmark, expected-answer target, or implementation branch. ACR-P10 closure is recorded in
`ALG_COST_COMPLETION_CLOSURE.md` and remains bounded to candidate-cover algebraic-cost readiness.

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

- `ALG_COST_COMPLETION_CLOSURE.md` records ACR-P10 closure and the exact claim boundary.
- `ALG_COST_FINAL_RED_TEAM_RESULTS.md` records closure red-team evidence.
- `ALG_COST_ROUTE_BUDGET_AUDIT.md`, `ALG_COST_DECOMPOSITION_AUDIT.md`, and
  `ALG_COST_NO_OVERFIT_AUDIT.md` record the final boundedness, decomposition, and anti-overfit
  audits.
- `evidence/ACR-P9/MECH_EVIDENCE.md` records the generic large-footprint stress-suite evidence and
  `reviews/ACR-P9/20260707-012120Z/` records reviewer PASS.

Related diagnostic reports:

- A prior timeout investigation report exists in this folder as non-authoritative background only.
  Its concrete input is not an acceptance item for this repair.
- `MIXTILINEAR_POST_GPSR_TIMEOUT_ROOT_CAUSE_REPORT.md` records the post-GPSR timeout diagnosis for
  the user-supplied external diagnostic input. It is not an acceptance item or permanent fixture.
- `MIXTILINEAR_POST_GSR_CORE_REPAIR_TIMEOUT_REPORT.md` records the post-GSR core repair timeout
  diagnosis for the same user-supplied external diagnostic input. It is not an acceptance item or
  permanent fixture.
- Earlier candidate-cover and generic-success closures remain historical background. Current
  readiness authority for the algebraic-cost candidate-cover layer is the ACR-P10 closure packet,
  not those earlier closures.

Authority note: this file is navigation only. It does not add, remove, or weaken requirements.
