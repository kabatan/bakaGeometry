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
