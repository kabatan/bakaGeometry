Purpose: final source-to-code conformance matrix
Status: evidence, non-authoritative

# Source-to-Code Conformance Matrix

Scope: RGDTPK-Q v4 finite candidate-cover layer. Exact-image equality/classification sections are out of scope except for the explicit scope guard in BS-R003 and BS-R122.

Fresh final commands: `evidence/final/final_commands.log`.

| Source section | Base Spec / MECH | Implementation evidence | Test / audit evidence | Reviewer decision |
|---|---|---|---|---|
| 0, 34 | BS-R000 | Change packet in `docs/ai/changes/RGDTPK-Q-v4-complete-conformance/`; active source lock and amended finite scope. | P0 evidence; final matrix and closure. | P0 spec reviewer PASS; P18 final reviewers PASS. |
| 1.1-1.3, 2.1, 32 | BS-R001 | `problem/input.rs`, `problem/validate.rs`, `api.rs`, `solver/orchestrator.rs`; roles/provenance do not dispatch. | `v4_candidate_cover_conformance::p18_public_api_ignores_roles_names_and_relation_order_for_mechanism`; strict audit. | P2/P17 reviewer PASS; P18 final reviewers PASS. |
| 2.2-2.3 | BS-R002 | `compose/final_support.rs`, `verify/verify_support.rs`, `roots/isolate.rs`, `roots/decode.rs`, `result/output.rs`. | `p14_full_pipeline_integration`, `p12_roots_decode_integration`, `v4_candidate_cover_conformance`. | P14/P15/P17 reviewer PASS; P18 final reviewers PASS. |
| 2.3-2.4, 27.1-27.4 | BS-R003 | `solver/options.rs`, `solver/orchestrator.rs`, `verify/replay.rs`, `result/status.rs`; exact-image request returns explicit out-of-scope guard, not success. | `p13_exact_image_semantics`, `p14_full_pipeline_integration`, `p15_acceptance_stress`, `v4_candidate_cover_conformance`. | P16/P17 reviewer PASS; P18 final reviewers PASS. |
| 3.1, 4.2, 33 | BS-R010 | `verify/run_certificate.rs` invariant flags; production code has no coordinate-list/RUR/QE/CAD/geometry fallback path. | `audit_v4_conformance.py --strict` findings 0; P18 adversarial tests. | P0-P17 reviewer PASS; P18 final reviewers PASS. |
| 3.3, 28.1 | BS-R011 | `result/status.rs`, `result/diagnostics.rs`, `problem/context.rs`, `result/output.rs`; finite failures carry evidence. | `p14_full_pipeline_integration`, `p15_acceptance_stress`, `v4_candidate_cover_conformance::p18_bounded_failure_returns_evidence_cost_trace_not_unsupported`. | P1/P2/P14/P17 reviewer PASS; P18 final reviewers PASS. |
| 6, 7.1 | BS-R020 | Fixed production module tree under `geosolver-core/src/**`; `lib.rs` exports module tree without solver logic. | strict audit required-file checks; full cargo test. | P1 reviewer PASS; P18 final reviewers PASS. |
| 5.1-5.3, 8.1 | BS-R030 | `types/ids.rs`, `types/hash.rs`, certificate hash functions across `verify/*` and `planner/*`. | unit tests in lib suite; `v4_candidate_cover_conformance::p18_normalization_and_hash_binding_are_order_independent`. | P1/P17 reviewer PASS; P18 final reviewers PASS. |
| 5.2-5.3, 8.2-8.5 | BS-R031 | `types/rational.rs`, `types/monomial.rs`, `types/polynomial.rs`, `types/univariate.rs`, `algebra/polynomial_ops.rs`. | lib type/algebra tests; full cargo test. | P1/P3 reviewer PASS; P18 final reviewers PASS. |
| 5.7, 8.6-8.7 | BS-R032 | `types/matrix.rs`, `types/interval.rs`, `algebra/sparse_matrix.rs`, `algebra/dense_matrix.rs`. | lib matrix/interval tests; full cargo test. | P1/P3 reviewer PASS; P18 final reviewers PASS. |
| 2.4, 5.4, 9.1-9.2 | BS-R040 | `problem/semantic.rs`, `problem/input.rs`, `problem/validate.rs`; semantic provenance preserved but not used as exact-image filter. | `p13_exact_image_semantics`, `p15_acceptance_stress`, P2 tests. | P2/P16 reviewer PASS; P18 final reviewers PASS. |
| 5.5, 9.3 | BS-R041 | `problem/canonicalize.rs`, canonical hashes and relation-id preservation. | P2 canonicalization tests; `p18_normalization_and_hash_binding_are_order_independent`. | P2 reviewer PASS; P18 final reviewers PASS. |
| 9.4-9.5 | BS-R042 | `problem/context.rs`; route budget, cooperative work accounting, `FiniteResourceFailure` evidence. | `p14_full_pipeline_integration`, `p15_acceptance_stress`, P18 bounded failure test. | P2/P17 reviewer PASS; P18 final reviewers PASS. |
| 10.1-10.4 | BS-R050 | `algebra/monomial_order.rs`, `algebra/polynomial_ops.rs`; elimination orders, leading terms, reductions. | P3/P4 algebra tests; full cargo test. | P3/P4 reviewer PASS; P18 final reviewers PASS. |
| 10.5-10.6 | BS-R051 | `algebra/modular.rs`, `algebra/crt.rs`, `algebra/rational_reconstruction.rs`; deterministic primes/CRT/reconstruction. | P3 tests; clippy/fmt/test final evidence. | P3 reviewer PASS; P18 final reviewers PASS. |
| 10.7-10.8 | BS-R052 | `algebra/sparse_matrix.rs`, `algebra/dense_matrix.rs`, `algebra/linear_solve.rs`; exact modular rank/nullspace and reconstruction traces. | P3/P9 tests; full cargo test. | P3/P9 reviewer PASS; P18 final reviewers PASS. |
| 10.8, 25.1 | BS-R053 | `algebra/normal_form.rs`, `verify/certificates.rs`; exact Q membership and normal-form certificate verification. | P3/P9/P11 tests; full cargo test. | P3/P9/P11 reviewer PASS; P18 final reviewers PASS. |
| 10.9-10.10 | BS-R054, MECH-02 | `algebra/groebner.rs`, `algebra/f4.rs`; production Groebner/F4 local elimination with exact certificates. | `fcr_p4_pure_planning`, P4 evidence, strict audit no test-only F4. | P4/P12 reviewer PASS; P18 final reviewers PASS. |
| 10.11 | BS-R055 | `algebra/elimination.rs`; declared elimination dispatcher and strategy-specific certificate routes. | `fcr_p4_pure_planning`, Universal tests, strict audit. | P4/P12 reviewer PASS; P18 final reviewers PASS. |
| 10.12-10.18 | BS-R056 | `algebra/resultant.rs`, `interpolation.rs`, `quotient.rs`, `krylov.rs`, `regular_chain.rs`, `norm_trace.rs`, `real_root.rs`, `sign.rs` as needed by candidate cover. | P10-P15 tests; strict audit. | P10-P15 reviewer PASS; P18 final reviewers PASS. |
| 11.1-11.6 | BS-R060 | `preprocess/compression.rs`; source-ordered compression with trace/guards/diagnostics. | P5 evidence and tests; full cargo test. | P5 reviewer PASS; P18 final reviewers PASS. |
| 11.1-11.6 | BS-R061 | `preprocess/definitional.rs`, `linear_affine.rs`, `binomial.rs`, `saturation.rs`, `independent.rs`; safety obligations. | P5 remediation tests; strict audit. | P5 reviewer PASS; P18 final reviewers PASS. |
| 12.1-12.5 | BS-R070 | `graph/hypergraph.rs`, `influence.rs`, `weighted_primal.rs`, `separators.rs`, `tree_decomposition.rs`, `metrics.rs`. | P6 evidence/tests; full cargo test. | P6 reviewer PASS; P18 final reviewers PASS. |
| 12.6-12.7 | BS-R071, MECH-04 | `graph/projection_dag.rs`; relation coverage, authorization hashes, duplication certificates. | P6/P14 tests; replay evidence. | P6/P14 reviewer PASS; P18 final reviewers PASS. |
| 13.1-13.2 | BS-R080 | `planner/cost_model.rs`, `planner/probes.rs`; deterministic algebraic estimates and probes. | P7/P10/P11/P15 stress tests. | P7 reviewer PASS; P18 final reviewers PASS. |
| 13.3 | BS-R081 | `planner/admission.rs`, `kernels/mod.rs`; all nine kernel admissions, Universal always admitted for well-formed blocks. | P7 tests; `generic_success_route_planner`, `gpsr_generic_planner_success_route`. | P7 reviewer PASS; P18 final reviewers PASS. |
| 13.4-13.6 | BS-R082, MECH-03 | `planner/kernel_plan.rs`, `planner/ladder.rs`, `planner/planner.rs`; declared ladder, plan hash, no hidden fallback. | P7/P12/P18 tests; strict audit hidden-route checks. | P7/P12 reviewer PASS; P18 final reviewers PASS. |
| 14.1-14.2 | BS-R090 | `kernels/traits.rs`, `kernels/mod.rs`; common message/cost/certificate contract. | P7-P14 tests; full cargo test. | P7-P14 reviewer PASS; P18 final reviewers PASS. |
| 15 | BS-R091 | `kernels/target_univariate.rs`; principal target support from local/child target relations. | P8/P14 tests; public pipeline tests. | P8/P14 reviewer PASS; P18 final reviewers PASS. |
| 16 | BS-R092 | `kernels/linear_affine.rs`; affine order, pivot guards, exact exported relation certificates. | P8/P15 tests. | P8 reviewer PASS; P18 final reviewers PASS. |
| 17 | BS-R093, MECH-01 | `kernels/target_relation_search.rs`, `planner/relation_schedule.rs`, `algebra/linear_solve.rs`; modular search plus exact Q verification. | `acr_p9_large_footprint_stress`, `gpsr_generic_planner_success_route`, P9 evidence. | P9 reviewer PASS; P18 final reviewers PASS. |
| 18 | BS-R094 | `algebra/resultant.rs`, `kernels/sparse_resultant.rs`; declared templates, resource traces, exact relation verification. | P10 tests; strict audit. | P10 reviewer PASS; P18 final reviewers PASS. |
| 19 | BS-R095 | `algebra/quotient.rs`, `algebra/krylov.rs`, `kernels/action_krylov.rs`; target-relevant quotient, no coordinate export, coverage certificate. | P11 tests; strict audit no coordinate export. | P11 reviewer PASS; P18 final reviewers PASS. |
| 20 | BS-R096 | `kernels/universal_elimination.rs`, `algebra/elimination.rs`, `algebra/f4.rs`; exact source section 20.4 internal strategy list. | P12 red-team/tests; strict audit forbids NormTrace/RegularChain/ActionKrylov as Universal internals. | P12 reviewer PASS; P18 final reviewers PASS. |
| 21 | BS-R097 | `algebra/regular_chain.rs`, `kernels/regular_chain_projection.rs`; guard/component semantics and certificate route. | P13/P15 tests. | P13 reviewer PASS; P18 final reviewers PASS. |
| 22 | BS-R098 | `algebra/norm_trace.rs`, `kernels/norm_trace_projection.rs`; explicit tower norm/trace without label dispatch. | P13/P15 tests; strict audit label-dispatch checks. | P13 reviewer PASS; P18 final reviewers PASS. |
| 23 | BS-R099 | `algebra/interpolation.rs`, `kernels/specialization_interpolation.rs`; deterministic samples and exact Q verification. | P13/P15 tests. | P13 reviewer PASS; P18 final reviewers PASS. |
| 24.1-24.3 | BS-R110, MECH-04 | `compose/message.rs`, `compose/compose.rs`, `compose/separator_elimination.rs`; postorder message-only composition. | P14 tests; replay tests. | P14 reviewer PASS; P18 final reviewers PASS. |
| 24.4, 31 | BS-R111 | `compose/final_support.rs`; primitive LCM support, exact target-only support verification, nonfinite handling. | P14/P15/nonfinite tests. | P14/P15 reviewer PASS; P18 final reviewers PASS. |
| 25.1-25.3 | BS-R112 | `verify/certificates.rs`, `verify/verify_message.rs`, `verify/verify_support.rs`; exact support/message proof. | P14/P17 replay and support tests. | P14/P17 reviewer PASS; P18 final reviewers PASS. |
| 25.4-25.5 | BS-R113 | `verify/run_certificate.rs`, `verify/replay.rs`; run certificate and replay bind input/canonical/DAG/messages/support/root/candidates. | P14/P17 replay tests; full cargo test. | P14/P17 reviewer PASS; P18 final reviewers PASS. |
| 26.1-26.2 | BS-R120, MECH-05 | `roots/squarefree.rs`, `roots/isolate.rs`, `algebra/real_root.rs`; exact Sturm and distinct Descartes/Vincent paths. | `p12_roots_decode_integration`, `p15_acceptance_stress`, P18 Descartes test. | P15 reviewer PASS; P18 final reviewers PASS. |
| 26.3 | BS-R121, MECH-05 | `roots/decode.rs`, `roots/algebraic_number.rs`; candidates bind target, support hash, root index, interval, candidate hash. | P15/P18 candidate hash tests. | P15 reviewer PASS; P18 final reviewers PASS. |
| 27.1-27.4 | BS-R122, MECH-06 | Exact-image classification is OUT_OF_SCOPE; implemented finite-scope guard in `solver/orchestrator.rs` and `verify/replay.rs`. | `p13_exact_image_semantics`, P16 exact-image tests, P18 exact-image scope test. | P16/P17 reviewer PASS; P18 final reviewers PASS. |
| 28.1-28.2 | BS-R130 | `result/status.rs`, `result/output.rs`, `api.rs`; success/failure finalizers preserve diagnostics and result fields. | P14/P17 tests; full cargo test. | P17 reviewer PASS; P18 final reviewers PASS. |
| 28.3, 30.1-30.3 | BS-R131, MECH-07 | `result/cost_trace.rs`, `solver/orchestrator.rs`, `planner/*`; global cost trace and failure-step cost trace. | P15/P17/P18 bounded failure tests. | P17 reviewer PASS; P18 final reviewers PASS. |
| 29.1-29.3 | BS-R140 | `solver/options.rs`, `solver/pipeline.rs`, `solver/orchestrator.rs`; source-ordered pipeline with stage functions and failure conversion. | P17 tests; `p14_stage_trace_executes_appendix_29_pipeline_in_order`. | P17 reviewer PASS; P18 final reviewers PASS. |
| 33 | BS-R150 | All implementation areas; final 16-condition checklist in `CLOSURE.md`. | Final fmt/clippy/test/audit all exit 0; P18 conformance suite. | P18 final reviewers PASS. |

