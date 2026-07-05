# FCR-P12 Review Packet

Requested final FCR claim after reviewer pass:

```text
CANDIDATE_COVER_CORE_READY
```

## Scope

FCR-P12 closes the Full Core Repair candidate-cover layer only. It does not close exact-image
readiness, source fidelity to the full v4 spec, full acceptance, P13/P14/P15/P16, final nonfinite
readiness with a public replay-bound certificate, performance, benchmark, or universal-completeness
claims.

## Changed Files

- `geosolver-core/src/lib.rs`
- `geosolver-core/src/verify/run_certificate.rs`
- `geosolver-core/tests/fcr_p10_acceptance_suite.rs`
- `geosolver-core/tests/fcr_p11_red_team_suite.rs`
- `docs/ai/changes/RGDTPK-Q-v4-core/FULL_CORE_ACCEPTANCE_RESULTS.md`
- `docs/ai/changes/RGDTPK-Q-v4-core/FULL_CORE_REPLAY_TAMPER_RESULTS.md`
- `docs/ai/changes/RGDTPK-Q-v4-core/FULL_CORE_COST_TRACE_SUMMARY.md`
- `docs/ai/changes/RGDTPK-Q-v4-core/FULL_CORE_INVARIANT_SCAN_BINDING.md`
- `docs/ai/changes/RGDTPK-Q-v4-core/CLOSURE.md`
- `docs/ai/ACTIVE_CONTEXT.md`
- `docs/ai/changes/RGDTPK-Q-v4-core/P12G_READINESS.md`
- `docs/ai/changes/RGDTPK-Q-v4-core/evidence/FCR-P12/*`

## Evidence Summary

- FCR-P10 support-producing acceptance suite: PASS, 12/12.
- FCR-P11 red-team suite: PASS, 10/10.
- Full all-target all-feature test run: PASS.
- Clippy with `-D warnings`: PASS after explicit non-behavioral lint allowlist.
- Static scans recorded with SHA256 output hashes.
- Raw deterministic static scan output is archived in `evidence/FCR-P12/scan_outputs.txt`.
- Final invariant evidence has positive and tamper tests.
- Final invariant evidence accepts only the concrete FCR-P12 evidence packet: deterministic scan
  hashes plus replay, red-team, and acceptance evidence hashes.
- Final DAG replay evidence accepts structurally bound actual DAG evidence and rejects missing or
  tampered evidence.
- Nonfinite readiness is excluded from `CANDIDATE_COVER_CORE_READY` because public nonfinite
  results still lack replay-bound nonfinite certificates.

## Reviewer Must Fail If

- `CANDIDATE_COVER_CORE_READY` is treated as exact-image readiness or full acceptance.
- Source-fidelity or R-ID `VERIFIED` is claimed.
- Static scans are treated as sufficient without replay/red-team evidence.
- The nonfinite exclusion is omitted from the final claim.
