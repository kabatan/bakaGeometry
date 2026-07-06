# Candidate-Cover Closure

Status: historical closure; current readiness claim suspended by
`ALG_COST_REPAIR_STATUS.md`.

Suspension note: `CANDIDATE_COVER_CORE_READY` and
`SOURCE_FAITHFUL_TO_V4_CANDIDATE_COVER_LAYER` are not current active claims until
`RGDTPK-Q-v4-algebraic-cost-completion-repair-v1` passes all phases in
`ALG_COST_COMPLETION_REPAIR_PLAN.md`.

Historical status: candidate-cover repair complete under the prior evidence boundary.

Allowed claim:

```text
CANDIDATE_COVER_CORE_READY
SOURCE_FAITHFUL_TO_V4_CANDIDATE_COVER_LAYER
```

Forbidden from this repair alone:

```text
exact target-image completion
full supplied-v4 source-fidelity completion
full acceptance completion
benchmark superiority
universal finite-system completeness
any R-ID VERIFIED status
```

## Correctness Statement

The implemented candidate-cover layer proves:

```text
true finite target values subset roots(S)
```

It does not prove the reverse inclusion. Candidate-cover mode is allowed to return extra roots and
does not run exact-image fiber/guard/slack/branch filtering unless `exact_image_mode=true`.

## Code Changes

- `verify/verify_support.rs`
  - preserves Route A target-only support-product verification;
  - adds Route B `ComposedIdealMembershipSupportCertificate`;
  - verifies exact rational identity `S(T) = sum q_i r_i`;
  - rejects multiplier tamper and removed composed relation evidence.
- `compose/compose.rs`
  - preserves separator-composition Route A;
  - allows Route B fallback only when the composed message ideal has a target eliminant;
  - still rejects incomplete message deletion.
- `compose/final_support.rs`
  - constructs support via Route A first;
  - constructs support via Route B composed-ideal elimination when Route A is unavailable;
  - prevents message-relation hard cases from becoming nonfinite without positive proof.
- `solver/orchestrator.rs`
  - emits `ExactImageFilteringNotRequested`;
  - emits `CandidateCoverMayContainSpuriousRoots`;
  - keeps candidate-cover mode separate from exact-image filtering.
- `tests/ccc_candidate_cover_completion.rs`
  - adds four public P11 spurious-root retention cases;
  - adds 16 fresh P12 public red-team algebraic inputs, including four new spurious-root cases;
  - rebuilds DAG/composition/support verification evidence per support-producing success and checks
    the replay-bound global support certificate hash.
- `tests/p13_exact_image_semantics.rs`
  - asserts candidate-cover diagnostics on the semantic spurious-root case.

## Evidence

Focused commands:

```text
cargo test --manifest-path geosolver-core/Cargo.toml verify_support -- --nocapture: PASS
cargo test --manifest-path geosolver-core/Cargo.toml ccc_route_b_final_support -- --nocapture: PASS
cargo test --manifest-path geosolver-core/Cargo.toml --test p13_exact_image_semantics -- --nocapture: PASS
cargo test --manifest-path geosolver-core/Cargo.toml --test ccc_candidate_cover_completion -- --nocapture: PASS
```

Final closure commands:

```text
cargo fmt --manifest-path geosolver-core/Cargo.toml -- --check: PASS
cargo clippy --manifest-path geosolver-core/Cargo.toml --all-targets --all-features -- -D warnings: PASS
cargo test --manifest-path geosolver-core/Cargo.toml --all-targets --all-features: PASS
git diff --check: PASS
```

## Static Scan Binding

Static scans for expected-answer/problem-id dispatch, QE/CAD/RUR/coordinate fallback, and
TODO/stub/unsupported production paths are recorded in `CANDIDATE_COVER_COST_TRACE_SUMMARY.md`.
Runtime `CoreInvariantFlags` are not used alone as static proof.

## Residual Scope

Exact-image equality and full supplied-v4 source-fidelity remain outside this candidate-cover
closure. Production F4 is not claimed. Geometry DSL, natural-language parsing, and diagram/image
understanding remain out of scope.
