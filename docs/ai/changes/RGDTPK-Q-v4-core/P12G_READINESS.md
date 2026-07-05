# P12G Readiness Before P13/P14

Status: P12G remediation readiness file after spec_verifier PASS and quality_reviewer PASS.

P13 may resume after the user accepts this P12G remediation report. P14 remains blocked after P12G
unless actual DAG/block replay replaces synthetic all-relations replay for final claims.

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
