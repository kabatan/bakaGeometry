# ACR-P6 MECH Evidence

Status: implementation evidence, not readiness authority.

Claim ceiling remains:

```text
CANDIDATE_COVER_PIPELINE_PRESENT_BUT_ALGEBRAIC_COST_INCOMPLETE
```

## Scope

Implemented the ACR-P6 `UniversalTargetElimination` success-route slice:

- Universal internal strategy plan steps now carry the same dominant-cost estimate and route-budget
  fields used by top-level kernel plans.
- Dense target-relation and sparse-resultant internal Universal stages are classified before
  execution and skipped when their internal algebraic cost is `CostProhibited`.
- Universal preserves the declared internal order for the remaining bounded stages.
- Universal refuses to execute a stage if a cost-prohibited stage somehow remains enabled.
- Universal binds the chosen stage estimate and route budget into the nested subplan before
  executing the delegated bounded kernel.
- Universal certificates now record strategy trace rows, skipped cost-prohibited stage hashes,
  failed strategy hashes, chosen strategy, and replayable exact relation evidence through the
  selected inner certificate payload.
- Projection-message verification reconstructs Universal strategy hashes from the certificate trace,
  including cost class, estimate hashes, route budget hashes, predicted work units, budget work cap,
  and elapsed-step cap.

## Changed Files

- `geosolver-core/src/kernels/universal_elimination.rs`
- `geosolver-core/src/planner/cost_model.rs`
- `geosolver-core/src/planner/kernel_plan.rs`
- `geosolver-core/src/solver/pipeline.rs`
- `geosolver-core/src/verify/certificates.rs`
- `geosolver-core/src/verify/replay.rs`
- `geosolver-core/src/verify/verify_message.rs`

## Verification Run

All cargo commands below were run from `geosolver-core`.

```text
cargo fmt --check
cargo check
cargo test --lib acr_p6 -- --nocapture
cargo test --lib kernels::universal_elimination::tests -- --nocapture
cargo test --lib verify::replay::tests -- --nocapture
cargo test --lib
rg -n -i "<forbidden diagnostic marker expression>" src -g "*.rs"
git diff --check
```

Observed result:

```text
cargo fmt --check: passed
cargo check: passed
acr_p6: 1 passed
kernels::universal_elimination::tests: 10 passed
verify::replay::tests: 16 passed
cargo test --lib: 238 passed
forbidden-marker scan over implementation Rust sources: no matches
git diff --check: exit 0, CRLF conversion warnings only
```

## Acceptance Evidence

The P6 stress test is:

```text
solver::pipeline::tests::acr_p6_universal_skips_cost_prohibited_dense_sparse_and_returns_candidate_cover
```

It verifies a generic high-footprint algebraic pipeline case through validate, canonicalize,
compression, graph/DAG planning, block execution, projection-message verification, composition,
support extraction, root extraction, certificate construction, and cost-trace validation.

The test forces the first planned block's declared route to Universal only, then verifies:

- final pipeline status is `CertifiedCandidateCover`;
- exactly one projection message is emitted for the block;
- the Universal certificate records dense target-relation and sparse-resultant internal stages as
  disabled `CostProhibited` stages before execution;
- skipped cost-prohibited stage hashes include the dense and sparse stage hashes;
- failed strategy hashes include the skipped dense and sparse stage hashes;
- Universal chooses the bounded target-action internal strategy after those skips;
- the selected inner payload is `TargetAction`;
- `verify_projection_message` accepts the generated Universal projection message.

## Anti-Overfit Boundary

No prior diagnostic input, expected-answer hook, domain-name dispatch, or fixed external benchmark
fixture was used in this implementation slice.

Reviews are required before ACR-P6 may be treated as closed.
