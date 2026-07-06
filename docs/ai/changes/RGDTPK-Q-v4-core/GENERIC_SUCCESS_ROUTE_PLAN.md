# Generic Success-Route Core Repair Plan v2

## Overview

This plan repairs the planner and local kernel routing so that a large dense-TRS-infeasible algebraic block does not cause timeout or solve-level failure when compact target-direct routes are available.

The plan must not include the concrete investigated geometry problem as a test, gate, fixture, benchmark, or code branch.

## Phase GSR-P0 — Agent reset and anti-overfit audit

### Goal

Prevent the agent from optimizing for a concrete problem or from treating preflight decline as success.

### Required artifacts

- `docs/ai/changes/RGDTPK-Q-v4-core/GENERIC_SUCCESS_ROUTE_AGENT_RESET.md`
- `docs/ai/changes/RGDTPK-Q-v4-core/GENERIC_SUCCESS_ROUTE_CURRENT_AUDIT.md`

### Required content

`GENERIC_SUCCESS_ROUTE_AGENT_RESET.md` must state:

```text
I will not add or use the investigated concrete geometry problem as a test, fixture, gate, benchmark, or route selector.
I will not add geometry-family dispatch or expected-answer dispatch.
I will not treat fast failure as success.
The repair is complete only if generic algebraic large-footprint stress cases produce CertifiedCandidateCover through api::solve_target.
Dense TargetRelationSearch cost prohibition is route-level only, not solve-level.
```

`CURRENT_AUDIT.md` must list the current state of:

```text
planner/relation_schedule.rs
planner/admission.rs
planner/ladder.rs
planner/planner.rs
solver/pipeline.rs
kernels/target_relation_search.rs
kernels/universal_elimination.rs
graph/tree_decomposition.rs
graph/separators.rs
graph/weighted_primal.rs
```

For each, record:

```yaml
file:
relevant_functions:
current_problem:
required_repair:
overfit_risk:
```

### Prohibited

- Mentioning the investigated concrete problem in new tests.
- Using exact variable counts / relation counts from the investigated case as constants in gates.

### Reviewer prompt

Use `GSR-REVIEW-P0`.

---

## Phase GSR-P1 — Dense TargetRelationSearch preflight and lazy schedule

### Goal

Make dense total-degree relation search impossible to materialize during admission unless it is proven feasible by closed-form estimates.

### Files

- `geosolver-core/src/planner/relation_schedule.rs`
- `geosolver-core/src/solver/options.rs`
- `geosolver-core/src/planner/admission.rs`
- `geosolver-core/src/kernels/target_relation_search.rs`

### Required implementation

1. Add `SaturatingCount`.
2. Add `RelationSearchPlanningCaps`.
3. Add `RelationSearchStageEstimate`.
4. Add `DenseRelationSearchPreflight`.
5. Add `monomial_count_total_degree_leq_saturating(n, d)`.
6. Add `estimate_dense_relation_search_schedule(...)`.
7. Ensure estimation never allocates `Vec<Monomial>`.
8. Add default planning caps used even when options are `None`.
9. Add route-level decline reason and preflight hash.
10. Ensure `build_dense_relation_search_schedule(...)` is called only after preflight passes.
11. Recheck caps in `execute_target_relation_search` before each stage materialization.

### Required tests

Use generic generated algebraic blocks only.

Tests must verify:

```text
- closed-form count equals enumeration for small n,d;
- count saturates safely for large n,d;
- infeasible dense schedule is declined without materialization;
- preflight hash is deterministic;
- no concrete investigated problem appears in test names or inputs.
```

### Reviewer prompt

Use `GSR-REVIEW-P1`.

---

## Phase GSR-P2 — Kernel admission isolation and route records

### Goal

One bad/infeasible route must not block collection of later admissions.

### Files

- `planner/admission.rs`
- `planner/kernel_plan.rs`
- `planner/planner.rs`
- `result/diagnostics.rs`
- `result/cost_trace.rs`

### Required implementation

1. Extend admission with route diagnostics:

```rust
pub enum KernelAdmissionStatus {
    Admitted,
    Declined { reason: String },
    CostProhibited { reason: String, estimate_hash: Hash },
    PlanProbeFailed { reason: String, constructed_object_hash: Hash },
}
```

or equivalent. It must remain distinguishable from solve-level failure.

2. Add `KernelRouteDiagnostic` / `KernelRouteTrace` containing:

```text
kernel_kind
block_id
admission_status
preflight_estimate_hash
estimated_rows
estimated_cols
estimated_rank
cost_class
decline_reason
```

3. `collect_kernel_admissions` must record every kernel kind, even if an earlier kernel is cost-prohibited.

4. Catch route-local `SolverError` from planning and convert to route-local decline, not solve-level failure.

5. Do not catch `ImplementationBug` silently. It must remain visible unless explicitly classified as route-local invariant failure with evidence.

### Required tests

Generic tests must verify:

```text
- a generated block where dense TRS is cost-prohibited still records admissions for all other kernels;
- Universal admission is present for every relation-bearing block;
- admissions vector length equals all planner kernel kinds;
- no route-local decline becomes solve-level failure during planning.
```

### Reviewer prompt

Use `GSR-REVIEW-P2`.

---

## Phase GSR-P3 — Success-route declared ladder and execution isolation

### Goal

The planner must build and execute a declared ladder that can reach support-producing routes after dense TRS is declined.

### Files

- `planner/ladder.rs`
- `planner/kernel_plan.rs`
- `planner/planner.rs`
- `solver/pipeline.rs`
- `kernels/traits.rs`

### Required implementation

1. Add a cost class:

```rust
pub enum RouteCostClass {
    PreferredCompact,
    Feasible,
    ExpensiveButAllowed,
    CostProhibited,
}
```

2. `build_declared_ladder` must exclude `CostProhibited` routes but must include feasible compact routes.

3. A relation-bearing block with no ladder is an implementation bug unless there is a documented structural relationless proof.

4. Universal must be last when present, unless a stronger exact route is the only admitted route and Universal is proven redundant. The simpler required behavior is: include Universal last for every relation-bearing block.

5. `execute_block_with_declared_ladder` must:
   - try every declared plan in order;
   - verify the returned `ProjectionMessage`;
   - record every route failure;
   - continue after non-success route failure;
   - return aggregate failure only after all declared plans fail.

6. Add `BlockProjectionFailureTrace`.

### Required tests

Generated algebraic stress must include:

```text
- dense TRS cost-prohibited;
- at least one compact target-direct route feasible;
- first feasible compact route intentionally fails in execution;
- second declared route succeeds;
- final public result is CertifiedCandidateCover;
- replay accepts.
```

No investigated concrete problem may be used.

### Reviewer prompt

Use `GSR-REVIEW-P3`.

---

## Phase GSR-P4 — UniversalTargetElimination as real generic success route

### Goal

Universal must be a declared generic target/separator projection kernel capable of trying compact algebraic strategies on large blocks.

### Files

- `kernels/universal_elimination.rs`
- `algebra/elimination.rs`
- `kernels/sparse_resultant.rs`
- `kernels/action_krylov.rs`
- `kernels/specialization_interpolation.rs`
- `kernels/regular_chain_projection.rs`
- `kernels/norm_trace_projection.rs`
- `verify/verify_message.rs`

### Required implementation

1. Universal plan must contain an internal strategy sequence with deterministic hashes.
2. Each internal strategy must be route-local and certificate-bound.
3. Dense TRS internal strategy must use preflight and skip materialization when cost-prohibited.
4. Universal must attempt subsequent strategies after a strategy declines/fails.
5. Universal must not return nonfinite from exhaustion.
6. Universal must emit a `ProjectionMessage` if any internal strategy yields exported relation generators.
7. Universal certificate must record:
   - internal strategies attempted;
   - chosen internal strategy;
   - failed internal strategies with route-local traces;
   - exact membership/resultant/action/interpolation certificate for chosen generators.
8. Universal must export only target/separator variables.

### Required tests

Generated algebraic stress must include at least three families:

```text
U1: large-footprint block where dense TRS is cost-prohibited and Universal succeeds through target-action quotient.
U2: large-footprint block where dense TRS is cost-prohibited and Universal succeeds through sparse/resultant or specialization.
U3: large-footprint block where first internal Universal strategy fails and a later strategy succeeds.
```

All must go through `api::solve_target` and return `CertifiedCandidateCover`.

### Reviewer prompt

Use `GSR-REVIEW-P4`.

---

## Phase GSR-P5 — Algebraic graph decomposition refinement

### Goal

Avoid unnecessary giant hard blocks when algebraic separators exist.

### Files

- `graph/weighted_primal.rs`
- `graph/separators.rs`
- `graph/tree_decomposition.rs`
- `graph/projection_dag.rs`
- `graph/metrics.rs`

### Required implementation

