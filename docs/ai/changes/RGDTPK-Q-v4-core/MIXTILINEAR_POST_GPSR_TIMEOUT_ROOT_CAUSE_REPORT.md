# Mixtilinear Post-GPSR Timeout Root-Cause Report

Status: diagnostic report.

Purpose: identify why the supplied `mixtilinear_candidate_cover_problem.rs` still fails to return
candidate-cover output within a 10 minute bound after the Generic Planner Success-Route repair.

Authority: evidence and engineering diagnosis only. This report does not amend the Base Spec, Plan,
claim ceiling, acceptance matrix, or any readiness status.

Claim boundary: this report does not claim exact-image completion, full v4 source fidelity, full
acceptance completion, benchmark readiness, or that the supplied problem is an acceptance fixture.
The supplied problem was used only as a temporary external diagnostic input requested by the user.

## Executive Summary

The current timeout is no longer caused by dense `TargetRelationSearch` materialization during
planning. The GPSR preflight now declines that dense route quickly and records a structured
`CostProhibitedDenseRoute` diagnostic.

The current evidence identifies the first executable route selected for the large block as the
route that enters the expensive diagnostic computation:

```text
BlockId(5) -> SparseResultantProjection
```

Planning completes in about 1.5 seconds. The declared first route for the large 33-variable,
29-relation block is `SparseResultantProjection`. The sparse-resultant micro-probe then follows the
same public resultant API used by that route and shows exact Sylvester resultants running without an
intermediate term-count, degree, keep-variable, coefficient-growth, or elapsed-time cap.
Intermediate relations blow up rapidly:

```text
eliminate VariableId(6):
  input pair terms = 28 and 1176
  Sylvester matrix = 4x4
  compute time = 87.612s
  output = degree 40, 628,925 terms, 16 variables

eliminate VariableId(25):
  input pair terms = 1,588 and 16,662
  Sylvester matrix = 3x3
  compute started at T=225.976s
  no corresponding compute_ok appears later in the captured stdout log
```

The supported root-cause diagnosis is therefore:

```text
The first declared large-block route is SparseResultantProjection. Its exact resultant subroutine
exhibits severe intermediate polynomial expression swell, and the route has no per-route work
budget. The captured diagnostic run reaches a 3x3 resultant after expression swell and does not
emit a completion line for that computation.
```

## Diagnostic Method

A temporary integration probe was created and deleted after investigation. It imported the supplied
problem file by path, then ran the public pipeline stages with timing:

```text
step_validate
step_canonicalize
step_compress
step_build_graphs
step_build_dag
step_plan
manual per-block execution probe
sparse-resultant micro-probe using public resultant APIs
```

The temporary probe was not added as a permanent test, fixture, benchmark, or acceptance gate.

The 600 second probe was run by launching the compiled test binary directly and redirecting stdout
to:

```text
geosolver-core\target\timeout_probe_600_stdout.log
```

That log is a temporary local evidence file, not a tracked artifact.

## Input and DAG Evidence

The supplied input file states that it directly lowers the geometry equations and does not normalize
the triangle:

```text
C:\Users\bakat\Downloads\mixtilinear_candidate_cover_problem.rs:5
```

It builds the target problem at:

```text
C:\Users\bakat\Downloads\mixtilinear_candidate_cover_problem.rs:199
C:\Users\bakat\Downloads\mixtilinear_candidate_cover_problem.rs:448
```

Observed input and compressed sizes:

```text
T=0.008s built input vars=52 equations=48 semantics=17
T=1.123s compressed variables=38 relations=34 diagnostics=0
```

Observed projection DAG:

```text
T=1.377s built dag blocks=11 root=BlockId(0)
BlockId(5): local_vars=33 relations=29 exported=2 children=0
BlockId(6)..BlockId(10): local_vars=3 relations=1 exported=2 children=0
```

The hard part is still `BlockId(5)`, which contains most variables and relations.

## Planning Evidence

Planning completed quickly:

```text
T=1.517s planned blocks=6
```

The declared ladder for the large block was:

```text
T=1.518s plan block=BlockId(5)
selected=SparseResultantProjection
ladder=SparseResultantProjection -> TargetActionKrylov -> RegularChainProjection ->
       SpecializationInterpolation -> UniversalTargetElimination
```

Dense `TargetRelationSearch` was not the active timeout path. It was declined by the new preflight:

```text
TargetRelationSearch status=Declined
CostProhibitedDenseRoute
first_export_degree=4
estimated_matrix_cols=5924738457
estimated_rows=44459430895
estimated_memory_bytes=6449173677056
matrix_col_cap=65536
matrix_row_cap=65536
memory_cap_bytes=268435456
stage_count=44
```

