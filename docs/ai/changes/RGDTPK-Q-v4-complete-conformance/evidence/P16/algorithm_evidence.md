# P16 Algorithm Evidence

Implemented/verified behavior:

- `exact_image_mode = true` no longer invokes exact-image classifier code from the solver/orchestrator path.
- Exact-image requests run the finite candidate-cover path through support verification, squarefree support, exact root isolation, and candidate decode.
- Exact-image requests return `CertificateDesignGap`, not `CertifiedExactTargetImage` or `CertifiedEmptyRealTargetImage`.
- Exact-image requests preserve unfiltered decoded candidates and root isolation records.
- `ExactImageOutOfScope` diagnostic binds input hash, support hash, squarefree support hash, candidate hashes, and candidate count.
- Exact-image requests whose candidate-cover path certifies nonfinite target image are also guarded as `CertificateDesignGap`, with `ExactImageOutOfScope` and `nonfinite_certificate_hash`.
- Run certificate binds `exact_image_mode` through solver options hash and keeps `exact_image_certificate_hash = None`.
- Replay accepts the scoped guard result only as an in-scope non-exact result and rejects exact-image certificate hashes or exact-image success statuses.

Regression coverage:

- `p16_exact_image_request_returns_scope_guard_without_filtering_slack_root`
- `p16_exact_image_empty_case_keeps_unfiltered_candidate_cover`
- `p16_branch_choice_semantics_do_not_filter_candidate_cover`
- `p16_public_exact_image_request_returns_scope_guard_with_candidates`
- `p16_public_exact_image_empty_semantics_do_not_filter_candidates`
- `p16_exact_image_scope_guard_suite`
- `p13_exact_image_nonfinite_requires_real_nonfinite_certificate`
- `p13_candidate_cover_mode_does_not_claim_exact_image_for_semantic_problem`
- replay subset tests rejecting exact-image certificate hashes/status paths

Static audit:

- `audit_v4_conformance.py --phase P16 --strict` checks P16 files, required scope guard symbols, absence of solver/orchestrator exact-image classifier calls, absence of reachable exact-image success statuses in orchestrator, certificate option binding, and replay rejection markers.
