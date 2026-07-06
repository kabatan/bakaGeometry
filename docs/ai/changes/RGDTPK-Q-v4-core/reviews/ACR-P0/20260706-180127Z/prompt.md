# ACR-P0 Guardian Boundary Review Prompt

Review phase: `ACR-P0`

Change: `RGDTPK-Q-v4-algebraic-cost-completion-repair-v1`

Use the ACR-P0 reviewer prompt from
`docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_COMPLETION_REVIEWER_PROMPTS.md`.

## Review Scope

Review only claim rollback and agent reset.

## Files to Review

```text
docs/ai/ACTIVE_CONTEXT.md
docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_REPAIR_STATUS.md
docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_AGENT_RESET.md
docs/ai/changes/RGDTPK-Q-v4-core/CANDIDATE_COVER_CLOSURE.md
docs/ai/changes/RGDTPK-Q-v4-core/CANDIDATE_COVER_ACCEPTANCE_RESULTS.md
docs/ai/changes/RGDTPK-Q-v4-core/CLOSURE.md
docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_COMPLETION_REPAIR_BASE_SPEC.md
docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_COMPLETION_REPAIR_PLAN.md
docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_ACCEPTANCE_MATRIX.yaml
```

## Required Checks

Verify:

1. old candidate-cover readiness is suspended;
2. the Agent explicitly acknowledges previous false-PASS modes;
3. the new max claim is `CANDIDATE_COVER_PIPELINE_PRESENT_BUT_ALGEBRAIC_COST_INCOMPLETE`;
4. no active file still presents old closure as current truth.

Fail if the wording allows "we already passed before; this is only a performance patch."

## Required Output

Return:

```yaml
schema_version: alg-cost-review-v1
phase_id: ACR-P0
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
  dense_trs_materialization_bounded: null
  sparse_resultant_swell_bounded: null
  route_budget_enforced: null
  ladder_non_monopolizing: null
  graph_decomposition_cost_aware: null
support_producing_checks:
  public_or_near_public_pipeline_used: false
  projection_message_verified: false
  support_verified: false
  replay_accepted: false
anti_overfit_checks:
  no_diagnostic_problem_fixture: true|false
  no_geometry_name_dispatch: true|false
  no_expected_answer_dispatch: true|false
```

Then concise findings. If FAIL, identify exact files and required edits.
