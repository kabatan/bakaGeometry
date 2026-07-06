RESULT: FAIL_FIXABLE

```yaml
schema_version: alg-cost-review-v1
phase_id: ACR-P2
review_status: FAIL
phase_closable: false
algorithmic_sufficiency: insufficient
source_spec_alignment: misaligned
blocking_findings:
  - "Production cost estimates do not use actual polynomial term counts: estimate_kernel_algebraic_work derives input/max terms from relation_count * variable_count, so a small matrix with huge polynomial entries can still be scored cheap."
  - "SparseResultant candidate pair scoring still sorts by matrix size and hashes only; it ignores input term count, keep-variable count, and expression-growth risk."
  - "RouteBudget is hash/certificate/message-bound, but I found no runtime enforcement against max_intermediate_terms/max_output_terms/max_work_units before/during route execution."
required_fixes:
  - "Make estimate_kernel_cost consume authorized relation polynomial footprints: total terms, max terms per relation/pair, keep-variable count, degree, coefficient height, and predicted intermediate/output growth."
  - "Change sparse resultant admission/pair scoring to reject or CostProhibit small-matrix large-entry cases before execution."
  - "Enforce RouteBudget in production execution paths, returning allowed route-local failure before exceeding declared budget."
reviewed_files:
  - docs/ai/changes/RGDTPK-Q-v4-core/reviews/ACR-P2/20260706-182041Z/prompt.md
  - docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_COMPLETION_REVIEWER_PROMPTS.md
  - geosolver-core/src/planner/algebraic_cost.rs
  - geosolver-core/src/planner/cost_model.rs
  - geosolver-core/src/planner/kernel_plan.rs
  - geosolver-core/src/planner/admission.rs
  - geosolver-core/src/planner/ladder.rs
  - geosolver-core/src/planner/planner.rs
  - geosolver-core/src/kernels/sparse_resultant.rs
  - geosolver-core/src/result/cost_trace.rs
  - geosolver-core/src/compose/message.rs
  - geosolver-core/src/verify/certificates.rs
  - geosolver-core/src/verify/replay.rs
reviewed_tests:
  - "Packet-reported evidence commands only; not rerun."
  - "Inspected acr_p2_plan_hash_changes_when_dominant_cost_estimate_changes"
  - "Inspected acr_p2_route_budget_is_certificate_and_trace_bound"
  - "Inspected acr_p2_sparse_resultant_small_matrix_large_entries_is_cost_prohibited"
adversarial_counterexamples:
  - name: "small-resultant-large-entry-footprint"
    algebraic_footprint: "Two relations, 4x4 resultant template, many keep-variable monomials / thousands of terms in entries."
    expected_behavior: "CostProhibited or ranked after safer bounded routes before execution."
    reviewer_result: "Not guaranteed: production estimate path ignores actual polynomial term counts and sparse pair scoring uses matrix size/hash ordering."
dominant_cost_checks:
  dense_trs_materialization_bounded: true
  sparse_resultant_swell_bounded: false
  route_budget_enforced: false
  ladder_non_monopolizing: false
  graph_decomposition_cost_aware: null
support_producing_checks:
  public_or_near_public_pipeline_used: false
  projection_message_verified: false
  support_verified: false
  replay_accepted: false
anti_overfit_checks:
  no_diagnostic_problem_fixture: true
  no_geometry_name_dispatch: true
  no_expected_answer_dispatch: true
```

Concise findings:

1. `cost_model.rs` computes `structural_terms = local_relation_count * local_variable_count` and stores that as `input_term_count` / `max_input_terms`. It does not inspect actual polynomial monomial counts, so the P2 condition for expression-growth-aware dominant cost is not met.

2. `sparse_resultant.rs` selects resultant pairs by `matrix_rows * matrix_cols` plus hashes. That is still matrix-first ranking and can miss the adversarial small-matrix, huge-entry case.

3. `kernel_plan.rs`, `certificates.rs`, and `message.rs` bind route budget/work hashes into plan/certificate/message identity. However, `pipeline.rs` executes routes without checking route budget fields, so the budget is not enforceable yet.

Forbidden claims: ACR-P2 closable, route budget fully enforced, SparseResultant expression-swell bounded, or production cost ranking source-faithful to dominant polynomial-entry growth.

Next action: fix production term-growth estimation and budget enforcement, then rerun ACR-P2 review with evidence.
