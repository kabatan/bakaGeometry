# FCR-P11 Review Packet

Claim ceiling after phase: `PARTIAL_MECHANISM_READY:MECH-011`

## Scope

FCR-P11 covers adversarial red-team execution, final-nonfinite gate selection, and closure
preconditions before FCR-P12. It does not close final candidate-cover readiness, exact-image
readiness, source fidelity, full acceptance, or any R-ID as `VERIFIED`.

## Changed Files

- `geosolver-core/tests/fcr_p11_red_team_suite.rs`
- `docs/ai/changes/RGDTPK-Q-v4-core/FULL_CORE_RED_TEAM_RESULTS.md`
- `docs/ai/changes/RGDTPK-Q-v4-core/FULL_CORE_NONFINITE_RESULTS.md`
- `docs/ai/changes/RGDTPK-Q-v4-core/FULL_CORE_INVARIANT_SCAN_BINDING.md`
- `docs/ai/changes/RGDTPK-Q-v4-core/evidence/FCR-P11/commands.txt`
- `docs/ai/changes/RGDTPK-Q-v4-core/evidence/FCR-P11/command_outputs.txt`

## Evidence Summary

- P11 red-team suite has 10 fresh algebraic inputs not used in FCR-P10. The no-positive-proof
  bounded case uses a distinct two-variable product/quadratic input, not the P10 B1 `x^2 = t`
  shape.
- Each input runs through public `api::solve_target`, with one near-public production composition
  replay check for the two-separator case.
- Support-producing cases assert `CertifiedCandidateCover` and replay acceptance.
- Positive nonfinite returns `CertifiedNonFiniteTargetImage`, but public nonfinite has no
  `CoreRunCertificate`; nonfinite readiness is explicitly excluded from `CANDIDATE_COVER_CORE_READY`.
- No-positive-proof bounded failure does not return nonfinite and retains cost trace evidence.
- Static scans are recorded as P11 preconditions and must be rerun/bound in FCR-P12.

## Verification

- `cargo fmt --manifest-path geosolver-core/Cargo.toml -- --check`: PASS
- `cargo test --manifest-path geosolver-core/Cargo.toml --test fcr_p11_red_team_suite -- --nocapture`: PASS, 10/10

## Required Reviewer Checks

- Confirm the 10 inputs are fresh enough and are not FCR-P10 fixture copies.
- Confirm required categories are represented.
- Confirm nonfinite readiness is not claimed without a machine-readable replay-bound public
  certificate.
- Confirm static scans and `CoreInvariantFlags` are only P12 closure preconditions here, not final
  proof by themselves.
