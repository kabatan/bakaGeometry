# Full Core Invariant and Static-Scan Binding

Status: FCR-P12 scan and replay binding recorded; reviewer approval required for final claim.

Timestamp: 20260706-063128+09:00

`CoreInvariantFlags` must not be treated as free booleans. Final closure must bind each invariant
flag to deterministic static scans and to dynamic replay/tamper or semantic evidence where
applicable. Raw scan output is archived in `evidence/FCR-P12/scan_outputs.txt`.

Required binding:

| Closure invariant | Required static evidence | Required dynamic evidence | Status |
| --- | --- | --- | --- |
| no geometry dispatch | dispatch scan hit_count=137, SHA256 `a66922f1042ad07a2166afe97d596566af1793954830af59da376f2b48787cac`; reviewed as terminology/invariant names rather than production dispatch | FCR-P11 red-team inputs are algebraic-only and avoid geometry labels | bound for FCR-P12 candidate-cover claim |
| no problem-id/fixture/expected-answer dispatch | same dispatch scan hash plus FCR-P11 fresh variable ids/relation orders | FCR-P11 suite mutates variable ids, relation order, and polynomial syntax outside FCR-P10 inputs | bound for FCR-P12 candidate-cover claim |
| no hidden fallback | temporary/stub scan hit_count=20, SHA256 `10dbf97a66fba04b6484b62b8a696599abbbef5eb1d11830f7e1e60e494e6dab`; reviewed as quarantined test helpers, explicit error constructor, and test-only synthetic certificates | support-producing P10/P11 cases require replay acceptance; failure cases keep trace | bound for FCR-P12 candidate-cover claim |
| no QE/CAD/RCF/full-coordinate fallback | QE/CAD/full-coordinate scan hit_count=3, SHA256 `46fa741ce667f57cccb05c973d65fa83a0dbef4c4c2ba97820913c532b5b4c62`; reviewed as rejection/error text | success paths expose target support, isolated target roots, and decoded target candidates only | bound for FCR-P12 candidate-cover claim |
| actual DAG replay | `verify::replay::tests` PASS 16/16; `verify::run_certificate::tests` PASS 6/6 | replay rejects message/certificate/support/root/candidate/DAG tamper; final DAG evidence accepts structurally bound actual DAG evidence and rejects tamper | bound for FCR-P12 candidate-cover claim |

Static scans remain necessary but not sufficient. Runtime `CoreRunCertificate` invariants do not
assert source-wide no-dispatch/no-QE flags as free booleans. The final-claim invariant gate accepts
only the concrete FCR-P12 candidate-cover evidence packet: the deterministic scan hashes above plus
FCR-P10 acceptance, FCR-P11 red-team execution, replay/tamper tests, and the final invariant/DAG
gate tests.
