RESULT: PASS

The prior blocker is fixed for P12G scope.

Quality reviewer verified that `require_final_claim_dag_replay_evidence` rejects both structurally
unbound evidence and structurally bound evidence, with the P14 blocker hard-coded in
`geosolver-core/src/verify/run_certificate.rs`. The structural helper can identify bound evidence,
but it is not a success path for final claims. `CoreRunCertificate` includes
`final_dag_replay_evidence_hash` in the run hash, and the binding test now verifies binding while
still expecting final-claim rejection.

Quality reviewer also verified that the actual-block replay helper is `pub(crate)` and no longer
test-only in `geosolver-core/src/verify/replay.rs`, and that the required tests are present:

- `p12g_replay_rejects_message_using_relation_outside_original_block`
- `p12g_replay_rejects_child_message_not_on_declared_dag_edge`
- `p12g_dag_authorization_hash_bound_into_run_certificate`
- `p12g_generality_stress_manifest_lists_required_cases`

Docs and evidence explicitly state that P14/P16 remain blocked until actual DAG/block replay
replaces synthetic all-relations replay for final claims.
