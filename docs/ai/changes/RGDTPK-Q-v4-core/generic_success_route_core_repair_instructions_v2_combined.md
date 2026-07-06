# Generic Success-Route Core Repair Pack v2



---

# FILE: GENERIC_SUCCESS_ROUTE_BASE_SPEC.md

# Generic Success-Route Core Repair Base Spec v2
Change ID: `RGDTPK-Q-v4-generic-success-route-core-repair-v2`

## 0. Purpose

This amendment repairs the planner / DAG / kernel-routing design exposed by large algebraic inputs derived from geometry IR.

The repair target is **not** a single investigated geometry problem and not any named geometry family. The repair target is the generic R-GDTPK-Q candidate-cover solver core:

```text
well-formed Q-polynomial target system
→ TargetProjectionDAG
→ deterministic declared kernel ladder per block
→ ProjectionMessage(s)
→ global support S(T)
→ exact support verification
→ exact real root isolation
→ decoded candidate cover
```

The repaired implementation must make the solver a **success-route planner**, not a failure-fast planner.

## 1. Non-negotiable interpretation

The solver has one unified top-level pipeline. Multiple local kernels are allowed only as declared, certificate-bound methods for constructing `ProjectionMessage`s inside that unified pipeline.

A local kernel route may be declined for a block when its algebraic cost footprint is infeasible, but that must not be treated as a solve-level failure while other declared target-direct routes remain viable.

```text
Correct:
  Dense TargetRelationSearch is cost-prohibited for this block.
  Continue collecting all other kernel admissions.
  Build a declared ladder containing feasible compact target-direct routes.
  Execute the ladder until a certified ProjectionMessage is produced.

Incorrect:
  Dense TargetRelationSearch is cost-prohibited.
  Return FiniteResourceFailure / AlgorithmicHardCase / CertificateDesignGap for the whole solve.
```

## 2. Concrete investigated cases are forbidden as implementation targets

The repair must not include any concrete investigated geometry problem as a required test, gate, fixture, benchmark, route selector, or expected-output oracle.

Forbidden in production code and tests created by this repair:

```text
- names of the investigated geometry family or problem;
- filenames of the investigated problem builder;
- exact variable IDs from the investigated input;
- exact equation counts or relation hashes from the investigated input as gate constants;
- expected target value or final support polynomial for the investigated input;
- geometry-family dispatch such as circle/triangle/tangent/mixtilinear-style branches;
- any special lowering/normalization inside solver core for a named geometry construction.
```

The repair may use only **general algebraic footprint classes**, such as:

```text
large local block
many local variables
many local relations
small exported separator set
dense total-degree relation search infeasible
compact quotient/action candidate
compact sparse resultant candidate
compact specialization/interpolation candidate
target-independent component
separator-rich algebraic graph
```

## 3. Root defect being repaired

The root defect is not candidate-cover semantics. The defect is planner scalability and route completion:

```text
A dense total-degree TargetRelationSearch route can be infeasible for a large local block.
The planner must recognize this cheaply and must continue toward a compact support-producing route.
```

This is a violation of the algebraic-cost compression goal when left unfixed, because a single dense monomial enumeration can make the pipeline scale with the total local variable count instead of with the best available local algebraic footprint.

## 4. Required repaired behavior

For every block with relations:

1. `collect_kernel_admissions` must complete without materializing infeasible dense monomial supports.
2. Each local kernel admission must be isolated: one cost-prohibited or hard kernel cannot block collection of other admissions.
3. `TargetRelationSearch(DenseTotalDegree)` must have a closed-form preflight and lazy support descriptors.
4. The planner must preserve a declared ladder of feasible target-direct routes.
5. The ladder executor must attempt declared routes in deterministic order and must not stop at the first failing route unless that route succeeds.
6. `UniversalTargetEliminationKernel` must remain a declared generic target/separator projection route for relation-bearing blocks.
7. A relation-bearing block must not become solve-level failure merely because dense `TargetRelationSearch` is cost-prohibited.
8. Success-producing generic algebraic stress cases must return `CertifiedCandidateCover` through `api::solve_target`, not only through helper functions.

## 5. Dense TargetRelationSearch preflight requirements

### GSR-001: closed-form counts

The implementation must provide saturating closed-form estimators for total-degree monomial counts.

```text
monomial_count_leq(n, d) = binomial(n + d, d)
```

The estimator must not allocate monomial vectors.

### GSR-002: preflight structure

Add a machine-readable preflight structure:

