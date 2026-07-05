FAIL_FIXABLE

**Reviewed R-IDs and MECHs**
Reviewed `RGQ-010`, `RGQ-019`, `RGQ-020`, `RGQ-025`; P3b MECH scope: continues `MECH-001`, starts `MECH-006`. No R-ID is marked verified.

**Files Inspected**
Inspected the requested Base Spec sections, Plan P3b/general rules, review schemas/prompts, all requested P3b code files, type files, and P3b evidence files. Also checked the linked review-summary schema mirror hash; it is byte-identical to `REVIEW_SUMMARY_SCHEMA.yaml`.

**Evidence and Commands Inspected**
Inspected `commands.txt`, `command_outputs.txt`, `static_scans.txt`, `function_implementation_table.yaml`, and `notes.md`.

Evidence claims:
- `cargo fmt --check`: exit 0.
- `cargo test`: exit 0, 44 tests passed.
- Static scans: no floating exact path, no geometry/fixture/answer dispatch, no ordinary `Unsupported`, no modular proof overclaim.
- I did not rerun tests due read-only review posture.

**Algorithmic Sufficiency Judgment**
Insufficient but fixable.

Most P3b primitives are real implementations: exact rational-to-Fp reduction, exact CRT, exact bounded rational reconstruction with nonunique failure, exact Fp rank/nullspace, and candidate-only modular solve traces.

However P3b cannot pass because `linear_solve` does not satisfy the full prime-avoidance and Appendix A §10.7 modular solve pipeline.

**Phase-Specific Checks**
Pass:
- `modular.rs` prime chooser is deterministic and skips denominators/nonzero coefficient numerators for the given polynomial list.
- `reduce_q_to_fp` uses exact modular inverse arithmetic, no floats.
- `crt.rs` uses exact integer GCD/inverse arithmetic; checked incompatible CRT path exists.
- `rational_reconstruction.rs` uses exact integer enumeration and returns `None` for nonunique bounded reconstruction.
- sparse/dense rank and nullspace are exact over Fp.
- modular solve result status is candidate-only: `CandidateOnlyRequiresExactQCheck`.

Fail:
- `geosolver-core/src/algebra/linear_solve.rs:138-144` chooses one avoided prime, then advances using raw `next_prime_after`. Later primes can divide matrix denominators or forbidden coefficient parts, causing panic in `reduce_rational_coeff` or rank/solve distortion.
- `solve_inhomogeneous_modular` chooses primes from matrix coefficients only; `geosolver-core/src/algebra/linear_solve.rs:100-102` reduces RHS entries under primes that were not screened against RHS denominators.
- Appendix A §10.7 requires CRT + rational reconstruction before handoff to exact Q checking. Current `linear_solve` returns only the last-prime basis/solution and traces; it does not combine residues or reconstruct rational candidates.

**Forbidden/Fail-Condition Scan Judgment**
Static forbidden scans pass for floats, geometry/fixture/answer dispatch, ordinary `Unsupported`, and modular proof overclaim.

The P3b fail condition is still triggered by incomplete prime avoidance in the modular solve sequence. This is algorithmic, not paperwork.

**Raw Response Consistency Implications**
A `review_summary.yaml` for this response must use:
- `review_status: FAIL_FIXABLE`
- `phase_closable: false`
- `algorithmic_sufficiency.verdict: insufficient`
- nonempty `blocking_findings` / `required_fixes`
- `raw_response_contains_blocker: true`
- `raw_response_contains_required_fix: true`
- `pass_matches_raw_response: false` if any summary attempts `PASS`

A PASS summary would contradict this raw response.

**Required Fixes**
1. Make every modular solve prime avoid all relevant matrix and RHS denominators and forbidden coefficient reductions, not just the first matrix-derived prime.
2. Implement Appendix A §10.7 CRT + rational reconstruction in modular solving, while still marking output as candidate-only pending exact Q verification by the caller.
3. Add tests for multi-prime avoidance, RHS denominator avoidance, and reconstruction/handoff behavior in modular solve.
