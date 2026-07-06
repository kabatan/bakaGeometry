# Mixtilinear Post-GSR Core Repair Timeout Report

Status: diagnostic report.

Purpose: record the supported root-cause diagnosis for why the user-supplied
`mixtilinear_candidate_cover_problem.rs` still failed to return candidate-cover output within a
10 minute timeout after the Generic Success Route Core Repair work.

Authority: evidence and engineering diagnosis only. This report does not amend the Base Spec,
Plan, claim ceiling, acceptance matrix, or readiness status.

Claim boundary: this report does not claim exact-image completion, full source fidelity, full
acceptance completion, benchmark readiness, or that the supplied problem is a permanent fixture.
The supplied file was used only as an external diagnostic input requested by the user.

## Executive Summary

The timeout is not caused by validation, canonicalization, compression, graph construction, DAG
construction, or planning. Those stages complete in about 0.2 seconds in the latest probe.

The direct stopping point is the first executable route for the large projection block:

```text
block=5
route=SparseResultantProjection
```

The route enters an exact sparse-resultant chain with severe symbolic expression swell. The chain
creates a 628,925-term intermediate relation at step 5, then later reaches step 15 where it starts a
3x3 Sylvester-resultant computation on 1,588-term and 16,662-term polynomial inputs. In the 180
second diagnostic run, the log contains the step 15 `TRACE COMPUTE START` line and no corresponding
`TRACE COMPUTE END` line.

The systemic defect is that `SparseResultantProjection` admission and planning bound only template
matrix dimensions and generic block-level costs. They do not bound intermediate polynomial term
growth, keep-variable count, determinant expansion work, or elapsed route work. Because the declared
ladder executes routes serially, the long-running first route prevents admitted fallback routes from
running within the 10 minute budget.

## Diagnostic Scope

Evidence was gathered with temporary local probes and then removed:

```text
geosolver-core/tests/mixtilinear_stage_probe.rs
temporary MIXTILINEAR_SPARSE_LOG instrumentation in geosolver-core/src/kernels/sparse_resultant.rs
```

The probes imported:

```text
C:\Users\bakat\Downloads\mixtilinear_candidate_cover_problem.rs
```

No permanent test fixture, benchmark, or acceptance gate was added for this input.

## User-Visible Failure

The release solver run was allowed approximately 10 minutes and timed out:

```text
release solver execution timed out after 604.054s
result output: none
candidate list: unavailable
```

This means the solver did not reach a final support polynomial, root isolation, candidate decoding,
or candidate-cover output for this input within the requested bound.

## Pipeline Stage Evidence

The 180 second stage probe recorded:

```text
[       1 ms] problem built: variables=52 relations=48 semantics=17
[       1 ms] END step_validate: elapsed_ms=0 result=ok
[       1 ms] END step_canonicalize: elapsed_ms=0 result=ok
[     152 ms] END step_compress: elapsed_ms=150 result=ok
[     188 ms] END step_build_graphs: elapsed_ms=36 result=ok
[     188 ms] END step_build_dag: elapsed_ms=0 result=ok
[     212 ms] END step_plan: elapsed_ms=23 result=ok
```

The hard projection block was:

```text
[     188 ms] block=5 locals=33 exports=2 relations=29 children=[]
```

The declared ladder for that block was:

```text
[     212 ms] plan block=5 ladder=[
  SparseResultantProjection,
  TargetActionKrylov,
  RegularChainProjection,
  SpecializationInterpolation,
  UniversalTargetElimination
]
```

Manual execution reached the first route and did not return from it:

```text
[     212 ms] ROUTE START block=5 kernel=SparseResultantProjection
```

No later `ROUTE OK` or `ROUTE ERR` line was emitted before the 180 second diagnostic timeout.

## Planner Evidence

The old dense-planning failure is not the current direct timeout path. Dense
`TargetRelationSearch` for block 5 was declined quickly by preflight:

