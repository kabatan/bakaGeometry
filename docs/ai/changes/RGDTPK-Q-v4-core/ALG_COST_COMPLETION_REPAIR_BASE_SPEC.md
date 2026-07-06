# Algebraic-Cost Completion Repair Base Spec v1

Change ID: `RGDTPK-Q-v4-algebraic-cost-completion-repair-v1`

## 0. Purpose

This amendment reopens the candidate-cover readiness claim until the implementation is made faithful to the v4 algorithmic-cost-compression requirements.

The previous implementation may contain a correct candidate-cover semantic pipeline, but it is not complete as an R-GDTPK-Q / ACCTP-Q solver core if a production route can monopolize the declared ladder with unbounded algebraic object growth.

This repair is not a local timeout patch. It is a source-spec completion repair for the algebraic-cost layer.

The final target is:

```text
Given a well-formed Q-polynomial target system F ⊂ Q[x1,...,xn,T],
produce a verified finite candidate cover S(T) without full coordinate roots,
and do so through algebraic-cost-compressed target/separator projection.
```

Spurious roots remain allowed in candidate-cover mode:

```text
required: true target values ⊆ roots(S)
not required: roots(S) ⊆ true target values
```

## 1. Source authority

The supplied v4 specification remains the source of truth.

This amendment makes explicit a requirement already present in v4:

```text
The solver's speed claim is not merely that target values are fewer than coordinate solutions.
The solver must avoid constructing huge full-coordinate algebraic objects by using
TargetProjectionDAG, local projection messages, separator width, quotient/action rank,
sparse template size, and support degree.
```

A kernel name, certificate type, planner entry, or acceptance result is not sufficient. A production kernel must be bounded by the algebraic costs that actually dominate that kernel.

## 2. Claim reset

Until every phase in `ALG_COST_COMPLETION_REPAIR_PLAN.md` passes, the repository must not claim:

```text
CANDIDATE_COVER_CORE_READY
SOURCE_FAITHFUL_TO_V4_CANDIDATE_COVER_LAYER
SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC
RGDTPK_Q_V4_ACCEPTANCE_COMPLETE
```

During this repair, the maximum allowed claim is:

```text
CANDIDATE_COVER_PIPELINE_PRESENT_BUT_ALGEBRAIC_COST_INCOMPLETE
```

The repair is complete only when the implementation can pass generic large-footprint algebraic support-producing stress without using geometry names, expected answers, or fixed problem fixtures.

## 3. Non-negotiable requirements

### ALGC-001: Algebraic-cost completeness is part of correctness for this core

A candidate-cover solver that is semantically correct only on small cases but allows a production route to expand unbounded symbolic objects is not source-faithful to v4.

A route must have declared bounds for its dominant algebraic objects:

```text
TargetRelationSearch:
  export support size, multiplier support size, row monomial count, matrix rows/cols, coefficient height

SparseResultantProjection:
  selected pair term counts, keep-variable count, total degree, Sylvester/Macaulay size,
  predicted determinant work, predicted output term growth, coefficient-height growth,
  chain length, intermediate relation sizes

TargetActionKrylov:
  quotient basis size, quotient rank, action matrix dimensions, coverage proof degree,
  basis construction work, normal-form certificate cost

SpecializationInterpolation:
  separator count, sample count, inner target-only cost, interpolation support size,
  verification cost

RegularChain / NormTrace:
  component count, tower degree, projected generator size, guard/component certificate cost

UniversalTargetElimination:
  all internal strategy costs, maximum local elimination steps, output generator size,
  resource-bound trace
```

### ALGC-002: No unbounded production route

Every declared route in a production ladder must be bounded. It must either:

```text
1. produce a verified ProjectionMessage within its declared bounds; or
2. return a route-local allowed failure before exceeding its declared algebraic work budget.
```

A route must not consume the user's wall-clock budget while preventing later declared routes from running.

### ALGC-003: SparseResultantProjection must be expression-swell aware

`SparseResultantProjection` must not admit or prioritize a route solely because a small finite Sylvester/resultant template exists.

It must estimate and bind:

```text
- pair input term counts
- degree in eliminated variable
- keep-variable count
- determinant / resultant backend work units
- predicted output term count
- predicted coefficient-height growth
- intermediate relation count and term growth
- chain step count
```

A 3x3 resultant with thousands of terms in the matrix entries is not a cheap route.

### ALGC-004: Recursive symbolic determinant is not a general production backend

Recursive determinant expansion over polynomial entries is allowed only under strict small-entry caps.

For production sparse resultant beyond those caps, implementation must use one of:

