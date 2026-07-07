# P14 Review Request

Reviewer prompt: RP-P14 from `REVIEWER_PROMPTS.md`.

Relevant R-IDs:

- BS-R110
- BS-R111
- BS-R112
- BS-R113

MECH:

- MECH-04

Files to inspect:

- `geosolver-core/src/compose/message.rs`
- `geosolver-core/src/compose/compose.rs`
- `geosolver-core/src/compose/separator_elimination.rs`
- `geosolver-core/src/compose/final_support.rs`
- `geosolver-core/src/verify/certificates.rs`
- `geosolver-core/src/verify/verify_message.rs`
- `geosolver-core/src/verify/verify_support.rs`
- `geosolver-core/src/verify/replay.rs`
- `geosolver-core/src/verify/run_certificate.rs`
- `geosolver-core/scripts/audit_v4_conformance.py`

Evidence:

- `source_to_code_map.md`
- `algorithm_evidence.md`
- `commands.log`
- `static_audit.log`
- `changed_files.txt`

Requested checks:

- Composition follows the projection DAG and consumes projection messages through `MessageIdeal`.
- Separator elimination uses message-only pseudo blocks and target-direct kernels.
- Fixed small threshold checks are not used as proof of no target eliminant.
- Final support uses target-only/composed-ideal routes and verifies exact membership.
- Nonfinite certification does not rely only on small rational witness search when real semantics/guards/saturations/feasibility obligations are present.
- Nonfinite positive proof records replayable target-free Groebner/dimension/algebraic-independence evidence; the bounded rational witness is only a properness witness.
- Replay recomputes input/canonical/compression/hypergraph/DAG/message/support/root/candidate/invariant bindings.

Requested decision: PASS / FAIL_FIXABLE / FAIL_BLOCKING / NEEDS_MORE_EVIDENCE for P14 only.
