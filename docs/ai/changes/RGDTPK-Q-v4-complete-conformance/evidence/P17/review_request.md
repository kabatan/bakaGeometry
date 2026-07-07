Purpose: P17 spec verifier request
Status: reviewer input packet, non-authoritative

# P17 Review Request

Reviewer role: spec_verifier

Use `REVIEWER_PROMPTS.md` section `RP-P17`.

Scope:
- Phase 17: Orchestrator, pipeline, result finalization, cost trace.

Relevant files:
- `docs/ai/changes/RGDTPK-Q-v4-complete-conformance/BASE_SPEC.md`
- `docs/ai/changes/RGDTPK-Q-v4-complete-conformance/PLAN.md`
- `docs/ai/changes/RGDTPK-Q-v4-complete-conformance/SOURCE_MAP.md`
- `docs/ai/changes/RGDTPK-Q-v4-complete-conformance/REVIEWER_PROMPTS.md`
- `docs/ai/changes/RGDTPK-Q-v4-complete-conformance/SCOPE_AMENDMENT_FINITE_CANDIDATE_COVER.md`
- `docs/ai/sources/geosolver_core_r_gdtpk_q_algorithm_spec_v4.md`
- `geosolver-core/src/solver/options.rs`
- `geosolver-core/src/solver/pipeline.rs`
- `geosolver-core/src/solver/orchestrator.rs`
- `geosolver-core/src/result/output.rs`
- `geosolver-core/src/result/cost_trace.rs`
- `geosolver-core/src/verify/run_certificate.rs`
- `geosolver-core/src/verify/replay.rs`
- `geosolver-core/scripts/audit_v4_conformance.py`

Evidence files:
- `docs/ai/changes/RGDTPK-Q-v4-complete-conformance/evidence/P17/changed_files.txt`
- `docs/ai/changes/RGDTPK-Q-v4-complete-conformance/evidence/P17/source_to_code_map.md`
- `docs/ai/changes/RGDTPK-Q-v4-complete-conformance/evidence/P17/commands.log`
- `docs/ai/changes/RGDTPK-Q-v4-complete-conformance/evidence/P17/static_audit.log`
- `docs/ai/changes/RGDTPK-Q-v4-complete-conformance/evidence/P17/algorithm_evidence.md`

Requested decision:
Return exactly one of PASS, FAIL, NEEDS_MORE_EVIDENCE.

Required review checks:
1. Pipeline order exactly matches the scoped candidate-cover source path.
2. Message verification occurs before composition.
3. Support verification occurs before roots.
4. Failure results preserve stage/cost evidence.
5. Cost trace contains every source parameter.
6. Final certificate binds all in-scope source fields.

Known scope boundary:
Exact-image classification remains OUT_OF_SCOPE. An exact-image request must return an explicit scope guard and cannot return exact-image success.
