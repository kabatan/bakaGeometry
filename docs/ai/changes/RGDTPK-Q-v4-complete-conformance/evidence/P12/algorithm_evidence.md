# P12 Algorithm Evidence

Implemented/verified behavior:

- Universal planning builds a fixed five-stage strategy sequence:
  `EliminationGroebnerLocal`, `F4EliminationLocal`, `TargetRelationSearchEscalated`,
  `ResultantIfSquareOrOverdetermined`, and `SpecializeProjectInterpolateVerify`.
- Plan validation rejects any reordered or non-fixed Universal strategy sequence.
- Universal production execution has no internal RegularChain, NormTrace, or TargetActionKrylov
  stage.
- Each stage carries resource bounds, algebraic work estimate, route budget, and reproducible
  `stage_hash`; certificates record attempted strategies, strategy records, skipped hashes, chosen
  strategy, failed strategy hashes, and executed failed strategy hashes.
- Local Groebner/F4 generators are filtered to nonzero primitive exported-variable generators and
  validated by exact membership certificates.
- Wrapped sub-kernel stages preserve the inner proof payload and map it back to the declared
  Universal strategy during replay.
- Empty local generator exhaustion returns `AlgorithmicHardCase` under
  `LocalNonfinitePolicy::NoLocalCertifiedNonFinite`; no local nonfinite claim is made.

Regression coverage:

- `p8d_universal_one_large_block_exports_target_relation`
- `p8d_universal_requires_fixed_strategy_sequence_and_resource_caps`
- `p8d_universal_exhaustion_uses_only_allowed_failure_statuses`
- `p8d_static_forbidden_fallback_apis_absent`
- `fcr_universal_one_large_block_multivariate_projection`
- `fcr_universal_keep_only_exports_no_coordinate_roots`
- `fcr_elimination_membership_certificates_replay`
- `fcr_production_f4_is_reachable`
- `dispatcher_executes_declared_f4_strategy_without_groebner_fallback`
- `f4_elimination_exports_keep_only_generators_with_certificates`
- `fcr_p11_red_team_05_one_large_block_universal_admission`

Static audit:

- `audit_v4_conformance.py --phase P12 --strict` checks P12 files, required symbols, exact
  `UniversalStrategy` enum variants, fixed strategy sequence markers, forbidden internal strategy
  absence, exact generator verification markers, and Universal replay markers.

