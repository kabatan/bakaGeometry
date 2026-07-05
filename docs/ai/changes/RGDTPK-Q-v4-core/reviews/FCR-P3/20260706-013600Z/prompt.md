Guardian boundary reviewer request for FCR-P3.

Workspace: C:\Users\bakat\OneDrive\ドキュメント\bakaGeometry
Branch: codex/full-core-repair
Review target: FCR-P3 only. Read-only review; do not edit files.

FCR-P3 requirement: public pipeline integration. PASS only if api::solve_target and solver::orchestrator::solve_with_context execute the candidate-cover pipeline from validation to final certificate, without temporary_pipeline_not_connected and without hand-built support/message finalizers.

Changed files to inspect:
- geosolver-core/src/solver/pipeline.rs
- geosolver-core/src/solver/orchestrator.rs
- geosolver-core/src/planner/planner.rs
- geosolver-core/src/verify/run_certificate.rs
- geosolver-core/src/verify/replay.rs (test-only initializer updates for new certificate input fields)
- geosolver-core/tests/p3_public_pipeline_integration.rs
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/FCR-P3/{commands.txt,command_outputs.txt,pipeline_static_scan.txt}

Important implementation notes:
- solve_with_context now calls step_validate -> step_canonicalize -> step_compress -> step_build_graphs -> step_build_dag -> step_plan -> step_execute -> step_verify_messages -> step_compose -> step_support -> step_roots -> step_core_certificate -> finalize_success_result.
- step_plan calls plan_all_blocks. plan_all_blocks now maps an empty declared ladder to AlgorithmicHardCase instead of leaking ImplementationBug for unsupported production shapes.
- step_execute executes declared ladder KernelExecutionPlans in DAG child-first order using kernels::all_kernels(), passes child ProjectionMessages in KernelContext, and verifies each emitted message with verify_projection_message.
- step_verify_messages verifies all messages against their actual DAG block and child-message context.
- finite success calls verify_global_support, squarefree/root isolation/decode, includes projection_messages, includes CoreRunCertificate, and records cost_trace.
- CoreRunCertificateInput now includes target_variable and final_dag_replay_evidence_hash. step_core_certificate builds final_dag_replay_evidence from actual DAG hash, message hashes, plan hashes, block IDs, source hashes, child dependencies, block authorization hashes, and edge authorization hashes.
- Nonfinite success remains available only through build_final_support_or_nonfinite positive certificate; otherwise unsupported cases return bounded failure.
- Because FCR-P2 quarantined advanced kernels, production success coverage is still limited to currently admitted base kernels. Do not require full generic kernel generalization in this FCR-P3 review; later FCR-P5+ owns that.

Fresh verification:
- cargo fmt: exit_code 0
- cargo check: exit_code 0
- cargo test: exit_code 0; 198 lib tests, p12 integration 1, p12g stress 1, p3 public pipeline integration 1, doctests 0
- static scan: no temporary_pipeline_not_connected call remains in api/solver; result/status definition remains only historical mapping
- static scan: all required FCR-P3 step calls and helper calls found
- git diff --check: exit_code 0, only LF->CRLF warnings

Please answer:
RESULT: PASS or RESULT: FAIL_FIXABLE or RESULT: FAIL_BLOCKED
Then concise findings. If PASS, state the phase is closable only under claim ceiling PARTIAL_MECHANISM_READY:MECH-011; do not claim CANDIDATE_COVER_CORE_READY, source-faithful, acceptance-complete, P13 readiness, or advanced kernel production readiness.
