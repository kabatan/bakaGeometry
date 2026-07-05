# FCR-P12 Reviewer Results

Timestamp: 20260706-063128+09:00

## Initial Review

The first FCR-P12 `spec_verifier` and `guardian_boundary_reviewer` runs returned `FAIL_FIXABLE`.
The blockers were:

- `CoreInvariantFlags` were treated as free booleans for source-wide no-dispatch/no-QE claims.
- Raw deterministic scan outputs were not archived.
- `FULL_CORE_SOURCE_SPEC_COMPLIANCE_MAP.md` contained stale FCR-P1A defect rows without a P12
  supersession addendum.

## Remediation

- Runtime `CoreRunCertificate` invariants no longer assert source-wide no-dispatch/no-QE flags.
- `require_final_claim_invariant_evidence` now accepts only the concrete FCR-P12 candidate-cover
  evidence packet: deterministic scan hashes plus replay/tamper, red-team, and acceptance evidence
  hashes.
- Raw deterministic scan output is archived in `scan_outputs.txt`.
- `FULL_CORE_SOURCE_SPEC_COMPLIANCE_MAP.md` now has an FCR-P12 supersession addendum that preserves
  exact-image/source-fidelity/full-acceptance exclusions.

## Rerun Review

- `spec_verifier`: PASS.
- `guardian_boundary_reviewer`: PASS.
- `quality_reviewer`: PASS.

Approved claim after all reviewer passes:

```text
CANDIDATE_COVER_CORE_READY
```

Still forbidden:

```text
EXACT_IMAGE_CORE_READY
SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC
RGDTPK_Q_V4_ACCEPTANCE_COMPLETE
final public replay-bound nonfinite readiness
performance or benchmark readiness
any R-ID VERIFIED status
```
