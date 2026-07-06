# Generic Planner Success-Route Repair Pack v1


# ===== GENERIC_PLANNER_SUCCESS_ROUTE_BASE_SPEC.md =====

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

# ===== GENERIC_PLANNER_SUCCESS_ROUTE_PLAN.md =====

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

# ===== GENERIC_PLANNER_SUCCESS_ROUTE_REVIEWER_PROMPTS.md =====

# Generic Planner Success-Route Reviewer Prompts v1

## Reviewer meta-protocol

For every phase, reviewer must answer:

```text
1. Did the patch avoid adding the investigated geometry problem as a test/gate/fixture?
2. Does the patch use only algebraic footprint information?
3. Does it preserve the single unified candidate-cover pipeline?
4. Does it prevent dense TargetRelationSearch from materializing huge supports in admission?
5. Does it route toward feasible declared kernels instead of fast failure?
6. Does it avoid hidden fallback, coordinate solving, expected-answer dispatch, and geometry-name dispatch?
```

A PASS is forbidden if any answer is missing or unsupported by code evidence.

## GPSR-RP0: Agent reset review

PASS only if `GENERIC_PLANNER_AGENT_RESET.md` explicitly rejects:

```text
- case-specific optimization;
- adding the investigated problem as a regression;
- fast failure as success;
- separate solvers;
- hidden fallback;
- dense route blocking the whole planner.
```

FAIL if the reset language implies that returning `FiniteResourceFailure` for hard large blocks is an acceptable closure for this repair.

## GPSR-RP1: Materialization audit review

Reviewer must inspect the listed files and verify the audit includes every planning/admission site that can allocate monomial supports or matrices.

Mandatory challenge:

```text
Point to the exact code that previously called dense schedule construction during TargetRelationSearch admission.
Point to the exact row in the audit requiring it to be preflighted or descriptorized.
```

FAIL if any relevant site is omitted.

## GPSR-RP2: Preflight review

PASS only if closed-form preflight computes estimates without enumerating monomials.

Reviewer must check:

```text
- combinatorial count uses saturating arithmetic;
- overflow cannot panic;
- default caps exist;
- explicit options cannot mean unbounded admission allocation;
- estimated rows/cols are hash-bound or traceable.
```

FAIL if `Vec<Monomial>` is allocated inside preflight.

## GPSR-RP3: Descriptor-first schedule review

PASS only if plan hashes can be deterministic without full support materialization.

FAIL if dense support lists are built during admission just to compute support hashes.

Reviewer must run or inspect a synthetic high-dimensional preflight test and confirm no materialized support count is proportional to `C(n+d,d)`.

## GPSR-RP4: TargetRelationSearch admission review

PASS only if dense infeasibility declines/cost-prohibits only the dense route, not the whole solve.

Reviewer must inspect `admission.rs` and confirm:

```text
build_dense_relation_search_schedule is not called before preflight passes;
TargetRelationSearch returns quickly on high-dimensional dense-infeasible footprint;
other kernels are still eligible after it declines.
```

FAIL if admission still materializes dense supports before preflight.

## GPSR-RP5: Admission isolation review

PASS only if one kernel's infeasibility cannot stop later kernels from being collected.

Mandatory code challenge:

```text
Show the loop over all kernel kinds.
Show how a cost-prohibited/declined TargetRelationSearch does not break or return early.
Show UniversalTargetElimination is still considered for relation-bearing blocks.
```

FAIL if the implementation silently skips later kernels after a dense route problem.

## GPSR-RP6: Declared ladder / cost policy review

PASS only if the ladder contains feasible declared target-direct plans and excludes cost-prohibited dense routes.

Reviewer must fail if the code:

```text
- ranks infeasible dense TRS above feasible compact routes;
- leaves the ladder empty for a well-formed relation-bearing block;
- uses geometry names, problem IDs, or expected answers in ranking.
```

## GPSR-RP7: Universal safeguard review

PASS only if Universal uses the same dense preflight and does not blindly call TargetRelationSearch escalation.

Mandatory challenge:

