# P12 Spec Verifier PASS After Integration Remediation

The read-only `spec_verifier` returned `PASS` after P12 remediation.

Blocking issues: none.

Reviewer findings:

- `decode_candidates` binds target, support hash, root index, interval, and candidate hash in `geosolver-core/src/roots/decode.rs`.
- `finalize_candidate_cover_result` computes squarefree support, exact root isolation, rejects empty real roots for support-producing result construction, and decodes candidates before returning `CertifiedCandidateCover` in `geosolver-core/src/compose/final_support.rs`.
- Replay rejects omitted or duplicate decoded candidates through length, index-set, interval, support, target, and recomputed hash checks in `geosolver-core/src/verify/replay.rs`.
- `p12_replay_rejects_candidate_omission_and_duplicates_even_when_hashes_match` covers omitted/duplicate decoded candidate tampering.
- `geosolver-core/tests/p12_roots_decode_integration.rs` exercises a nonzero support case with exact roots and non-placeholder decoded candidates.

Accepted claim ceiling:

```text
PARTIAL_MECHANISM_READY:MECH-011
```

Residual limits:

P12 does not verify P13 exact-image semantics, P14 public pipeline orchestration, P15 acceptance suites, P16 final readiness, source-faithful status, acceptance-complete status, or any R-ID as VERIFIED.
