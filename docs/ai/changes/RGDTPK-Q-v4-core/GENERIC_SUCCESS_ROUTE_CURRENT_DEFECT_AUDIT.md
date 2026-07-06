# Generic Success-Route Current Defect Audit v2

## Current observed defect class

A large algebraic block can make dense total-degree TargetRelationSearch infeasible. A previous repair added preflight decline for dense TargetRelationSearch, but external real-problem tests still fail, indicating the repair likely stopped at timeout avoidance rather than success-route completion.

## Distinction

```text
Timeout avoidance:
  Dense route does not materialize billions of monomials.

Success-route repair:
  Dense route is cost-prohibited, other declared routes are collected,
  ladder execution reaches a certified ProjectionMessage,
  final support S(T) is verified,
  api::solve_target returns CertifiedCandidateCover on generic large-footprint stress.
```

The second is required.

## Suspected remaining defects

```text
1. Admission is improved, but support-producing feasibility is not guaranteed.
2. Planned routes may still fail in execution without ladder continuing correctly.
3. Universal may plan quickly but not produce a message on large blocks.
4. Decomposition may create oversized relation-heavy blocks even when algebraic separators exist.
5. Generic stress may still be too toy-like and not force success after dense TRS is cost-prohibited.
```

## Required repair target

Do not optimize for any concrete investigated geometry problem. Repair the generic route planner and Universal/decomposition logic so that large-footprint algebraic blocks with compact target-direct routes produce candidate-cover success.
