# P11 Guardian Boundary Reviewer Result

The P11 MECH-completion boundary review returned `FAIL_FIXABLE`.

Blockers reported:

- `CoreRunCertificate` carried `compression_hash` and `hypergraph_hash`, and `hash_core_run_certificate` included them, but `replay_run_certificate` did not recompute or check those fields. A self-consistently rehashed tamper could pass the earlier replay path.
- `derive_core_invariant_flags` set `no_geometry_dispatch`, `no_problem_id_dispatch`, `no_expected_answer_dispatch`, and `no_qe_cad` to literal `true` without concrete enforcement evidence, which was too broad for the P11 prompt requirement that invariant flags not be true without enforcement.

Required remediation:

- Add replay checks and negative tests for self-consistent `compression_hash`/`hypergraph_hash` tamper.
- Either enforce the literal invariant flags with concrete evidence checks or narrow/derive them so unsupported flags cannot be true.

Remediation applied:

- `CoreRunCertificateInput` now carries concrete `compression_hash` and `hypergraph_hash` values. `build_core_run_certificate` records those values instead of placeholder sentinels.
- `replay_run_certificate` recomputes `CompressionState::from_system(canonical).to_compressed_system().compressed_hash` and `build_relation_variable_hypergraph(&compressed).hypergraph_hash`, then rejects mismatches.
- The P11 replay tamper test now self-consistently rehashes altered `compression_hash` and `hypergraph_hash` fields and asserts both are rejected.
- `derive_core_invariant_flags` no longer sets `no_geometry_dispatch`, `no_problem_id_dispatch`, `no_expected_answer_dispatch`, or `no_qe_cad` to literal `true`. Those final-claim flags remain `false` at P11; replay requires only the P11-supported invariant subset via `p11_replay_enforced`.