```text
Show what Universal does when dense escalation is infeasible.
Show that it tries other declared local target/separator strategies.
Show that it does not convert dense exhaustion to CertifiedNonFiniteTargetImage.
```

## GPSR-RP8: Generic stress review

PASS only if tests are generic algebraic footprint tests, not the investigated problem.

Reviewer must scan tests/docs for forbidden names and exact fixture imports.

Support-producing tests must prove:

```text
- status = CertifiedCandidateCover;
- support is verified by exact Q certificate;
- replay accepts;
- dense TRS infeasibility did not cause solve failure;
- produced route is through declared kernel ladder.
```

FAIL if tests only check “planning returns quickly” without at least two generic support-producing successes.

## GPSR-RP9: Trace review

PASS only if cost-prohibited dense route decisions are observable in diagnostics or cost trace and do not become solve-level failures when other routes succeed.

FAIL if route decisions disappear from evidence.

## GPSR-RP10: Closure review

PASS only if closure claims are limited to planner success-route readiness and dense admission safety, unless the full candidate-cover suite is rerun.

FAIL if closure claims:

```text
- benchmark superiority;
- exact-image completion;
- full v4 source-fidelity;
- success on the investigated problem as a gated result.
```

# ===== GENERIC_PLANNER_ACCEPTANCE_MATRIX.yaml =====

schema_version: generic-planner-success-route-v1
claim_target:
  allowed_after_repair:
    - PLANNER_SUCCESS_ROUTE_READY
    - DENSE_TRS_ADMISSION_SAFE
  allowed_only_if_full_existing_candidate_cover_suite_rerun:
    - CANDIDATE_COVER_CORE_READY
    - SOURCE_FAITHFUL_TO_V4_CANDIDATE_COVER_LAYER
  forbidden_from_this_repair_alone:
    - BENCHMARK_SUPERIORITY
    - EXACT_IMAGE_CORE_READY
    - SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC
    - RGDTPK_Q_V4_ACCEPTANCE_COMPLETE

forbidden_case_specific_content:
  must_not_include:
    - mixtilinear
    - mixtilinear_candidate_cover_problem
    - expected_cos_value
    - geometry_family_dispatch
    - problem_id_dispatch
    - exact_polynomial_hash_from_reported_problem

required_tests:
  - id: G1
    name: dense_relation_search_preflight_no_materialization
    required_behavior:
      - closed_form_estimate_only
      - no_vec_monomial_allocation
      - returns_under_short_time
      - decision_cost_prohibited_when_caps_exceeded
  - id: G2
    name: admission_isolation_after_dense_trs_decline
    required_behavior:
      - target_relation_search_declined_or_cost_prohibited
      - later_kernel_admissions_collected
      - universal_admitted_for_relation_block
      - no_solve_level_failure_during_admission
  - id: G3
    name: generic_large_footprint_compact_quotient_success
    required_status: CertifiedCandidateCover
    required_path: api_or_near_public_pipeline
    forbidden_behavior:
      - exact_image_filtering_required
      - expected_answer_dispatch
      - geometry_name_dispatch
      - dense_trs_materialization
  - id: G4
    name: generic_large_footprint_sparse_or_specialization_success
    required_status: CertifiedCandidateCover
    required_path: api_or_near_public_pipeline
    required_evidence:
      - support_verified_exact_Q
      - replay_accepts
      - dense_trs_not_executable_due_to_preflight
      - compact_route_used
  - id: G5
    name: universal_ladder_preserved_after_dense_route_prohibited
    required_behavior:
      - universal_in_ladder
      - universal_not_hidden_fallback
      - no_nonfinite_from_dense_exhaustion

review_requirements:
  - scan_for_forbidden_case_specific_content
  - inspect_admission_code_for_early_return
  - inspect_relation_schedule_for_preflight_before_materialization
  - inspect_universal_for_internal_preflight
  - verify_support_producing_generic_stress
  - verify_trace_records_cost_prohibited_dense_route

# ===== GENERIC_PLANNER_AGENT_RESET.md =====

# Generic Planner Agent Reset

