# P11 Spec Verifier PASS

Result: PASS.

The read-only spec verifier found no blocking issues by R-ID and did not mark any R-ID as VERIFIED.

Key anchors inspected:

- Public replay now uses exact verifier: `geosolver-core/src/kernels/traits.rs` gates `exact_replay_result` on `verify_projection_message`, and all nine kernel `replay` implementations delegate to it.
- Synthetic and `BindingOnly` certificates are rejected by projection-message verification and do not derive production replay invariants.
- TargetAction replay rebuilds the production quotient handle, Krylov sequence, coverage, annihilator, and output relation in `verify_target_action_payload`.
- RegularChain replay rebuilds DAG, projections, and combined outputs from source, variable, and guard evidence in `verify_regular_chain_payload`.
- Production TargetAction and RegularChain messages carry replayable payloads.
- Forged TargetAction and RegularChain tests assert both `verify_projection_message` rejection and public `kernel.replay` rejection.

The verifier relied on the fresh local evidence for focused P11 tests and full-suite pass in `command_outputs.txt`.
