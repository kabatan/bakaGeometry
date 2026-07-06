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
