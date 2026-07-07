# P8 Spec Reviewer Result

Status: PASS

Reviewer: spec_verifier

Accepted remediation:

- Guarded affine replay now passes `KernelContext` into replay and rejects nonconstant pivots unless
  `denominator_guard_hash` matches an authorized `ctx.system.guards` record with the same pivot
  factor hash.
- Regression `p8_guarded_affine_replay_rejects_tampered_denominator_guard_hash` verifies a valid
  guarded affine message, tampers the guard hash, recomputes the certificate binding/package hashes,
  and confirms `verify_projection_message` rejects it.

Fresh checks accepted:

- `cargo fmt --manifest-path geosolver-core\Cargo.toml --check`
- `python geosolver-core\scripts\audit_v4_conformance.py --phase P8 --strict`
- `cargo test --manifest-path geosolver-core\Cargo.toml --lib target_univariate -- --nocapture`
- `cargo test --manifest-path geosolver-core\Cargo.toml --lib linear_affine -- --nocapture`
- `cargo test --manifest-path geosolver-core\Cargo.toml --lib verify_message -- --nocapture`
- `cargo test --manifest-path geosolver-core\Cargo.toml --no-run`
