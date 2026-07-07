# P2 Review Request

Reviewer prompt: RP-P2 from `REVIEWER_PROMPTS.md`.

Relevant R-IDs:

- BS-R001
- BS-R040
- BS-R041
- BS-R042

Files to inspect:

- `geosolver-core/src/problem/input.rs`
- `geosolver-core/src/problem/semantic.rs`
- `geosolver-core/src/problem/validate.rs`
- `geosolver-core/src/problem/canonicalize.rs`
- `geosolver-core/src/problem/context.rs`
- `geosolver-core/src/preprocess/compression.rs`
- `geosolver-core/src/verify/replay.rs`
- `geosolver-core/scripts/audit_v4_conformance.py`

Evidence:

- `source_to_code_map.md`
- `algorithm_evidence.md`
- `commands.log`
- `static_audit.log`
- `changed_files.txt`

Requested decision: PASS / FAIL_FIXABLE / FAIL_BLOCKING / NEEDS_MORE_EVIDENCE for P2 only.