This proves the old planning-time dense monomial enumeration defect was not the current stopping
point.

## Execution Route Evidence

The 600 second stdout log directly records the large-block plan:

```text
T=1.517s planned blocks=6
T=1.518s plan block=BlockId(5)
selected=SparseResultantProjection
```

The pipeline executes the declared ladder serially. `step_execute` dispatches
`execute_block_with_declared_ladder`, which iterates over `plan.declared_ladder` and returns only
after a kernel returns a message, returns an allowed error, or returns a fatal error:

```text
geosolver-core/src/solver/pipeline.rs:120
geosolver-core/src/solver/pipeline.rs:156
geosolver-core/src/solver/pipeline.rs:500
```

Because the selected first route is `SparseResultantProjection`, and because the sparse-resultant
micro-probe demonstrates the same route's exact-resultant subroutine entering a severe expression
swell step on the same block data, the evidence identifies this route as the supported timeout
cause. The code path also shows that a serial first route with no work budget can delay observation
of later fallback kernels.

## Sparse-Resultant Micro-Probe Evidence

The micro-probe replicated the sparse-resultant chain on `BlockId(5)` using the same public
resultant API and the same pair-selection rule shape.

Initial sparse-resultant block profile:

```text
T=1.527s sparse_micro block=BlockId(5)
current_relations=29
eliminated=31
exported=2
max_dim=104
```

Selected intermediate steps:

```text
VariableId(1):
  pair terms=(28, 25), matrix=2x2
  output degree=10, terms=158, elapsed=0.007s

VariableId(2):
  pair terms=(9, 158), matrix=2x2
  output degree=13, terms=589, elapsed=0.019s

VariableId(5):
  pair terms=(34, 26), matrix=3x3
  output degree=16, terms=1176, elapsed=0.068s

VariableId(6):
  pair terms=(28, 1176), matrix=4x4
  output degree=40, terms=628925, vars=16, elapsed=87.612s
```

After the 628,925-term intermediate relation appeared, even pair selection became expensive because
the selection logic repeatedly scans polynomial terms to compute variable degrees. Examples:

```text
VariableId(18) compute_ok at T=98.677s
VariableId(19) choose at T=127.164s
selection gap ~= 28.487s

VariableId(19) compute_ok at T=127.167s
VariableId(20) choose at T=152.315s
selection gap ~= 25.148s

VariableId(20) compute_ok at T=152.319s
VariableId(21) choose at T=173.825s
selection gap ~= 21.506s
```

The decisive diagnostic evidence is the later resultant step:

```text
T=225.897s sparse_micro choose eliminate=VariableId(25)
current_relations=14
pair=(12, 13)
elim_degrees=(2, 1)
total_degrees=(15, 22)
terms=(1588, 16662)
keep_vars=24
matrix=3x3

T=225.976s sparse_micro compute_start eliminate=VariableId(25) pair=(12, 13) matrix=3x3
```

The same captured stdout log contains no later `compute_ok` for `VariableId(25)`. The last flushed
line for this computation is the `compute_start` line above. This supports the diagnosis that the
sparse-resultant chain is spending the remaining observed runtime inside that resultant step.

## Code-Path Evidence

`SparseResultantProjection` execution calls `build_sparse_resultant_trace` before producing a
projection message:

```text
geosolver-core/src/kernels/sparse_resultant.rs:245
geosolver-core/src/kernels/sparse_resultant.rs:261
```

Inside the trace loop, each selected pair is computed as an exact resultant and then immediately
verified:

```text
geosolver-core/src/kernels/sparse_resultant.rs:395
geosolver-core/src/kernels/sparse_resultant.rs:416
geosolver-core/src/kernels/sparse_resultant.rs:417
```

`verify_resultant_certificate` recomputes the resultant:

```text
geosolver-core/src/algebra/resultant.rs:193
```

So the actual kernel is at least as expensive as the micro-probe. The micro-probe measured only
`compute_resultant_relation`; production execution also performs exact certificate recomputation.

The resultant implementation uses Sylvester matrices and recursive determinant expansion:

```text
geosolver-core/src/algebra/resultant.rs:170
geosolver-core/src/algebra/resultant.rs:324
geosolver-core/src/algebra/resultant.rs:367
geosolver-core/src/algebra/resultant.rs:377
geosolver-core/src/algebra/resultant.rs:393
```

The implementation has a matrix-dimension cap:

