# P1 Review Request

Reviewer prompt: RP-P1 from `REVIEWER_PROMPTS.md`.

Relevant R-IDs:

- BS-R020
- BS-R030
- BS-R031
- BS-R032
- BS-R130

Files to inspect:

- `geosolver-core/src/lib.rs`
- `geosolver-core/src/api.rs`
- `geosolver-core/src/types/ids.rs`
- `geosolver-core/src/types/rational.rs`
- `geosolver-core/src/types/monomial.rs`
- `geosolver-core/src/types/polynomial.rs`
- `geosolver-core/src/types/univariate.rs`
- `geosolver-core/src/types/matrix.rs`
- `geosolver-core/src/types/interval.rs`
- `geosolver-core/src/types/hash.rs`
- `geosolver-core/src/result/status.rs`
- `geosolver-core/src/result/diagnostics.rs`
- `geosolver-core/src/result/cost_trace.rs`
- `geosolver-core/src/result/output.rs`
- `geosolver-core/src/verify/replay.rs`
- public API compatibility test updates under `geosolver-core/tests/*`
- `geosolver-core/scripts/audit_v4_conformance.py`

Evidence:

- `source_to_code_map.md`
- `algorithm_evidence.md`
- `commands.log`
- `static_audit.log`
- `changed_files.txt`

Requested decision: PASS / FAIL / NEEDS_MORE_EVIDENCE for P1 only.
