# P8 Algorithm Evidence

Status: P8 implementation evidence before spec reviewer.

Implemented/verified behavior:

- TargetUnivariate considers child messages and local relations. Regression:
  `p7_target_univariate_uses_child_message_target_relation`.
- TargetUnivariate support uses primitive/squarefree-compatible construction. Regression:
  `p7_target_univariate_support_is_primitive_lcm_squarefree`.
- TargetUnivariate rejects separator-only planner admission and tampered authorization/source/child
  message bindings.
- LinearAffine succeeds with safe constant pivots and rejects unsafe nonconstant pivots.
- LinearAffine rejects stale/incomplete plans, authorization tampering, and source hash tampering.
- Guarded affine replay rejects a certificate whose nonconstant pivot
  `denominator_guard_hash` is tampered after recomputing the certificate binding hash. Regression:
  `p8_guarded_affine_replay_rejects_tampered_denominator_guard_hash`.
- Preprocess/LinearAffine guard tests cover guarded rational affine pivots and preserved guard
  records.

Static audit guard added:

- `audit_v4_conformance.py --phase P8 --strict` checks TargetUnivariate child-message scanning,
  primitive LCM/squarefree support markers, PrincipalSupport, source membership binding, LinearAffine
  safe pivot/guard markers, guarded affine certificate route, and message verifier export/guard
  authorization rejection markers.

Claim boundary:

- This evidence supports only Phase 8 TargetUnivariate/LinearAffine conformance.
- It does not claim finite candidate-cover completion or full source-faithful completion.
