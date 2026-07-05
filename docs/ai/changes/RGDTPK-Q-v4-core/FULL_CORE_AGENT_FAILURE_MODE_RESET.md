# Full Core Agent Failure-Mode Reset

Status: active execution constraint for Full Core Repair.

This file is created for `FCR-P0A`. It is not a confession, summary, or retrospective note. It is a
runtime constraint for every later FCR phase and every handoff.

## Operating Acknowledgement

I will treat Full Core Repair as implementation of the production R-GDTPK-Q candidate-cover solver
core, not as a document or review-gate exercise.

The following hazards are active constraints:

- Passing Guardian gates is not implementation.
- Documenting a limitation is not implementing the missing algorithm.
- A minimum example is not a generic mechanism.
- A module-only test is not a production pipeline proof.
- A reviewer PASS is invalid if the prompt did not force algorithmic sufficiency review.
- Specialized kernels are optimizers inside the generic pipeline, not the whole solver.
- If a v4 obligation cannot be implemented without narrowing scope, I must raise
  `AlgorithmDefect` instead of shrinking scope.

## Failure Modes And Concrete Prevention

```yaml
- failure_mode: gate_or_review_pass_substitutes_for_algorithm
  concrete_prevention_in_this_phase: >
    Every FCR phase must point to production code paths, public or near-public pipeline execution,
    and exact replay/certificate behavior. Review archives are supporting evidence only.

- failure_mode: narrow_algebraic_shape_treated_as_generic
  concrete_prevention_in_this_phase: >
    Any success path limited to target-univariate, affine-only, alias-univariate, binary-only,
    explicit-tower-only, triangular-only, or fixture-shaped input must be marked partial and either
    generalized, quarantined, or removed from production reachability.

- failure_mode: heavy_fallback_hidden_under_generic_name
  concrete_prevention_in_this_phase: >
    Generic fallback labels must be backed by declared deterministic bounds, exact-Q certificates,
    resource accounting, and allowed failure semantics. Unbounded global elimination, QE/CAD,
    coordinate solution export, and hidden fallback are forbidden.

- failure_mode: plan_time_execution_hidden_as_planning
  concrete_prevention_in_this_phase: >
    Planning may select schedules, bounds, and cost/resource probes only. Any plan path that
    constructs final output relations, support polynomials, regular-chain projections, resultants,
    norm traces, or interpolated relations must trigger PlanDefect unless moved to execute with
    replayable certificates.

- failure_mode: helper_or_module_test_substitutes_for_public_pipeline
  concrete_prevention_in_this_phase: >
    Acceptance evidence must run through `api::solve_target` or a near-public pipeline that
    constructs DAG blocks, plans and executes kernels, composes projection messages, verifies
    support, squarefrees, isolates roots, decodes candidates, and emits a run certificate.

- failure_mode: certificate_or_dag_is_decorative
  concrete_prevention_in_this_phase: >
    Certificates must bind source relations, output relations, plan hashes, block authorization,
    child-message dependencies, replay hashes, and cost/resource traces where required. Synthetic
    all-relations replay cannot justify final candidate-cover readiness.

- failure_mode: exact_image_or_source_fidelity_overclaim
  concrete_prevention_in_this_phase: >
    FCR may target `CANDIDATE_COVER_CORE_READY` only. `EXACT_IMAGE_CORE_READY`,
    `SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC`, and `RGDTPK_Q_V4_ACCEPTANCE_COMPLETE` remain forbidden
    until exact-image semantics and final source-fidelity work are implemented and reviewed.
```

## PlanDefect / AlgorithmDefect Trigger Table

| Trigger | Classification | Required Response |
| --- | --- | --- |
| A phase plan permits a narrow algebraic shape as the only support-producing path. | `PlanDefect` | Stop the phase, amend the plan or downgrade the claim before code changes continue. |
| A production registry reaches non-production, test-only, fake, or fixture-specific code. | `PlanDefect` | Quarantine/remove the path or replace it with a production implementation before claiming progress. |
| A required v4 candidate-cover algorithm cannot be implemented without narrowing scope. | `AlgorithmDefect` | Stop and report the missing algorithmic obligation; do not replace it with a limited slice. |
| A planner computes final output relations or support polynomials instead of a replayable plan. | `PlanDefect` | Move relation construction to execute or classify the path as non-production. |
| `api::solve_target` cannot be connected without bypassing DAG/planner/kernel/message/compose. | `AlgorithmDefect` | Stop and report the missing production pipeline mechanism. |
| A reviewer prompt mainly checks evidence consistency rather than algorithmic sufficiency. | `PlanDefect` | Repair the reviewer prompt and rerun the review. |
| A support-producing case passes through a helper that skips the production pipeline. | `PlanDefect` | Move the case to test support only or make it pass through the production pipeline. |
| Actual DAG/block replay cannot replace synthetic all-relations replay for final claims. | `AlgorithmDefect` | Stop before any final readiness claim and implement replay or report the blocker. |
| A specialized kernel is the only reason a required stress category succeeds. | `PlanDefect` | Add the generic path required by v4 or quarantine the specialized path as an optimizer only. |
| A later-phase exact-image function is needed to justify candidate-cover readiness. | `PlanDefect` | Separate candidate-cover proof from exact-image semantics or block the claim. |

## Handoff Requirement

Every FCR phase handoff must include:

```yaml
phase_id: <FCR phase>
plan_defect_triggered: true|false
algorithm_defect_triggered: true|false
trigger_ids: []
if_triggered_response: <stop/amend/generalize/quarantine/report>
claim_ceiling_after_phase: <claim label>
forbidden_claims_reaffirmed: true
```

If either trigger flag is true, implementation must stop at the current boundary until the defect is
resolved or explicitly reported to the user.
