# P11 Quality Reviewer FAIL_FIXABLE

The P11 quality reviewer returned `FAIL_FIXABLE`.

Blocking finding:

- Universal replay did not authorize nested `inner_payload` sources. `payload_sources_are_authorized` checked only the top-level Universal `source_relations`, while `verify_payload_for_outputs` replayed the inner payload directly. A forged Universal certificate could therefore keep wrapper sources authorized while using arbitrary source polynomials inside the inner payload.

Remediation applied:

- `payload_source_hashes` now recursively appends `UniversalProjectionCertificate.inner_payload` source hashes to the top-level Universal source hashes before authorization.
- Added `p11_rejects_universal_inner_payload_with_unauthorized_sources`, which constructs a hash-consistent forged Universal wrapper whose inner `TargetOnlySupport` payload uses an unauthorized source relation.
- The test asserts rejection through `verify_projection_message`, public `UniversalTargetEliminationKernel::replay`, and `replay_run_certificate` with the run hashes recomputed.

Verification after remediation:

- `cargo test --manifest-path geosolver-core/Cargo.toml p11_ -- --nocapture`: 9 passed.
- `cargo test --manifest-path geosolver-core/Cargo.toml -- --nocapture`: 167 passed.
- `cargo fmt --manifest-path geosolver-core/Cargo.toml -- --check`: passed.
- `git diff --check`: passed with only CRLF conversion warnings.

Second P11 quality reviewer checkpoint also returned `FAIL_FIXABLE`.

Blocking finding:

- `projection_message_dependency_indices` could manufacture dependency cycles for valid replay data when a source polynomial hash was already authorized by canonical input but another projection message also output the same polynomial hash.

Remediation applied:

- Added `hash_projection_message_dag_binding_with_authorized_sources`, and run replay now computes canonical/compressed input relation hashes and input polynomial hashes as base-authorized sources.
- `projection_message_dependency_indices` now skips base-authorized source hashes before creating message-to-message dependency edges.
- Added `p11_replay_accepts_duplicate_hash_when_source_is_input_authorized`, proving duplicate message outputs do not create a false cycle when the source hash is already authorized by input.

Verification after remediation:

- `cargo test --manifest-path geosolver-core/Cargo.toml p11_ -- --nocapture`: 11 passed.
- `cargo test --manifest-path geosolver-core/Cargo.toml -- --nocapture`: 169 passed.
- `cargo fmt --manifest-path geosolver-core/Cargo.toml -- --check`: passed.
- `git diff --check`: passed with only CRLF conversion warnings.

Third P11 quality reviewer checkpoint also returned `FAIL_FIXABLE`.

Blocking findings:

- SparseResultant replay verified resultant certificates internally but did not bind certificate relation hashes to message outputs or require certificate inputs to come from the payload's authorized/generated relation closure.
- NormTrace replay did not include tower source hashes in payload authorization and did not recompute tower/step hashes before accepting a tower norm relation.

Remediation applied:

- Added `verify_sparse_resultant_payload`, which verifies each resultant certificate, checks each certificate input against the available generated closure, adds raw and primitive resultant hashes to that closure, and requires message outputs to be generated resultants.
- `payload_source_hashes` now includes `NormTraceProjectionCertificate.tower.source_relation_hashes`.
- `verify_norm_tower_plan_relation` now calls `verify_tower_plan_hashes`, which requires the tower source hash set to contain the target expression hash and every minimal polynomial hash, then recomputes every tower step hash and the tower hash before norm replay.
- Added `p11_rejects_forged_sparse_resultant_payload_in_message_and_kernel_replay`.
- Added `p11_rejects_forged_norm_trace_payload_source_and_tower_hash`.

Verification after remediation:

- `cargo test --manifest-path geosolver-core/Cargo.toml p11_ -- --nocapture`: 13 passed.
- `cargo test --manifest-path geosolver-core/Cargo.toml -- --nocapture`: 171 passed.
- `cargo fmt --manifest-path geosolver-core/Cargo.toml -- --check`: passed.
- `git diff --check`: passed with only CRLF conversion warnings.
