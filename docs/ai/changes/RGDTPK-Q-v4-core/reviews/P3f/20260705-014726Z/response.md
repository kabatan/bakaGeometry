RESULT: PASS

P3f passes the requested primitive-layer boundary review.

Checked against `BASE_SPEC.md` RGQ-023, RGQ-024, RGQ-028, RGQ-029, RGQ-058, RGQ-059; Appendix A sections 10.16-10.19; `PLAN.md` P3f; `REVIEWER_PROMPTS.md#P3f`; evidence files and schemas; and the four changed files.

Reasons:
- Required P3f functions are implemented in `regular_chain.rs`, `norm_trace.rs`, `real_root.rs`, and `sign.rs`; they are not empty shells, unconditional verifiers, or deferred hooks.
- Regular-chain code preserves guards/component semantics and uses projection semantics rather than dropping them.
- Tower detection is algebraic-form based; no geometry-name dispatch found.
- Norm verification recomputes the resultant relation and compares primitive-normalized relations; it is not unconditional.
- Root/sign helpers use exact rational arithmetic, rational intervals, Sturm logic, and return `RefinementRequired` when sign constancy is not certified.
- Evidence hashes match the packet, post-fix evidence shows focused P3f tests and full crate tests passing, and static scans found no forbidden P3f markers.
- No P3f evidence claims later kernel integration, root decode, exact-image fiber classification, candidate-cover readiness, acceptance completion, or MECH closure.

Blockers: none.

Required fixes: none.

Forbidden claims:
- Do not claim R-IDs are VERIFIED.
- Do not claim `RegularChainProjectionKernel` or `NormTraceProjectionKernel` is integrated.
- Do not claim candidate-cover, exact-image, root decode, acceptance suite completion, or MECH closure.

Claim ceiling: `SCAFFOLD_READY`. P3f starts executable primitive support for `MECH-007`, `MECH-011`, and `MECH-012`, but does not close them.
