# P12 Source-to-Code Map

Scope: BS-R096 / RP-P12 UniversalTargetEliminationKernel.

| Requirement | Implementation |
| --- | --- |
| Universal admission for Q-polynomial relation blocks | `geosolver-core/src/kernels/universal_elimination.rs` `admit_universal_elimination`, `plan_universal_elimination_with_messages` |
| Fixed source section 20.4 internal strategies only | `geosolver-core/src/planner/kernel_plan.rs` `UniversalStrategy`; `universal_elimination.rs` `fixed_universal_strategy_sequence`, `validate_fixed_strategy_sequence` |
| No NormTrace/RegularChain/ActionKrylov internal stages | `universal_elimination.rs` uses only local Groebner/F4, TargetRelationSearch, SparseResultant, and SpecializationInterpolation internal execution branches |
| Strategy declared and certificate-bound | `UniversalStagePlan`, `universal_stage_hash`, `UniversalProjectionCertificate.strategy_records`, `verify_universal_strategy_trace` |
| Exported generators are in Q[Z] and exactly verified | `algebra/elimination.rs` `validate_local_elimination_result`; `universal_elimination.rs` `extract_verified_export_generators`; `verify_message.rs` `verify_membership_outputs` |
| Empty generator path | `universal_elimination.rs` returns `AlgorithmicHardCase` when no nonzero exported generator exists under `NoLocalCertifiedNonFinite` |
| Production F4 path | `algebra/f4.rs` `f4_elimination_local`; `universal_elimination.rs` local F4 branch |

