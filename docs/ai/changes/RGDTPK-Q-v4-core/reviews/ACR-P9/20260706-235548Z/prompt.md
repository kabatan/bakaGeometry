# ACR-P9 Reviewer Prompt

Use the ACR-P9 reviewer prompt in `ALG_COST_COMPLETION_REVIEWER_PROMPTS.md`.

Scope: ACR-P9 only.

Review goal: determine whether the generic large-footprint stress suite is sufficient to close ACR-P9 under `ALG_COST_COMPLETION_REPAIR_PLAN.md` and `ALG_COST_COMPLETION_REPAIR_BASE_SPEC.md`.

Files to inspect:

- `geosolver-core/tests/acr_p9_large_footprint_stress.rs`
- `geosolver-core/src/solver/pipeline.rs`
- `geosolver-core/src/solver/orchestrator.rs`
- `geosolver-core/src/verify/run_certificate.rs`
- `geosolver-core/src/verify/replay.rs`
- `geosolver-core/src/kernels/sparse_resultant.rs`
- `geosolver-core/src/kernels/universal_elimination.rs`
- `geosolver-core/src/kernels/regular_chain_projection.rs`
- `geosolver-core/src/planner/admission.rs`
- `geosolver-core/src/planner/cost_model.rs`
- `geosolver-core/src/planner/kernel_plan.rs`
- `geosolver-core/src/planner/ladder.rs`
- `geosolver-core/src/planner/planner.rs`
- `geosolver-core/src/graph/projection_dag.rs`
- `geosolver-core/src/compose/compose.rs`
- `geosolver-core/src/compose/separator_elimination.rs`

Evidence:

- `docs/ai/changes/RGDTPK-Q-v4-core/evidence/ACR-P9/MECH_EVIDENCE.md`
- `docs/ai/changes/RGDTPK-Q-v4-core/evidence/ACR-P9/stress_matrix.yaml`
- `docs/ai/changes/RGDTPK-Q-v4-core/evidence/ACR-P9/commands.txt`

Fresh command results:

```text
cargo fmt --manifest-path geosolver-core\Cargo.toml --check
PASS

forbidden-marker scan over Rust sources and P9-related tests
PASS: no matches

cargo test --manifest-path geosolver-core\Cargo.toml --test acr_p9_large_footprint_stress -- --test-threads=1
PASS: 8 passed; 0 failed; finished in 83.74s
```

Required reviewer output:

- PASS, FAIL_FIXABLE, or FAIL_BLOCKING.
- Record each S1-S8 family with:
  - `dense_trs_status`
  - `sparse_resultant_status`
  - `successful_route`
  - `projection_message_verified`
  - `support_verified`
  - `replay_accepted`
- Include at least one adversarial algebraic counterexample or challenge.
- Do not grant ACR-P10 closure, exact-image acceptance, source-fidelity, or final readiness.
