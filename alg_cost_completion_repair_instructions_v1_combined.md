# Algebraic-Cost Completion Repair Pack v1



---

# File: ALG_COST_COMPLETION_REPAIR_BASE_SPEC.md

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


---

# File: ALG_COST_COMPLETION_REPAIR_PLAN.md

# Algebraic-Cost Completion Repair Plan v1

Change ID: `RGDTPK-Q-v4-algebraic-cost-completion-repair-v1`

This plan must be executed before any candidate-cover readiness claim is restored.

The diagnostic problem itself must not be added as a permanent test, fixture, benchmark, gate, or acceptance input.

## ACR-P0 — Claim rollback and failure-mode reset

### Required changes

1. Add `ALG_COST_REPAIR_STATUS.md`.
2. Mark prior `CANDIDATE_COVER_CORE_READY` / `SOURCE_FAITHFUL_TO_V4_CANDIDATE_COVER_LAYER` claims as suspended.
3. Add `ALG_COST_AGENT_RESET.md`.
4. Agent must explicitly acknowledge:
   - previous PASS was not sufficient;
   - route existence is not algorithmic sufficiency;
   - preflight is not completion;
   - fast failure is not candidate-cover success;
   - expression swell is a first-class algebraic cost;
   - no diagnostic-problem-specific optimization is allowed.

### Acceptance

PASS requires source files and docs to reflect:

```text
Current max claim:
  CANDIDATE_COVER_PIPELINE_PRESENT_BUT_ALGEBRAIC_COST_INCOMPLETE
```

Reviewer must fail if old closure still claims candidate-cover readiness without this repair.

## ACR-P1 — Source-spec and current-implementation gap audit

### Required artifact

Create:

```text
ALG_COST_SOURCE_SPEC_GAP_MAP.md
```

For each v4 section 1, 3, 4, 12, 13, 17, 18, 19, 20, 23, 24, 25, 30, 32, 33, list:

```yaml
spec_section:
required_algorithmic_obligation:
current_implementation_status:
dominant_costs_accounted_for:
dominant_costs_missing:
risk:
repair_action:
```

### Mandatory audit targets

- `planner/cost_model.rs`
- `planner/ladder.rs`
- `planner/admission.rs`
- `planner/relation_schedule.rs`
- `kernels/target_relation_search.rs`
- `kernels/sparse_resultant.rs`
- `kernels/action_krylov.rs`
- `kernels/specialization_interpolation.rs`
- `kernels/universal_elimination.rs`
- `graph/separators.rs`
- `graph/tree_decomposition.rs`
- `graph/projection_dag.rs`
- `algebra/resultant.rs`
- `types/polynomial.rs`
- `solver/pipeline.rs`

### Acceptance

The audit must identify SparseResultant expression-swell and serial route monopolization as algorithmic-cost blockers. It must not frame them as mere benchmark or timeout tuning issues.

## ACR-P2 — Route budget and dominant-cost architecture

### Required implementation

Add core data structures:

```rust
pub struct AlgebraicWorkEstimate {
    pub local_variable_count: usize,
    pub local_relation_count: usize,
    pub exported_variable_count: usize,
    pub input_term_count: usize,
    pub max_input_terms: usize,
    pub max_total_degree: usize,
    pub max_keep_variable_count: usize,
    pub matrix_rows: Option<usize>,
    pub matrix_cols: Option<usize>,
    pub quotient_rank_estimate: Option<usize>,
    pub predicted_output_terms: Option<SaturatingCount>,
    pub predicted_intermediate_terms: Option<SaturatingCount>,
    pub predicted_coefficient_height_bits: Option<SaturatingCount>,
    pub predicted_work_units: SaturatingCount,
    pub estimate_hash: Hash,
}

pub struct RouteBudget {
    pub max_work_units: SaturatingCount,
    pub max_elapsed_steps: usize,
    pub max_input_terms_per_pair: usize,
    pub max_intermediate_terms: usize,
    pub max_output_terms: usize,
    pub max_keep_variables: usize,
    pub max_total_degree: usize,
    pub max_coefficient_height_bits: usize,
    pub budget_hash: Hash,
}

pub enum RouteCostClass {
    PreferredCompact,
    Feasible,
    ExpensiveButAllowed,
    CostProhibited,
}
```

