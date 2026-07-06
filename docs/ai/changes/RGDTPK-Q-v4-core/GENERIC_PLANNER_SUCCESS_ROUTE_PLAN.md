# Generic Planner Success-Route Repair Plan v1

## Phase GPSR-P0: Agent failure-mode reset

Create `GENERIC_PLANNER_AGENT_RESET.md` and explicitly acknowledge:

```text
- Do not add the investigated geometry problem as a regression.
- Do not optimize for a named case.
- Do not close by returning fast failure.
- Keep one unified pipeline.
- Fix generic planner success routing.
```

Required evidence:

```text
docs/ai/changes/RGDTPK-Q-v4-core/GENERIC_PLANNER_AGENT_RESET.md
```

Reviewer prompt: `GPSR-RP0`.

## Phase GPSR-P1: Direct code audit of planning materialization

Audit and record every path where admission/planning materializes monomial supports or matrices.

Must inspect at least:

```text
geosolver-core/src/planner/admission.rs
geosolver-core/src/planner/relation_schedule.rs
geosolver-core/src/planner/planner.rs
geosolver-core/src/planner/ladder.rs
geosolver-core/src/kernels/target_relation_search.rs
geosolver-core/src/kernels/universal_elimination.rs
geosolver-core/src/kernels/specialization_interpolation.rs
```

Create `GENERIC_PLANNER_MATERIALIZATION_AUDIT.md` with rows:

```yaml
path: <file::function>
phase: admission|plan|execute|universal_internal
materializes_support: true|false
materializes_matrix: true|false
uses_closed_form_preflight: true|false
can_block_admission_collection: true|false
required_action: keep|descriptorize|preflight_guard|move_to_execute|delete
```

Reviewer prompt: `GPSR-RP1`.

## Phase GPSR-P2: Add closed-form dense relation-search preflight

Implement in `planner/relation_schedule.rs` or a new `planner/relation_preflight.rs`.

Required types:

```rust
pub struct SaturatingCount { pub value: Option<u128>, pub saturated: bool }

pub struct RelationSearchPlanningCaps {
    pub max_export_cols: SaturatingCount,
    pub max_multiplier_cols: SaturatingCount,
    pub max_matrix_cols: SaturatingCount,
    pub max_row_monomials: SaturatingCount,
    pub max_estimated_memory_bytes: SaturatingCount,
    pub max_materialized_stages: usize,
}

pub struct RelationSearchStageEstimate {
    pub export_degree: usize,
    pub multiplier_total_degree: usize,
    pub estimated_export_cols: SaturatingCount,
    pub estimated_multiplier_cols: SaturatingCount,
    pub estimated_matrix_cols: SaturatingCount,
    pub estimated_row_monomials_upper_bound: SaturatingCount,
    pub feasible_under_caps: bool,
    pub estimate_hash: Hash,
}

pub struct DenseRelationSearchPreflight {
    pub eliminated_variables: Vec<VariableId>,
    pub exported_variables: Vec<VariableId>,
    pub z_seed: usize,
    pub e_cap: usize,
    pub d_max: usize,
    pub stages: Vec<RelationSearchStageEstimate>,
    pub first_feasible_stage: Option<usize>,
    pub first_infeasible_stage: Option<usize>,
    pub preflight_hash: Hash,
}
```

Required functions:

```rust
pub fn monomial_count_total_degree_leq(nvars: usize, degree: usize) -> SaturatingCount;
pub fn default_relation_search_planning_caps(options: &SolverOptions) -> RelationSearchPlanningCaps;
pub fn estimate_dense_relation_search_schedule(...)-> DenseRelationSearchPreflight;
pub fn preflight_allows_materialization(preflight: &DenseRelationSearchPreflight)-> bool;
```

Rules:

```text
- No Vec<Monomial> allocation in preflight.
- All arithmetic saturates.
- Explicit SolverOptions caps override defaults but cannot cause unbounded allocation during admission.
```

Reviewer prompt: `GPSR-RP2`.

## Phase GPSR-P3: Convert dense schedules to descriptor-first planning

Add support descriptors so plan hashes can be deterministic without full monomial materialization.

Required behavior:

```text
- Admission stores preflight and descriptor hashes.
- Materialized monomial supports are built only after preflight allows it.
- If preflight rejects dense route, no dense support Vec is allocated.
```

Implementation options:

```text
A. Add descriptor fields to DenseRelationSearchSchedule and keep materialized fields only for feasible stages.
B. Add a separate DenseRelationSearchPlanDescriptor and build full schedule in execution only.
```

Forbidden:

```text
building dense monomial supports just to compute schedule hash
```

Reviewer prompt: `GPSR-RP3`.

## Phase GPSR-P4: Make TargetRelationSearch admission non-blocking

Modify `planner/admission.rs`.

Required behavior:

