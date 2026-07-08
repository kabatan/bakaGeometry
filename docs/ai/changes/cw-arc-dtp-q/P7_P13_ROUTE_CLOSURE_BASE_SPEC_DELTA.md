# P7-P13 Route Closure Base Spec Delta

Purpose: controlling delta for the P7-P13 route-closure implementation and review.
Status: admitted route-closure authority after guardian boundary review on 2026-07-08.
Authority: this file narrows and strengthens the active V3 Base Spec for P7-P13 only. `BASE_SPEC.md` remains controlling outside this delta.

## Source Intent

The user required the following review constraints before P7-P13 implementation:

1. P7-P12 route closure must not pass only because the P4-P6 foundation exists.
2. Each route must have route-forcing, no-fallback, exact-proof-gate, and tamper tests.
3. The existence of `FairProofSchedule::unbounded()` and top-level final-spec unbounded ideal execution are separate claims.
4. Every phase must check that `FactorizationResult::ResourceFailure` or `Partial` is not treated as `Complete`.
5. `origin_evidence` remains ranking evidence, not an adoption condition.

## Controlling Requirements

P7-P13-R1. P7-P12 route closure requires route-specific production data-flow matching Base Spec section 8. Passing P4-P6 shared primitives is insufficient evidence for any route PASS.

P7-P13-R2. Each P7-P12 candidate route must have executable tests for all of:

- route-forcing with only that origin enabled;
- complete fallback disabled;
- candidate adoption only through fixed exact proof and verifier replay;
- meaningful certificate tamper rejection when the route reaches solver success, plus route/candidate evidence tamper rejection where the route produces route evidence.

Source inspection is required during review, but it cannot replace these tests.

P7-P13-R3. `FairProofSchedule::unbounded()` may be tested as a lazy fair tuple iterator, but that alone never proves top-level solver conformance to final unbounded ideal execution. Any top-level unbounded claim must inspect solver orchestration, fallback reachability, and resource behavior separately.

P7-P13-R4. Every implementation and review checkpoint for P7-P13 must inspect factor scheduling and candidate adoption paths for false completion. `FactorizationResult::ResourceFailure` and `FactorizationResult::Partial` must never be treated as `FactorizationStatus::Complete`, either directly or through fallback cloning of the original polynomial.

P7-P13-R5. `origin_evidence` may affect deterministic candidate ranking only after exact/reconstructed status, degree, and prime evidence. It must not be used as a certificate, route adoption condition, proof shortcut, or substitute for fixed proof.

P7-P13-R6. LocalizedSchur is included in P13 but is not a candidate-origin PASS route unless it constructs an exact target-only certificate replayable against the original system. Support information alone is not solver success.

## Claim Boundary

After admission, this delta authorizes and restricts only the P7-P13 route-closure implementation scope. It does not itself close P7-P13, and it does not close P14+, P15, P16, final V3 completion, source-faithfulness, production-safety, readiness, acceptance-complete, or any R-ID verified claim.