`KernelExecutionPlan` must carry route-specific `AlgebraicWorkEstimate` and `RouteBudget`, not only generic matrix rows/cols.

### Acceptance

- Cost estimate hash must change when predicted term growth changes.
- Cost class must become `CostProhibited` for large predicted expression swell even if matrix rows/cols are small.
- Tests must prove a route with small matrix dimensions but huge polynomial-entry terms is not considered cheap.

## ACR-P3 — SparseResultant expression-swell-aware planning

### Required changes

Modify `kernels/sparse_resultant.rs`.

1. Replace `probe_sparse_resultant_plan` with a probe that simulates the selected elimination chain without computing resultants.
2. For every candidate pair, estimate:
   - left/right term count;
   - degree in eliminated variable;
   - keep-variable count;
   - Sylvester/Macaulay dimension;
   - determinant entry term product;
   - output term upper-bound or risk class;
   - coefficient-height growth.
3. Pair score must include expression-swell risk.
4. A candidate pair with small matrix but huge entry terms must be rejected or ranked behind safer routes.
5. The probe must produce `SparseResultantSwellPreflight`.

### Acceptance

Create generic tests:

```text
- small matrix / huge entries -> CostProhibited
- larger matrix / tiny sparse entries -> Feasible if exact verification route exists
- pair selection avoids intermediate relations with huge term count
- plan hash changes when swell estimate changes
```

No diagnostic problem file may be used.

## ACR-P4 — SparseResultant bounded execution and backend repair

### Required changes

1. Add runtime guards in `build_sparse_resultant_trace`:
   - max pair input terms;
   - max intermediate output terms;
   - max keep variables;
   - max total degree;
   - max coefficient-height bits;
   - max chain steps;
   - max route work units.
2. If a guard is exceeded, return an allowed route-local `FiniteResourceFailure` or `AlgorithmicHardCase` carrying the route trace.
3. Do not let a guard failure abort the whole solver if later declared routes are allowed.
4. Restrict recursive symbolic determinant to small-entry caps.
5. Add modular/evaluation-interpolation/subresultant backend for larger but feasible resultant routes.
6. The backend must end with exact Q verification.

### Acceptance

- A route that creates a huge intermediate polynomial must stop before the next huge resultant.
- Later declared route must execute after SparseResultant guard failure.
- Recursive determinant test must fail if invoked with large polynomial entries.
- Modular/evaluation-interpolation resultant must produce an exactly verified relation on generic algebraic stress.

## ACR-P5 — Ladder execution must be bounded and non-monopolizing

### Required changes

Modify `solver/pipeline.rs` and `planner/ladder.rs`.

1. Every route in declared ladder must have an enforceable route budget.
2. `execute_block_with_declared_ladder` must record:
   - route start;
   - route success;
   - route allowed failure;
   - route budget stop;
   - route elapsed/work summary.
3. A route-local budget stop must continue to later routes when declared failure behavior allows it.
4. A single route must not monopolize block execution.
5. Aggregate ladder failure must include all attempted route summaries.

### Acceptance

Generic stress:

```text
- first route intentionally exceeds work budget
- second route produces ProjectionMessage
- final result is CertifiedCandidateCover
- route trace shows first route yielded and second route succeeded
```

This must be public or near-public pipeline, not a unit-only helper.

## ACR-P6 — UniversalTargetElimination success-route completion

### Required changes

Modify `kernels/universal_elimination.rs`.

1. Universal internal stages must use the same route budgets and cost estimates as top-level kernels.
2. Universal must skip cost-prohibited dense/sparse stages before execution.
3. Universal must try remaining bounded stages in declared order.
4. Universal must emit verified target/separator relation on generic large-footprint stress.
5. Universal must not finish as a hardcase if an internal lower-cost strategy can produce a relation.
6. Universal's certificate must include:
   - attempted strategies;
   - skipped cost-prohibited strategies;
   - failed strategy hashes;
   - chosen strategy;
   - exact relation verification.

### Acceptance

Generic stress:

```text
dense TRS cost-prohibited
sparse resultant cost-prohibited by swell
Universal chooses TargetAction or Specialization or bounded local elimination
ProjectionMessage verified
CertifiedCandidateCover returned
```

No geometry names or diagnostic input.

## ACR-P7 — Algebraic-cost-aware graph decomposition

### Required changes

Modify graph modules:

