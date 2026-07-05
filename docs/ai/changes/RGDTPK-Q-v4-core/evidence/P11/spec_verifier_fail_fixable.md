# P11 Initial Spec Verifier Result

The first P11 spec-verifier checkpoint returned `FAIL_FIXABLE`.

Blockers reported:

- `verify_projection_message` was still largely route/hash/shape oriented and did not replay variant-specific proof evidence.
- `replay_run_certificate` did not call `verify_projection_message`, so replay could accept synthetic certificates when hashes were internally consistent.
- Run certificate support and DAG binding were incomplete; stale or arbitrary global-support/DAG hashes could be accepted if unrelated fields were recomputed.
- Invariant flags were set to true without an evidence hash tying them to the replayed artifacts.

Remediation applied:

- Added route-specific `KernelCertificatePayload` variants and required all production P7-P10 kernels to emit payload-bearing certificates.
- Added exact payload replay for target support, dense membership, guarded affine, sparse resultant, target action, regular chain, norm trace, specialization/interpolation, and Universal wrapped proofs.
- Made replay verify every projection message through `verify_projection_message`.
- Added projection-message DAG binding, global support certificate hash binding, and invariant evidence hashing.
- Added negative tests for hash-consistent synthetic projection certificates, global support certificate tamper, support certificate hash tamper, DAG tamper, and invariant evidence tamper.

Second P11 spec-verifier checkpoint also returned `FAIL_FIXABLE`.

Additional blockers reported:

- TargetAction replay did not reconstruct quotient/action matrix evidence and could accept a hash-consistent forged characteristic relation.
- RegularChain replay only recombined supplied projection generators and did not recompute DAG/projection identity from authorized source relations.
- Invariant flags were better hash-bound but still derived too weakly while proof variants could be forged.

Additional remediation applied:

- `TargetActionProjectionCertificate` now carries `target` and `ProductionQuotientHandleInput`; replay rebuilds the production quotient handle, reruns Krylov recurrence recovery, verifies characteristic support coverage and annihilator, and compares output relation.
- `RegularChainProjectionCertificate` now carries source relations, variable order, exported variables, and guards; replay rebuilds the regular-chain DAG and projections from those inputs and compares combined outputs.
- Projection-message verification excludes the message itself when authorizing certificate and payload source hashes from child messages.
- Run-certificate invariant flags are derived from verifiable payload presence, no-coordinate export evidence, and plan binding, and replay recomputes those flags after message/support verification.
- Added forged TargetAction and forged RegularChain P11 negative tests.

Third P11 spec-verifier checkpoint also returned `FAIL_FIXABLE`.

Additional blocker reported:

- Public `TargetProjectionKernel::replay` implementations for TargetAction and RegularChain, and structurally similar replay paths in other kernels, still used shape/hash checks instead of delegating to exact payload replay. A forged payload rejected by `verify_projection_message` could therefore still be accepted by a direct `kernel.replay` caller.

Additional remediation applied:

- Added `exact_replay_result` in `geosolver-core/src/kernels/traits.rs`, which accepts only when the message kind matches and `verify_projection_message(message, ctx)` succeeds.
- Updated all nine public kernel `replay` implementations to delegate to `exact_replay_result`.
- Removed the old structural `basic_replay_result` helper.
- Renamed and extended forged TargetAction and forged RegularChain tests so they assert rejection through both `verify_projection_message` and public `kernel.replay`.

Fourth P11 spec-verifier checkpoint also returned `FAIL_FIXABLE`.

Additional blocker reported:

- Run replay built each message's replay `KernelContext` with all projection messages as `child_messages`, allowing mutually supporting forged projection messages to authorize each other's source hashes without a declared acyclic DAG edge.

Additional remediation applied:

- Added `projection_message_dependency_indices`, deriving message-to-message dependencies from certificate source hashes plus recursive payload source hashes.
- `hash_projection_message_dag_binding` now includes derived dependency edges in the run certificate DAG binding.
- `replay_run_certificate` now rejects cyclic dependency graphs and calls `verify_projection_message` with only each message's derived child dependencies, not the full projection-message list.
- Added `p11_replay_rejects_mutual_projection_message_source_cycle`, proving two individually replayable hash-consistent forged messages are rejected when their source dependencies form a cycle.

Fifth P11 spec-verifier checkpoint also returned `FAIL_FIXABLE`.

Additional blocker reported:

- SparseResultant and NormTrace replay still had hash-only algebraic gaps. SparseResultant did not prove the advertised output was generated by the carried resultant certificates, and NormTrace accepted a self-consistent tower hash without binding the tower body to authorized source/minimal-polynomial relations.

Additional remediation applied:

- `verify_sparse_resultant_payload` now starts from authorized payload source relations, verifies each carried resultant certificate, recomputes each resultant relation, and requires every advertised SparseResultant output to be generated by that certificate closure.
- `verify_tower_plan_hashes` now recomputes each tower step hash and requires `source_relation_hashes` to include both the target-minus-expression polynomial and every minimal-polynomial relation used by the tower.
- `payload_source_hashes` now includes `NormTraceTower.source_relation_hashes`, so run replay dependency derivation and projection-message source authorization bind the tower body, not just the output relation.
- Added P11 negative tests covering a forged SparseResultant output and a self-consistent-but-unauthorized NormTrace tower body through both `verify_projection_message` and public kernel replay.
