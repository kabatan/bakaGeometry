# P12G Readiness Before P13/P14

Status: superseded by Full Core Repair overlay.

This file records the pre-FCR P12G readiness state only. `FULL_CORE_REPAIR_BASE_SPEC.md`,
`FULL_CORE_REPAIR_PLAN.md`, `CLOSURE.md`, and `ACTIVE_CONTEXT.md` now define current claim
authority. Do not use this P12G readiness file to resume P13 or to claim general candidate-cover
core readiness.

Historical P12G result before FCR: P12G spec and quality reviews passed for the narrowed P12G
remediation scope. FCR later determined that the current implementation remains insufficient as a
general R-GDTPK candidate-cover core.

## Historical Required State At Pre-FCR P12G

- Claim ceiling remained `PARTIAL_MECHANISM_READY:MECH-011`.
- `TargetActionKrylov` Route A is implemented for the required non-target-only quotient/action case.
- `CertifiedProbePlan` is present for TargetActionKrylov plan-time probe work and is replayed in execute.
- Candidate-cover finalization accepts nonzero support with no real roots as an empty cover.
- Final invariant and actual-DAG replay hooks blocked P14/P16 final claims without evidence. Later
  FCR-P12 evidence may supersede this historical blocker for the candidate-cover-only closure claim.
- Nonfinite certificates carry positive proof kind.
- P12G G1-G8 stress tests pass locally.
- `P13_P14_READINESS_AFTER_P12G.md` answers the post-P12G readiness questions explicitly.

## Still Not Complete

- P13 exact-image semantics
- P14 public `solve_target` candidate-cover pipeline connection
- P15 acceptance suites
- P16 final source-fidelity closure
- performance claim
- `SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC`
- `RGDTPK_Q_V4_ACCEPTANCE_COMPLETE`