- `graph/weighted_primal.rs`
- `graph/separators.rs`
- `graph/tree_decomposition.rs`
- `graph/projection_dag.rs`
- `graph/metrics.rs`

Implement separator scoring that includes:

```text
relation arity
relation degree
monomial count
coefficient-height estimate
predicted local projection cost
linear/definitional eliminability
target distance
separator width
relation duplication certificate cost
```

If a large block remains, planner diagnostics must state why no separator improved estimated cost.

### Acceptance

Generic large-footprint stress must show:

```text
- decomposition reduces max block width when algebraic separators exist;
- if one-large-block remains, diagnostics explain why;
- no geometry-family dispatch or role-based solver branch appears.
```

## ACR-P8 — Sparse/lazy TargetRelationSearch completion

### Required changes

Dense total-degree TRS is not sufficient for large blocks.

Implement at least one sparse/lazy support strategy from the v4 spec:

```text
SparseFromProjectionFootprint
SpecializedInterpolationFootprint
```

The sparse/lazy strategy must:

```text
- build small support descriptors without full dense monomial enumeration;
- generate candidate relations;
- verify membership over Q exactly;
- participate in planner cost estimates;
- be used when dense TRS is cost-prohibited but sparse footprint is feasible.
```

### Acceptance

Generic stress:

```text
dense TRS cost-prohibited
sparse/lazy TRS feasible
sparse/lazy TRS produces verified ProjectionMessage
final result CertifiedCandidateCover
```

## ACR-P9 — Generic large-footprint support-producing stress suite

### Mandatory rule

Do not include the diagnostic problem, its name, its builder, its variable IDs, its polynomial hashes, or expected target value.

### Required stress families

Create at least 8 generated algebraic stress families:

```text
S1: large block, dense TRS prohibited, TargetAction succeeds
S2: large block, dense TRS prohibited, sparse/lazy TRS succeeds
S3: sparse resultant feasible with modular/evaluation backend, exact verified
S4: sparse resultant expression-swell prohibited, later route succeeds
S5: specialization-interpolation succeeds after dense/sparse prohibitions
S6: Universal succeeds after at least two internal strategy failures
S7: graph decomposition reduces a high-cost block through algebraic separators
S8: no useful separator one-large-block still succeeds through bounded Universal
```

Each support-producing stress must assert:

```text
status == CertifiedCandidateCover
support_polynomial is Some and nonzero
verify_global_support passes
replay_run_certificate accepts
diagnostics include route trace and cost trace
no exact-image filtering required
```

### Anti-overfit variants

Each stress family must run at least two deterministic isomorphic variants:

```text
- variable ID permutation
- relation order permutation
- coefficient scaling by nonzero rationals
```

Expected support polynomial must not be hardcoded.

## ACR-P10 — Reviewer red-team and final closure

### Required artifacts

For every phase, create:

```text
reviews/ACR-P*/<timestamp>/prompt.md
reviews/ACR-P*/<timestamp>/response.md
reviews/ACR-P*/<timestamp>/review_summary.yaml
reviews/ACR-P*/<timestamp>/evidence_manifest.yaml
```

Final closure must include:

```text
ALG_COST_COMPLETION_CLOSURE.md
ALG_COST_FINAL_RED_TEAM_RESULTS.md
ALG_COST_ROUTE_BUDGET_AUDIT.md
ALG_COST_DECOMPOSITION_AUDIT.md
ALG_COST_NO_OVERFIT_AUDIT.md
```

### Final claim

Only after all phases pass may the repo restore:

```text
CANDIDATE_COVER_CORE_READY
SOURCE_FAITHFUL_TO_V4_CANDIDATE_COVER_LAYER
```

Full exact-image/source-fidelity labels remain out of scope unless exact-image requirements are also completed.


---

# File: ALG_COST_COMPLETION_REVIEWER_PROMPTS.md

# Algebraic-Cost Completion Repair Reviewer Prompts v1

These prompts are mandatory. A phase cannot close unless its reviewer uses the corresponding prompt and archives prompt, response, review_summary.yaml, and evidence_manifest.yaml.

## Reviewer Meta-Protocol

You are not reviewing whether the code compiles or whether a gate was satisfied. You are reviewing whether the R-GDTPK-Q / ACCTP-Q candidate-cover algorithm is now source-faithful to the v4 algebraic-cost-compression design.

