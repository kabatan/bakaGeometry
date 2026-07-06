# Active Context - GeoSolver R-GDTPK-Q-v4-core

Active change: `docs/ai/changes/RGDTPK-Q-v4-core/`

Current instruction pack: `generic_planner_success_route_repair_pack_v1`.

Active claim ceiling for this repair:

```text
PLANNER_SUCCESS_ROUTE_READY
DENSE_TRS_ADMISSION_SAFE
```

Candidate-cover means every true finite target value is contained in `roots(S)`. It does not mean
`roots(S)` equals the exact target image. Extra real roots are allowed and must not be treated as a
candidate-cover failure.

This repair is generic planner success-route work only. It must not use the prior timeout input as a
fixture, gate, benchmark, expected-answer target, or implementation branch.

Read first:

1. `GENERIC_PLANNER_SUCCESS_ROUTE_BASE_SPEC.md`
2. `GENERIC_PLANNER_SUCCESS_ROUTE_PLAN.md`
3. `GENERIC_PLANNER_ACCEPTANCE_MATRIX.yaml`
4. `GENERIC_PLANNER_CURRENT_DEFECT_AUDIT.md`
5. `GENERIC_PLANNER_PATCH_NOTES.md`
6. `GENERIC_PLANNER_AGENT_RESET.md`
7. `GENERIC_PLANNER_QUICK_GUARDIAN_PROMPT.md`
8. `GENERIC_PLANNER_SUCCESS_ROUTE_REVIEWER_PROMPTS.md`
9. `GENERIC_PLANNER_MATERIALIZATION_AUDIT.md`
10. `GENERIC_PLANNER_SUCCESS_ROUTE_RESULTS.md`
11. `GENERIC_PLANNER_REPLAY_AND_TRACE_RESULTS.md`
12. `GENERIC_PLANNER_CLOSURE.md`
13. `BASE_SPEC.md`
14. `PLAN.md`
15. `SOURCE_MAP.md`

Related diagnostic reports:

- A prior timeout investigation report exists in this folder as non-authoritative background only.
  Its concrete input is not an acceptance item for this repair.
- `MIXTILINEAR_POST_GPSR_TIMEOUT_ROOT_CAUSE_REPORT.md` records the post-GPSR timeout diagnosis for
  the user-supplied external diagnostic input. It is not an acceptance item or permanent fixture.

Authority note: this file is navigation only. It does not add, remove, or weaken requirements.
