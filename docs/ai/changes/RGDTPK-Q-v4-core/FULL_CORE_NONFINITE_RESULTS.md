# Full Core Nonfinite Results

Status: FCR-P11 route selected; final closure must exclude nonfinite readiness from
`CANDIDATE_COVER_CORE_READY`.

Timestamp: 20260706-055251+09:00

Pre-P11 correction: certified nonfinite target image is not counted as an FCR-P10 support-producing
acceptance category. The previous P10 public certified-nonfinite check has been moved to the final
nonfinite semantics gate.

FCR-P11 chose route 2 below because the public `TargetSolveResult` still does not carry a
machine-readable nonfinite certificate inside `CoreRunCertificate` replay:

1. Implement a public, machine-readable, replay-bound nonfinite certificate that is carried by the
   final result and accepted by `replay_run_certificate`, with tamper rejection evidence.
2. Keep nonfinite readiness out of `CANDIDATE_COVER_CORE_READY`, record the limitation here, and
   leave `CertifiedNonFiniteTargetImage` final readiness to a later nonfinite task.

Required checks before final closure:

| Check | Status | Evidence |
| --- | --- | --- |
| positive nonfinite case returns `CertifiedNonFiniteTargetImage` only with positive proof | PASS | `fcr_p11_red_team_07_positive_nonfinite_kept_out_of_candidate_cover_claim`; target-free relation with rational witness returns `CertifiedNonFiniteTargetImage` |
| similar no-positive-proof case does not return `CertifiedNonFiniteTargetImage` | PASS | `fcr_p11_red_team_08_target_free_without_positive_witness_is_not_nonfinite`; fresh two-variable product/quadratic input under bounded target relation search returns hard/resource/certificate status |
| relation-search exhaustion does not become nonfinite | PASS | `fcr_p11_red_team_08_target_free_without_positive_witness_is_not_nonfinite` and FCR-P10 B1 |
| resource-bounded failure does not become nonfinite | PASS | `fcr_p11_red_team_08_target_free_without_positive_witness_is_not_nonfinite` uses a distinct two-variable algebraic input and retains cost trace |
| public result carries replay-bound nonfinite certificate, or claim explicitly excludes nonfinite readiness | PASS by explicit exclusion | public nonfinite result has no `CoreRunCertificate`; `CANDIDATE_COVER_CORE_READY` must not include nonfinite readiness |

Final closure must fail if it implies final nonfinite readiness. A later nonfinite task may replace
this exclusion only by carrying a machine-readable, replay-bound nonfinite certificate in the public
result and adding tamper rejection evidence.
