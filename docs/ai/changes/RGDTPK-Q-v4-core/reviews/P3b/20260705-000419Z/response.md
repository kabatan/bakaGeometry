FAIL_FIXABLE

**Reviewed R-IDs and MECHs**
Reviewed `RGQ-010`, `RGQ-019`, `RGQ-020`, `RGQ-025`, Appendix A §10.3-§10.7, and P3b scope: continues `MECH-001`, starts `MECH-006`. No R-ID is marked verified.

**Files Inspected**
Inspected all requested Base Spec, Plan, schema, prompt, implementation, type, and P3b evidence files, including the prior failed review at `reviews/P3b/20260704-235154Z/response.md`.

**Evidence and Commands Inspected**
Inspected `commands.txt`, `command_outputs.txt`, `static_scans.txt`, `function_implementation_table.yaml`, and `notes.md`.

Evidence reports:
- `cargo fmt --check`: exit 0.
- `cargo test`: exit 0, 50 tests passed.
- Static scans: no floating exact path, no geometry/fixture/answer dispatch, no ordinary `Unsupported`, no modular proof overclaim.
- I did not rerun tests because this was a read-only review.

**Algorithmic Sufficiency Judgment**
Insufficient but fixable.

The prior three blockers are materially fixed:
- Every modular solve prime is selected through `choose_prime_avoiding_denominators` for the full matrix coefficient set, and inhomogeneous solving includes RHS coefficients.
- Modular reduction uses exact rational-to-Fp arithmetic.
- CRT plus rational reconstruction is now present in `linear_solve.rs`, and outputs remain `CandidateOnlyRequiresExactQCheck`.
- Tests cover deterministic prime selection, CRT round trip and incompatible failure, rational reconstruction success and nonunique failure, matrix rank/nullspace, multi-prime avoidance, RHS denominator avoidance, and candidate-only reconstruction/handoff.

New blocker:
- Appendix A §10.7 requires continuing until the **rank profile** stabilizes. `PrimeSolveTrace` records only `prime`, `rank`, and `nullity` in `geosolver-core/src/algebra/linear_solve.rs:31`; both solve loops compare only previous rank at `linear_solve.rs:80` and `linear_solve.rs:128`. Pivot columns are computed in helper paths but are not exposed in the modular solve trace or used for stability. This means CRT/reconstruction may combine samples after equal ranks but different pivot profiles.

**Phase-Specific Checks**
Pass:
- Prime screening covers matrix coefficients and RHS coefficients.
- Exact rational-to-Fp, CRT, rational reconstruction, sparse/dense rank, and nullspace paths are integer/finite-field based.
- Unstable/nonunique rational reconstruction fails.
- Modular solve exposes candidate fields and does not certify a Q relation.

Fail:
- Modular linear solving does not implement Appendix A §10.7 rank-profile stabilization. It stabilizes rank only.
- Tests do not include a same-rank/different-pivot-profile case that would prevent premature reconstruction.

**Forbidden/Fail-Condition Scan Judgment**
Static scans pass for floating exact paths, geometry/fixture/answer dispatch, ordinary `Unsupported`, and modular proof overclaim. No support-vs-deliverable confusion was found for modular candidates: the code marks them candidate-only.

**Raw Response Consistency Implications**
A `review_summary.yaml` for this response must use:
- `review_status: FAIL_FIXABLE`
- `phase_closable: false`
- `algorithmic_sufficiency.verdict: insufficient`
- nonempty `blocking_findings` and `required_fixes`
- `raw_response_contains_blocker: true`
- `raw_response_contains_required_fix: true`
- `pass_matches_raw_response: false` if any summary attempts `PASS`

`REVIEW_SUMMARY_SCHEMA.yaml` and `schemas/review_summary.schema.yaml` are byte-identical. The current `reviews/P3b/20260705-000419Z` directory contained only `prompt.md`, so no current `review_summary.yaml` or `evidence_manifest.yaml` was available to validate.

**Forbidden Claims**
Do not claim P3b `PASS`, `phase_closable: true`, full Appendix A §10.7 completion, or P3b algorithmic sufficiency until rank-profile stabilization is implemented and tested.

**Required Fixes**
1. Extend modular solve traces to include rank profile, at minimum pivot columns for each prime.
2. Stabilize homogeneous and inhomogeneous modular solving on rank profile, not rank alone, before CRT/reconstruction handoff.
3. Add a regression test with equal rank but different pivot columns across selected primes, proving reconstruction waits for stable rank profile.