You must fail the phase if the implementation does any of the following:

```text
- treats preflight as completion;
- treats admission/planning as ProjectionMessage success;
- produces fast failure instead of support-producing success where success is required;
- leaves any production route with unbounded symbolic object growth;
- lets a route monopolize the declared ladder;
- relies on geometry names, problem IDs, expected answers, or diagnostic fixtures;
- implements only narrow slices and calls them generic;
- hides a heavy global coordinate-first fallback;
- lacks exact Q verification for produced relations;
- fails to provide route-level cost trace and replay-bound certificates.
```

You must write down at least one adversarial algebraic counterexample for the phase you review, even if the provided tests pass.

## Required `review_summary.yaml`

```yaml
schema_version: alg-cost-review-v1
phase_id: ACR-Px
review_status: PASS|FAIL
phase_closable: true|false
algorithmic_sufficiency: sufficient|insufficient
source_spec_alignment: aligned|misaligned
blocking_findings: []
required_fixes: []
reviewed_files: []
reviewed_tests: []
adversarial_counterexamples:
  - name: ""
    algebraic_footprint: ""
    expected_behavior: ""
    reviewer_result: ""
dominant_cost_checks:
  dense_trs_materialization_bounded: true|false|null
  sparse_resultant_swell_bounded: true|false|null
  route_budget_enforced: true|false|null
  ladder_non_monopolizing: true|false|null
  graph_decomposition_cost_aware: true|false|null
support_producing_checks:
  public_or_near_public_pipeline_used: true|false
  projection_message_verified: true|false
  support_verified: true|false
  replay_accepted: true|false
anti_overfit_checks:
  no_diagnostic_problem_fixture: true|false
  no_geometry_name_dispatch: true|false
  no_expected_answer_dispatch: true|false
```

If any boolean under required checks is false, review_status must be FAIL.

## ACR-P0 Reviewer Prompt

Review the claim rollback and agent reset.

You must verify:

1. old candidate-cover readiness is suspended;
2. the Agent explicitly acknowledges previous false-PASS modes;
3. the new max claim is `CANDIDATE_COVER_PIPELINE_PRESENT_BUT_ALGEBRAIC_COST_INCOMPLETE`;
4. no file still presents old closure as current truth.

Fail if the wording allows "we already passed before; this is only a performance patch."

## ACR-P1 Reviewer Prompt

Review `ALG_COST_SOURCE_SPEC_GAP_MAP.md`.

You must compare the implementation against v4 sections 1, 3, 4, 12, 13, 17, 18, 19, 20, 23, 24, 25, 30, 32, and 33.

You must fail if the gap map misses any of:

```text
- SparseResultant expression swell;
- serial route monopolization;
- dense TRS large-block materialization risk;
- route-level budget absence;
- graph decomposition leaving high-cost blocks without evidence;
- Universal not guaranteeing a bounded success route;
- lack of sparse/lazy TargetRelationSearch for large blocks;
- planner confusing matrix dimensions with actual symbolic work.
```

## ACR-P2 Reviewer Prompt

Review route budget architecture.

Inspect `KernelExecutionPlan`, `ResourceBounds`, `RouteBudget`, `AlgebraicWorkEstimate`, cost trace, and related hashes.

You must fail if:

```text
- route budget exists only in docs;
- budget does not include expression-growth fields;
- cost estimates still rank by kernel name and matrix rows/cols only;
- plan hash is unchanged when dominant-cost estimates change;
- route budget is not replay/certificate-bound.
```

## ACR-P3 Reviewer Prompt

Review SparseResultant expression-swell planning.

Inspect `kernels/sparse_resultant.rs` and `planner/cost_model.rs`.

You must fail if:

```text
- admission checks only matrix rows/cols;
- pair scoring ignores input term count or keep-variable count;
- a small Sylvester matrix with large polynomial entries can be ranked as cheap;
- intermediate output term-growth is not estimated;
- dangerous pair chains are still admitted as Feasible;
- tests use the diagnostic problem or its values.
```

You must construct an adversarial pair of polynomials with:
- small resultant matrix dimension;
- hundreds or thousands of terms in entries;
- many keep variables.

The review must state whether it is cost-prohibited before execution.

## ACR-P4 Reviewer Prompt

Review SparseResultant bounded execution and backend.

You must fail if:

