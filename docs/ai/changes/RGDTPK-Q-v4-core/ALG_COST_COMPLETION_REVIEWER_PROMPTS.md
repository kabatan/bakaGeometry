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