```rust
pub struct DenseRelationSearchPreflight {
    pub eliminated_variables: Vec<VariableId>,
    pub exported_variables: Vec<VariableId>,
    pub z_seed: usize,
    pub e_cap: usize,
    pub d_max: usize,
    pub stages: Vec<RelationSearchStageEstimate>,
    pub materialization_allowed: bool,
    pub cost_prohibited_reason: Option<String>,
    pub preflight_hash: Hash,
}

pub struct RelationSearchStageEstimate {
    pub export_degree: usize,
    pub multiplier_total_degree: usize,
    pub estimated_export_cols: SaturatingCount,
    pub estimated_multiplier_cols: SaturatingCount,
    pub estimated_matrix_cols: SaturatingCount,
    pub estimated_row_monomials_upper_bound: SaturatingCount,
    pub stage_cost_class: RouteCostClass,
    pub estimate_hash: Hash,
}
```

`SaturatingCount` must distinguish exact finite counts from saturated counts.

### GSR-003: default caps

Default options must include conservative planner caps for dense total-degree relation search. These caps are not solve-scope restrictions; they are route-scope restrictions.

If user options are `None`, default planner caps must still prevent multi-billion monomial materialization.

### GSR-004: lazy schedule descriptor

Planning must not materialize dense supports unless preflight says the route is feasible under caps.

Preferred design:

```text
DenseRelationSearchSchedule contains support descriptors and hashes of descriptors.
Actual monomial support materialization occurs only in execution, stage by stage, after rechecking caps.
```

Minimum acceptable design:

```text
Admission uses preflight and calls build_dense_relation_search_schedule only when all stages to be materialized are cap-feasible.
```

### GSR-005: no failure-by-preflight

If dense relation search is cost-prohibited for a block, the admission for that kernel is declined with a trace. The block and solve continue.

## 6. Success-route planner requirements

### GSR-006: distinguish admission from support-producing feasibility

The code and docs must distinguish:

```text
Admission:
  The route is structurally relevant.

Plan:
  The route has declared bounds, cost, certificate route, and resource limits.

Support-producing executable route:
  Execution is expected to produce a ProjectionMessage under the declared plan, or the failure mode is known and does not block other routes.
```

The planner must not use "planned" as a synonym for "will produce support".

### GSR-007: admission isolation

`collect_kernel_admissions` must gather every kernel admission independently.

A route-specific panic, combinatorial estimate overflow, resource-prohibited preflight, or certificate-design gap must not prevent collection of later admissions.

The output must contain a diagnostic admission record for every kernel kind.

### GSR-008: ladder completeness

For every relation-bearing block, the declared ladder must contain at least one support-producing candidate route or the `UniversalTargetEliminationKernel`.

If every route is rejected before execution, this is a planner bug unless the block is structurally relationless.

### GSR-009: ladder execution isolation

`execute_block_with_declared_ladder` must attempt every declared plan in deterministic order until one returns a verified `ProjectionMessage`.

If a route fails, its failure is recorded and execution proceeds to the next route.

Only after the entire declared ladder fails may the block return failure. That failure must include all per-route failure records and algebraic cost footprints.

### GSR-010: no hidden fallback

A route tried at execution must appear in the declared ladder. A route not in the ladder must not be called as fallback.

## 7. UniversalTargetElimination requirements

### GSR-011: generic large-block responsibility

UniversalTargetElimination is not a placeholder and not a failure label. It is the final declared generic target/separator projection kernel.

For large blocks where dense TargetRelationSearch is cost-prohibited, Universal must still attempt compact declared internal strategies.

### GSR-012: Universal internal strategy sequence

Universal’s plan must include a declared internal strategy sequence. At minimum:

```text
1. TargetRelationSearch with preflight; skip dense materialization if cost-prohibited.
2. Sparse/resultant eliminant if sparse footprint admits a bounded template.
3. TargetActionKrylov if finite target-relevant quotient/action is certifiable.
4. SpecializationInterpolation if separators/exported variables make it viable.
5. RegularChain/NormTrace if algebraically detected.
6. Bounded local elimination to exported variables only.
```

The exact internal order may be cost-based, but it must be deterministic and recorded.

### GSR-013: Universal must not return nonfinite from exhaustion

Universal stage exhaustion, no relation found, dense route cost prohibition, or compact route failure must not become `CertifiedNonFiniteTargetImage` without a positive nonfinite certificate.

### GSR-014: Universal output discipline

