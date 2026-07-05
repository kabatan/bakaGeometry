# Full Core Replay And Tamper Results

Status: FCR-P12 closure evidence prepared; reviewer approval required for final claim.

Timestamp: 20260706-063128+09:00

## Replay Evidence

| Evidence | Result |
| --- | --- |
| `verify::replay::tests` through all-target test run | PASS, 16/16 |
| support-producing FCR-P10 cases call `replay_run_certificate` | PASS through `fcr_p10_acceptance_suite` |
| support-producing FCR-P11 red-team cases call `replay_run_certificate` | PASS through `fcr_p11_red_team_suite` |
| `verify::run_certificate::tests` through all-target test run | PASS, 6/6 |

## Tamper Coverage

The replay tests reject:

- input, canonical system, DAG, plan, squarefree support, support root, and decoded-candidate tamper;
- projection-message deletion;
- duplicate projection messages and candidate omission/duplication;
- child-message edge mismatch;
- message use of relation outside original block authorization;
- mutual projection-message source cycles;
- forged TargetAction, SparseResultant, RegularChain, NormTrace, and Universal payloads;
- synthetic projection certificates even when hashes are shaped to match;
- global support relation tampering.

## Final-Claim Gates

`require_final_claim_dag_replay_evidence` now accepts hash-bound actual DAG replay evidence and
rejects missing or tampered evidence. `require_final_claim_invariant_evidence` accepts only the
concrete FCR-P12 candidate-cover invariant packet: deterministic scan-output hashes plus replay,
red-team, and acceptance evidence hashes. It rejects missing evidence, arbitrary synthetic hashes,
and tampered scan evidence.

These gates do not grant exact-image readiness, source fidelity, full acceptance, or final
nonfinite readiness.
