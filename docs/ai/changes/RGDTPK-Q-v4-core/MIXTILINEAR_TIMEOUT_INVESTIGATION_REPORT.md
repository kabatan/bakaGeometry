# Mixtilinear Candidate-Cover Timeout Investigation Report

Status: diagnostic report.

Purpose: record evidence for why the attached `mixtilinear_candidate_cover_problem.rs` did not
return a candidate-cover result within the requested 10 minute timeout.

Authority: evidence and engineering diagnosis only. This file does not amend the Base Spec, Plan,
claim ceiling, or acceptance status.

Claim boundary: this report does not claim exact-image completion, full supplied-v4 source
fidelity, full acceptance completion, benchmark readiness, or any R-ID `VERIFIED` status.

## Executive Summary

The timeout was caused before projection execution, composition, support construction, root
isolation, or candidate decoding. The solver reached the planning phase quickly, then stalled while
planning `TargetRelationSearch` for one large projection block.

The specific large block had:

```text
local_vars=33
relations=29
exported=2
max_degree=7
```

For this block, the default `TargetRelationSearch` schedule computes:

```text
d_max=7
z_seed=4
e_cap=47
variables_for_multipliers=33
```

At the first schedule stage alone, the planner attempts to materialize about 5.9 billion dense
multiplier-support columns:

```text
stage e=4:
  multiplier_total_degree=11
  export_cols=15
  multiplier_cols=5,924,738,442
  total_cols=5,924,738,457
```

This is a planner/admission scalability defect: the planner builds a dense relation-search schedule
by enumerating all monomials before rejecting the route. It does not use a cheap preflight cap to
decline infeasible dense schedules. Therefore, a geometry-derived algebraic input with one
33-variable block can consume the timeout in planning even though other kernel planners return
quickly.

## User-Visible Failure

The direct attempt to solve the attached problem was run with a 10 minute timeout.

Observed outcome:

```text
command timed out after 604021 milliseconds
```

Process observation after timeout:

```text
run_mixtilinear CPU ~= 603 seconds
```

No solver result was returned. Therefore:

```text
status: unavailable
candidate list: unavailable
support polynomial: unavailable
```

## Input Size Evidence

A temporary inspection example was used and then deleted. It called only
`mixtilinear_candidate_cover_problem::build_problem()` and inspected the resulting
`RationalTargetProblem`.

Observed input size:

```text
declared_variables=52
variables_seen_in_equations=52
equations=48
semantic_encodings=17
target=VariableId(0)
max_total_degree=6
max_terms_per_equation=34
total_terms=283
```

Per-equation profile excerpt:

```text
eq[00] degree=3 terms=7 vars=7
eq[01] degree=4 terms=9 vars=7
eq[34] degree=6 terms=34 vars=9
eq[37] degree=3 terms=9 vars=9
eq[45] degree=3 terms=9 vars=9
```

After canonicalization and compression in the solver pipeline, the problem became:

```text
compressed variables=38
compressed relations=34
diagnostics=0
```

This is already much larger than the public acceptance/red-team cases that previously exercised the
candidate-cover core.

## Pipeline Stage Evidence

A temporary pipeline profiler was used and then deleted. It called the public pipeline stages in
order:

```text
step_validate
step_canonicalize
step_compress
step_build_graphs
step_build_dag
step_plan
```

Observed timings:

```text
input variables=52 equations=48 semantics=17 target=VariableId(0)
t=   0.020s built problem
t=   0.030s validated
t=   0.041s canonicalized
compressed variables=38 relations=34 diagnostics=0
t=   1.214s compressed
t=   1.484s built graphs
dag blocks=11 root=BlockId(0)
t=   1.581s built dag
```

The process then remained inside `step_plan` and was terminated after 60 seconds in the profiler
run. This proves the earlier 10 minute timeout was not caused by:

- projection-message execution;
- message verification;
- composition;
- final support construction;
- support verification;
- root isolation;
- candidate decoding.

The timeout occurs before those stages are reached.

## DAG Structure Evidence

The generated projection DAG had 11 blocks:

```text
dag blocks=11 root=BlockId(0)
block BlockId(0): local_vars=38 relations=0 exported=1 children=2
block BlockId(1): local_vars=37 relations=0 exported=2 children=2
block BlockId(2): local_vars=36 relations=0 exported=2 children=2
block BlockId(3): local_vars=35 relations=0 exported=2 children=2
block BlockId(4): local_vars=34 relations=0 exported=2 children=2
block BlockId(5): local_vars=33 relations=29 exported=2 children=0
block BlockId(6): local_vars=3 relations=1 exported=2 children=0
block BlockId(7): local_vars=3 relations=1 exported=2 children=0
block BlockId(8): local_vars=3 relations=1 exported=2 children=0
block BlockId(9): local_vars=3 relations=1 exported=2 children=0
block BlockId(10): local_vars=3 relations=1 exported=2 children=0
```