Universal output may contain only target/separator relations in the block’s exported variables.

No coordinate roots, coordinate solution lists, full coordinate RUR, or geometry-specific formulas are allowed.

## 8. DAG / decomposition repair requirements

### GSR-015: cost-aware decomposition audit

If a decomposition produces a large relation-heavy block while the weighted graph has algebraic separator candidates, the decomposition must be refined.

The repair must add algebraic footprint metrics to guide decomposition:

```text
relation arity
relation total degree
relation monomial count
coefficient height
target distance
affine/definitional eliminability
separator candidate score
estimated local projection cost
estimated dense relation-search cost
estimated quotient/action rank
```

### GSR-016: no geometry-name normalization

DAG/decomposition repair must not use geometry names or coordinate-gauge fixes inside solver core.

Algebraic preprocessing may use only algebraic form, e.g., affine definitions, explicit nonzero guards, independent components, sparse support, incidence graph structure.

### GSR-017: relation assignment and duplication discipline

Relation assignment must preserve authorization and certificates. If duplicating relations across blocks is necessary to create useful separators, duplication certificates must be generated and replay-bound.

## 9. Generic algebraic acceptance requirements

This repair must not include the investigated concrete problem as a test. Instead, it must include generated algebraic stress families that encode the same **general failure mode**:

```text
large local footprint
dense total-degree TargetRelationSearch infeasible
at least one compact target-direct route exists
candidate-cover success required
```

Support-producing generic tests must pass through `api::solve_target`.

Failure status is not acceptable for support-producing generic stress.

## 10. Review archive requirements

Every phase must have:

```text
prompt.md
response.md
review_summary.yaml
evidence_manifest.yaml
```

A reviewer PASS is invalid if it does not explicitly inspect the files/functions named in the phase prompt.

Reviewer must reject:

```text
- tests using investigated problem names or builders;
- route-specific early failure accepted as success;
- dense TRS preflight without support-producing alternative route;
- planned/admitted route treated as execution success;
- Universal that merely returns hardcase for large blocks;
- hidden fallback;
- geometry-family dispatch;
- expected-answer or fixture dispatch.
```

## 11. Allowed final claim after this repair

After all phases pass:

```text
GENERIC_SUCCESS_ROUTE_PLANNER_READY
```

This is a planner/kernels routing readiness claim. It does not by itself claim full benchmark readiness.

It may be combined with existing:

```text
CANDIDATE_COVER_CORE_READY
SOURCE_FAITHFUL_TO_V4_CANDIDATE_COVER_LAYER
```

only if the final generic success-route tests prove support-producing candidate-cover successes through the public pipeline.

## 12. Forbidden final claim from this repair alone

```text
EXACT_IMAGE_CORE_READY
SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC
RGDTPK_Q_V4_ACCEPTANCE_COMPLETE
BENCHMARK_SUPERIORITY
```


---

# FILE: GENERIC_SUCCESS_ROUTE_PLAN.md

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


---

# FILE: GENERIC_SUCCESS_ROUTE_REVIEWER_PROMPTS.md

# Generic Success-Route Core Repair Reviewer Prompts v2

## Reviewer Meta-Protocol

For every phase, the reviewer must answer:

```text
1. Did the implementation add or use a concrete investigated problem as a test, gate, fixture, benchmark, route selector, or expected answer?
2. Did it add geometry-family dispatch?
3. Did it turn timeout into faster failure instead of support-producing success on generic algebraic stress?
4. Did it confuse admission/planning with execution success?
5. Did it leave a path where dense TargetRelationSearch materializes huge monomial supports during admission?
6. Did it preserve a single unified TargetProjectionDAG pipeline?
7. Did it preserve declared ladders and avoid hidden fallback?
8. Did it verify final support exactly and replay-bind messages/support/roots/candidates?
```

If any answer is bad, the phase must be FAIL.

A reviewer PASS is invalid if it only says tests passed. The reviewer must inspect the named source files and cite exact functions.

---

## GSR-REVIEW-P0

Review `GENERIC_SUCCESS_ROUTE_AGENT_RESET.md` and `GENERIC_SUCCESS_ROUTE_CURRENT_AUDIT.md`.

Reject if:
- the concrete investigated geometry problem or family name appears as a required regression;
- exact variable/relation counts from the investigated case are used as gate constants;
- the agent frames the repair as fast failure;
- the audit omits any required planner/DAG/kernel file.

