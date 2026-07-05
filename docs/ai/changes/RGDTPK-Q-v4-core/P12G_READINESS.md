# P12G Readiness Before P13/P14

Status: superseded by Full Core Repair overlay.

This file records the pre-FCR P12G readiness state only. `FULL_CORE_REPAIR_BASE_SPEC.md` and
`FULL_CORE_REPAIR_PLAN.md` now block P13, P14, P15, and P16. Do not use this P12G readiness file to
resume P13 or to claim general candidate-cover core readiness.

Historical P12G result before FCR: P12G spec and quality reviews passed for the narrowed P12G
remediation scope. FCR later determined that the current implementation remains insufficient as a
general R-GDTPK candidate-cover core.

## Required State

- Claim ceiling remains `PARTIAL_MECHANISM_READY:MECH-011`.
- `TargetActionKrylov` Route A is implemented for the required non-target-only quotient/action case.
- `CertifiedProbePlan` is present for TargetActionKrylov plan-time probe work and is replayed in execute.
- Candidate-cover finalization accepts nonzero support with no real roots as an empty cover.
- Final invariant and actual-DAG replay hooks block P14/P16 final claims without evidence. The P12G
  implementation binds final DAG replay evidence into `CoreRunCertificate`, but even structurally
  bound caller-supplied evidence cannot close final claims until full actual replay is implemented.
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