```text
kernel=TargetRelationSearch block_id=5 admission_status=CostProhibited
estimated_matrix_cols=242588986
estimated_rows=1809761560
estimated_memory_bytes=262700869888
matrix_col_cap=65536
matrix_row_cap=65536
memory_cap_bytes=268435456
stage_count=46
```

The same planner diagnostics admitted `SparseResultantProjection`:

```text
kernel=SparseResultantProjection block_id=5 admission_status=Admitted
cost_class=ExpensiveButAllowed
eliminated_variables=31
estimated_rows=29
estimated_cols=64
exported_variables=2
```

`TargetActionKrylov`, `RegularChainProjection`, `SpecializationInterpolation`, and
`UniversalTargetElimination` were also admitted for the same block, but they were behind
`SparseResultantProjection` in the declared ladder.

## Sparse-Resultant Trace Evidence

The 180 second sparse-resultant trace started with:

```text
TRACE START relations=29 eliminated=31 exported=[VariableId(0), VariableId(31)] max_dim=51
```

Early steps completed quickly enough to log outputs. The first severe expression swell occurred at
step 5:

```text
TRACE STEP SELECTED step=5 eliminate=VariableId(6)
  left_terms=28
  right_terms=1176
  deg_left=2
  deg_right=2
  keep_vars=16
  matrix=4x4

TRACE COMPUTE END step=5 eliminate=VariableId(6)
  resultant_terms=628925
  resultant_degree=40
```

The chain continued with this 628,925-term polynomial in the current relation set:

```text
TRACE STEP START step=15 eliminate=VariableId(25)
current_relations=14
current_terms=[8, 7, 3, 8, 7, 7, 7, 7, 628925, 52, 305, 80, 1588, 16662]
```

The step selected a small symbolic matrix but very large polynomial entries:

```text
TRACE STEP SELECTED step=15 eliminate=VariableId(25)
  left=12
  right=13
  left_terms=1588
  right_terms=16662
  deg_left=2
  deg_right=1
  keep_vars=24
  matrix=3x3
```

The final sparse trace line before the 180 second timeout was:

```text
TRACE COMPUTE START step=15 eliminate=VariableId(25)
```

There was no matching `TRACE COMPUTE END step=15` line. This identifies the active computation at
timeout as `compute_resultant_relation` for the step 15 sparse-resultant pair.

## Code-Path Evidence

The pipeline executes declared ladder routes serially:

```text
geosolver-core/src/solver/pipeline.rs:502
geosolver-core/src/solver/pipeline.rs:523
```

The planner sorts executable plans by the cost model and declared kernel order:

```text
geosolver-core/src/planner/ladder.rs:6
geosolver-core/src/planner/ladder.rs:15
geosolver-core/src/planner/cost_model.rs:99
geosolver-core/src/planner/cost_model.rs:132
```

`SparseResultantProjection` currently has a fixed planner order and certificate cost:

```text
geosolver-core/src/planner/cost_model.rs:137
geosolver-core/src/planner/cost_model.rs:151
```

`SparseResultantProjection` planning builds a template plan from `probe_sparse_resultant_plan`:

```text
geosolver-core/src/kernels/sparse_resultant.rs:136
geosolver-core/src/kernels/sparse_resultant.rs:351
```

That probe accumulates template matrix rows and columns from selected pairs. It does not simulate
the execution chain's intermediate resultant outputs or term growth.

Production execution calls `build_sparse_resultant_trace` before producing a projection message:

```text
geosolver-core/src/kernels/sparse_resultant.rs:245
geosolver-core/src/kernels/sparse_resultant.rs:395
```

The trace loop computes exact resultants and pushes each new relation into the current relation set.
There is no hard cap in this loop on intermediate term count, total degree, keep-variable count,
coefficient growth, or elapsed work.

The selected pair search repeatedly scans relation pairs:

```text
geosolver-core/src/kernels/sparse_resultant.rs:505
```

