# P15 Review Request

Reviewer prompt: RP-P15 from `REVIEWER_PROMPTS.md`.

Relevant R-IDs:

- BS-R120
- BS-R121

MECH:

- MECH-05

Files to inspect:

- `geosolver-core/src/algebra/real_root.rs`
- `geosolver-core/src/algebra/sign.rs`
- `geosolver-core/src/roots/squarefree.rs`
- `geosolver-core/src/roots/isolate.rs`
- `geosolver-core/src/roots/decode.rs`
- `geosolver-core/src/roots/algebraic_number.rs`
- `geosolver-core/scripts/audit_v4_conformance.py`

Evidence:

- `source_to_code_map.md`
- `algorithm_evidence.md`
- `commands.log`
- `static_audit.log`
- `changed_files.txt`

Requested checks:

- Squarefree rejects zero support and uses exact squarefree computation.
- Sturm isolation is exact.
- Descartes/Vincent is distinct exact implementation and not a Sturm alias.
- Root intervals are rational and isolate one root each.
- Candidate hashes bind target, support hash, root index, and interval.
- No float-only approximation or fixed insufficient split cap is used.

Requested decision: PASS / FAIL_FIXABLE / FAIL_BLOCKING / NEEDS_MORE_EVIDENCE for P15 only.
