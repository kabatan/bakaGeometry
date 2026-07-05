# Current Defect Audit — Why P1–P12G Cannot Continue

This file is a corrective audit target for the Agent. It is not a substitute for direct code review. Every item below must be confirmed against current HEAD and then repaired.

---

## D0 — Public pipeline is not connected

Observed production risk:

```text
solver/orchestrator.rs returns temporary_pipeline_not_connected.
```

Why this fails:

```text
The v4 solver core is defined by the full solve_target pipeline. A disconnected public pipeline means no end-to-end candidate-cover algorithm exists.
```

Required action:

```text
Implement solve_with_context as the full production pipeline.
```

---

## D1 — TargetActionKrylov is still a partial algebraic form

Observed production risk:

```text
TargetActionKrylov selection space consists of target-only univariate and local-univariate plus linear target alias forms.
```

Why this fails:

```text
This is not the generic quotient/action algorithm in the v4 spec. It does not build target-relevant quotient bases for arbitrary authorized local ideals.
```

Required action:

```text
Implement generic quotient/action construction from authorized relations, or fail this repair. Do not treat local-univariate alias support as completion.
```

---

## D2 — Plan-time hidden execution remains

Known plan-time output construction risks:

```text
SparseResultant:
  plan_sparse_resultant_with_messages -> build_sparse_resultant_trace -> compute_resultant_relation

SpecializationInterpolation:
  plan_specialization_interpolation_with_messages -> build_specialization_interpolation_trace
  -> execute_inner_target_only_kernel
  -> eliminate_to_keep_variables(LocalGroebner)

RegularChain:
  plan_regular_chain_projection -> build_regular_chain_trace
  -> local_regular_chain_decomposition
  -> project_chain_to_variables

NormTrace:
  plan_norm_trace_projection -> build_norm_trace_trace
  -> norm relation construction
```

Why this fails:

```text
The planner must declare schedules, templates, and bounds. Execute must construct projection messages. If planning computes final relations, the declared ladder becomes a hidden execution layer.
```

Required action:

```text
Move final relation construction to execute for every kernel. CertifiedProbePlan must not be used to hide final output construction during plan.
```

---

## D3 — F4 is non-production

Observed production risk:

```text
algebra/f4.rs exposes NotProductionF4 and test-backed Groebner batch functions.
```

Why this fails:

```text
The v4 spec allows exact local F4/F5-like sparse linear algebra, not a fake F4 wrapper. A non-production F4 path cannot support performance or Universal claims.
```

Required action:

```text
Implement real production local F4 with certificates or remove all production F4 claims and routes.
```

---

## D4 — SparseResultant is not generic enough

Observed production risk:

```text
Current SparseResultantProjection is centered on selecting pairs and chaining binary resultants.
```

Why this fails:

```text
The v4 SparseResultantProjection requires declared support/template construction for the intended eliminant, exact reconstruction, and exact verification. Binary pair chains may be an optimization but not generic completion.
```

Required action:

```text
Implement the generic resultant/eliminant contract or remove SparseResultant from generic completion claims.
```

---

## D5 — RegularChain and NormTrace are admitted-structure optimizers, not generic coverage

Observed production risk:

```text
RegularChain depends on detected triangular/decomposition forms.
NormTrace depends on explicit tower detection.
```

Why this fails:

```text
These are valid optimizers only if integrated into a generic pipeline. They cannot be used as evidence that the solver covers arbitrary algebraic common IR.
```

Required action:

```text
Implement their v4 contracts without plan-time output construction and keep all generic coverage claims grounded in the full pipeline.
```

---

## D6 — Replay is not final source-faithful

Observed production risk:

```text
Main replay constructs synthetic all-relations blocks for message verification.
Actual-block replay helper exists but is not the main accepted path.
```

Why this fails:

```text
The final solver certificate must prove each message used only its authorized DAG block relations and declared child messages.
```

Required action:

```text
Make actual TargetProjectionDAG/block replay the main replay path.
```

---

## D7 — Stress is still too module-local

Observed production risk:

```text
Several P12G stress cases are module tests for preprocess or individual kernels rather than full pipeline-fragment tests.
```

Why this fails:

```text
A module test can prove a helper works, but cannot prove that the R-GDTPK production algorithm narrows target values from algebraic common IR.
```

Required action:

```text
Promote all support-producing stress cases to planner/kernel/message/compose/support/root pipeline tests, and to public solve_target tests for final readiness.
```

---

## D8 — Review evidence has been too permissive

Observed production risk:

```text
P12G review PASS did not catch TargetActionKrylov partiality or plan-time hidden execution across all kernels.
```

Why this fails:

```text
Evidence consistency is not algorithmic sufficiency.
```

Required action:

```text
Re-run reviews with Full Core Repair reviewer prompts. Any reviewer must fail on partial-slice completion or hidden execution, even if prior Guardian summaries passed.
```

---

## D9 — Required immediate conclusion

The current code must be treated as an incomplete partial implementation. It may contain useful components, especially TargetRelationSearch and parts of Universal, but it cannot be used as the basis for P13/P14 continuation until the Full Core Repair Plan passes.

