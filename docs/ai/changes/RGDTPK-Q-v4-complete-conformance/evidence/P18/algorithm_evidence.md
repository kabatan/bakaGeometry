Purpose: P18 algorithm evidence
Status: evidence, non-authoritative

# P18 Algorithm Evidence

Fresh command evidence is in `../final/final_commands.log`.

## Final Commands

- `cargo fmt --manifest-path geosolver-core\Cargo.toml --check`: exit 0.
- `cargo clippy --manifest-path geosolver-core\Cargo.toml --all-targets -- -D warnings`: exit 0.
- `cargo test --manifest-path geosolver-core\Cargo.toml`: exit 0.
- `python geosolver-core\scripts\audit_v4_conformance.py --strict`: exit 0, findings 0.

## P18 Conformance Tests

`geosolver-core/tests/v4_candidate_cover_conformance.rs` adds five public/near-public tests:

1. `p18_public_api_ignores_roles_names_and_relation_order_for_mechanism`
   - Exercises the public API through renamed variables, misleading roles, and relation order permutations.
   - Checks that the mechanism returns a candidate cover with stable support/candidate counts, not a fixture/role dispatch.
2. `p18_descartes_option_decodes_hash_bound_candidates`
   - Selects the Descartes/Vincent root-isolation option.
   - Checks candidate count, support hash binding, root index binding, and nonempty candidate hashes.
3. `p18_exact_image_request_is_explicit_scope_guard_not_success`
   - Requests exact-image mode and verifies it does not return exact-image success.
   - Checks the explicit out-of-scope diagnostic path.
4. `p18_normalization_and_hash_binding_are_order_independent`
   - Constructs the same target algebraic system with different relation/term order.
   - Checks candidate-cover support hash and decoded candidate hashes remain stable.
5. `p18_bounded_failure_returns_evidence_cost_trace_not_unsupported`
   - Uses intentionally tight route limits.
   - Checks failure is evidence-backed and includes cost trace/failure-stage data rather than an unsupported status.

## Regression Tests Updated In P18

- `geosolver-core/tests/acr_p9_large_footprint_stress.rs`
  - Updated stale route-specific expectations so the suite verifies candidate-cover success, allowed failed-route evidence, and graph split behavior rather than requiring Universal to be the successful route when a stronger admitted generic route succeeds.
- `geosolver-core/tests/generic_success_route_planner.rs`
  - Updated generic success-route assertions to distinguish a required failed dense/Universal route trace from the successful route.
  - Removed stale message-construction assumptions that conflicted with the final declared ladder.

## Strict Audit Adjustment

`audit_v4_conformance.py` now treats these as allowed safety references in strict all-phase mode:

- no-coordinate-root/RUR export guard strings and booleans,
- the `CertifiedExactTargetImage` enum declaration in `result/status.rs`,
- the replay guard that rejects exact-image success statuses in finite candidate-cover certificates.

This was required because strict mode previously applied the broad token scan without the phase-specific safety-marker allowlist, producing false positives against required guard code.
