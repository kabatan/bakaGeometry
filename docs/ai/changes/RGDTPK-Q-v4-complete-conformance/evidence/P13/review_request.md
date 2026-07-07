# P13 Review Request

Reviewer prompt: RP-P13 from `REVIEWER_PROMPTS.md`.

Relevant R-IDs:

- BS-R097
- BS-R098
- BS-R099

Files to inspect:

- `geosolver-core/src/algebra/regular_chain.rs`
- `geosolver-core/src/kernels/regular_chain_projection.rs`
- `geosolver-core/src/algebra/norm_trace.rs`
- `geosolver-core/src/kernels/norm_trace_projection.rs`
- `geosolver-core/src/algebra/interpolation.rs`
- `geosolver-core/src/kernels/specialization_interpolation.rs`
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

- RegularChain has component/guard/projection semantics and certificates.
- RegularChain components now carry `RegularityEvidence` and `GuardConditionEvidence`; nonconstant
  initials require an explicit matching guard and replay checks/tamper-regresses this evidence.
- NormTrace detects algebraic tower by form, not geometry label.
- Norm relation is exactly verified.
- Specialization sample points are deterministic and certificate-bound.
- Interpolated relation is accepted only after exact Q verification.

Scope note:

- `p13_exact_image_semantics` has one non-P13 exact-image/nonfinite failure carried forward to P14.
  Please judge P13 only against BS-R097/098/099.

Requested decision: PASS / FAIL_FIXABLE / FAIL_BLOCKING / NEEDS_MORE_EVIDENCE for P13 only.
