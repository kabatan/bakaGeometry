# P0 Algorithm Evidence

Status: harness evidence only.

P0 did not change solver behavior. It added a static conformance harness that checks:

- production Rust file layout;
- an initial set of source-named production APIs;
- `todo!`, `unimplemented!`, and TODO panic markers;
- `Unsupported` status/diagnostic markers;
- `kernel_not_ready_error` in production route;
- possible geometry/fixture/expected-answer dispatch tokens;
- possible coordinate/RUR/global-lex production paths;
- exact-image success statuses in the scoped finite candidate-cover repair;
- Descartes/Vincent isolation delegating to Sturm.

The first strict run intentionally fails with 12 findings. This is acceptable for P0 because P0
closes the audit harness, not the implementation.

No R-ID is verified by this evidence.
