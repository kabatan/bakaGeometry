Read-only final Guardian review for `RGDTPK-Q-v4-generic-success-route-core-repair-v2`.

Workspace: `C:\Users\bakat\OneDrive\ドキュメント\bakaGeometry`

Review scope:

- Base spec: `docs/ai/changes/RGDTPK-Q-v4-core/GENERIC_SUCCESS_ROUTE_BASE_SPEC.md`
- Plan: `docs/ai/changes/RGDTPK-Q-v4-core/GENERIC_SUCCESS_ROUTE_PLAN.md`
- Acceptance matrix: `docs/ai/changes/RGDTPK-Q-v4-core/GENERIC_PLANNER_ACCEPTANCE_MATRIX.yaml`
- Current evidence: `GENERIC_SUCCESS_ROUTE_ACCEPTANCE_RESULTS.md`, `GENERIC_SUCCESS_ROUTE_STATIC_SCAN.md`, `GENERIC_SUCCESS_ROUTE_CLOSURE.md`
- Implementation files in planner, kernels, graph, solver, verify, and tests touched by this change.

Required checks:

1. Dense TargetRelationSearch large-footprint preflight is descriptor/lazy and does not materialize cost-prohibited supports or matrices.
2. Cost-prohibited dense routes are machine-readable and cannot enter the declared ladder.
3. Kernel admissions are collected independently and route-local planning failures are represented as `PlanProbeFailed` or `CostProhibited`.
4. Declared ladder execution tries declared routes in deterministic order, records route-local failures, and has no hidden fallback.
5. `UniversalTargetElimination` remains a real declared generic route, is forced last when present, and has a certificate-bound fixed internal strategy sequence.
6. Universal strategy trace verification rejects mismatched attempted/chosen/inner-payload traces.
7. Graph decomposition has generic algebraic metrics and separator candidate classes; it is not geometry-family or concrete-problem dispatch.
8. P6/GSR stress evidence covers generic algebraic route families without using the investigated concrete problem.
9. Static scans show no production geometry dispatch, expected-answer dispatch, concrete-problem dispatch, full coordinate solution enumeration, QE/CAD fallback, hidden fallback, or implementation placeholders.
10. Final claim is bounded to `GENERIC_SUCCESS_ROUTE_PLANNER_READY` and is not confused with exact-image readiness, full supplied-v4 source fidelity, full acceptance, or benchmark superiority.

Known requirement to evaluate:

- Because GSR-P3 requires Universal to be last, the G4 stress case must demonstrate public `api::solve_target` execution where earlier declared routes fail route-locally, Universal remains last, Universal executes, a later Universal internal strategy succeeds, and replay accepts. Decide whether the current evidence satisfies that requirement.

Report format:

- Return `PASS`, `FAIL_FIXABLE`, or `FAIL_BLOCKING`.
- If failing, cite file/line references and state the minimal required fix.
- If passing, state that the only supported final claim is `GENERIC_SUCCESS_ROUTE_PLANNER_READY`.
