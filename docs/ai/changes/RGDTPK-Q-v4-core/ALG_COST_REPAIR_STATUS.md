# Algebraic-Cost Completion Repair Status

Status: active repair; prior readiness claims suspended.

Authority: status and claim-boundary artifact for
`RGDTPK-Q-v4-algebraic-cost-completion-repair-v1`.

## Current Claim Boundary

The following previous claims are suspended until every phase in
`ALG_COST_COMPLETION_REPAIR_PLAN.md` passes:

```text
CANDIDATE_COVER_CORE_READY
SOURCE_FAITHFUL_TO_V4_CANDIDATE_COVER_LAYER
```

The following claims remain forbidden without separate exact-image and full source-fidelity closure:

```text
SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC
RGDTPK_Q_V4_ACCEPTANCE_COMPLETE
```

Current maximum claim during this repair:

```text
CANDIDATE_COVER_PIPELINE_PRESENT_BUT_ALGEBRAIC_COST_INCOMPLETE
```

## Reason for Rollback

The previous candidate-cover closure did not prove algebraic-cost completeness for production
routes. In particular, it did not prove that every declared production route is bounded by its
dominant algebraic objects or that a long-running first route cannot monopolize ladder execution.

The post-GSR core repair timeout report identified the current blocker as uncapped symbolic
expression swell in `SparseResultantProjection`, reached through a declared first route for a large
projection block.

## Active Repair Scope

This repair is not a diagnostic-problem patch. It must complete the v4 algebraic-cost-compressed
candidate-cover layer through generic algebraic mechanisms:

```text
route-level algebraic work estimates
route budgets
SparseResultant expression-swell preflight
SparseResultant runtime growth guards
bounded resultant backend policy
non-monopolizing declared ladder execution
bounded UniversalTargetElimination success route
algebraic-cost-aware graph decomposition
sparse/lazy TargetRelationSearch
generic large-footprint support-producing stress
```

## Historical Artifact Treatment

Earlier closure and acceptance files remain historical evidence only. They must not be read as
current readiness authority unless this algebraic-cost completion repair closes with PASS.

The active navigation authority is `docs/ai/ACTIVE_CONTEXT.md`.
