# Quick Guardian Prompt: Generic Planner Success-Route Repair

Implement `RGDTPK-Q-v4-generic-planner-success-route-v1`.

Read in order:

1. `GENERIC_PLANNER_SUCCESS_ROUTE_BASE_SPEC.md`
2. `GENERIC_PLANNER_SUCCESS_ROUTE_PLAN.md`
3. `GENERIC_PLANNER_SUCCESS_ROUTE_REVIEWER_PROMPTS.md`
4. `GENERIC_PLANNER_ACCEPTANCE_MATRIX.yaml`
5. `GENERIC_PLANNER_AGENT_RESET.md`
6. `GENERIC_PLANNER_CURRENT_DEFECT_AUDIT.md`

Hard constraints:

```text
- Do not include the investigated geometry problem as a test, fixture, benchmark, or gate.
- Do not add geometry-specific or expected-answer dispatch.
- Do not close by returning fast failure for large blocks.
- Fix generic planner routing so infeasible dense TargetRelationSearch cannot block other kernels.
- Add closed-form dense relation-search preflight before any dense support materialization.
- Ensure later kernels and Universal are still collected and planned.
- Add generic algebraic footprint tests only.
```

Closure requires all GPSR phases and reviewer prompts to pass.
