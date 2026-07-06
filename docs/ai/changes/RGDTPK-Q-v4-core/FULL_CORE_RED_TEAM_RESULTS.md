# Full Core Red-Team Results

Status: FCR-P11 implementation evidence recorded; reviewer recheck required before FCR-P12 closure.

Timestamp: 20260706-055251+09:00

FCR-P11 added `geosolver-core/tests/fcr_p11_red_team_suite.rs`. The suite uses fresh variable ids,
relation order, coefficients, and syntactic forms not present in FCR-P10. Each case runs through
`api::solve_target` and support-producing cases require `replay_run_certificate` acceptance.

Required coverage:

| Required input class | Fresh case id | Public/near-public path | Result | Evidence |
| --- | --- | --- | --- | --- |
| multivariate finite quotient/action not alias-univariate | `fcr_p11_red_team_01_multivariate_action_not_alias_univariate` | public `api::solve_target`, `TargetActionKrylov` prioritized | `CertifiedCandidateCover` | nonlinear target `t = xy + y`; replay accepted |
| two-separator composition | `fcr_p11_red_team_02_two_separator_public_composition` | public `api::solve_target`, production DAG/message composition replayed near-public | `CertifiedCandidateCover` | multiple projection messages and message-only separator elimination |
| sparse resultant not target-univariate degree-1 | `fcr_p11_red_team_03_sparse_resultant_higher_degree_eliminant` | public `api::solve_target`, `SparseResultantProjection` prioritized | `CertifiedCandidateCover` | resultant from `x^2 = t + 1`, `x^3 = 5`; replay accepted |
| guarded rational affine with nonconstant denominator | `fcr_p11_red_team_04_guarded_rational_affine_nonconstant_denominator` | public `api::solve_target` | `CertifiedCandidateCover` | denominator `x + 2` with witness relation; replay accepted |
| one-large-block no-separator | `fcr_p11_red_team_05_one_large_block_universal_admission` | public `api::solve_target`, `UniversalTargetElimination` prioritized but declared last | `CertifiedCandidateCover` | Universal admission trace retained; replay accepted |
| target-independent feasibility obligation | `fcr_p11_red_team_06_target_independent_feasibility_component` | public `api::solve_target` | `CertifiedCandidateCover` | independent `z^2 = 4` with target support `t^2 = 11`; replay accepted |
| positive nonfinite | `fcr_p11_red_team_07_positive_nonfinite_kept_out_of_candidate_cover_claim` | public `api::solve_target` | `CertifiedNonFiniteTargetImage` | support absent; public certificate absent, so not candidate-cover readiness evidence |
| similar case without positive nonfinite proof | `fcr_p11_red_team_08_target_free_without_positive_witness_is_not_nonfinite` | public `api::solve_target`, bounded `TargetRelationSearch` | hard/resource/certificate status, never nonfinite | fresh two-variable product/quadratic input `xy = t`, `x^2 + y^2 = 3`; cost trace retained |
| additional fresh algebraic input 9 | `fcr_p11_red_team_09_regular_chain_fresh_input` | public `api::solve_target`, `RegularChainProjection` prioritized | `CertifiedCandidateCover` | shifted target relation `(t + 1) = y`; replay accepted |
| additional fresh algebraic input 10 | `fcr_p11_red_team_10_norm_trace_fresh_two_step_tower` | public `api::solve_target`, `NormTraceProjection` prioritized | `CertifiedCandidateCover` | two-step tower `a^2 = 3`, `b^2 = a`, `tb = 2`; replay accepted |

Verification:

```text
cargo test --manifest-path geosolver-core/Cargo.toml --test fcr_p11_red_team_suite -- --nocapture
result: PASS, 10 passed, 0 failed
```

FCR-P12 final closure must still fail if an independent reviewer cannot reproduce or falsify these
fresh inputs through the public or near-public pipeline.
