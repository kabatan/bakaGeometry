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
