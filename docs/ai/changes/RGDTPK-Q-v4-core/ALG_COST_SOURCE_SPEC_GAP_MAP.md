# Algebraic-Cost Completion Source-Spec Gap Map

Status: ACR-P1 gap audit.

Change: `RGDTPK-Q-v4-algebraic-cost-completion-repair-v1`

Authority: diagnostic mapping from v4 source obligations to current implementation gaps. This file
does not close any requirement and does not restore readiness claims.

Current maximum claim remains:

```text
CANDIDATE_COVER_PIPELINE_PRESENT_BUT_ALGEBRAIC_COST_INCOMPLETE
```

## Audit Scope

Mandatory audit targets inspected for this gap map:

```text
planner/cost_model.rs
planner/ladder.rs
planner/admission.rs
planner/relation_schedule.rs
kernels/target_relation_search.rs
kernels/sparse_resultant.rs
kernels/action_krylov.rs
kernels/specialization_interpolation.rs
kernels/universal_elimination.rs
graph/separators.rs
graph/tree_decomposition.rs
graph/projection_dag.rs
algebra/resultant.rs
types/polynomial.rs
solver/pipeline.rs
```

## Gap Entries

```yaml
- spec_section: "Appendix A section 1 / RGQ-000 source authority"
  required_algorithmic_obligation: >
    The solver's claimed strength is algebraic-cost compression: avoid huge full-coordinate
    objects by using local block width, separator width, quotient/action rank, sparse template
    size, final support degree, and certificate cost.
  current_implementation_status: >
    The semantic candidate-cover pipeline exists, but route selection and sparse-resultant
    execution can still construct huge symbolic intermediate objects.
  dominant_costs_accounted_for:
    - generic matrix rows/cols in KernelCostEstimate
    - quotient rank estimate
    - coefficient height estimate
    - existing GlobalCostTrace block/matrix fields
  dominant_costs_missing:
    - route-specific expression-swell estimate
    - intermediate relation term growth
    - route elapsed/work budget
    - determinant polynomial-entry work
    - sparse/lazy support descriptor cost
  risk: >
    A small apparent template can be selected while the actual algebraic object growth is
    unbounded, invalidating the source's cost-compression premise.
  repair_action: "ACR-P2 through ACR-P9: route estimates, budgets, bounded kernels, stress suite."

- spec_section: "Appendix A section 3 / RGQ-004-RGQ-006 forbidden paths and failure semantics"
  required_algorithmic_obligation: >
    Production paths must not use hidden fallbacks or geometry dispatch, and failures must report
    algebraic blockers with evidence rather than Unsupported.
  current_implementation_status: >
    Static structure mostly avoids geometry dispatch and hidden coordinate roots. However, a
    declared route can monopolize execution without producing a bounded failure status.
  dominant_costs_accounted_for:
    - dense TRS preflight cost-prohibited diagnostics
    - route-local allowed failure statuses for some kernel errors
  dominant_costs_missing:
    - route-local budget-stop status and trace
    - dominant-cost evidence for SparseResultant and Universal internal stages
  risk: >
    A well-formed input can time out inside a declared route instead of returning a finite resource
    or algorithmic hard-case status with route evidence.
  repair_action: "ACR-P2 and ACR-P5: route budget model and non-monopolizing ladder execution."

- spec_section: "Appendix A section 4 / RGQ-011-RGQ-016 top-level pipeline and ladder"
  required_algorithmic_obligation: >
    Build DAG, plan deterministic declared ladders, execute target/separator kernels, compose,
    verify global support, isolate roots, decode candidates, and record algebraic cost blockers.
  current_implementation_status: >
    The public pipeline and declared ladder exist. `execute_block_with_declared_ladder` iterates
    serially and continues on allowed route errors, but it does not enforce per-route algebraic work
    budgets.
  dominant_costs_accounted_for:
    - global trace total variables/relations/terms
    - block matrix rows/cols
    - failure trace for returned route errors
  dominant_costs_missing:
    - route start/success/budget-stop summary in the public run trace
    - execution-time work metering
    - aggregate ladder summary for non-returning route attempts
  risk: >
    Admission/planning can be confused with operational support-producing success, and a serial
    first route can block every later declared route.
  repair_action: "ACR-P5: enforce budgets and record all attempted route outcomes."

- spec_section: "Appendix A section 12 / graph decomposition"
  required_algorithmic_obligation: >
    Construct algebraic weighted graphs, choose separators that improve estimated local projection
    cost, and explain one-large-block outcomes when no useful separator exists.
  current_implementation_status: >
    Graph metrics include algebraic fields such as arity, degree, monomial count, coefficient
    height, dense-TRS class, quotient rank, and sparse template size. Separator scoring has some
    algebraic penalties, but there is no complete route-dominant cost model shared with planner
    budgets, and one-large-block explanations are limited.
  dominant_costs_accounted_for:
    - relation arity/degree/monomial metrics
    - separator width penalty
    - target distance hint
    - affine/definitional counts
    - dense TRS and quotient-rank hints
  dominant_costs_missing:
    - predicted local projection cost using route-specific budgets
    - relation duplication certificate-cost accounting in separator decisions
    - diagnostic proof that no separator improved cost
  risk: >
    The decomposition can leave a high-cost block without sufficient evidence that algebraic
    separator choices were exhausted.
  repair_action: "ACR-P7: share algebraic work estimates with graph separator scoring and diagnostics."

- spec_section: "Appendix A section 13 / planner"
  required_algorithmic_obligation: >
    Planner cost estimates must be deterministic and must rank declared ladder routes by real
    dominant algebraic cost, not by kernel name or shallow matrix dimensions.
  current_implementation_status: >
    `KernelCostEstimate` uses generic Macaulay rows/cols, quotient rank, coefficient height, fixed
    certificate cost, and kernel-order penalty. `KernelExecutionPlan` has `ResourceBounds`, but no
    route-specific `AlgebraicWorkEstimate` or `RouteBudget`.
  dominant_costs_accounted_for:
    - matrix row/column product
    - quotient rank estimate
    - fixed certificate cost
    - fixed kernel ordering
  dominant_costs_missing:
    - predicted output/intermediate terms
    - route-specific dominant work units
    - SparseResultant determinant polynomial-entry work
    - TargetAction basis/normal-form work
    - Specialization sample/interpolation support work
    - Universal internal strategy costs
  risk: >
    `SparseResultantProjection` can be ranked ahead of safer routes because its matrix dimensions
    look small, even when expression swell risk is huge.
  repair_action: "ACR-P2 and ACR-P3: add algebraic estimates/budgets and integrate them into cost hashing and ordering."

- spec_section: "Appendix A section 17 / TargetRelationSearch"
  required_algorithmic_obligation: >
    Build target/separator relations through deterministic support schedules, without dense
    materialization before feasibility is proven, and support sparse/lazy footprints for large
    blocks.
  current_implementation_status: >
    Dense total-degree preflight now produces structured cost-prohibited diagnostics and prevents
    obvious huge materialization. Sparse/lazy relation search remains incomplete as a production
    route for large-block success.
  dominant_costs_accounted_for:
    - export support count
    - multiplier support count
    - row count
    - matrix column count
    - memory estimate
  dominant_costs_missing:
    - production sparse/lazy support descriptor strategy
    - exact Q membership verification for sparse/lazy TRS output
    - large-footprint success stress where dense is prohibited and sparse/lazy succeeds
  risk: >
    Dense TRS can correctly decline large blocks, but the solver still lacks the required alternate
    sparse/lazy support-producing workhorse for those blocks.
  repair_action: "ACR-P8: implement SparseFromProjectionFootprint or SpecializedInterpolationFootprint."

- spec_section: "Appendix A section 18 / SparseResultantProjection"
  required_algorithmic_obligation: >
    Use sparse resultant/eliminant templates only when finite and exactly verifiable, with costs
    dominated by sparse template size and bounded algebraic object growth.
  current_implementation_status: >
    Planning checks finite template applicability and matrix dimension. Execution uses exact
    Sylvester resultants and recursive symbolic determinant expansion. It does not preflight or
    guard expression swell.
  dominant_costs_accounted_for:
    - Sylvester matrix dimension
    - cumulative template matrix rows/cols
    - final relation degree bound
  dominant_costs_missing:
    - selected pair term counts
    - keep-variable count
    - determinant entry product work
    - predicted output term count
    - intermediate relation term cap
    - coefficient-height growth cap
    - backend choice policy for large entries
  risk: >
    A 3x3 resultant with thousands of terms can be treated as cheap and can monopolize execution.
  repair_action: "ACR-P3 and ACR-P4: swell-aware planning, runtime guards, and bounded backend policy."

- spec_section: "Appendix A section 19 / TargetActionKrylov"
  required_algorithmic_obligation: >
    When quotient/action rank is feasible, produce a target annihilator only with verified
    characteristic support coverage, without coordinate roots or full coordinate RUR.
  current_implementation_status: >
    A TargetActionKrylov route exists with certificate structures and coverage checks. Its planner
    cost remains generic and is not compared against SparseResultant swell risk by a shared route
    budget model.
  dominant_costs_accounted_for:
    - quotient rank estimate in generic cost model
    - action matrix/certificate cost trace fields
  dominant_costs_missing:
    - basis construction work estimate
    - normal-form certificate cost estimate
    - route budget tied to quotient rank and basis/matvec work
  risk: >
    A compact action route can be placed behind a dangerous SparseResultant route due to shallow
    cost ordering.
  repair_action: "ACR-P2: represent TargetAction dominant costs; ACR-P9: support-producing stress where TargetAction succeeds after prohibited routes."

- spec_section: "Appendix A section 20 / UniversalTargetElimination"
  required_algorithmic_obligation: >
    Universal is a planned generic target/separator projection kernel, not a hidden fallback. It
    must try bounded internal strategies, skip cost-prohibited ones, and emit verified relations
    when a lower-cost strategy is feasible.
  current_implementation_status: >
    Universal has a fixed strategy sequence and records attempted/chosen/failed strategy hashes.
    Internal stages do not yet share route budgets and dominant-cost estimates with top-level
    kernels, and support-producing Universal stress is not complete for the new algebraic-cost bar.
  dominant_costs_accounted_for:
    - fixed strategy sequence
    - failed strategy hash prefix
    - local Groebner cap guard
  dominant_costs_missing:
    - per-internal-stage route budget
    - cost-prohibited skip evidence for sparse-resultant swell
    - verified Universal success after dense and sparse internal stage failures
  risk: >
    Universal can still behave as a failure aggregator rather than a bounded production success
    route for large generic footprints.
  repair_action: "ACR-P6: Universal internal budgets, skip records, and support-producing stress."

- spec_section: "Appendix A section 23 / SpecializationInterpolation"
  required_algorithmic_obligation: >
    Use specialization/interpolation to reduce separator coefficient swell, then accept only after
    exact Q verification.
  current_implementation_status: >
    A specialization/interpolation kernel exists with multiseparator interpolation and exact
    verification concepts. Its sample count, interpolation support size, inner target-only cost,
    and verification cost are not route-budget-bound in KernelExecutionPlan.
  dominant_costs_accounted_for:
    - degree bound
    - sample/interpolation helpers
    - exact verification route in certificates
  dominant_costs_missing:
    - sample count budget
    - interpolation support size estimate
    - inner route dominant-cost estimate
    - route failure trace for prohibited inner strategies
  risk: >
    The planner cannot reliably prefer specialization/interpolation when it is the bounded route or
    prohibit it when interpolation support is too large.
  repair_action: "ACR-P2 and ACR-P9: budget fields and stress family where specialization succeeds after prohibitions."

- spec_section: "Appendix A section 24 / composition and final support"
  required_algorithmic_obligation: >
    Compose verified projection messages without reconstructing full coordinates, eliminate
    separators through target-direct kernels, and build nonzero target support or evidence-backed
    failure.
  current_implementation_status: >
    Composition and Route B support verification exist. Separator elimination invokes target-direct
    planning over message relations, but inherited route budgets and route-attempt summaries are
    not fully represented.
  dominant_costs_accounted_for:
    - composition relation count before/after
    - support verification route evidence
  dominant_costs_missing:
    - separator-elimination route budget trace
    - composition cost tied to separator width and support degree
  risk: >
    Separator composition can be semantically correct without proving cost-bounded route behavior
    in the subproblem planner.
  repair_action: "ACR-P5 and ACR-P9: route traces and replay evidence for public support-producing cases."

- spec_section: "Appendix A section 25 / verification and replay"
  required_algorithmic_obligation: >
    Verify every projection message and global support over Q, and replay the run certificate
    against source-bound plans and costs.
  current_implementation_status: >
    Message verification, global support verification, and replay exist. The current certificate
    and trace language does not bind route-specific algebraic work estimates and budgets for all
    production kernels.
  dominant_costs_accounted_for:
    - certificate route kind
    - projection cost trace fields
    - replay-bound plan hashes
  dominant_costs_missing:
    - route budget hash in KernelExecutionPlan
    - algebraic work estimate hash in KernelExecutionPlan
    - route outcome trace replay binding
  risk: >
    Replay can confirm the chosen relation but not prove that the route obeyed its intended
    algebraic work budget.
  repair_action: "ACR-P2, ACR-P5, and ACR-P10: hash-bound route budget/work estimates and final route budget audit."

- spec_section: "Appendix A section 30 / cost design"
  required_algorithmic_obligation: >
    Cost trace must record local projection costs in terms of block width, separator width,
    quotient/action rank, sparse template/relation-search matrix sizes, final support degree, and
    certificate size.
  current_implementation_status: >
    `GlobalCostTrace` records many coarse fields, but not the route-specific dominant costs that
    caused the timeout: input pair terms, keep variables, predicted output terms, determinant work,
    and route work units.
  dominant_costs_accounted_for:
    - total n/m/s/d/h
    - max block width
    - max separator width
    - matrix rows/cols
    - quotient rank
    - final support degree
  dominant_costs_missing:
    - predicted and observed route work units
    - predicted/observed intermediate term growth
    - route budget stop reason
    - Universal internal stage cost summary
  risk: >
    Cost trace can look complete while omitting the actual algebraic object that made the algorithm
    non-terminating within budget.
  repair_action: "ACR-P2/P5/P10: expand cost trace and audit."

- spec_section: "Appendix A section 32 / geometry-derived footprint"
  required_algorithmic_obligation: >
    Use algebraic footprints such as sparsity, low degree, small separators, affine eliminability,
    and explicit towers; never dispatch on geometry names or problem identity.
  current_implementation_status: >
    Current code does not intentionally dispatch on the diagnostic problem. Existing graph metrics
    use algebraic signals. However, stress and implementation must be strengthened without
    importing the diagnostic geometry file or its details.
  dominant_costs_accounted_for:
    - algebraic graph metrics
    - dense TRS footprint estimates
  dominant_costs_missing:
    - generic large-footprint stress variants proving no overfit
    - static scan tied to ACR completion artifacts
  risk: >
    A repair could accidentally become a local timeout workaround rather than a generic
    algebraic-footprint algorithm.
  repair_action: "ACR-P9 and ACR-P10: anti-overfit stress variants and no-overfit audit."

- spec_section: "Appendix A section 33 / completion criteria"
  required_algorithmic_obligation: >
    Every well-formed Q-polynomial target system enters the generic pipeline; successful cases
    return verified support and replay; failures have evidence-backed statuses; hidden fallback and
    narrow-slice completion are impossible.
  current_implementation_status: >
    The generic pipeline exists, but algebraic-cost completion is not established because
    production routes can be admitted/planned without dominant-cost budgets and support-producing
    large-footprint stress is incomplete under the ACR requirements.
  dominant_costs_accounted_for:
    - public pipeline status/certificate structure
    - generic route registry
    - some previous success-route stress
  dominant_costs_missing:
    - eight ACR large-footprint support-producing stress families
    - anti-overfit variants for each stress family
    - final route budget/decomposition/no-overfit audits
  risk: >
    The repository can overclaim readiness while still lacking the algebraic-cost-compressed
    success behavior required by v4.
  repair_action: "ACR-P9/P10: full stress suite, reviewers, closure artifacts."
```

## Cross-Cutting Blockers

1. `SparseResultantProjection` expression swell is a correctness blocker for algebraic-cost
   completion, not just a performance issue.
2. Serial ladder execution is acceptable only if every declared route has an enforceable budget and
   returns an allowed route-local failure before monopolizing execution.
3. Dense `TargetRelationSearch` preflight is necessary but not sufficient; sparse/lazy support
   production is still required for large blocks.
4. Planner cost must include route-specific dominant work estimates rather than generic matrix
   dimensions and fixed kernel order.
5. Graph decomposition must explain high-cost one-block outcomes and use route cost predictions in
   separator scoring.
6. Universal must be a bounded success route with verified messages in generic large-footprint
   stress, not only a final aggregate failure route.

## ACR-P1 Conclusion

The current implementation is correctly classified as:

```text
CANDIDATE_COVER_PIPELINE_PRESENT_BUT_ALGEBRAIC_COST_INCOMPLETE
```

The next required work is ACR-P2 route budget and dominant-cost architecture.
