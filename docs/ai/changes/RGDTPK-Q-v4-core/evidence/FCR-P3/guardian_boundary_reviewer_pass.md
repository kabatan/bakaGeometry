RESULT: PASS

Findings:
- `api::solve_target` calls `solver::orchestrator::solve_with_context`; `solve_with_context` now runs the expected public pipeline stages through validation, canonicalization, compression, graph/DAG build, planning, execution, message verification, composition, support, roots, core certificate, and success finalization.
- No `temporary_pipeline_not_connected` call remains in `api.rs`, `solver/orchestrator.rs`, or `solver/pipeline.rs`; only the historical status constructor remains.
- `step_plan` uses `plan_all_blocks`, and empty declared ladders now map to `AlgorithmicHardCase`.
- `step_execute` uses declared ladder plans, `kernels::all_kernels()`, child message contexts, and verifies emitted messages.
- Finite success verifies global support, isolates/decodes roots, includes projection messages, cost trace, and a `CoreRunCertificate` with target variable and final DAG replay evidence hash.
- Evidence is adequate: fmt/check/test passed, static scan confirms required calls and no temporary path, and the P3 public pipeline integration test exercises public API success.

FCR-P3 is closable only under `PARTIAL_MECHANISM_READY:MECH-011`. This does not claim `CANDIDATE_COVER_CORE_READY`, source fidelity, acceptance completion, P13 readiness, or advanced kernel production readiness. No files edited.
