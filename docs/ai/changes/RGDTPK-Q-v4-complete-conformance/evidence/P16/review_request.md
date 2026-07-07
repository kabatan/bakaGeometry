# P16 Review Request

Reviewer prompt: RP-P16 from `REVIEWER_PROMPTS.md`.

Relevant R-IDs:

- BS-R003
- BS-R040
- BS-R122

MECH:

- MECH-06

Files to inspect:

- `geosolver-core/src/solver/options.rs`
- `geosolver-core/src/solver/pipeline.rs`
- `geosolver-core/src/solver/orchestrator.rs`
- `geosolver-core/src/result/status.rs`
- `geosolver-core/src/result/output.rs`
- `geosolver-core/src/problem/semantic.rs`
- `geosolver-core/src/verify/run_certificate.rs`
- `geosolver-core/src/verify/replay.rs`
- `geosolver-core/scripts/audit_v4_conformance.py`

Evidence:

- `source_to_code_map.md`
- `algorithm_evidence.md`
- `commands.log`
- `static_audit.log`
- `changed_files.txt`

Requested checks:

- Every exposed exact-image option/status/finalizer path is audited.
- Exact-image request returns explicit `ExactImageOutOfScope` diagnostic and allowed failure status.
- Exact-image nonfinite outcome also returns explicit `ExactImageOutOfScope` diagnostic and allowed failure status, not `CertifiedNonFiniteTargetImage`.
- No `CertifiedExactTargetImage` or `CertifiedEmptyRealTargetImage` success is reachable in this scoped repair.
- Candidate-cover mode does not call exact-image classification or filter candidates by slack/guard semantics.
- Semantic provenance remains available but not used to classify/filter candidates.
- The scope guard is replay/certificate-bound.

Requested decision: PASS / FAIL_FIXABLE / FAIL_BLOCKING / NEEDS_MORE_EVIDENCE for P16 only.
