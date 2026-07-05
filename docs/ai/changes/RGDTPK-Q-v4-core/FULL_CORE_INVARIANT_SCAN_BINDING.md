# Full Core Invariant and Static-Scan Binding

Status: FCR-P11 preconditions recorded; FCR-P12 must rerun fresh final scans and bind hashes.

Timestamp: 20260706-055251+09:00

`CoreInvariantFlags` must not be treated as free booleans. Final closure must bind each invariant
flag to fresh static scans and to dynamic replay/tamper or semantic evidence where applicable.

Required binding:

| Closure invariant | Required static evidence | Required dynamic evidence | Status |
| --- | --- | --- | --- |
| no geometry dispatch | `rg` no-dispatch scan over `geosolver-core/src`: hit_count=137, reviewed as terminology/invariant names rather than production dispatch | P11 red-team inputs are algebraic-only and avoid geometry labels | P11 precondition recorded; rerun in FCR-P12 |
| no problem-id/fixture/expected-answer dispatch | same no-dispatch scan plus P11 fresh variable ids/relation orders | P11 suite mutates variable ids, relation order, and polynomial syntax outside FCR-P10 inputs | P11 precondition recorded; rerun in FCR-P12 |
| no hidden fallback | fallback/stub scan over `geosolver-core/src`: hit_count=20, reviewed as quarantined/pre-existing markers | support-producing P11 cases require replay acceptance; failure case keeps trace | P11 precondition recorded; rerun in FCR-P12 |
| no QE/CAD/RCF/full-coordinate fallback | QE/CAD/full-coordinate scan over `geosolver-core/src`: hit_count=3, reviewed as rejection/error text | success paths expose target support, isolated target roots, and decoded target candidates only | P11 precondition recorded; rerun in FCR-P12 |
| actual DAG replay | inspect `CoreRunCertificate`/`FinalDagReplayEvidence` and run replay tests | P11 support-producing cases call `replay_run_certificate`; FCR-P12 must rerun replay/tamper suite | P11 precondition recorded; rerun in FCR-P12 |

P11 does not close final invariant evidence. FCR-P12 final closure must rerun scans, record command
outputs, bind scan hashes where required, and keep static scans as necessary but insufficient
without dynamic red-team and replay/tamper evidence.
