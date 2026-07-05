RESULT: PASS

P11 may be closed. Claim ceiling: P11 closes `MECH-010` and advances `MECH-016` only; P12 may start. No R-ID is VERIFIED, and there is no final readiness, source-faithful, acceptance-complete, root/decode completion, exact-image, orchestration, or performance-readiness claim.

Blockers: none.

Basis: `CoreRunCertificateInput` now carries `compression_hash` and `hypergraph_hash`, builder stores them, and `run_hash` includes them. Replay recomputes both and rejects mismatches. The tamper test self-consistently rehashes compression/hypergraph tampering and rejects both. `derive_core_invariant_flags` leaves final anti-dispatch/QE-CAD flags false at P11, while replay requires only the P11-supported subset via `p11_replay_enforced`. Static scan evidence reports no removed sentinels or broad invariant helpers. Latest evidence shows spec reviewer PASS, quality reviewer PASS, 13 focused P11 tests passed, full `geosolver-core` tests passed, `cargo fmt --check` passed, and `git diff --check` had no whitespace errors beyond CRLF warnings.

Forbidden claims: any R-ID VERIFIED, final readiness, source-faithful, acceptance-complete, P12/P13/P14/P15/P16 completion, performance readiness.

Next action: close P11 at the stated ceiling and admit/start P12 separately.
