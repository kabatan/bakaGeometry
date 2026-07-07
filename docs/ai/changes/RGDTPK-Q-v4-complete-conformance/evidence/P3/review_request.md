# P3 Review Request

Reviewer prompt: RP-P3 from `REVIEWER_PROMPTS.md`.

Relevant R-IDs:

- BS-R050
- BS-R051
- BS-R052
- BS-R053

Files to inspect:

- `geosolver-core/src/algebra/monomial_order.rs`
- `geosolver-core/src/algebra/polynomial_ops.rs`
- `geosolver-core/src/algebra/modular.rs`
- `geosolver-core/src/algebra/crt.rs`
- `geosolver-core/src/algebra/rational_reconstruction.rs`
- `geosolver-core/src/algebra/sparse_matrix.rs`
- `geosolver-core/src/algebra/dense_matrix.rs`
- `geosolver-core/src/algebra/linear_solve.rs`
- `geosolver-core/src/algebra/normal_form.rs`
- `geosolver-core/src/types/matrix.rs`
- `geosolver-core/scripts/audit_v4_conformance.py`

Evidence:

- `source_to_code_map.md`
- `algorithm_evidence.md`
- `commands.log`
- `static_audit.log`
- `changed_files.txt`

Requested decision: PASS / FAIL_FIXABLE / FAIL_BLOCKING / NEEDS_MORE_EVIDENCE for P3 only.
