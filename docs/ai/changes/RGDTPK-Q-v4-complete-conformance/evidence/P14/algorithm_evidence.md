# P14 Algorithm Evidence

Implemented/verified behavior:

- Projection message composition exposes a `MessageIdeal` API and composition consumes projection message relations through this API.
- Separator elimination creates a pseudo block from message relations only; it does not load original coordinate equations during separator elimination.
- The previous bounded `message_relations_have_target_eliminant` shortcut was removed. The check now uses an exact Groebner eliminant attempt and returns `CertificateDesignGap` if that proof cannot be constructed.
- Final support construction accepts target-only root relations and composed-ideal membership relations, producing normalized nonzero Q[T] support.
- Support verification proves either the target-only route or exact composed-ideal membership route; separator-elimination messages are replayed.
- Nonfinite certification records and replays target-free Groebner basis hashes, a target algebraic-independence hash, a proper ideal witness hash, and `dimension_lower_bound >= 1`. The rational witness is used only to prove the target-free ideal is proper.
- Nonfinite exact-image mode refuses guard/saturation/semantic/feasibility/unregistered-nonzero-witness cases with `CertificateDesignGap` rather than claiming a real nonfinite certificate from a small witness alone.
- Run certificate replay recomputes original input hash, canonicalization, compression hash, hypergraph hash, DAG hash, message hashes, support hash, squarefree hash, root hash, candidate hash, and invariant evidence.

Regression coverage:

- `p10_multiblock_composition_eliminates_separator_from_message_relations_only`
- `fcr_p9_support_verifier_replays_separator_elimination_certificate`
- `p12g_g6_multiseparator_composition_requires_child_message`
- `ccc_route_b_final_support_uses_composed_ideal_membership_when_route_a_unavailable`
- `composed_ideal_membership_route_verifies_support_without_target_only_root_relation`
- `composed_ideal_membership_route_rejects_multiplier_tamper`
- `p11_replay_fails_on_input_canonical_dag_plan_and_squarefree_tamper`
- `p11_replay_fails_on_support_root_and_candidate_tamper`
- `p12g_replay_rejects_message_using_relation_outside_original_block`
- `p14_public_candidate_cover_success_has_all_result_fields_and_trace`
- `p14_public_certified_nonfinite_is_finalized_without_panic`
- `fcr_final_nonfinite_public_certified_nonfinite_requires_positive_proof`
- `fcr_final_nonfinite_bounded_search_failure_is_not_nonfinite`
- `p13_exact_image_nonfinite_with_guard_or_saturation_returns_gap_without_real_proof`

Static audit:

- `audit_v4_conformance.py --phase P14 --strict` checks P14 files, required symbols, message-only separator markers, fixed-heuristic removal, final support/nonfinite markers, support verifier markers, and replay recomputation markers.

Scope note:

- Exact-image equality/classification remains outside the finite candidate-cover success claim. Replay explicitly rejects exact-image status as a finite candidate-cover replay success. P16 will close the exact-image scope guard separately.
