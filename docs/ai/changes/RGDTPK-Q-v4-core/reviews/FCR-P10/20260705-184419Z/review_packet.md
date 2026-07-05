# FCR-P10 Review Packet

Phase: FCR-P10 - Full algebraic support-producing acceptance suite
Timestamp: 20260705-184419Z
Claim ceiling after phase: PARTIAL_MECHANISM_READY:MECH-011

## Source Requirements

- `FULL_CORE_REPAIR_PLAN.md` FCR-P10 now requires A1-A11 support-producing categories plus B1 bounded failure semantics through public or near-public pipeline.
- Support-producing cases must return `CertifiedCandidateCover` with nonempty support, real root/candidate construction, projection messages, and certificates.
- The suite must include permutation/nontrivial variable IDs and avoid expected-answer or fixture dispatch.
- Reviewer prompt fails helper-only tests, support-producing hard/resource/certificate/nonfinite results, target-action univariate/alias-only stress, no Universal one-large-block case, and no multiseparator composition.

## Changed Files

- `geosolver-core/src/kernels/mod.rs`
- `geosolver-core/src/api.rs`
- `geosolver-core/src/planner/admission.rs`
- `geosolver-core/src/kernels/sparse_resultant.rs`
- `geosolver-core/src/kernels/specialization_interpolation.rs`
- `geosolver-core/src/kernels/target_relation_search.rs`
- `geosolver-core/src/kernels/universal_elimination.rs`
- `geosolver-core/src/graph/projection_dag.rs`
- `geosolver-core/src/planner/ladder.rs`
- `geosolver-core/src/planner/planner.rs`
- `geosolver-core/src/preprocess/compression.rs`
- `geosolver-core/src/preprocess/definitional.rs`
- `geosolver-core/src/result/output.rs`
- `geosolver-core/src/solver/options.rs`
- `geosolver-core/src/solver/orchestrator.rs`
- `geosolver-core/src/solver/pipeline.rs`
- `geosolver-core/src/verify/replay.rs`
- `geosolver-core/src/verify/run_certificate.rs`
- `geosolver-core/tests/fcr_p10_acceptance_suite.rs`
- `docs/ai/changes/RGDTPK-Q-v4-core/evidence/FCR-P10/*`

## Implementation Summary

- Promoted advanced projection kernels from test-only/quarantine into production `all_kernels`, `kernel_by_kind`, and planner admission.
- Added public `api::solve_target` FCR-P10 suite covering A1-A11 plus B1.
- Pre-P11 correction moved certified nonfinite behavior out of P10 and into the final nonfinite semantics gate.
- Fixed replay to recompute compressed systems with production `pre_kernel_compress`.
- Fixed cascading definitional elimination by recomputing candidates after each substitution.
- Preserved sparse resultant pure planning: plan probe remains shape-only; execution recomputes probe and separately verifies resultant trace.
- Added empty compressed relation nonfinite path through the existing positive zero-target-elimination certificate.
- Added generic production `SolverOptions.kernel_priority` to exercise admitted named kernels through the public API without fixture/problem-id dispatch.
- Preserved relationless structural DAG blocks without emitting messages, while recursively hash-binding child messages in replay evidence.
- Added public A3/A5 projection-message recomposition/removal checks, a near-public two-separator message-only composition check, and deterministic nonzero rational scaling for every P10 problem relation.
- Preserved matrix-dimension cost trace evidence in public FiniteResourceFailure results.
- Preserved requested public target fallback for failure finalization and derived finite-resource cost trace kernel identity from failure stage.

## Evidence

- `evidence/FCR-P10/commands.txt`
- `evidence/FCR-P10/command_outputs.txt`
- `evidence/FCR-P10/static_scans.txt`
- `evidence/FCR-P10/acceptance_matrix.md`

## Fresh Local Verification

- `cargo fmt --manifest-path geosolver-core/Cargo.toml -- --check`: PASS
- `cargo check --manifest-path geosolver-core/Cargo.toml`: PASS
- `cargo test --manifest-path geosolver-core/Cargo.toml --test fcr_p10_acceptance_suite -- --nocapture`: PASS, 12/12 after pre-P11 correction
- `cargo test --manifest-path geosolver-core/Cargo.toml --test fcr_final_nonfinite_semantics -- --nocapture`: PASS, 2/2
- `cargo test --manifest-path geosolver-core/Cargo.toml`: PASS
- `cargo test --manifest-path geosolver-core/Cargo.toml verify::replay::tests -- --nocapture`: PASS, 16/16
- `cargo test --manifest-path geosolver-core/Cargo.toml --test fcr_p4_pure_planning fcr_plan_sparse_resultant_does_not_construct_output_relation -- --nocapture`: PASS, 1/1
- `cargo test --manifest-path geosolver-core/Cargo.toml --test fcr_p10_acceptance_suite fcr_p10_b1_public_resource_bounded_hard_case_has_spec_status -- --nocapture`: PASS, 1/1
- `git diff --check`: PASS, CRLF warnings only

## Known Limitations

- This phase does not claim final source fidelity, exact-image readiness, P13/P14/P15/P16 readiness, or `CANDIDATE_COVER_CORE_READY`.
- FCR-P11 red-team/final-nonfinite gate remains required before any generic readiness claim, and
  FCR-P12 remains final closure only.