The critical block is `BlockId(5)`. It contains nearly all hard relations and most variables:

```text
mode=summary block=BlockId(5) local_vars=33 relations=29 exported=2
probes:
  vars=33
  rels=29
  max_degree=7
  total_monomials=381
  template=29x64
  rank_est=18446744073709551615
  height_bits=1050
```

The `rank_est=18446744073709551615` value is `usize::MAX`, showing that the structural rank
estimate saturated for this block. That is a warning sign that the block is outside the practical
range of the dense planner path.

Selected relation profile for `BlockId(5)`:

```text
RelationId(15): degree=6 terms=28 vars=9
RelationId(17): degree=6 terms=28 vars=9
RelationId(34): degree=6 terms=34 vars=9
RelationId(35): degree=6 terms=26 vars=10
RelationId(36): degree=6 terms=28 vars=10
RelationId(37): degree=7 terms=25 vars=13
RelationId(42): degree=6 terms=26 vars=9
```

## Planner Isolation Evidence

A temporary planner probe was used and then deleted. It tested `BlockId(5)` planner components
individually with a 30 second cap per mode.

Observed results:

```text
target-relation: TIMEOUT_30S
linear-affine: 1.0686ms, declined
sparse-resultant: 100.7516ms, planned
target-action: 1.0849ms, planned
norm-trace: 3.9065ms, hardcase
regular-chain: 619.9us, planned
specialization: 2.0081ms, planned
universal: 785.4us, planned
collect_kernel_admissions: TIMEOUT_30S
```

This isolates the stall to `TargetRelationSearch` planning. Other candidate-cover kernel planners
for the same block return quickly.

## Code Path Evidence

`plan_all_blocks` calls `run_cost_probes`, then `collect_kernel_admissions`, then builds the
declared ladder:

```text
geosolver-core/src/planner/planner.rs:24
geosolver-core/src/planner/planner.rs:25
geosolver-core/src/planner/planner.rs:31
```

In `collect_kernel_admissions`, the `TargetRelationSearch` branch immediately calls
`build_dense_relation_search_schedule`:

```text
geosolver-core/src/planner/admission.rs:160
geosolver-core/src/planner/admission.rs:164
```

The default degree cap is:

```rust
z_seed.max(2 * d_max + eliminated.len() + exported.len())
```

Source location:

```text
geosolver-core/src/planner/relation_schedule.rs:43
```

For the critical block:

```text
d_max=7
eliminated.len()=31
exported.len()=2
z_seed=4
e_cap=max(4, 2*7 + 31 + 2)=47
```

Then `build_dense_relation_search_schedule` iterates:

```rust
for e in z_seed..=e_cap
```

Source location:

```text
geosolver-core/src/planner/relation_schedule.rs:82
```

At each stage it calls `build_multiplier_supports`, which calls
`monomials_total_degree_leq` on the union of eliminated and exported variables:

```text
geosolver-core/src/planner/relation_schedule.rs:89
geosolver-core/src/planner/relation_schedule.rs:175
geosolver-core/src/planner/relation_schedule.rs:189
```

`monomials_total_degree_leq` recursively enumerates and stores every monomial:

```text
geosolver-core/src/planner/relation_schedule.rs:227
geosolver-core/src/planner/relation_schedule.rs:235
```

This means planning is not just estimating a dense schedule. It materializes large monomial support
vectors during admission.

## Combinatorial Explosion Calculation

For `BlockId(5)`, multiplier supports use 33 variables. The number of monomials in `n` variables of
total degree at most `D` is:

```text
C(n + D, D)
```

The first relation-search stage is `e=4`, and the schedule uses:

```text
multiplier_total_degree = e + d_max = 11
```

For each source relation, the multiplier degree is:

```text
11 - relation_degree
```

Because many relations have degree 2, 3, 4, 6, and 7, the resulting multiplier supports are already
massive. The measured closed-form estimate for the first stage was:

```text
stage e=4:
  multiplier_total_degree=11
  export_cols=15
  multiplier_cols=5,924,738,442
  total_cols=5,924,738,457
```

The second stage is even larger:

```text
stage e=5:
  multiplier_total_degree=12
  export_cols=21
  multiplier_cols=25,735,295,351
  total_cols=25,735,295,372
```

The final default stage is completely infeasible:

```text
stage e=47:
  multiplier_total_degree=54
  export_cols=1,176
  multiplier_cols=7,158,568,357,581,507,113,343,770
  total_cols=7,158,568,357,581,507,113,344,946
```

The planner times out while trying to construct schedule data structures long before it could reach
matrix solving or candidate-cover verification.

## Why Resource Bounds Did Not Stop It

The default solver options leave all major resource caps unset:

```text
max_relation_search_export_degree: None
max_memory_bytes: None
max_matrix_rows: None
max_matrix_cols: None
max_coefficient_height_bits: None
```

Source location:

```text
geosolver-core/src/solver/options.rs:30
```

`TargetRelationSearchKernel` has `enforce_matrix_limits`, but that check is in execution and only
uses `ctx.options.max_matrix_rows` and `ctx.options.max_matrix_cols`:

```text
geosolver-core/src/kernels/target_relation_search.rs:1095
```

Since the default options have no matrix row/column caps, this check would not reject a huge matrix
even if execution were reached.

More importantly, this problem does not reach execution. The schedule is materialized during
admission/planning, before execution-level matrix-limit checks are applied.

## Root Cause

The root cause is a missing cheap feasibility gate in `TargetRelationSearch` admission/planning.

Current behavior:

1. The planner sees that the large block has local relations.
2. It admits `TargetRelationSearch` by constructing a full dense relation-search schedule.
3. Schedule construction enumerates all multiplier monomial supports.
4. The first stage alone requires billions of monomial entries.
5. Planning never returns within the requested timeout.

Expected behavior for a high-speed solver:

1. Estimate schedule dimensions from closed-form monomial counts.
2. Decline dense `TargetRelationSearch` immediately when estimated rows/columns exceed practical
   caps.
3. Continue to cheaper or more specialized kernels already available in the ladder.

This is not a candidate-cover semantic issue. It is a planner scalability issue.

## Secondary Observations

The decomposition produced a large 33-variable block with 29 relations. The geometric input was
lowered without normalizing the triangle or fixing coordinate gauges for `A`, `B`, or `C`. That
leaves many Euclidean degrees of freedom in the algebraic system. A geometric normalization such as
placing `A=(0,0)`, `B=(1,0)`, and representing `C` with fewer variables would likely reduce block
width substantially. However, the immediate timeout still reflects a solver-side missing preflight
guard: the planner should not attempt to instantiate a multi-billion-column dense schedule.

Also, some other planners for the large block returned quickly:

```text
sparse-resultant planned 50x50
target-action planned 256x256
regular-chain planned 29x2
specialization planned 219x219
universal planned with declared resource caps
```

Because `collect_kernel_admissions` processes all kernel admissions and stalls on
`TargetRelationSearch`, those other candidate routes never get a chance to form the final ladder for
execution.

## Recommended Fixes

Recommended immediate production fixes:

1. Add a closed-form preflight estimator to `build_dense_relation_search_schedule` or the
   `TargetRelationSearch` admission branch. It should compute estimated export columns, multiplier
   columns, and row monomial upper bounds without allocating `Vec<Monomial>`.

2. Decline `TargetRelationSearch` before materialization when estimated dimensions exceed either:
   explicit solver options, or conservative default planner caps.

3. Make resource caps apply at planning/admission time, not only at execution time.

4. Treat saturated probe values such as `rank_est=usize::MAX` as a strong signal to avoid dense
   relation search on that block unless the user explicitly overrides caps.

5. Consider reordering admission so cheap planners can be collected even if one expensive generic
   route would be infeasible, or isolate each admission so one bad route cannot block the entire
   ladder.

Recommended geometry/input-side improvement:

1. Normalize Euclidean gauge before algebraic lowering for geometry problems. The attached file
   explicitly says it does not normalize `A`, `B`, or `C`. That choice produced unnecessary
   coordinate freedom and contributed to the 33-variable projection block.

## Minimal Regression Test Shape

A focused regression test should not need the full mixtilinear problem. It should construct or load
a block with:

```text
eliminated variables >= 31
exported variables = 2
d_max >= 7
relations >= 29
```

Then assert that `TargetRelationSearch` admission returns quickly with `Declined` or
`FiniteResourceFailure` under default caps, without materializing monomial supports. The test should
also assert that other kernel admissions are still considered.

## Final Diagnosis

The solver timed out because the planner attempted dense target-relation search on a 33-variable
block and tried to materialize a relation-search schedule whose first stage has approximately
5.9 billion columns. The timeout is therefore caused by missing admission-time resource gating and
eager monomial-support enumeration, not by root isolation, exact-image filtering, replay, support
verification, or candidate decoding.
