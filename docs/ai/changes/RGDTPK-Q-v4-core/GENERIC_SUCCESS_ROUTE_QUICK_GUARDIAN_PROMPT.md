# Generic Success-Route Repair Quick Guardian Prompt v2

You are implementing `RGDTPK-Q-v4-generic-success-route-core-repair-v2`.

Read these files first:

1. `GENERIC_SUCCESS_ROUTE_BASE_SPEC.md`
2. `GENERIC_SUCCESS_ROUTE_PLAN.md`
3. `GENERIC_SUCCESS_ROUTE_REVIEWER_PROMPTS.md`
4. `GENERIC_PLANNER_ACCEPTANCE_MATRIX.yaml`

The repair is not allowed to include the concrete investigated geometry problem as a test, fixture, gate, benchmark, route selector, or expected-answer oracle.

The goal is generic:

```text
When dense total-degree TargetRelationSearch is infeasible for a large algebraic block,
the unified TargetProjectionDAG pipeline must quickly continue to compact declared target-direct routes,
execute a successful certified ProjectionMessage route when one exists,
and return CertifiedCandidateCover through api::solve_target on generic algebraic stress families.
```

Do not pass by returning `FiniteResourceFailure`, `AlgorithmicHardCase`, or `CertificateDesignGap` for support-producing generic stress.

Do not add geometry-family dispatch.

Do not add hidden fallback.

Do not call full coordinate solve, full coordinate RUR, QE, or CAD.

Implement phases GSR-P0 through GSR-P7 exactly. Each phase must have reviewer archive files using the phase-specific reviewer prompts in `GENERIC_SUCCESS_ROUTE_REVIEWER_PROMPTS.md`.

A phase is not closable unless the reviewer explicitly confirms:
- no concrete case overfit;
- no fast-failure completion;
- no hidden fallback;
- support-producing generic stress succeeds through public pipeline;
- route trace and replay evidence are present.
