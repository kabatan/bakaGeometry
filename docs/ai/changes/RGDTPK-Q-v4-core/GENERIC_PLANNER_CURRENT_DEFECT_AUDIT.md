# Generic Planner Current Defect Audit

## Defect class

A dense TargetRelationSearch admission can attempt to materialize enormous monomial supports during planning. This can prevent the planner from collecting other kernel admissions.

This is a design defect because v4 requires deterministic planning with algebraic cost estimates, not unbounded admission-time allocation.

## What must be fixed

```text
1. Dense TargetRelationSearch must have closed-form feasibility preflight.
2. Dense schedule materialization must be delayed until after caps pass.
3. Admission collection must be isolated per kernel.
4. Cost-prohibited dense routes must not abort the solve.
5. Declared ladder must choose feasible compact target-direct routes.
6. Universal must not call dense relation search blindly.
```

## What must not be fixed by this repair

```text
- Do not add the reported geometry problem as regression.
- Do not add geometry-specific dispatch.
- Do not normalize the geometry input in solver core.
- Do not change candidate-cover semantics.
- Do not accept fast failure as success.
```

## Generic evidence expected

```text
- synthetic algebraic footprint proving dense preflight works;
- synthetic support-producing algebraic families proving planner routes around dense infeasibility;
- static scan proving no case-specific content;
- cost trace proving dense route was cost-prohibited while compact route succeeded.
```