```text
- `build_sparse_resultant_trace` can compute indefinitely without checking term growth;
- recursive symbolic determinant is called on large-entry templates;
- modular/evaluation/subresultant backend lacks exact Q verification;
- route guard failure aborts the entire solver instead of allowing later declared routes when allowed;
- runtime guard evidence is not included in diagnostics/cost trace.
```

You must inspect tests proving later routes run after SparseResultant guard stop.

## ACR-P5 Reviewer Prompt

Review declared ladder execution.

You must fail if:

```text
- route budgets are not enforced in `execute_block_with_declared_ladder`;
- route failure trace is missing;
- first route can monopolize the solver;
- aggregate failure hides individual route causes;
- tests only verify fast failure, not next-route success.
```

A required reviewer challenge:

```text
Build or inspect a generic stress where first route is budget-stopped and second route returns a verified ProjectionMessage.
```

## ACR-P6 Reviewer Prompt

Review UniversalTargetElimination.

You must fail if:

```text
- Universal is merely last in ladder but not a real bounded projector;
- internal stages do not carry route budgets;
- cost-prohibited stages are executed anyway;
- no generic stress shows Universal producing a message after internal stage failures;
- Universal uses full coordinate roots, full coordinate RUR, or hidden global fallback;
- Universal returns hardcase while a feasible bounded internal route exists.
```

## ACR-P7 Reviewer Prompt

Review algebraic-cost-aware graph decomposition.

You must inspect graph weighting, separator scoring, and projection DAG construction.

You must fail if:

```text
- decomposition ignores relation degree, arity, monomial count, coefficient height;
- high-cost block remains without diagnostic explanation;
- separator improvement is measured only by variable count;
- relation duplication lacks certificate;
- geometry names or variable roles drive solver dispatch.
```

Reviewer must construct at least one generic hypergraph where cost-aware decomposition must split a block that variable-count-only scoring would keep.

## ACR-P8 Reviewer Prompt

Review sparse/lazy TargetRelationSearch.

You must fail if:

```text
- DenseTotalDegree remains the only production relation-search strategy;
- sparse/lazy strategy lacks exact Q membership verification;
- sparse/lazy support descriptors still enumerate dense monomials;
- large-block stress cannot produce a message when dense TRS is prohibited;
- sparse heuristic output is accepted without exact membership proof.
```

## ACR-P9 Reviewer Prompt

Review generic large-footprint stress suite.

You must fail if:

```text
- any stress contains the diagnostic problem name or imported file;
- any expected support polynomial is hardcoded;
- fewer than 8 required stress families exist;
- support-producing cases return failure statuses;
- tests only use helper-level APIs;
- no anti-overfit variants exist;
- no route trace proves which route succeeded.
```

For each stress family, reviewer must record:

```yaml
dense_trs_status:
sparse_resultant_status:
successful_route:
projection_message_verified:
support_verified:
replay_accepted:
```

## ACR-P10 Reviewer Prompt

Review final closure.

You must fail closure unless all previous reviews are PASS and the final artifacts demonstrate:

```text
- no diagnostic fixture dependency;
- all production routes bounded by dominant algebraic costs;
- SparseResultant expression swell cannot monopolize execution;
- dense TRS cannot materialize huge supports;
- graph decomposition is cost-aware;
- Universal emits verified messages in generic large-footprint stress;
- public or near-public pipeline returns CertifiedCandidateCover on required support-producing stress;
- exact Q support verification and replay succeed.
```

Do not PASS merely because the repo says `CANDIDATE_COVER_CORE_READY`.


---

# File: ALG_COST_ACCEPTANCE_MATRIX.yaml

schema_version: alg-cost-completion-acceptance-v1

claim_policy:
  suspended_until_all_pass:
    - CANDIDATE_COVER_CORE_READY
    - SOURCE_FAITHFUL_TO_V4_CANDIDATE_COVER_LAYER
  max_claim_during_repair:
    - CANDIDATE_COVER_PIPELINE_PRESENT_BUT_ALGEBRAIC_COST_INCOMPLETE
  forbidden_always_without_exact_image:
    - SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC
    - RGDTPK_Q_V4_ACCEPTANCE_COMPLETE

