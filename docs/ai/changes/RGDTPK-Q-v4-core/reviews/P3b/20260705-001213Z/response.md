FAIL_FIXABLE

**Reviewed R-IDs and MECHs**
Reviewed `RGQ-010`, `RGQ-019`, `RGQ-020`, `RGQ-025`, Appendix A §10.3-§10.7, Plan P3b, and P3b MECH scope: continues `MECH-001`, starts `MECH-006` linear backbone. No R-ID is marked verified.

**Files Inspected**
Inspected all required files: Base Spec sections, Plan P3b/general rules, review archive/schema files, reviewer prompt P3b/general rules, all listed algebra/type implementation files, P3b evidence files, and the two prior failed review responses. Also checked `schemas/review_summary.schema.yaml` for mirror identity.

**Evidence and Commands Inspected**
Inspected `commands.txt`, `command_outputs.txt`, `static_scans.txt`, `function_implementation_table.yaml`, and `notes.md`.

Evidence reports:
- `cargo fmt --manifest-path geosolver-core\Cargo.toml -- --check`: exit 0.
- `cargo test --manifest-path geosolver-core\Cargo.toml -- --nocapture`: exit 0, 51 tests passed.
- Static scans report no floating exact paths, geometry/fixture/answer dispatch, ordinary `Unsupported`, or modular proof overclaim.
- I did not rerun the full test suite. I did run read-only scans/status/hash checks. The repo has no commits and the relevant files are untracked, so freshness is judged by packet evidence/timestamps, not git history.

**Algorithmic Sufficiency Judgment**
Insufficient but fixable.

The second-review blockers are partly fixed:
- Modular solve traces now include `pivot_columns`.
- The solve loops wait for consecutive matching `pivot_columns` before breaking.
- There is a same-rank/different-pivot regression test.
- Prime screening, exact rational-to-Fp reduction, CRT, rational reconstruction, sparse/dense rank/nullspace, and candidate-only proof status are materially present.

Remaining blocker:
- `geosolver-core/src/algebra/linear_solve.rs:82` adds every homogeneous basis sample to `basis_samples`, including samples from pivot profiles that were later rejected as unstable. `linear_solve.rs:97-100` then reconstructs using the whole sample list.
- `linear_solve.rs:129-130` similarly adds every inhomogeneous solution sample to `solution_samples`, and `linear_solve.rs:146-149` reconstructs from the whole list.
- The profile-change branch at `linear_solve.rs:83-87` / `132-136` resets only the stability counter. It does not reset or filter the CRT/reconstruction sample buffers. Therefore reconstruction handoff is not stabilized on rank profile; it may combine residues from different pivot-column profiles.

This is algorithmic, not paperwork: the loop waits longer, but the CRT/reconstruction input is still contaminated by pre-stability samples.

**Phase-Specific Checks**
Pass:
- Every selected modular solve prime is screened through `choose_prime_avoiding_denominators`; inhomogeneous solving includes RHS coefficients via `extra_coefficients`.
- Modular traces expose rank, nullity, and pivot-column rank profile.
- Modular reduction is exact rational-to-Fp arithmetic with integer modular inverse; no float path found.
- CRT and rational reconstruction use integer arithmetic; nonunique reconstruction fails.
- Sparse/dense rank and nullspace are exact over Fp.
- Modular solving exposes traces and reconstructed candidates only; `CandidateOnlyRequiresExactQCheck` does not certify a Q relation.

Fail:
- Homogeneous and inhomogeneous CRT/reconstruction handoff is not limited to the stable rank-profile suffix.
- The regression test at `linear_solve.rs:471-507` proves the trace waits for a third prime, but it does not prove reconstructed candidates exclude the unstable first-profile sample.

**Forbidden/Fail-Condition Scan Judgment**
Static scans pass for floating exact paths, geometry/fixture/answer dispatch, ordinary `Unsupported`, placeholder/stub markers, and modular proof overclaim. No support-vs-deliverable confusion was found in proof status.

**Review Summary Implications**
`REVIEW_SUMMARY_SCHEMA.yaml` and `schemas/review_summary.schema.yaml` are byte-identical; SHA-256 is `CA9A11D4E5511218222D1CD5B675223D3E3017A989CFB776795AC6EF1B352EC0`.

The latest P3b review directory inspected contains only `prompt.md`, so no current `review_summary.yaml` is available to validate. A summary for this response must use:
- `review_status: FAIL_FIXABLE`
- `phase_closable: false`
- `algorithmic_sufficiency.verdict: insufficient`
- nonempty `blocking_findings` and `required_fixes`
- `raw_response_contains_blocker: true`
- `raw_response_contains_required_fix: true`

**Forbidden Claims**
Do not claim P3b `PASS`, `phase_closable: true`, P3b algorithmic sufficiency, full Appendix A §10.7 completion, or “rank-profile-stable reconstruction handoff” until reconstruction uses only stable-profile samples.

**Required Fixes**
1. Reset or filter `basis_samples` and `solution_samples` whenever `pivot_columns` changes.
2. Reconstruct only from the final stable pivot-profile suffix.
3. Extend the same-rank/different-pivot regression to assert CRT/reconstruction candidates are based only on the stable profile, including inhomogeneous coverage or an explicit waiver for why homogeneous coverage is sufficient.