```text
- Run dense preflight before calling build_dense_relation_search_schedule.
- If dense route exceeds caps, return a declined/cost-prohibited admission for TargetRelationSearch.
- Do not throw solve-level failure from admission for cost-prohibited dense route.
- Preserve source relation hashes and diagnostic estimates for cost trace.
```

If `TargetRelationSearch` has a feasible sparse-footprint strategy, it may admit that strategy. Dense infeasibility must not imply total TargetRelationSearch impossibility if another declared relation-search support strategy is feasible.

Reviewer prompt: `GPSR-RP4`.

## Phase GPSR-P5: Admission isolation for all kernels

Modify `collect_kernel_admissions` so every kernel admission is isolated.

Required behavior:

```text
- One kernel's cost-prohibited result does not stop collection.
- Panics are not acceptable; true implementation bugs should be reported but not disguised.
- All later kernels are still attempted after TargetRelationSearch is declined.
- UniversalTargetElimination is included when the block has relations.
```

Add a generic test that forces dense TargetRelationSearch infeasibility and asserts later kernel admissions are still present.

The test must not import or mention the investigated geometry problem.

Reviewer prompt: `GPSR-RP5`.

## Phase GPSR-P6: Success-route cost policy and declared ladder repair

Modify `planner/cost_model.rs` and `planner/ladder.rs`.

Required behavior:

```text
- Cost-prohibited routes are excluded from executable ladder.
- Feasible compact target-direct routes are ranked ahead of infeasible dense relation search.
- UniversalTargetElimination remains last executable generic route when admitted.
- The final ladder is nonempty for well-formed blocks with relations.
```

Add algebraic-footprint scoring terms:

```text
dense_trs_estimated_cols
sparse_template_size
quotient_rank_estimate
specialization_template_size
regular_chain_detected
tower_detected
separator_width
certificate_route_strength
```

Reviewer prompt: `GPSR-RP6`.

## Phase GPSR-P7: Universal internal safeguard

Modify `kernels/universal_elimination.rs` so every internal TargetRelationSearch escalation uses the same preflight.

Required behavior:

```text
- Universal does not call dense TargetRelationSearch blindly.
- If dense escalation is infeasible, Universal proceeds to other declared strategies.
- Universal remains target/separator-export-only and certificate-bound.
- Universal does not return nonfinite because dense relation search is too large.
```

Reviewer prompt: `GPSR-RP7`.

## Phase GPSR-P8: Generic algebraic stress tests only

Add tests that verify the generic design without including the investigated problem.

Required families:

```text
1. Dense-preflight infeasible synthetic footprint:
   build a block or schedule with many eliminated variables and degree enough to exceed caps.
   Assert preflight returns quickly and no monomial supports are materialized.

2. Admission isolation:
   same synthetic footprint; assert TargetRelationSearch is cost-prohibited/declined and later kernels are still collected.

3. Support-producing compact quotient/action family:
   a parameterized multivariate finite quotient/action problem that would make dense TRS large under inflated auxiliary variables, but has a compact quotient/action route.
   Must return CertifiedCandidateCover through public or near-public pipeline.

4. Support-producing sparse/resultant or specialization family:
   a parameterized problem with many irrelevant or auxiliary local variables but sparse eliminant structure.
   Must return CertifiedCandidateCover.

5. Universal ladder preservation:
   a block where specialized kernels may decline but Universal remains in ladder and planning completes.
```

Rules:

```text
- No geometry names.
- No imported generated geometry problem file.
- Randomized/deterministic seed variable renaming and rational coefficient scaling.
- Do not assert an expected final answer unless the answer follows from the constructed algebraic family and is verified by support certificate.
```

Reviewer prompt: `GPSR-RP8`.

## Phase GPSR-P9: Cost trace and diagnostics

Ensure cost-prohibited dense routes are visible in diagnostics/cost trace but do not become solve-level failures when another route succeeds.

Required evidence fields:

```text
kernel_kind = TargetRelationSearch
route = DenseTotalDegree
estimated_matrix_cols
estimated_multiplier_cols
planning_cap
decision = CostProhibitedDenseRoute
block_id
```

If existing `GlobalCostTrace` cannot store declined-route diagnostics, add a planner diagnostic record or admission trace that is serialized in result diagnostics.

Reviewer prompt: `GPSR-RP9`.

## Phase GPSR-P10: Final review and closure

Create:

```text
GENERIC_PLANNER_SUCCESS_ROUTE_RESULTS.md
GENERIC_PLANNER_REPLAY_AND_TRACE_RESULTS.md
GENERIC_PLANNER_CLOSURE.md
```

Allowed claim after this repair:

```text
PLANNER_SUCCESS_ROUTE_READY
DENSE_TRS_ADMISSION_SAFE
```

This repair may preserve but must not re-claim the entire candidate-cover layer unless the full existing candidate-cover suite is rerun and passes.

Forbidden claims from this repair alone:

```text
benchmark superiority
source-faithful full v4 completion
exact-image completion
success on the investigated geometry problem as a gate
```

Reviewer prompt: `GPSR-RP10`.
