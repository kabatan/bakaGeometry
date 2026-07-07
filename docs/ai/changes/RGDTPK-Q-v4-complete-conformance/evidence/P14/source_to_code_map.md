# P14 Source To Code Map

Phase: P14 - projection message verification, composition, final support, nonfinite certificate, replay.

## BS-R110 - Projection Message Composition

- `geosolver-core/src/compose/message.rs`
  - `MessageIdeal`
  - `message_to_relations`
  - `merge_messages`
  - `hash_message_ideal`
- `geosolver-core/src/compose/compose.rs`
  - `compose_projection_messages`
  - validates message package hashes before composition
  - uses `message_to_relations`/`merge_messages`
  - invokes `eliminate_separators_from_message_relations` when no target-only root relation exists
- `geosolver-core/src/compose/separator_elimination.rs`
  - builds `message_only_system`
  - builds `message_only_block`
  - executes target-direct relation search on message relations only
  - `verify_separator_elimination_message` replays the message-only proof

## BS-R111 - Final Support Polynomial

- `geosolver-core/src/compose/final_support.rs`
  - `build_global_support_polynomial`
  - `support_from_target_only_relations`
  - `support_from_composed_ideal_membership`
  - computes squarefree primitive univariate support via repeated `squarefree_part_uni(univariate_mul(...))`, which is the candidate-cover LCM path for squarefree Q[T] factors.
  - `TargetEliminationZeroCertificate` now includes target-free Groebner basis hashes, a proper ideal witness hash, a target algebraic-independence hash, and a dimension lower bound for nonfinite certification.
  - `verify_zero_target_elimination_certificate` recomputes the target-free Groebner basis hashes and rejects target-bearing dimension evidence.

## BS-R112 - Support Verification

- `geosolver-core/src/verify/verify_support.rs`
  - `verify_global_support`
  - `GlobalSupportProofRoute::TargetOnlyRootRelationProduct`
  - `GlobalSupportProofRoute::ComposedIdealMembership`
  - `verify_composed_ideal_membership_support_certificate`
  - replay of separator elimination evidence through `verify_separator_elimination_message`

## BS-R113 - Run Certificate And Replay

- `geosolver-core/src/verify/run_certificate.rs`
  - `CoreRunCertificate`
  - `CoreInvariantFlags`
  - `FinalDagReplayEvidence`
  - `build_core_run_certificate`
- `geosolver-core/src/verify/replay.rs`
  - `replay_run_certificate`
  - recomputes input/canonical/compression/hypergraph/DAG
  - re-verifies messages against actual DAG blocks
  - recomposes messages and verifies support/root/candidate hashes

## P14 Remediation Notes

- Removed fixed small-threshold target-eliminant heuristic from `compose.rs`.
- If composed-ideal eliminant proof cannot be computed, composition now returns `CertificateDesignGap` instead of silently treating bounded heuristic failure as proof.
- Exact-image nonfinite guard/saturation cases now return `CertificateDesignGap` instead of claiming real nonfinite proof when semantic guards, saturations, feasibility obligations, or unregistered nonzero-witness relations are present.
- Nonfinite certification no longer treats the bounded rational witness as the proof by itself. The witness is only the proper-ideal evidence and is replay-bound together with target algebraic-independence/dimension evidence.
