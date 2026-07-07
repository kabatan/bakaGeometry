# P11 Review Request

Reviewer prompt: RP-P11 from `REVIEWER_PROMPTS.md`.

Relevant R-IDs:

- BS-R095

Files to inspect:

- `geosolver-core/src/algebra/quotient.rs`
- `geosolver-core/src/algebra/krylov.rs`
- `geosolver-core/src/kernels/action_krylov.rs`
- `geosolver-core/src/verify/certificates.rs`
- `geosolver-core/src/verify/verify_message.rs`
- `geosolver-core/scripts/audit_v4_conformance.py`

Evidence:

- `source_to_code_map.md`
- `algorithm_evidence.md`
- `commands.log`
- `static_audit.log`
- `changed_files.txt`

Requested checks:

- Quotient handle does not expose coordinate roots, coordinate solution list, full coordinate RUR,
  or target-unrelated full quotient basis.
- Krylov sequence uses deterministic coverage probes.
- Coverage certificate cannot miss target-relevant eigenvalues.
- `verify_annihilator` is exact.
- No candidate polynomial returns without coverage.
- Forged coverage payload is rejected after recomputing wrapper hashes.

Requested decision: PASS / FAIL_FIXABLE / FAIL_BLOCKING / NEEDS_MORE_EVIDENCE for P11 only.

