# P2 Algorithm Evidence

Status: P2 implementation evidence.

Implemented changes:

- Added input-hash binding over target, variables, equations, semantic encodings, and variable
  roles, plus `make_problem_with_roles` for role-provenance construction without post-hoc stale
  hashes.
- Reused the same input hash helper in replay, avoiding divergence between construction and
  verification.
- Added canonicalization-time semantic consistency checking after zero relation removal.
- Bound canonical and compressed hashes to semantic provenance hashes.
- Added route-budget failure evidence test for cooperative `FiniteResourceFailure`.
- Added P2 phase support to `audit_v4_conformance.py`.

Behavior evidence:

```text
cargo fmt --manifest-path geosolver-core\Cargo.toml --check
python geosolver-core\scripts\audit_v4_conformance.py --phase P2 --strict
cargo test --manifest-path geosolver-core\Cargo.toml --lib problem -- --nocapture
cargo test --manifest-path geosolver-core\Cargo.toml --lib compression -- --nocapture
cargo test --manifest-path geosolver-core\Cargo.toml --lib replay -- --nocapture
cargo test --manifest-path geosolver-core\Cargo.toml --test p3_public_pipeline_integration -- --nocapture
cargo test --manifest-path geosolver-core\Cargo.toml --test p14_full_pipeline_integration p14_public_candidate_cover_success_has_all_result_fields_and_trace -- --nocapture
cargo test --manifest-path geosolver-core\Cargo.toml --no-run
```

All listed commands exited 0.

Known non-P2 blockers remain governed by later phases.
