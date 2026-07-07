# P13 Algorithm Evidence

Implemented/verified behavior:

- Regular-chain decomposition records component semantics, source relation hashes, guards, main/free
  variables, regularity evidence for every initial, guard-condition evidence, component hashes, and
  projection hashes.
- Nonconstant regular-chain initials require an explicit matching guard; replay rejects tampered
  regularity evidence after wrapper hashes are recomputed.
- Regular-chain projection checks keep-variable output and replay recomputes DAG, per-component
  projections, and union/intersection combination.
- NormTrace detection uses algebraic tower form and validates the target expression against exported
  variables plus algebraic variables; it does not use geometry labels.
- NormTrace computes norm relations through exact resultant/norm steps and verifies them by
  recomputing the tower plan relation.
- SpecializationInterpolation uses deterministic sample points, declared support, declared inner
  target-only kernel plans, exact sample replay, and final exact elimination verification.

Regression coverage:

- `regular_chain_projection_preserves_guards_and_keep_variables`
- `nonconstant_initial_requires_and_records_guard_evidence`
- `duplicate_main_variables_create_component_dag`
- `p9_regular_chain_guard_binding_is_operational`
- `p13_regular_chain_replay_rejects_tampered_regularity_evidence_after_rehash`
- `p11_rejects_forged_regular_chain_payload_in_message_and_kernel_replay`
- `detects_explicit_tower_by_algebraic_form`
- `norm_relation_is_exactly_recomputed`
- `multistep_tower_norm_eliminates_each_algebraic_variable`
- `p11_rejects_forged_norm_trace_payload_source_and_tower_hash`
- `specialization_points_are_deterministic_and_specialization_is_exact`
- `bad_interpolation_sample_fails_final_verification`
- `bad_multiseparator_sample_fails_exact_interpolation_verification`
- `p12g_specialization_interpolation_inner_schedule_is_declared`

Static audit:

- `audit_v4_conformance.py --phase P13 --strict` checks P13 files, required symbols, regular-chain
  guard/projection markers, NormTrace algebraic-form and exact verification markers, interpolation
  sample/support/hash markers, and verifier replay markers.

P14 carry-forward:

- Exploratory `p13_exact_image_semantics` run found one exact-image/nonfinite semantics failure in
  `p13_exact_image_nonfinite_with_guard_or_saturation_returns_gap_without_real_proof`; this is not
  a P13 kernel requirement and is carried to P14 nonfinite/replay work.
