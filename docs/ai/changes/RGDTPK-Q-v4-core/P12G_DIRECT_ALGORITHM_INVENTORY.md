# P12G Direct Algorithm Inventory

Status: active P12G inventory before P13/P14.

Current HEAD at P12G-a rebind start: `d958d1a26266414f67ee6c4251c648a8558375ca`.

Current maximum claim ceiling:

```text
PARTIAL_MECHANISM_READY:MECH-011
```

## Inventory

| Area | Current P12G state | Claim limit |
| --- | --- | --- |
| TargetActionKrylov | Route A implemented for target-only univariate relations and local-univariate plus linear target-alias relations. The non-target-only stress case `x^2 - 2 = 0`, `T - x = 0` produces `T^2 - 2` through `TargetActionKrylov`. | Not a generic quotient-basis constructor for arbitrary local ideals. |
| Plan execution boundary | `KernelExecutionPlan` carries `PlanWorkClassification`; TargetActionKrylov uses `CertifiedProbePlan` and execute replays source/output/trace hashes. | Other kernels retain `PurePlan` unless they are later upgraded with explicit certified probe models. |
| Candidate cover finalization | Nonzero support with no real roots returns `CertifiedCandidateCover` with empty root and candidate vectors. | This is candidate-cover semantics only, not exact-image completion. |
| Invariant flags | Final invariant evidence hook requires explicit scan/evidence hashes before final claims. Missing evidence returns `CertificateDesignGap`. | P14/P16 final claims remain blocked without evidence. |
| DAG replay | Final DAG replay evidence hook requires actual projection DAG hash and block authorization hashes before final claims. | Current P11 replay remains sufficient only for its phase scope. |
| Nonfinite certificates | `NonFiniteCertificate` has `NonFiniteProofKind`, and verification rejects proof-kind/evidence mismatch. | Bounded witness failure does not become certified nonfinite. |
| Stress battery | P12G G1-G8 direct tests are present under `p12g_` test names. | These are pre-P13/P14 stress tests, not P15 acceptance suites. |

## Negative Claims

P12G does not complete P13 exact-image semantics, P14 public orchestration, P15 acceptance suites,
P16 final closure, performance readiness, `SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC`, or
`RGDTPK_Q_V4_ACCEPTANCE_COMPLETE`.