1. Extend graph metrics with:
   - relation arity,
   - total degree,
   - monomial count,
   - coefficient height,
   - target distance,
   - affine/definitional eliminability,
   - estimated dense TRS cost,
   - estimated quotient/action rank,
   - sparse template estimate.

2. Extend separator candidates:
   - articulation variables,
   - min-fill candidates,
   - bounded min-cut candidates,
   - construction-like intermediate variables recognized algebraically,
   - low-degree definitional/affine variables.

3. Scoring must penalize:
   - huge relation-heavy leaf blocks,
   - saturated rank estimates,
   - infeasible dense TRS estimates.

4. If no separator improves cost, one-large-block is allowed, but the cost trace must show why.

5. Relation duplication, if added, must be certificate-bound.

### Required tests

Use generated relation-variable hypergraphs, not geometry cases.

Tests must verify:

```text
- separator-rich large algebraic graph decomposes into smaller projection blocks;
- relation assignment remains complete and authorized;
- no geometry role/name influences separator choice;
- no useful separator case still produces one-large-block and Universal ladder.
```

### Reviewer prompt

Use `GSR-REVIEW-P5`.

---

## Phase GSR-P6 — Generic support-producing stress suite

### Goal

Prove the repair is not merely timeout avoidance.

### Files

- `geosolver-core/tests/generic_success_route_planner.rs`
- `docs/ai/changes/RGDTPK-Q-v4-core/GENERIC_SUCCESS_ROUTE_ACCEPTANCE_RESULTS.md`

### Prohibited

The tests must not include the investigated concrete problem, its builder, its family name, its expected answer, or fixed exact input profile.

### Required generated stress families

All support-producing cases must call `api::solve_target` and return `CertifiedCandidateCover`.

#### G1: large footprint compact quotient/action

Generate a family parameterized by `(seed, layer_count, width)` such that:

```text
- dense total-degree TRS preflight is cost-prohibited;
- TargetActionKrylov or Universal-internal target-action route produces support;
- support is verified exactly;
- replay accepts.
```

#### G2: large footprint sparse/resultant or specialization

Generate a family parameterized by `(seed, eliminated_count, sparse_chain_length)` such that:

```text
- dense TRS is cost-prohibited;
- sparse/resultant or specialization route produces a message;
- public result is CertifiedCandidateCover.
```

#### G3: separator-rich composition

Generate a family with multiple blocks and separators such that:

```text
- at least two ProjectionMessages are required;
- removing one message changes support or makes replay fail;
- final support is verified.
```

#### G4: Universal later-strategy success

Generate a family where:

```text
- initial declared route fails;
- Universal tries internal strategies;
- a later Universal strategy succeeds;
- result is CertifiedCandidateCover.
```

#### G5: no useful separator one-large-block

Generate a family where:

```text
- no useful separator is found;
- one-large-block is produced;
- Universal remains in ladder;
- support-producing success occurs through a declared generic route.
```

### Required assertions for each support-producing case

```text
status == CertifiedCandidateCover
support_polynomial.is_some()
squarefree_support_polynomial.is_some()
projection_messages not empty
certificate.is_some()
replay_run_certificate(...).accepted
cost_trace includes per-block route trace
at least one dense TRS route is cost-prohibited in G1/G2/G4/G5
the successful route is not geometry-specific
```

### Reviewer prompt

Use `GSR-REVIEW-P6`.

---

## Phase GSR-P7 — Anti-overfit and source-fidelity closure

### Goal

Ensure the repair is generic, source-faithful, and not a new narrow gate.

### Required artifacts

- `GENERIC_SUCCESS_ROUTE_CLOSURE.md`
- `GENERIC_SUCCESS_ROUTE_STATIC_SCAN.md`
- `GENERIC_SUCCESS_ROUTE_REVIEW_SUMMARY.md`

### Required static scans

Search production code and new tests for:

```text
investigated problem/family names
problem_id
expected_answer
fixture
golden
circle solver
triangle solver
tangent solver
mixtilinear-style dispatch
coordinate solution enumeration
full coordinate RUR
QE
CAD
unimplemented!
todo!
placeholder
```

Classify every hit.

### Closure must state

```text
This repair does not add a concrete-problem regression.
This repair makes planner route selection generic and algebraic-footprint based.
Dense TRS cost prohibition is route-local, not solve-level failure.
Support-producing generic large-footprint stresses pass through api::solve_target.
```

### Reviewer prompt

Use `GSR-REVIEW-P7`.
