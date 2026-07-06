# Generic Planner Success-Route Repair Base Spec v1
Change ID: `RGDTPK-Q-v4-generic-planner-success-route-v1`

## 0. Purpose

This repair fixes a generic planner scalability and success-routing defect. It must not add a test, branch, fixture, string match, or heuristic for the previously investigated geometry problem.

The required design outcome is:

```text
The unified candidate-cover pipeline must never allow one infeasible dense TargetRelationSearch admission to block planning of other target-direct kernels.
For large algebraic blocks, the planner must choose algebraic-cost-compressed success routes from generic footprint evidence,
not by recognizing a named geometry problem and not by returning a failure status early.
```

This repair is not allowed to include the reported geometry problem as a gate, regression, fixture, benchmark, or acceptance item. If the repair is correct, that problem should improve as a consequence of the generic planning design, but the plan must not optimize toward it.

## 1. Source authority

The v4 algorithm spec remains authoritative. The solver has a single pipeline:

```text
ValidateInput
CanonicalizeSystem
PreKernelAlgebraicCompression
BuildRelationVariableHypergraph
BuildTargetInfluenceGraph
BuildWeightedProjectionGraph
BuildTargetProjectionDAG
PlanProjectionMessages
ExecuteLocalProjectionKernels
ComposeProjectionMessages
BuildGlobalSupportPolynomial
VerifyGlobalSupport
SquarefreeSupport
ExactRealRootIsolation
DecodeTargetCandidates
FinalizeResultAndCertificate
```

Multiple kernels are not multiple solvers. They are declared local projection methods inside `PlanProjectionMessages` / `ExecuteLocalProjectionKernels`.

## 2. Forbidden case-specific content

The patch, tests, comments, docs, and review artifacts for this repair must not contain or depend on:

```text
mixtilinear
mixtilinear_candidate_cover_problem
circle tangent problem names
triangle/circle/tangent geometry-family branching
specific variable IDs from the investigated problem
specific equation counts from the investigated problem
specific polynomial hashes from the investigated problem
expected cos value
known final support polynomial for that problem
```

The diagnostic report may be cited in human review as evidence of the failure class, but its concrete input must not become an implementation target.

## 3. Required semantic boundary

This repair targets candidate-cover performance and planning. It must preserve:

```text
true finite target values ⊆ roots(S)
spurious roots are allowed
exact-image filtering is not part of candidate-cover mode
```

It must not modify exact-image semantics to make candidate-cover pass.

## 4. Non-negotiable requirements

### GPSR-001: Unified pipeline only

Do not add separate solvers, geometry-specific handlers, or problem-shape dispatch. The only allowed path is the existing generic algebraic pipeline and declared projection-kernel ladder.

### GPSR-002: Admission isolation

`collect_kernel_admissions` must be isolation-safe:

```text
One kernel admission must not be able to hang, allocate huge monomial vectors, or prevent other kernel admissions from being collected.
```

Every admission must be bounded by cheap preflight, symbolic descriptors, or explicit cost caps. Expensive support materialization during admission is forbidden.

### GPSR-003: Dense TargetRelationSearch preflight

Before materializing any dense monomial support, the planner must compute closed-form estimates:

```text
export_cols = #monomials(|Z|, export_degree)
multiplier_cols_i = #monomials(|Y∪Z|, multiplier_degree_i)
total_multiplier_cols = Σ_i multiplier_cols_i
estimated_matrix_cols = export_cols + total_multiplier_cols
estimated_row_monomials_upper_bound
estimated_memory_bytes_upper_bound
```

The monomial count must use saturating exact integer arithmetic. It must not enumerate monomials to estimate feasibility.

### GPSR-004: Dense route infeasibility is local to that route

If dense TargetRelationSearch is infeasible, only that dense route becomes non-executable for that block. The block must continue planning with other projection kernels.

Forbidden behavior:

```text
returning solve-level failure because dense TargetRelationSearch is too large
stopping collect_kernel_admissions after dense TargetRelationSearch is infeasible
omitting UniversalTargetElimination because dense TargetRelationSearch is infeasible
```

### GPSR-005: Success-route planner policy

The planner must form a declared ladder from feasible routes. It must prefer compact certified routes over infeasible dense total-degree relation search.

The planner must use algebraic footprint only:

```text
local variable count
exported variable count
relation count
monomial count
maximum degree
coefficient height
separator width
sparse template estimate
quotient/action rank estimate
specialization/interpolation template estimate
regular-chain/tower detectability
dense TRS preflight estimates
```

### GPSR-006: UniversalTargetElimination uses the same safeguards

UniversalTargetElimination must not internally invoke dense TargetRelationSearch without preflight. If an internal dense escalation is infeasible, Universal must proceed to its other declared local target/separator strategies.

### GPSR-007: No failure-as-success gates

This repair must not be accepted merely because planning returns quickly with a failure status. Generic support-producing stress cases must still return `CertifiedCandidateCover`.

However, this plan must not include the investigated geometry problem as such a stress case. Use abstract algebraic stress generators instead.

### GPSR-008: Generic stress, not case regression

Acceptance tests must use algebraically generated families, not the investigated problem. They must include randomized or parameterized variable renaming and rational scaling so that test success cannot depend on exact IDs or fixed polynomial hashes.

Required stress families:

```text
1. Large dense-TRS-infeasible block with an alternative compact quotient/action route.
2. Large dense-TRS-infeasible block with a sparse/resultant or specialization route.
3. Large dense-TRS-infeasible block where Universal must remain in the declared ladder.
4. Admission-isolation test proving all kernels after TargetRelationSearch are still collected.
5. Descriptor-only planning test proving dense supports are not materialized in admission.
```

These are generic algebraic footprint tests. They must not use geometry names or the investigated problem file.

### GPSR-009: Lazy support descriptors

Dense relation-search planning should represent supports using descriptors until execution.

Required descriptor idea:

```rust
enum SupportDescriptor {
    DenseTotalDegree { variables: Vec<VariableId>, degree: usize, estimated_count: SaturatingCount },
    SparseFootprint { variables: Vec<VariableId>, support_hash: Hash, estimated_count: SaturatingCount },
}
```

A descriptor hash may be used in plan hashes. Full monomial vectors may be materialized only after caps pass and only in execution or bounded schedule construction.

### GPSR-010: Default planning caps

Default options must include safe planning caps for dense TargetRelationSearch. `None` must no longer mean “unbounded dense support allocation during planning.”

Explicit user overrides may raise caps, but default public `solve_target` must be safe.

### GPSR-011: Cost trace and diagnostics

When dense TargetRelationSearch is cost-prohibited, the run must record a trace/diagnostic such as:

```text
kernel = TargetRelationSearch
route = DenseTotalDegree
estimated_matrix_cols = ...
estimated_rows = ...
cap = ...
decision = CostProhibitedDenseRoute
```

This trace is diagnostic only. It must not become a solve-level failure if another route succeeds.

### GPSR-012: Review standard

Reviewer must fail the repair if:

```text
- the investigated geometry problem appears in tests/gates/comments as a target fixture;
- dense TargetRelationSearch still materializes monomials during admission;
- one infeasible admission can block other kernel admissions;
- the planner returns fast failure instead of building a feasible declared ladder in generic stress cases;
- Universal internally calls dense TargetRelationSearch without preflight;
- the patch introduces geometry/problem/expected-answer dispatch;
- support-producing generic stress passes only through exact-image filtering.
```