## Exact-Image-Only Source Sections

| Source section | Status | Evidence |
|---|---|---|
| 27.1-27.4 exact target image equality, real-fiber/slack final classification, Hermite/Thom final filtering | OUT_OF_SCOPE except scope guard | `SCOPE_AMENDMENT_FINITE_CANDIDATE_COVER.md`; BS-R003/BS-R122; exact-image request tests verify no exact-image success. |

## Required MECH Coverage

| MECH | Implemented by | Evidence |
|---|---|---|
| MECH-01 TargetRelationSearch exact membership | `kernels/target_relation_search.rs`, `verify/certificates.rs`, `algebra/linear_solve.rs` | P9 reviewer PASS; final cargo test. |
| MECH-02 F4 local elimination | `algebra/f4.rs`, `algebra/elimination.rs`, `kernels/universal_elimination.rs` | P4/P12 reviewer PASS; strict audit. |
| MECH-03 declared ladder / no hidden fallback | `planner/*`, `kernels/mod.rs`, `solver/orchestrator.rs` | P7/P12/P18 tests; strict audit. |
| MECH-04 Projection DAG and composition | `graph/projection_dag.rs`, `compose/*`, `verify/replay.rs` | P6/P14/P17 reviewer PASS. |
| MECH-05 exact root isolation and candidate binding | `algebra/real_root.rs`, `roots/*` | P15/P18 tests. |
| MECH-06 exact-image scope guard | `solver/options.rs`, `solver/orchestrator.rs`, `verify/replay.rs` | P16/P18 exact-image tests. |
| MECH-07 algebraic cost trace | `result/cost_trace.rs`, `problem/context.rs`, `solver/orchestrator.rs` | P17/P18 failure evidence tests. |
