P12G-f is implemented as an actual-DAG replay readiness hook and P14/P16 blocker.
`CoreRunCertificate` now binds `final_dag_replay_evidence_hash`; `FinalDagReplayEvidence` carries
the TargetProjectionDAG hash, projection message hashes, plan hashes, message block ids,
per-message source relation hashes, child dependency hashes, per-block authorization hashes, edge
authorization hashes, and an explicit `actual_dag_replay_verified` flag.

Final claims are rejected when that evidence is missing or structurally unbound. Even structurally
bound evidence still returns a P14 blocker in P12G, because no production path derives it from an
actual TargetProjectionDAG replay yet. The production run-certificate builder still sets this field
to `None`, so P14 cannot close until actual DAG/block replay replaces synthetic all-relations replay
for final claims.

Regressions:

- `p12g_final_dag_claim_is_blocked_without_actual_dag_replay_evidence`
- `p12g_dag_authorization_hash_bound_into_run_certificate`
- `p12g_replay_rejects_message_using_relation_outside_original_block`
- `p12g_replay_rejects_child_message_not_on_declared_dag_edge`
