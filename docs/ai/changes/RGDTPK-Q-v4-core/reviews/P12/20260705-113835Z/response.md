RESULT: PASS

Blockers: none.

P12 is closable only for `PARTIAL_MECHANISM_READY:MECH-011`. `PLAN.md` scopes P12 to MECH-011 and roots/decode work at `PLAN.md:608`, with tasks through exact isolation, decode binding, algebraic-root records, and integration tests at `PLAN.md:615`.

Prior FAIL_FIXABLE issues are adequately remediated:

- Production decode wiring exists in `finalize_candidate_cover_result`, which calls squarefree, exact isolation, and `decode_candidates` before returning `CertifiedCandidateCover`: `geosolver-core/src/compose/final_support.rs:240`.
- Replay now rejects omitted/duplicate candidates by length, index set, interval, support/target, and recomputed hash checks: `geosolver-core/src/verify/replay.rs:224`.
- Tamper tests cover omission and duplicate candidates with recomputed certificates: `geosolver-core/src/verify/replay.rs:495`.
- Integration test proves non-placeholder exact roots/candidates from the production finalizer: `geosolver-core/tests/p12_roots_decode_integration.rs:21`.

Evidence is sufficient: archived focused P12 tests pass `6 + 1`, full suite passes `177 + 1 + doc-tests`, fmt/check and `git diff --check` pass, and static scans report no placeholder/floating/stub matches: `docs/ai/changes/RGDTPK-Q-v4-core/evidence/P12/command_outputs.txt:13`, `docs/ai/changes/RGDTPK-Q-v4-core/evidence/P12/command_outputs.txt:31`, `docs/ai/changes/RGDTPK-Q-v4-core/evidence/P12/static_scans.txt:13`.

Forbidden claims: no P13 exact-image semantics, no P14 public orchestration, no P15 acceptance, no P16 final readiness, no source-faithful or acceptance-complete status. No R-ID is verified.

Next action: archive this as P12 MECH completion PASS and close MECH-011 only.
