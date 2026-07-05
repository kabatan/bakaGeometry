# Closure Packet - RGDTPK-Q-v4-core

Status: P12 completed, with P12G generality remediation active before P13/P14.

Current maximum claim ceiling:

```text
PARTIAL_MECHANISM_READY:MECH-011
```

## Current State

P0 through P12 have PASS review archives in the current worktree history. P12G is a mandatory
post-P12 remediation inserted by `P12G_BASE_SPEC_AMENDMENT.md` and `P12G_PLAN.md`; P13 and P14
remain blocked until P12G-a through P12G-h are closed.

P12G preserves the current claim ceiling. It does not claim exact-image completion, public
orchestration, performance readiness, final acceptance, `SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC`, or
`RGDTPK_Q_V4_ACCEPTANCE_COMPLETE`.

## P12G Remediation Scope

P12G addresses the direct algorithm audit findings before continuation:

- TargetActionKrylov now has a provenance-bound non-target-only quotient/action path for local
  univariate relation plus target alias relation, in addition to the target-only companion path.
- Kernel plans can carry a typed `CertifiedProbePlan`; TargetActionKrylov binds plan-time probe
  source hashes, output hash, and trace hash, then replays them during execute.
- Candidate-cover finalization keeps nonzero support with zero real roots as
  `CertifiedCandidateCover` with empty roots/candidates.
- Final invariant evidence and final DAG replay evidence are hash-bound blockers. P14/P16 final
  claims remain blocked until actual DAG/block replay replaces synthetic all-relations replay for
  final claims.
- Nonfinite certificates carry an explicit proof kind and reject proof-kind/evidence mismatch.
- P12G G1-G8 stress tests are present at direct module or pipeline-fragment level.

## Explicit Negative Claims

The following are still not complete:

- P13 exact-image semantics
- P14 public orchestration and `solve_target` candidate-cover pipeline connection
- P15 acceptance suites
- P16 final closure and source-fidelity audit
- performance claim
- `SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC`
- `RGDTPK_Q_V4_ACCEPTANCE_COMPLETE`
- any R-ID marked `VERIFIED`

## Commit Binding

The current P12G working state is bound by fresh local evidence under
`docs/ai/changes/RGDTPK-Q-v4-core/evidence/P12G-*` and review archives under
`docs/ai/changes/RGDTPK-Q-v4-core/reviews/P12G-*` once each subphase closes. Historical P0-P12
archives remain evidence for their original scopes only and do not override P12G.
