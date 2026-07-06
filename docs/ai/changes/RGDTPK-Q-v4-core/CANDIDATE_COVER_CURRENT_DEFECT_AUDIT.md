# Candidate-Cover Current Defect Audit

This audit replaces the over-broad exact-image-focused defect audit for the current work.

## Corrected interpretation

The current algorithm is intended to return a finite candidate cover, not an exact target image.

Therefore, the following are not candidate-cover defects:

```text
- spurious roots remain;
- exact_image_certificate is absent in candidate-cover mode;
- mixed coordinate real fiber classification is incomplete;
- guard/slack semantics are not used to filter candidate-cover candidates.
```

They are exact-image-layer defects only.

## Still candidate-cover defects

The following remain defects for the current layer:

```text
1. support S(T) is not exact-Q verified;
2. support may omit a true target value;
3. success depends on expected answer or fixture dispatch;
4. production path uses full coordinate solutions or full RUR;
5. generic pipeline is actually a narrow slice portfolio;
6. TargetActionKrylov is target-only or alias-only;
7. support verification only supports target-only product route when composed-ideal membership route is required;
8. Universal exhaustion maps to nonfinite without positive proof;
9. root isolation or candidate decode is placeholder or absent;
10. replay does not bind DAG/messages/support/roots/candidates.
```

## Current known likely work items

Based on prior review, inspect and either fix or explicitly clear:

```text
- verify/verify_support.rs:
    Add composed-ideal membership route if absent.

- compose/final_support.rs:
    Ensure S(T) proof is containment proof, not exact-image proof.

- kernels/action_krylov.rs and algebra/quotient.rs:
    Verify generic quotient/action path is not alias-only.

- kernels/universal_elimination.rs:
    Verify declared local generic route, no hidden coordinate fallback.

- algebra/f4.rs:
    Either production F4 or non-production-only with no readiness dependency.

- compose/final_support.rs nonfinite:
    Positive proof only; no general nonfinite completeness required for candidate-cover.

- result/output.rs and solver/orchestrator.rs:
    Candidate-cover mode must not filter spurious roots.

- verify/replay.rs:
    Replay must bind actual DAG/messages/support/root/candidate and nonfinite certificate if present.
```
