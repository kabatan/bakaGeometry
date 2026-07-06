# Generic Planner Success-Route Patch Notes

This pack replaces the earlier case-regression framing.

Key correction:

```text
The reported geometry problem must not be part of the repair plan or gate.
```

Reason:

```text
Including that problem in the plan would invite the Agent to optimize toward that fixture.
The correct repair is generic: dense TargetRelationSearch must not block declared success routes on any large algebraic footprint.
```

Expected outcome:

```text
After this generic repair, external experiments may rerun the reported problem, but that run is not part of the Guardian acceptance gate.
```
