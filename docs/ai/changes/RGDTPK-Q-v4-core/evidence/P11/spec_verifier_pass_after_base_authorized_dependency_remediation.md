# P11 Spec Verifier PASS After Base-Authorized Dependency Remediation

Result: PASS.

The read-only spec verifier found no blocking issues for the requested P11 / MECH-010 / MECH-016 scope and did not mark any R-ID as VERIFIED.

Key anchors inspected:

- `replay_run_certificate` computes base-authorized input relation and polynomial hashes, binds the DAG with `hash_projection_message_dag_binding_with_authorized_sources`, derives per-message dependencies, rejects cycles, and replays each message with only derived children.
- `verify_projection_message` rejects synthetic and binding-only certificates, recursively authorizes Universal inner payload sources, replays TargetAction quotient/Krylov/annihilator evidence, and rebuilds RegularChain DAG/projection evidence.
- Public replay routes through `exact_replay_result`, which gates acceptance on `verify_projection_message`, and all nine kernel `replay` implementations delegate to it.
- Focused P11 tests include duplicate input-authorized regression, Universal unauthorized inner payload, forged TargetAction/RegularChain public replay, synthetic rejection, deletion/tamper, and dependency-cycle cases.

Runtime evidence exists in `command_outputs.txt`: focused P11 tests passed and the full suite passed.
