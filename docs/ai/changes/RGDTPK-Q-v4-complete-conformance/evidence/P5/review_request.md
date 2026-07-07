# P5 Review Request

Reviewer prompt: RP-P5 from `REVIEWER_PROMPTS.md`.

Relevant R-IDs:

- BS-R060
- BS-R061

Files to inspect:

- `geosolver-core/src/preprocess/compression.rs`
- `geosolver-core/src/preprocess/definitional.rs`
- `geosolver-core/src/preprocess/linear_affine.rs`
- `geosolver-core/src/preprocess/binomial.rs`
- `geosolver-core/src/preprocess/saturation.rs`
- `geosolver-core/src/preprocess/independent.rs`
- `geosolver-core/src/problem/semantic.rs`
- `geosolver-core/src/result/diagnostics.rs`
- `geosolver-core/scripts/audit_v4_conformance.py`

Evidence:

- `source_to_code_map.md`
- `algorithm_evidence.md`
- `commands.log`
- `static_audit.log`
- `changed_files.txt`

Remediation focus after prior P5 FAIL_FIXABLE:

- `simplify_binomial_relations` no longer drops semantically referenced duplicate relation IDs.
  If a primitive/binomial duplicate group contains any relation referenced by semantic encodings, the
  full group is retained. Pure unreferenced duplicates are still deduplicated.
- Added regression coverage where `RelationId(1)` is the semantic NonZero witness relation and is a
  scaled duplicate of `RelationId(0)`; binomial simplification retains relation 1 and saturation
  still records the witness.

Requested decision: PASS / FAIL_FIXABLE / FAIL_BLOCKING / NEEDS_MORE_EVIDENCE for P5 only.
