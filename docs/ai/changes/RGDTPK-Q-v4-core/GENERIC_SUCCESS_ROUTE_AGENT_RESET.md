# Generic Success-Route Agent Reset

Status: active guardrail for `RGDTPK-Q-v4-generic-success-route-core-repair-v2`.

I will not add or use the investigated concrete geometry problem as a test, fixture, gate,
benchmark, or route selector.

I will not add geometry-family dispatch or expected-answer dispatch.

I will not treat fast failure as success.

The repair is complete only if generic algebraic large-footprint stress cases produce
`CertifiedCandidateCover` through `api::solve_target`.

Dense `TargetRelationSearch` cost prohibition is route-level only, not solve-level.

This repair uses only generic algebraic footprints: local variable count, relation count,
separator/exported variables, polynomial degree, monomial count, sparse template estimates,
quotient/action feasibility, and declared route traces.