anti_overfit:
  forbidden_strings_in_tests_and_code:
    - mixtilinear
    - mixtilinear_candidate_cover_problem
    - circumcircle
    - incircle
    - tangent_solver
    - expected_cos
  forbidden_dependency:
    - diagnostic_problem_file
    - known_support_polynomial
    - specific_external_problem_hash

route_cost_requirements:
  dense_target_relation_search:
    closed_form_preflight: required
    no_materialization_before_feasible: required
    sparse_or_lazy_strategy_for_large_blocks: required
  sparse_resultant_projection:
    expression_swell_preflight: required
    runtime_growth_guards: required
    recursive_determinant_large_entries: forbidden
    modular_or_evaluation_backend_with_exact_verification: required
  declared_ladder:
    route_budget: required
    route_failure_trace: required
    later_route_runs_after_allowed_failure: required
  universal:
    bounded_internal_stages: required
    verified_projection_message_on_generic_stress: required
  graph_decomposition:
    algebraic_cost_aware_separator_scoring: required
    large_block_explanation: required

support_producing_stress:
  required_families:
    - id: S1
      name: large_block_dense_trs_prohibited_target_action_succeeds
      required_status: CertifiedCandidateCover
    - id: S2
      name: large_block_dense_trs_prohibited_sparse_lazy_trs_succeeds
      required_status: CertifiedCandidateCover
    - id: S3
      name: sparse_resultant_modular_backend_succeeds
      required_status: CertifiedCandidateCover
    - id: S4
      name: sparse_resultant_swell_prohibited_later_route_succeeds
      required_status: CertifiedCandidateCover
    - id: S5
      name: specialization_interpolation_succeeds_after_route_prohibition
      required_status: CertifiedCandidateCover
    - id: S6
      name: universal_succeeds_after_internal_stage_failures
      required_status: CertifiedCandidateCover
    - id: S7
      name: decomposition_reduces_high_cost_block
      required_status: CertifiedCandidateCover
    - id: S8
      name: one_large_block_bounded_universal_success
      required_status: CertifiedCandidateCover
  per_case_required_evidence:
    - public_or_near_public_pipeline
    - projection_message_verified
    - support_polynomial_nonzero
    - exact_Q_support_verification
    - replay_accepts
    - cost_trace_present
    - successful_route_recorded
    - dense_trs_status_recorded
    - sparse_resultant_status_recorded
  anti_overfit_variants_minimum: 2
  variants:
    - variable_id_permutation
    - relation_order_permutation
    - nonzero_rational_scaling

review_archive:
  required_per_phase:
    - prompt.md
    - response.md
    - review_summary.yaml
    - evidence_manifest.yaml
  required_final:
    - ALG_COST_COMPLETION_CLOSURE.md
    - ALG_COST_FINAL_RED_TEAM_RESULTS.md
    - ALG_COST_ROUTE_BUDGET_AUDIT.md
    - ALG_COST_DECOMPOSITION_AUDIT.md
    - ALG_COST_NO_OVERFIT_AUDIT.md

failure_conditions:
  fail_if:
    - route_monopolizes_ladder
    - sparse_resultant_swell_unbounded
    - dense_trs_materializes_before_preflight
    - planned_is_treated_as_executed
    - helper_only_stress
    - failure_status_used_for_support_producing_stress
    - diagnostic_problem_fixture_used
    - geometry_dispatch_used
    - expected_answer_dispatch_used


---

# File: ALG_COST_AGENT_RESET_TEMPLATE.md

# Algebraic-Cost Completion Agent Reset Template v1

Before writing code, fill this out and commit it under the change directory.

## 1. Previous false-PASS acknowledgement

I acknowledge that the prior candidate-cover readiness judgment was insufficient because it did not ensure source-faithful algebraic-cost compression on large geometry-derived algebraic footprints.

I acknowledge that:
- a route can be semantically correct on small cases and still be algorithmically incomplete;
- a small matrix dimension is not sufficient cost evidence;
- expression swell is a first-class algebraic cost;
- preflight and planning are not execution success;
- fast failure is not candidate-cover success;
- review evidence is not a substitute for a working algorithm.

## 2. Current repair target

The target is not to pass a diagnostic problem.

The target is to implement the v4 algebraic-cost-compressed candidate-cover algorithm so that large-footprint algebraic inputs can be processed through bounded target-direct routes.

## 3. Forbidden shortcuts

