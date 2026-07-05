# P9 Reviewer FAIL_FIXABLE Remediation

Reviewer result before remediation: `FAIL_FIXABLE`.

That response is a failed review only. It does not close P9, does not close `MECH-007`, and is not executable proof.

Findings addressed:

1. `RegularChainProjection` dropped compressed-system guards and overclaimed the single-chain regular-chain helper as P9-ready.
2. `NormTraceProjection` rejected every tower except exactly one non-exported algebraic variable and overclaimed the single-variable norm/trace helper as P9-ready.

Fixes:

1. `geosolver-core/src/algebra/regular_chain.rs` now builds a component DAG for admitted regular-chain structures with duplicate main-variable components and preserves guards on component chains.
2. `geosolver-core/src/kernels/regular_chain_projection.rs` now passes `CompressedSystemQ.guards` into planning and execution. The new guard-binding test verifies execution mismatch after guard deletion and certificate-hash change after guard mutation.
3. `geosolver-core/src/algebra/norm_trace.rs` now exposes `TowerPlanDescription`, `TowerStep`, `detect_explicit_tower_plan`, `norm_relation_for_tower_plan`, and `verify_norm_tower_plan_relation`.
4. `geosolver-core/src/kernels/norm_trace_projection.rs` now plans and executes against the multi-step tower plan and exact recomputation verifier.

Fresh verification after fixes:

- `cargo fmt --manifest-path geosolver-core/Cargo.toml --check`: pass.
- `cargo test --manifest-path geosolver-core/Cargo.toml p9_ -- --nocapture`: 6 passed, 0 failed.
- `cargo test --manifest-path geosolver-core/Cargo.toml`: 154 passed, 0 failed.
- Forbidden P9 shortcut scan: no matches.
- `git diff --check`: exit 0, with CRLF conversion warnings only.
