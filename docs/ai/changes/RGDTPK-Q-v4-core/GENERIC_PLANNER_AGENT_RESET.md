# Generic Planner Agent Reset

Before implementing this repair, the Agent must accept the following.

```text
I am not fixing one named geometry problem.
I must not add that problem as a test, gate, benchmark, or hidden target.
I am fixing a generic planner design defect: an infeasible dense relation-search route can block all other declared local projection kernels.
```

Correct goal:

```text
Use algebraic footprint to avoid infeasible dense route materialization and route planning toward feasible declared target-direct kernels.
```

Incorrect goals:

```text
- Make planning return failure quickly.
- Add a geometry-specific heuristic.
- Recognize variable IDs or polynomial shapes from one reported problem.
- Remove TargetRelationSearch entirely.
- Treat specialized kernels as separate solvers.
```

The pipeline remains unified. Multiple kernels are declared local projection methods inside the unified TargetProjectionDAG pipeline.