Required response sections:
```yaml
phase: GSR-P0
status: PASS|FAIL
concrete_case_overfit_found: true|false
fast_failure_framing_found: true|false
missing_files: []
blocking_findings: []
required_fixes: []
```

---

## GSR-REVIEW-P1

Review dense TRS preflight and lazy schedule.

Must inspect:
```text
planner/relation_schedule.rs
solver/options.rs
planner/admission.rs
kernels/target_relation_search.rs
```

Reject if:
- `build_dense_relation_search_schedule` can be called in admission before preflight passes;
- preflight allocates monomial vectors;
- no default planner caps exist;
- execution does not recheck caps before materializing supports;
- tests use the investigated concrete problem;
- estimated counts can overflow silently.

Required adversarial check:
Explain how the code handles a hypothetical 30+ variable, degree >= 7 local block without materializing monomial supports.

---

## GSR-REVIEW-P2

Review admission isolation.

Must inspect:
```text
planner/admission.rs
planner/kernel_plan.rs
planner/planner.rs
result/cost_trace.rs
result/diagnostics.rs
```

Reject if:
- one route-local failure prevents later kernel admissions;
- admissions vector does not contain records for all kernel kinds;
- cost-prohibited route is converted into solve-level failure during planning;
- Universal is missing for relation-bearing blocks;
- route diagnostics are not machine-readable.

Required adversarial check:
Describe the behavior when TargetRelationSearch is cost-prohibited but TargetActionKrylov and Universal are admissible.

---

## GSR-REVIEW-P3

Review declared ladder and execution isolation.

Must inspect:
```text
planner/ladder.rs
planner/kernel_plan.rs
planner/planner.rs
solver/pipeline.rs
kernels/traits.rs
```

Reject if:
- the ladder can be empty for relation-bearing blocks;
- Universal is not last/present for relation-bearing blocks;
- `execute_block_with_declared_ladder` stops after the first failed route;
- failure traces from failed routes are lost;
- any executed route was not declared in the ladder;
- a "planned" route is counted as success without executing and verifying a `ProjectionMessage`.

Required adversarial check:
Describe a generated case where route 1 fails and route 2 succeeds. Verify code supports it.

---

## GSR-REVIEW-P4

Review UniversalTargetElimination.

Must inspect:
```text
kernels/universal_elimination.rs
algebra/elimination.rs
kernels/sparse_resultant.rs
kernels/action_krylov.rs
kernels/specialization_interpolation.rs
verify/verify_message.rs
```

Reject if:
- Universal is a placeholder or only returns hardcase for large blocks;
- Universal invokes dense TRS without preflight;
- Universal does not try later strategies after earlier strategy failure;
- Universal returns nonfinite from exhaustion;
- Universal output contains non-exported variables;
- Universal certificate does not record chosen and failed internal strategies.

Required adversarial check:
Explain how Universal handles a large block where dense TRS is cost-prohibited but a compact target-action or sparse route exists.

---

## GSR-REVIEW-P5

Review decomposition repair.

Must inspect:
```text
graph/weighted_primal.rs
graph/separators.rs
graph/tree_decomposition.rs
graph/projection_dag.rs
graph/metrics.rs
```

Reject if:
- separator scoring ignores algebraic projection cost;
- large relation-heavy blocks are not penalized;
- geometry role/name affects decomposition;
- relation assignment loses authorization;
- relation duplication, if present, lacks certificate;
- no-useful-separator case is treated as unsupported.

Required adversarial check:
Provide a generic algebraic graph where separator-rich structure should split. Explain what the implementation does.

---

## GSR-REVIEW-P6

Review generic success-route stress suite.

Must inspect:
```text
tests/generic_success_route_planner.rs
planner/*
kernels/*
solver/*
compose/*
verify/*
```

Reject if:
- tests include the concrete investigated problem or family name;
- stress is helper-only and does not call `api::solve_target`;
- support-producing cases return failure statuses;
- dense TRS is not cost-prohibited in large-footprint cases;
- successful route is geometry-specific or expected-answer based;
- replay is not checked;
- cost trace / route trace is absent.

Required response must list every generated stress family:
```yaml
families:
  - id:
    generated_parameters:
    dense_trs_status:
    successful_kernel:
    public_status:
    replay_accepted:
    route_trace_present:
```

---

## GSR-REVIEW-P7

Final anti-overfit and closure review.

