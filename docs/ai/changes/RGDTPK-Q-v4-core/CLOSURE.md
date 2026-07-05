# Closure Packet - RGDTPK-Q-v4-core

Status: Full Core Repair active before P13/P14/P15/P16.

Current maximum claim ceiling:

```text
PARTIAL_MECHANISM_READY:MECH-011
```

## Current State

P0 through P12G have historical PASS/review evidence in the current worktree history for their
original scopes. `FULL_CORE_REPAIR_BASE_SPEC.md` and `FULL_CORE_REPAIR_PLAN.md` are now mandatory
corrective overlays inserted before P13/P14/P15/P16.

Full Core Repair reopens the current generality and public-pipeline claims. P13, P14, P15, and P16
remain blocked until all FCR phases pass. The current implementation must not be described as close
to full candidate-cover completion, and old P12G PASS results must not be used as proof of generic
core readiness.

## P12G Remediation Scope

P12G addressed a direct algorithm audit before FCR, but FCR now classifies the remaining gaps as
larger core-repair blockers:

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

## Full Core Repair Overlay

FCR requires repair or removal of narrow production paths, including alias/univariate-only
TargetActionKrylov, module-only stress proof, synthetic replay substitutes, plan-time execution
paths, fake or non-generic kernel claims, and any public pipeline gap in `api::solve_target`.

Required final FCR claim target is `CANDIDATE_COVER_CORE_READY`, not another partial mechanism
label. That claim is unavailable until FCR acceptance evidence and reviews pass.

## Explicit Negative Claims

The following are still not complete:

- P13 exact-image semantics
- P14 public orchestration and `solve_target` candidate-cover pipeline connection
- P15 acceptance suites
- P16 final closure and source-fidelity audit
- FCR generic candidate-cover core readiness
- generic TargetActionKrylov over target-relevant quotient/action, beyond alias/univariate Route A
- actual DAG/block replay as the main replay path
- performance claim
- `CANDIDATE_COVER_CORE_READY`
- `SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC`
- `RGDTPK_Q_V4_ACCEPTANCE_COMPLETE`
- any R-ID marked `VERIFIED`

## Commit Binding

The current FCR import is bound by `FULL_CORE_REPAIR_PACK_MANIFEST.sha256`, the source zip SHA256
recorded in FCR-P0 evidence, and local evidence under
`docs/ai/changes/RGDTPK-Q-v4-core/evidence/FCR-P0/`. Historical P0-P12G archives remain evidence
for their original scopes only and do not override FCR.
