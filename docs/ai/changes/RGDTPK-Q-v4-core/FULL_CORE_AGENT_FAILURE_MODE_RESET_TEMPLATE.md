# Full Core Agent Failure-Mode Reset Template

The Agent must fill this file before writing production code for the repair.

## Non-negotiable mental model

The task is to implement the v4 R-GDTPK-Q candidate-cover solver core. The task is not to pass Guardian gates, write plausible evidence, preserve previous phase claims, or make a small suite pass.

## Failure modes to actively avoid

For each item, fill `concrete_prevention_in_this_phase` before closing any phase.

```yaml
- failure_mode: gate_or_review_pass_substitutes_for_algorithm
  concrete_prevention_in_this_phase: TODO
- failure_mode: narrow_algebraic_shape_treated_as_generic
  concrete_prevention_in_this_phase: TODO
- failure_mode: heavy_fallback_hidden_under_generic_name
  concrete_prevention_in_this_phase: TODO
- failure_mode: plan_time_execution_hidden_as_planning
  concrete_prevention_in_this_phase: TODO
- failure_mode: helper_or_module_test_substitutes_for_public_pipeline
  concrete_prevention_in_this_phase: TODO
- failure_mode: certificate_or_dag_is_decorative
  concrete_prevention_in_this_phase: TODO
- failure_mode: exact_image_or_source_fidelity_overclaim
  concrete_prevention_in_this_phase: TODO
```

## Stop conditions

The Agent must stop and declare `AlgorithmDefect` or `PlanDefect` if any of the following is true:

```text
- a required v4 candidate-cover algorithm cannot be implemented without narrowing scope;
- the only support-producing implementation is target-univariate, affine-only, alias-only, binary-only, tower-only, or example-specific;
- a plan function computes final output relations;
- public solve_target cannot be connected without bypassing DAG/planner/kernel/compose;
- a reviewer prompt would pass evidence but not inspect algorithmic generality.
```