I will not:
- add diagnostic problem as a fixture;
- branch on geometry names;
- branch on expected answers;
- branch on problem hashes;
- claim readiness because routes are merely admitted;
- pass support-producing stress with a failure status;
- rely on recursive determinant expansion for large symbolic entries;
- leave an unbounded route in a declared ladder.

## 4. Required proof mindset

For each route, I will ask:
- what algebraic objects dominate its cost?
- are those objects bounded before construction?
- can the route yield to a later declared route?
- can it produce a verified ProjectionMessage?
- is the final support verified over Q?


---

# File: ALG_COST_CURRENT_DEFECT_AUDIT.md

# Algebraic-Cost Completion Current Defect Audit v1

## Defect class

The latest diagnostic report shows a post-GSR timeout in projection execution, not planning. The first executable route for the large block is `SparseResultantProjection`; it enters an exact sparse-resultant chain with severe expression swell.

## Direct evidence to account for

The report records:
- validation/canonicalization/compression/graph/DAG/planning complete in about 0.2 seconds;
- block 5 has 33 local variables, 29 relations, 2 exports;
- declared ladder begins with SparseResultantProjection;
- step 5 produces a 628,925-term intermediate relation;
- step 15 starts a 3x3 Sylvester resultant on 1,588-term and 16,662-term inputs;
- no compute end line appears before timeout.

## Why previous PASS was invalid

The previous closure focused on candidate-cover semantic correctness and some generic stress. It did not require every production route to be bounded by its real dominant algebraic cost.

The missing requirement was:

```text
A production route must not be admitted, prioritized, or executed solely because a shallow proxy
(matrix dimension, route name, or template existence) looks small.
```

## Required correction

This repair must introduce:
- expression-swell-aware SparseResultant planning;
- runtime growth guards;
- bounded alternate resultant backend;
- non-monopolizing ladder route budgets;
- graph/decomposition cost awareness;
- sparse/lazy TRS;
- Universal success-route evidence;
- generic large-footprint support-producing stress.

## Not acceptable

- Adding the diagnostic problem as a regression gate.
- Returning a faster failure and calling that success.
- Moving SparseResultant after another route without bounding it.
- Disabling SparseResultant entirely without providing generic success routes.
- Keeping recursive symbolic determinant as an unbounded production backend.


---

# File: ALG_COST_QUICK_GUARDIAN_PROMPT.md

# Quick Guardian Prompt: Algebraic-Cost Completion Repair v1

You are implementing `RGDTPK-Q-v4-algebraic-cost-completion-repair-v1`.

Do not treat this as a local timeout fix. This is a source-spec repair caused by a false PASS: the implementation did not fully satisfy the v4 algebraic-cost-compressed candidate-cover algorithm.

Read these files first:

```text
ALG_COST_COMPLETION_REPAIR_BASE_SPEC.md
ALG_COST_COMPLETION_REPAIR_PLAN.md
ALG_COST_COMPLETION_REVIEWER_PROMPTS.md
ALG_COST_ACCEPTANCE_MATRIX.yaml
ALG_COST_AGENT_RESET_TEMPLATE.md
ALG_COST_CURRENT_DEFECT_AUDIT.md
```

Mandatory principles:

```text
- Do not add the diagnostic geometry problem as a test, fixture, benchmark, or gate.
- Do not branch on geometry names, variable IDs, relation IDs, problem hashes, or expected answers.
- Do not pass by returning faster failures.
- Do not confuse admission/planning with ProjectionMessage success.
- Do not let any production route monopolize execution.
- Do not leave SparseResultant expression swell unbounded.
- Do not use recursive symbolic determinant for large polynomial entries.
- Do not claim candidate-cover readiness until all ACR phases pass.
```

Required outcome:

```text
Generic large-footprint algebraic stress must reach CertifiedCandidateCover through public or near-public pipeline, with exact Q support verification and replay, while dense TRS and dangerous SparseResultant routes are bounded and cannot block later routes.
```

Proceed phase-by-phase:

```text
ACR-P0 -> ACR-P1 -> ACR-P2 -> ACR-P3 -> ACR-P4 -> ACR-P5 -> ACR-P6 -> ACR-P7 -> ACR-P8 -> ACR-P9 -> ACR-P10
```

Each phase must be reviewed with the corresponding reviewer prompt. Do not close a phase without archived reviewer prompt, response, review_summary.yaml, and evidence_manifest.yaml.
