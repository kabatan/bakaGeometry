# P11 Spec Verifier PASS After NormTrace Body Binding

Result: PASS.

The read-only spec verifier found no blocking issues for the requested P11 / MECH-010 / MECH-016 scope and did not mark any R-ID as VERIFIED.

Key anchors inspected:

- `verify_projection_message` performs package/block/export/source authorization and variant-specific exact replay.
- `BindingOnly` and synthetic payloads are rejected and do not derive enforced invariants.
- SparseResultant replay binds generated closure and outputs.
- NormTrace replay requires tower source hashes to cover `target_minus_expression.hash` and every `minimal_polynomial.hash`, then recomputes step/tower hashes.
- Run replay derives acyclic dependencies while treating base input hashes as authorized sources.
- All public kernel replay paths delegate through `exact_replay_result`, which calls `verify_projection_message`.
- Focused negative tests include SparseResultant, NormTrace self-consistent tower-body rehash, Universal inner unauthorized source, dependency cycle, duplicate input-authorized source, and public replay rejection.

Runtime checks reported by the verifier:

- `cargo test --manifest-path geosolver-core/Cargo.toml p11_ -- --nocapture`: 13 passed.
- `cargo test --manifest-path geosolver-core/Cargo.toml -- --nocapture`: 171 passed plus doc-tests passed.