Before implementing this repair, the Agent must accept the following.

```text
I am not fixing one named geometry problem.
I must not add that problem as a test, gate, benchmark, or hidden target.
I am fixing a generic planner design defect: an infeasible dense relation-search route can block all other declared local projection kernels.
```

Correct goal:

```text
Use algebraic footprint to avoid infeasible dense route materialization and route planning toward feasible declared target-direct kernels.
```

Incorrect goals:

```text
- Make planning return failure quickly.
- Add a geometry-specific heuristic.
- Recognize variable IDs or polynomial shapes from one reported problem.
- Remove TargetRelationSearch entirely.
- Treat specialized kernels as separate solvers.
```

The pipeline remains unified. Multiple kernels are declared local projection methods inside the unified TargetProjectionDAG pipeline.

# ===== GENERIC_PLANNER_CURRENT_DEFECT_AUDIT.md =====

# Generic Planner Current Defect Audit

## Defect class

A dense TargetRelationSearch admission can attempt to materialize enormous monomial supports during planning. This can prevent the planner from collecting other kernel admissions.

This is a design defect because v4 requires deterministic planning with algebraic cost estimates, not unbounded admission-time allocation.

## What must be fixed

```text
1. Dense TargetRelationSearch must have closed-form feasibility preflight.
2. Dense schedule materialization must be delayed until after caps pass.
3. Admission collection must be isolated per kernel.
4. Cost-prohibited dense routes must not abort the solve.
5. Declared ladder must choose feasible compact target-direct routes.
6. Universal must not call dense relation search blindly.
```

## What must not be fixed by this repair

```text
- Do not add the reported geometry problem as regression.
- Do not add geometry-specific dispatch.
- Do not normalize the geometry input in solver core.
- Do not change candidate-cover semantics.
- Do not accept fast failure as success.
```

## Generic evidence expected

```text
- synthetic algebraic footprint proving dense preflight works;
- synthetic support-producing algebraic families proving planner routes around dense infeasibility;
- static scan proving no case-specific content;
- cost trace proving dense route was cost-prohibited while compact route succeeded.
```

# ===== GENERIC_PLANNER_QUICK_GUARDIAN_PROMPT.md =====

# Quick Guardian Prompt: Generic Planner Success-Route Repair

Implement `RGDTPK-Q-v4-generic-planner-success-route-v1`.

Read in order:

1. `GENERIC_PLANNER_SUCCESS_ROUTE_BASE_SPEC.md`
2. `GENERIC_PLANNER_SUCCESS_ROUTE_PLAN.md`
3. `GENERIC_PLANNER_SUCCESS_ROUTE_REVIEWER_PROMPTS.md`
4. `GENERIC_PLANNER_ACCEPTANCE_MATRIX.yaml`
5. `GENERIC_PLANNER_AGENT_RESET.md`
6. `GENERIC_PLANNER_CURRENT_DEFECT_AUDIT.md`

Hard constraints:

```text
- Do not include the investigated geometry problem as a test, fixture, benchmark, or gate.
- Do not add geometry-specific or expected-answer dispatch.
- Do not close by returning fast failure for large blocks.
- Fix generic planner routing so infeasible dense TargetRelationSearch cannot block other kernels.
- Add closed-form dense relation-search preflight before any dense support materialization.
- Ensure later kernels and Universal are still collected and planned.
- Add generic algebraic footprint tests only.
```

Closure requires all GPSR phases and reviewer prompts to pass.

# ===== GENERIC_PLANNER_PATCH_NOTES.md =====

# Generic Planner Success-Route Patch Notes

This pack replaces the earlier case-regression framing.

Key correction:

```text
The reported geometry problem must not be part of the repair plan or gate.
```

Reason:

```text
Including that problem in the plan would invite the Agent to optimize toward that fixture.
The correct repair is generic: dense TargetRelationSearch must not block declared success routes on any large algebraic footprint.
```

Expected outcome:

```text
After this generic repair, external experiments may rerun the reported problem, but that run is not part of the Guardian acceptance gate.
```