```text
geosolver-core/src/algebra/resultant.rs:87
geosolver-core/src/algebra/resultant.rs:119
```

but no cap on:

```text
intermediate polynomial term count
intermediate total degree
number of keep variables
coefficient growth
number of polynomial multiplications
elapsed time
```

For this input, the dangerous computation is not a large numeric matrix dimension. It is polynomial
expression swell inside small symbolic determinant matrices.

## Planner/Coverage Evidence

The planner builds all admissions and then sorts executable plans into the declared ladder:

```text
geosolver-core/src/planner/planner.rs:32
geosolver-core/src/planner/planner.rs:39
geosolver-core/src/planner/ladder.rs:6
geosolver-core/src/planner/ladder.rs:15
```

The cost estimate is based on generic block-level probe dimensions and fixed kernel penalties:

```text
geosolver-core/src/planner/cost_model.rs:45
geosolver-core/src/planner/cost_model.rs:50
geosolver-core/src/planner/cost_model.rs:56
geosolver-core/src/planner/cost_model.rs:100
geosolver-core/src/planner/cost_model.rs:114
```

It does not estimate sparse-resultant intermediate expression swell. Consequently,
`SparseResultantProjection` is admitted and selected first even though its execution path is
impractical for this block.

## Non-Causes

The evidence does not support these as the current timeout cause:

```text
dense TargetRelationSearch planning materialization
projection DAG construction
message composition
global support construction
support verification
squarefree support
real root isolation
candidate decoding
exact-image filtering
```

Those later stages are not reached in the timed runs.

## Root Cause

Supported immediate root cause:

```text
BlockId(5)'s first ladder route is SparseResultantProjection. The same sparse-resultant chain, run
through the public resultant API on the block data, enters an uncapped exact Sylvester-resultant
computation after intermediate polynomial term counts have exploded. The captured stdout log reaches
a 3x3 resultant on 1,588-term and 16,662-term polynomials and contains no completion line for that
computation.
```

Systemic root causes:

```text
1. SparseResultantProjection admission/planning only bounds template matrix dimension, not symbolic
   expression swell.
2. The planner cost model does not include sparse-resultant intermediate term-growth estimates.
3. The declared ladder executes kernels serially, but a long-running first kernel has no elapsed
   budget or continuable timeout, so later admitted kernels can be delayed indefinitely.
4. The exact resultant implementation uses recursive determinant expansion on polynomial entries,
   which is unsuitable once polynomial entries have thousands of terms and many keep variables.
5. Production sparse-resultant execution recomputes resultants for certificate verification,
   multiplying the cost of already expensive resultant steps.
```

## Recommended Fix Direction

Required solver-side fixes before treating this input as evidence of fast generic completion:

```text
1. Add SparseResultantProjection execution preflight with symbolic expression-swell estimates:
   input term counts, total degrees, keep-variable count, Sylvester dimension, estimated product
   terms, and estimated determinant expansion work.

2. Add hard intermediate caps for SparseResultantProjection:
   max input terms per selected pair, max output terms, max keep variables, max total degree,
   max coefficient height, and max elapsed work units.

3. If a sparse-resultant route exceeds those caps, return an allowed
   FiniteResourceFailure/AlgorithmicHardCase for that route so the declared ladder can continue
   to TargetActionKrylov, RegularChainProjection, SpecializationInterpolation, or Universal.

4. Extend the planner cost model so sparse-resultant term-growth estimates affect ordering.

5. Replace recursive polynomial determinant expansion with a bounded modular/evaluation-
   interpolation resultant method, or keep the current exact method only for very small term-count
   templates.
```

Input-side improvement:

```text
The supplied generated problem does not normalize A, B, or C. A geometry lowering that fixes an
Euclidean gauge, for example A=(0,0) and B=(1,0), would likely reduce the large block width. This is
secondary: the solver should still decline or bound an infeasible sparse-resultant route instead of
spending the entire timeout inside it.
```

## Final Diagnosis

The current post-GPSR timeout is explained by the planned first large-block route:
`SparseResultantProjection` on `BlockId(5)`. Planning succeeds quickly and dense TRS is declined by
preflight. The selected sparse-resultant route then encounters severe symbolic expression swell: a
measured 4x4 resultant took about 88 seconds and produced a 628,925-term relation, and the 600
second diagnostic stdout reaches a later 3x3 resultant on 1,588-term and 16,662-term inputs with no
completion line. The supported cause is uncapped sparse-resultant expression swell and missing
per-route fallback budgeting, not dense TRS planning, final support/root processing, or
candidate-cover semantics.
