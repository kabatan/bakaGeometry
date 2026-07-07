# P13 Source-to-Code Map

Scope: BS-R097, BS-R098, BS-R099 / RP-P13.

| Requirement | Implementation |
| --- | --- |
| Regular-chain component/guard/projection semantics | `geosolver-core/src/algebra/regular_chain.rs` `RegularChainDAG`, `RegularChain`, `RegularityEvidence`, `GuardConditionEvidence`, `ProjectionGenerators`, `local_regular_chain_decomposition`, `verify_regular_chain_dag_evidence`, `project_chain_to_variables`, `combine_chain_projections` |
| RegularChainProjection certificate | `geosolver-core/src/kernels/regular_chain_projection.rs` emits `RegularChainProjectionCertificate`; `verify_message.rs` replays DAG/projections/combination |
| NormTrace tower detection by algebraic form | `geosolver-core/src/algebra/norm_trace.rs` `detect_explicit_tower_plan`, `validate_tower_expression`; no geometry label dispatch |
| Exact norm verification | `norm_relation_for_tower_plan`, `verify_norm_tower_plan_relation`; `norm_trace_projection.rs` rejects unverified relations |
| Deterministic specialization samples | `geosolver-core/src/algebra/interpolation.rs` `choose_multiseparator_specialization_points`; `specialization_interpolation.rs` hashes sample points and samples |
| Declared inner kernel plans | `specialization_interpolation.rs` `execute_inner_target_only_kernel`, `p12g_specialization_interpolation_inner_schedule_is_declared` |
| Exact Q verification of interpolation | `specialization_interpolation.rs` `verify_interpolated_relation_by_elimination`; `verify_message.rs` replays interpolation certificate and exact elimination result |
