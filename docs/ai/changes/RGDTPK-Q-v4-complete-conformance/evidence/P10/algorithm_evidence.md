# P10 Algorithm Evidence

Implemented/verified behavior:

- Resultant templates bind input polynomials, eliminated variable, keep variables, support sets,
  matrix dimensions, and template hash.
- SparseResultant planning estimates pairwise template chains, swell, output support, matrix rows,
  matrix columns, route work, and resource budgets before execution.
- Execution recomputes the planned template chain, checks route-local finite resource guards, and
  rejects stale swell/template evidence.
- Resultant computation returns candidate-only proof status and accepts only exact recomputation via
  `verify_resultant_certificate`.
- Resultant certificate replay now rejects missing modular traces and recomputes the expected
  `ModularOptions::default()` traces before accepting; modular traces must be nonempty and exactly
  equal to the replayed trace list.
- Output relation variables are checked against exported variables before message construction.
- SparseResultant certificates carry template/input support binding, modular traces, relation hash,
  backend kind, and exact verification hash.
- Message replay recomputes resultant certificates and rejects forged payloads.

Regression coverage:

- `resultant_support_template_and_certificate_are_exact`
- `acr_p9_quadratic_subresultant_backend_is_exact_and_replayable`
- `resultant_certificate_rejects_tampered_relation_hash`
- `resultant_certificate_rejects_tampered_trace_prime_without_panic`
- `resultant_certificate_rejects_nonprime_trace_modulus`
- `resultant_certificate_rejects_missing_modular_traces`
- `p8b_sparse_resultant_kernel_produces_exact_exported_relation`
- `p10_sparse_resultant_replay_rejects_missing_modular_traces_after_rehash`
- `p12g_sparse_resultant_template_plan_does_not_overclaim_binary_chain`
- finite resource guard tests under `acr_p4_*`.

Static audit:

- `audit_v4_conformance.py --phase P10 --strict` checks template/certificate/resultant markers,
  finite resource markers, sparse-resultant execution markers, verifier exact replay markers, and
  the non-overclaiming binary-chain regression marker.
