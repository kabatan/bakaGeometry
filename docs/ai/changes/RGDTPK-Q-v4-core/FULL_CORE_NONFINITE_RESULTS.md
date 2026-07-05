# Full Core Nonfinite Results

Status: pending FCR-P11.

This file is a required final-closure artifact. It is not evidence yet.

Pre-P11 correction: certified nonfinite target image is not counted as an FCR-P10 support-producing
acceptance category. The previous P10 public certified-nonfinite check has been moved to the final
nonfinite semantics gate.

FCR-P11 must choose one of these routes:

1. Implement a public, machine-readable, replay-bound nonfinite certificate that is carried by the
   final result and accepted by `replay_run_certificate`, with tamper rejection evidence.
2. Keep nonfinite readiness out of `CANDIDATE_COVER_CORE_READY`, record the limitation here, and
   leave `CertifiedNonFiniteTargetImage` final readiness to a later nonfinite task.

Required checks before final closure:

| Check | Status | Evidence |
| --- | --- | --- |
| positive nonfinite case returns `CertifiedNonFiniteTargetImage` only with positive proof | pending | pending |
| similar no-positive-proof case does not return `CertifiedNonFiniteTargetImage` | pending | pending |
| relation-search exhaustion does not become nonfinite | pending | pending |
| resource-bounded failure does not become nonfinite | pending | pending |
| public result carries replay-bound nonfinite certificate, or claim explicitly excludes nonfinite readiness | pending | pending |

Final closure must fail if nonfinite readiness is implied without one of the two approved routes.
