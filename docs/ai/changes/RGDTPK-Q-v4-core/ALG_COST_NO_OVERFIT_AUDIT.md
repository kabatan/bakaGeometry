# Algebraic-Cost No-Overfit Audit

Scope: ACR-P10 anti-overfit, static-scan, and claim-boundary audit.

## Forbidden Fixture Scan

Command:

```text
rg -n -i "mixtilinear|mixtilinear_candidate_cover_problem|circumcircle|incircle|tangent_solver|expected_cos|expected target value|expected support polynomial|known_support_polynomial|diagnostic_problem|problem hash|geometry name" geosolver-core\src geosolver-core\tests
```

Result: PASS, no matches.

The same terms can appear in normative docs as forbidden examples. That is not implementation
dispatch and is not counted as a source/test fixture dependency.

## Anti-Overfit Stress Evidence

ACR-P9 runs S1-S8 through three deterministic variants:

- baseline IDs with nonzero rational coefficient scaling
- non-base variable-ID renaming with coefficient scaling
- relation-order permutation with coefficient scaling

The stress suite asserts route/cost trace, exact support verification, projection-message
verification, and certificate replay. It does not hardcode expected support polynomials.

## CoreInvariantFlags Boundary

Runtime `CoreInvariantFlags` are intentionally conservative:

- Per-run certificates derive runtime flags from actual messages and keep source-wide dispatch/QE
  claims false unless separate final invariant evidence is supplied.
- `fcr_p12_candidate_cover_final_invariant_flags()` and
  `fcr_p12_candidate_cover_final_invariant_evidence()` bind source-wide no-dispatch/no-QE/no-hidden
  fallback claims to scan/evidence hashes.
- `require_final_claim_invariant_evidence()` rejects final invariant claims without exactly matching
  final evidence.

This ties CoreInvariantFlags and static scans to closure without treating ordinary runtime
certificates as source-wide proof.

## No-Dispatch / No-QE-CAD Scan Boundary

Static scan for QE/CAD and coordinate fallback terms returns expected references in invariant flags,
tests, and explicit rejection messages. The closure claim is therefore evidence-bound:

- no geometry/problem-id/expected-answer dispatch is supported by the forbidden fixture scan and
  final invariant evidence;
- no QE/CAD claim is only valid through final invariant evidence, not from a generic runtime flag;
- action/Krylov and quotient paths explicitly reject coordinate-root or full-coordinate-RUR exports.

## Candidate-Cover vs Exact-Image

`CANDIDATE_COVER_CORE_READY` means the finite target image is covered by `roots(S)`. It does not
mean exact target-image equality. Spurious roots remain valid in candidate-cover mode and are tested
by the P12/CCC red-team cases.

This audit forbids using ACR-P10 closure as `RGDTPK_Q_V4_ACCEPTANCE_COMPLETE` or full exact-image
acceptance.