Once a 628,925-term relation is present, even pair selection becomes expensive. The 180 second probe
still progressed to step 15, but the large intermediate relation remains part of the current set.

The exact resultant implementation is a Sylvester determinant:

```text
geosolver-core/src/algebra/resultant.rs:170
geosolver-core/src/algebra/resultant.rs:324
geosolver-core/src/algebra/resultant.rs:367
```

The determinant implementation uses recursive Laplace expansion over polynomial entries. Polynomial
multiplication is a nested term-pair product followed by normalization:

```text
geosolver-core/src/types/polynomial.rs:108
geosolver-core/src/types/polynomial.rs:30
```

For step 15, even a 3x3 matrix is not cheap because entries contain thousands of terms and 24 keep
variables. The matrix dimension cap is therefore not a sufficient work bound.

## Non-Causes

The evidence does not support these as the current direct timeout cause:

```text
validation
canonicalization
compression
graph construction
projection DAG construction
planning as a whole
dense TargetRelationSearch materialization
message composition
global support construction
support verification
squarefree support
real root isolation
candidate decoding
exact-image filtering
```

Those later stages are not reached in the timed execution path.

## Root Cause

Immediate root cause:

```text
The first declared route for block 5 is SparseResultantProjection. That route enters an exact
sparse-resultant chain whose intermediate polynomial expression swell is not bounded. The latest
probe reaches step 15, starts a 3x3 Sylvester resultant on 1,588-term and 16,662-term polynomials,
and does not complete that computation before the timeout.
```

Systemic causes:

```text
1. SparseResultantProjection admission only checks whether some finite resultant template is
   available; it does not reject blocks with dangerous expression-swell risk.
2. SparseResultantProjection planning estimates template matrix dimensions, not intermediate
   resultant term growth or determinant polynomial work.
3. The cost model uses generic block-level matrix/rank estimates and fixed kernel penalties, so it
   can place SparseResultantProjection before other admitted fallback routes despite the route's
   practical cost.
4. The declared ladder is serial. A long-running first route prevents later admitted routes from
   being tried within the user's wall-clock budget.
5. The exact resultant backend uses recursive polynomial determinant expansion, which is unsuitable
   after expression swell produces thousands to hundreds of thousands of terms.
```

## Required Fix Direction

Required solver-side fixes before claiming this class of input can be handled quickly:

```text
1. Add SparseResultantProjection preflight estimates for:
   - selected pair input term counts
   - keep-variable count
   - total degree
   - Sylvester dimension
   - estimated determinant expansion work
   - estimated output term growth

2. Add runtime guards inside build_sparse_resultant_trace:
   - max pair input terms
   - max intermediate output terms
   - max keep variables
   - max total degree
   - max coefficient height
   - max work units or elapsed route budget

3. When a guard is exceeded, return an allowed FiniteResourceFailure or AlgorithmicHardCase for
   the route so the declared ladder can continue to TargetActionKrylov, RegularChainProjection,
   SpecializationInterpolation, or UniversalTargetElimination.

4. Feed sparse-resultant expression-swell estimates into the planner cost model so dangerous
   sparse-resultant plans are either ordered later or not admitted.

5. Replace recursive polynomial determinant expansion with a bounded modular/evaluation-
   interpolation resultant path, or restrict the current exact determinant implementation to very
   small symbolic templates.
```

Secondary input-side improvement:

```text
The supplied generated geometry problem appears not to normalize an Euclidean gauge for the
triangle. A lowering that fixes a coordinate frame, for example A=(0,0), B=(1,0), may reduce block
width. This is secondary: the solver must still decline or bound infeasible sparse-resultant routes
instead of spending the entire timeout inside them.
```

## Cleanup and Reproducibility Notes

The temporary probe test, temporary source instrumentation, and target logs used for this diagnosis
were removed after evidence capture. Before adding this report, the working tree was clean:

```text
## main...origin/main
```

This report records the relevant observed excerpts so the diagnosis is reviewable without keeping
the temporary probe as a permanent test or fixture.
