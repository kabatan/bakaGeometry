# P11 Source-to-Code Map

Scope: BS-R095 / RP-P11 TargetActionKrylovKernel.

| Requirement | Implementation |
| --- | --- |
| Target-relevant quotient handle without coordinate roots/RUR export | `geosolver-core/src/algebra/quotient.rs` `ProductionQuotientHandleInput`, `BasisScope::TargetRelevant`, `validate_production_input` |
| Finite-rank quotient proof and normal-form/action-column certificates | `geosolver-core/src/algebra/quotient.rs` `build_production_target_relevant_quotient_input_from_relations`, `normal_form_basis_certificate`, `make_action_column_certificate`, `verify_action_column_certificate` |
| Deterministic block Krylov probes | `geosolver-core/src/kernels/action_krylov.rs` `build_target_action_krylov_trace` uses all unit vectors `(0..handle.basis_size())` |
| Coverage certificate cannot miss target-relevant eigenvalues | `geosolver-core/src/algebra/krylov.rs` `certify_krylov_coverage` requires recovered recurrence to equal the characteristic polynomial of the exact target action matrix |
| Exact annihilator verification | `geosolver-core/src/algebra/krylov.rs` `verify_annihilator` recomputes the target action matrix, characteristic polynomial, and Cayley-Hamilton hash |
| No candidate without coverage | `geosolver-core/src/kernels/action_krylov.rs` constructs output relation only after `certify_krylov_coverage` and `verify_annihilator`; `verify_message.rs` replays both |
| Replay rejects forged coverage | `geosolver-core/src/kernels/action_krylov.rs` `p11_action_krylov_replay_rejects_tampered_coverage_after_rehash` |

