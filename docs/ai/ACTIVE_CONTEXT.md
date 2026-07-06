# Active Context - GeoSolver R-GDTPK-Q-v4-core

Active change: `docs/ai/changes/RGDTPK-Q-v4-core/`

Current instruction pack: `generic_success_route_core_repair_pack_v2`.

Active claim ceiling for this repair:

```text
GENERIC_SUCCESS_ROUTE_PLANNER_READY
```

Candidate-cover means every true finite target value is contained in `roots(S)`. It does not mean
`roots(S)` equals the exact target image. Extra real roots are allowed and must not be treated as a
candidate-cover failure.

This repair is generic success-route core work only. It must not use the prior timeout input as a
fixture, gate, benchmark, expected-answer target, or implementation branch.

Read first:

1. `GENERIC_SUCCESS_ROUTE_BASE_SPEC.md`
2. `GENERIC_SUCCESS_ROUTE_PLAN.md`
3. `GENERIC_PLANNER_ACCEPTANCE_MATRIX.yaml`
4. `GENERIC_SUCCESS_ROUTE_CURRENT_DEFECT_AUDIT.md`
5. `GENERIC_SUCCESS_ROUTE_AGENT_RESET.md`
6. `GENERIC_SUCCESS_ROUTE_QUICK_GUARDIAN_PROMPT.md`
7. `GENERIC_SUCCESS_ROUTE_REVIEWER_PROMPTS.md`
8. `GENERIC_SUCCESS_ROUTE_ACCEPTANCE_RESULTS.md`
9. `GENERIC_SUCCESS_ROUTE_STATIC_SCAN.md`
10. `GENERIC_SUCCESS_ROUTE_CLOSURE.md`
11. `GENERIC_SUCCESS_ROUTE_REVIEW_SUMMARY.md`
12. `BASE_SPEC.md`
13. `PLAN.md`
14. `SOURCE_MAP.md`

Related diagnostic reports:

- A prior timeout investigation report exists in this folder as non-authoritative background only.
  Its concrete input is not an acceptance item for this repair.
- `MIXTILINEAR_POST_GPSR_TIMEOUT_ROOT_CAUSE_REPORT.md` records the post-GPSR timeout diagnosis for
  the user-supplied external diagnostic input. It is not an acceptance item or permanent fixture.

Authority note: this file is navigation only. It does not add, remove, or weaken requirements.