```text
- modular resultant with rational reconstruction and exact Q verification
- evaluation-interpolation resultant with exact Q verification
- subresultant / PRS-style resultant with exact Q verification
- sparse/Macaulay null-relation computation with exact Q verification
```

The backend may generate candidates using modular or evaluation methods, but correctness must be exact over Q.

### ALGC-005: Dense TargetRelationSearch remains bounded and sparse-aware

Dense total-degree `TargetRelationSearch` is allowed only when closed-form preflight proves that all materialized supports and matrices are within caps.

For large blocks, the solver must prefer sparse/lazy/footprint relation-search strategies rather than treating dense total-degree search as the central route.

The following functions and any equivalents must never enumerate dense support before feasibility passes:

```text
estimate_dense_relation_search_schedule
hash_dense_relation_search_preflight
dense_relation_search_decline_reason
admit_target_relation_search
Universal::TargetRelationSearchEscalated
```

### ALGC-006: Planner must rank by real dominant cost, not kernel name

`build_declared_ladder` and `estimate_kernel_cost` must include route-specific dominant-cost estimates.

Forbidden:

```text
SparseResultantProjection appears before other routes only because its matrix has few rows/cols
while expression-swell risk is huge.
```

Required:

```text
dangerous SparseResultantProjection -> CostProhibited or placed after safer routes
small quotient/action route -> preferred when quotient rank and coverage are feasible
specialization/interpolation route -> preferred when it has bounded samples and exact verification
Universal route -> final generic route but with bounded internal stages
```

### ALGC-007: Declared ladder is bounded serial, not unbounded serial

The declared ladder may remain serial, but each route must have an enforceable route budget.

If a route exceeds a declared budget, execution must return an allowed route-local failure and continue to later declared routes.

This is not hidden fallback because the ladder is declared in the plan and certificate.

### ALGC-008: UniversalTargetElimination must be a real success route

`UniversalTargetElimination` is not a hardcase-returning placeholder and not a hidden full-coordinate fallback.

It must:

```text
- plan its internal strategy sequence before execution;
- apply the same dense/sparse/resultant/action/interpolation work estimates as top-level kernels;
- skip cost-prohibited stages;
- execute remaining bounded stages;
- export only target/separator relations;
- verify every exported relation over Q;
- record failed strategy hashes and dominant-cost diagnostics.
```

### ALGC-009: Graph/DAG decomposition must be algebraic-cost aware

The graph layer must not routinely leave one huge hard block when algebraic separators are available.

Separator scoring must include:

```text
- relation arity
- total degree
- monomial count
- coefficient-height estimate
- predicted local projection cost
- linear/definitional eliminability
- target distance
- separator width penalty
```

A large one-block result is allowed only when no cost-improving separator is found, and the evidence must explain why.

### ALGC-010: Planned is not support-producing

A kernel admission or plan object is not proof that the kernel can produce a `ProjectionMessage`.

Every acceptance stress must distinguish:

```text
admitted
planned
executed
verified ProjectionMessage produced
composed into S(T)
verified global support
decoded candidates
```

PASS requires the last five for support-producing cases.

### ALGC-011: No case-specific repair

The implementation and tests for this repair must not mention or depend on:

```text
mixtilinear
mixtilinear_candidate_cover_problem
triangle / circle / tangent family names
specific external diagnostic problem file names
specific variable IDs from a diagnostic input
specific relation IDs from a diagnostic input
known final support polynomial or expected cosine value
problem hash from the diagnostic input
```

The repair must use algebraic footprint stress only.

### ALGC-012: Generic large-footprint support-producing stress is mandatory

The repair must include generated or hand-written algebraic stress families that do not encode the diagnostic problem, but do encode the general failure patterns:

```text
- large block where dense TRS is cost-prohibited
- sparse resultant has dangerous expression-swell risk and is not first
- another compact route can produce candidate-cover success
- route budget forces a dangerous route to yield to the next route
- graph decomposition reduces a high-cost block using algebraic separator scoring
- Universal executes bounded internal stages and emits a verified message
```

### ALGC-013: Reviewers must reject previous false-PASS modes

A reviewer must fail the phase if any of the following is true:

```text
- a production route has no dominant-cost bound;
- SparseResultant uses recursive symbolic determinant beyond small-entry caps;
- a route can monopolize ladder execution without returning;
- a stress only proves fast failure, not candidate-cover success;
- tests use the diagnostic problem or its details;
- support-producing success relies on geometry names or expected answers;
- admission/planning is treated as equivalent to execution success;
- large-footprint stress does not exercise public or near-public pipeline;
- Universal only aggregates failures and never emits a verified exported relation in the stress suite.
```