Must inspect:
```text
GENERIC_SUCCESS_ROUTE_CLOSURE.md
GENERIC_SUCCESS_ROUTE_STATIC_SCAN.md
GENERIC_SUCCESS_ROUTE_ACCEPTANCE_RESULTS.md
all modified production source files
all new tests
```

Reject if:
- closure claims benchmark superiority;
- closure claims exact-image completion;
- closure claims full v4 source-fidelity beyond candidate-cover/routing readiness;
- concrete investigated case appears in tests or production dispatch;
- static scan hits are unclassified;
- failures are accepted as completion for support-producing generic stress;
- reviewer cannot trace a generic large-footprint input through public pipeline to `CertifiedCandidateCover`.

Required final answer:
```yaml
phase: GSR-P7
status: PASS|FAIL
allowed_claims:
  - GENERIC_SUCCESS_ROUTE_PLANNER_READY
  - CANDIDATE_COVER_CORE_READY_if_existing_candidate_cover_closure_remains_valid
forbidden_claims:
  - EXACT_IMAGE_CORE_READY
  - RGDTPK_Q_V4_ACCEPTANCE_COMPLETE
blocking_findings: []
required_fixes: []
```


---

# FILE: GENERIC_PLANNER_ACCEPTANCE_MATRIX.yaml

schema_version: generic-success-route-v2
claim_target:
  allowed_after_all_phases:
    - GENERIC_SUCCESS_ROUTE_PLANNER_READY
  compatible_existing_claims:
    - CANDIDATE_COVER_CORE_READY
    - SOURCE_FAITHFUL_TO_V4_CANDIDATE_COVER_LAYER
  forbidden_from_this_repair_alone:
    - EXACT_IMAGE_CORE_READY
    - SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC
    - RGDTPK_Q_V4_ACCEPTANCE_COMPLETE
    - BENCHMARK_SUPERIORITY

hard_prohibitions:
  no_concrete_investigated_problem_tests: true
  no_geometry_family_dispatch: true
  no_expected_answer_dispatch: true
  no_fast_failure_as_success: true
  no_hidden_fallback: true
  no_coordinate_solution_enumeration: true

required_generic_success_families:
  - id: G1
    name: large_footprint_compact_quotient_action
    public_path: api::solve_target
    dense_target_relation_search: cost_prohibited
    required_success: CertifiedCandidateCover
    required_success_route_one_of:
      - TargetActionKrylov
      - UniversalTargetElimination_internal_TargetActionKrylov
    required_evidence:
      - support_polynomial
      - exact_Q_support_verification
      - projection_messages
      - replay_accepts
      - route_trace
      - cost_trace
  - id: G2
    name: large_footprint_sparse_or_specialization_success
    public_path: api::solve_target
    dense_target_relation_search: cost_prohibited
    required_success: CertifiedCandidateCover
    required_success_route_one_of:
      - SparseResultantProjection
      - SpecializationInterpolation
      - UniversalTargetElimination_internal_SparseOrSpecialization
    required_evidence:
      - support_polynomial
      - exact_Q_support_verification
      - replay_accepts
  - id: G3
    name: separator_rich_composition
    public_path: api::solve_target
    required_success: CertifiedCandidateCover
    required_evidence:
      - projection_messages_at_least_2
      - deleting_message_changes_support_or_fails_replay
      - exact_Q_support_verification
  - id: G4
    name: universal_later_strategy_success
    public_path: api::solve_target
    required_success: CertifiedCandidateCover
    required_evidence:
      - first_declared_route_fails
      - later_declared_route_succeeds
      - UniversalTargetElimination_records_internal_strategy_trace
      - replay_accepts
  - id: G5
    name: no_useful_separator_one_large_block_success
    public_path: api::solve_target
    required_success: CertifiedCandidateCover
    required_evidence:
      - one_large_block
      - Universal_present_in_ladder
      - dense_TRS_not_materialized_if_cost_prohibited
      - certified_projection_message

review_required_fields:
  every_phase:
    - prompt.md
    - response.md
    - review_summary.yaml
    - evidence_manifest.yaml
  final_review_must_include:
    - static_scan_no_concrete_case
    - static_scan_no_geometry_dispatch
    - static_scan_no_expected_answer
    - dense_trs_materialization_audit
    - success_route_trace_audit
    - universal_strategy_trace_audit


---

# FILE: GENERIC_SUCCESS_ROUTE_QUICK_GUARDIAN_PROMPT.md

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


---

# FILE: GENERIC_SUCCESS_ROUTE_CURRENT_DEFECT_AUDIT.md

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
