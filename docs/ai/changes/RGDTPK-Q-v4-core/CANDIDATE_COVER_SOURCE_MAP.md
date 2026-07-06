# Candidate-Cover Source Map

Status: active source map for `v4_candidate_cover_completion_pack_v1`.

This file uses the required CCC-P1 classification fields. It supports only the candidate-cover
layer claim and must not be cited as full supplied-v4 source fidelity.

## Appendix A Section Classification

```yaml
- item: SRC-ALG-v4 sections 0-1 objective/scope
  candidate_cover_required: true
  exact_image_later: false
  optimizer_optional: false
  implemented_status: implemented
  current_path: docs/ai/changes/RGDTPK-Q-v4-core/CANDIDATE_COVER_CLOSURE.md
  repair_action: keep
  notes: Candidate-cover closure states containment, not exact target-image equality.

- item: SRC-ALG-v4 section 2 problem/output semantics
  candidate_cover_required: true
  exact_image_later: true
  optimizer_optional: false
  implemented_status: implemented
  current_path: geosolver-core/src/result/status.rs; geosolver-core/src/result/output.rs
  repair_action: keep
  notes: CertifiedCandidateCover is finite containment; exact-image statuses remain separate.

- item: SRC-ALG-v4 section 3 forbidden paths and source hierarchy
  candidate_cover_required: true
  exact_image_later: false
  optimizer_optional: false
  implemented_status: implemented
  current_path: geosolver-core/src/verify/run_certificate.rs; CANDIDATE_COVER_COST_TRACE_SUMMARY.md
  repair_action: keep
  notes: Runtime flags are not overclaimed; static scans are separately recorded.

- item: SRC-ALG-v4 section 4 top-level pipeline
  candidate_cover_required: true
  exact_image_later: false
  optimizer_optional: false
  implemented_status: implemented
  current_path: geosolver-core/src/api.rs::solve_target; geosolver-core/src/solver/orchestrator.rs::solve_with_context
  repair_action: keep
  notes: Public pipeline reaches support, roots, decode, certificate, replay, and cost trace.

- item: SRC-ALG-v4 section 5 data model
  candidate_cover_required: true
  exact_image_later: false
  optimizer_optional: false
  implemented_status: implemented
  current_path: geosolver-core/src/types; geosolver-core/src/problem; geosolver-core/src/result
  repair_action: keep
  notes: Exact rational, polynomial, interval, hash, problem, result, and diagnostic data are required.

- item: SRC-ALG-v4 section 6 crate/module layout
  candidate_cover_required: true
  exact_image_later: false
  optimizer_optional: false
  implemented_status: implemented
  current_path: geosolver-core/src/lib.rs
  repair_action: keep
  notes: Candidate-cover modules are present; exact-image modules are retained, not deleted.

- item: SRC-ALG-v4 sections 7-8 public API and algebra types
  candidate_cover_required: true
  exact_image_later: false
  optimizer_optional: false
  implemented_status: implemented
  current_path: geosolver-core/src/api.rs; geosolver-core/src/types
  repair_action: keep
  notes: Public API is algebraic Q-polynomial target solving, not geometry dispatch.

- item: SRC-ALG-v4 section 9 validation/canonicalization
  candidate_cover_required: true
  exact_image_later: false
  optimizer_optional: false
  implemented_status: implemented
  current_path: geosolver-core/src/problem/validate.rs; geosolver-core/src/problem/canonicalize.rs
  repair_action: keep
  notes: Validation/canonicalization preserve semantic provenance without controlling dispatch.

- item: SRC-ALG-v4 sections 10-14 primitives, graph, planner, registry
  candidate_cover_required: true
  exact_image_later: false
  optimizer_optional: false
  implemented_status: implemented
  current_path: geosolver-core/src/algebra; geosolver-core/src/graph; geosolver-core/src/planner; geosolver-core/src/kernels/mod.rs
  repair_action: keep
  notes: Generic planner and kernel registry remain candidate-cover required.

- item: SRC-ALG-v4 sections 15-16 TargetUnivariate and LinearAffine
  candidate_cover_required: true
  exact_image_later: false
  optimizer_optional: true
  implemented_status: implemented
  current_path: geosolver-core/src/kernels/target_univariate.rs; geosolver-core/src/kernels/linear_affine.rs
  repair_action: keep
  notes: These are useful projection routes but do not define solver scope.

- item: SRC-ALG-v4 section 17 TargetRelationSearch
  candidate_cover_required: true
  exact_image_later: false
  optimizer_optional: false
  implemented_status: implemented
  current_path: geosolver-core/src/kernels/target_relation_search.rs; geosolver-core/src/planner/relation_schedule.rs
  repair_action: keep
  notes: Generic deterministic exact-Q support route; not optional for candidate-cover readiness.

- item: SRC-ALG-v4 sections 18,21-23 specialized optimizers
  candidate_cover_required: true
  exact_image_later: false
  optimizer_optional: true
  implemented_status: implemented
  current_path: geosolver-core/src/kernels/sparse_resultant.rs; regular_chain_projection.rs; norm_trace_projection.rs; specialization_interpolation.rs
  repair_action: keep
  notes: Optimizers must verify exactly and cannot replace generic fallback routes.

- item: SRC-ALG-v4 section 19 TargetActionKrylov
  candidate_cover_required: true
  exact_image_later: false
  optimizer_optional: true
  implemented_status: implemented
  current_path: geosolver-core/src/kernels/action_krylov.rs; geosolver-core/src/algebra/quotient.rs; geosolver-core/src/algebra/krylov.rs
  repair_action: keep
  notes: Production handle forbids coordinate roots/RUR and requires verified characteristic support coverage.

- item: SRC-ALG-v4 section 20 UniversalTargetElimination
  candidate_cover_required: true
  exact_image_later: false
  optimizer_optional: false
  implemented_status: implemented
  current_path: geosolver-core/src/kernels/universal_elimination.rs
  repair_action: keep
  notes: Final generic local route; exhaustion is hard/resource/certificate status, not nonfinite.

- item: SRC-ALG-v4 section 24 composition/final support
  candidate_cover_required: true
  exact_image_later: false
  optimizer_optional: false
  implemented_status: implemented
  current_path: geosolver-core/src/compose/compose.rs; geosolver-core/src/compose/final_support.rs
  repair_action: implement
  notes: Route A target-only support and Route B composed-ideal support construction are production support paths.

- item: SRC-ALG-v4 section 25 certificates/replay
  candidate_cover_required: true
  exact_image_later: false
  optimizer_optional: false
  implemented_status: implemented
  current_path: geosolver-core/src/verify/verify_message.rs; verify_support.rs; replay.rs; run_certificate.rs
  repair_action: implement
  notes: Support, DAG/messages, roots, decoded candidates, and final invariant evidence are replay-bound.

- item: SRC-ALG-v4 section 26 root isolation/decode
  candidate_cover_required: true
  exact_image_later: false
  optimizer_optional: false
  implemented_status: implemented
  current_path: geosolver-core/src/roots/isolate.rs; roots/squarefree.rs; roots/decode.rs
  repair_action: keep
  notes: Exact squarefree support, real root isolation, and decoded candidates are candidate-cover required.

- item: SRC-ALG-v4 section 27 exact-image fiber/guard/slack/branch API
  candidate_cover_required: false
  exact_image_later: true
  optimizer_optional: false
  implemented_status: implemented
  current_path: geosolver-core/src/fiber/exact_image.rs; hermite.rs; thom.rs; slack_semantics.rs
  repair_action: defer_exact_image
  notes: Retained as later API/provenance; candidate-cover must not depend on exact-image filtering.

- item: SRC-ALG-v4 sections 28-30 result/orchestrator/cost
  candidate_cover_required: true
  exact_image_later: false
  optimizer_optional: false
  implemented_status: implemented
  current_path: geosolver-core/src/result; geosolver-core/src/solver
  repair_action: keep
  notes: Candidate-cover success and failure include status, diagnostics, certificates, and cost trace.

- item: SRC-ALG-v4 section 31 nonfinite target image
  candidate_cover_required: true
  exact_image_later: true
  optimizer_optional: false
  implemented_status: implemented
  current_path: geosolver-core/src/compose/final_support.rs; geosolver-core/src/verify/replay.rs
  repair_action: keep
  notes: Positive nonfinite proof safety is required; general nonfinite completeness is not a candidate-cover blocker.

- item: SRC-ALG-v4 section 32 geometry-derived footprint
  candidate_cover_required: true
  exact_image_later: false
  optimizer_optional: true
  implemented_status: implemented
  current_path: geosolver-core/src/preprocess; geosolver-core/src/kernels
  repair_action: keep
  notes: Footprints may guide algebraic cost, never geometry-name dispatch.

- item: SRC-ALG-v4 section 33 completion criteria except condition 13
  candidate_cover_required: true
  exact_image_later: false
  optimizer_optional: false
  implemented_status: implemented
  current_path: CANDIDATE_COVER_ACCEPTANCE_RESULTS.md; CANDIDATE_COVER_CLOSURE.md
  repair_action: keep
  notes: Applied only to candidate-cover layer and Suite A/failure-safety evidence.

- item: SRC-ALG-v4 section 33 condition 13 exact-image/full acceptance
  candidate_cover_required: false
  exact_image_later: true
  optimizer_optional: false
  implemented_status: not_applicable
  current_path: CANDIDATE_COVER_CLOSURE.md
  repair_action: defer_exact_image
  notes: Exact-image/full acceptance remains forbidden without a separate exact-image closure.
```

## File And Function Classification

```yaml
- item: api.rs::solve_target
  candidate_cover_required: true
  exact_image_later: false
  optimizer_optional: false
  implemented_status: implemented
  current_path: geosolver-core/src/api.rs::solve_target
  repair_action: keep
  notes: Public entry for all support-producing acceptance and red-team cases.

- item: solver/orchestrator.rs::solve_with_context
  candidate_cover_required: true
  exact_image_later: false
  optimizer_optional: false
  implemented_status: implemented
  current_path: geosolver-core/src/solver/orchestrator.rs::solve_with_context
  repair_action: keep
  notes: Candidate-cover mode does not run exact-image filtering and emits spurious-root diagnostics.

- item: compose/compose.rs::compose_projection_messages
  candidate_cover_required: true
  exact_image_later: false
  optimizer_optional: false
  implemented_status: implemented
  current_path: geosolver-core/src/compose/compose.rs::compose_projection_messages
  repair_action: generalize
  notes: Preserves Route A separator composition and admits Route B only when message ideal has a target eliminant.

- item: compose/final_support.rs::build_final_support_or_nonfinite_with_system
  candidate_cover_required: true
  exact_image_later: false
  optimizer_optional: false
  implemented_status: implemented
  current_path: geosolver-core/src/compose/final_support.rs::build_final_support_or_nonfinite_with_system
  repair_action: implement
  notes: Tries Route A, then Route B composed-ideal support, then positive nonfinite/hardcase gate.

- item: verify/verify_support.rs::verify_global_support
  candidate_cover_required: true
  exact_image_later: false
  optimizer_optional: false
  implemented_status: implemented
  current_path: geosolver-core/src/verify/verify_support.rs::verify_global_support
  repair_action: implement
  notes: Verifies Route A or exact-Q Route B membership certificate.

- item: verify/verify_support.rs::ComposedIdealMembershipSupportCertificate
  candidate_cover_required: true
  exact_image_later: false
  optimizer_optional: false
  implemented_status: implemented
  current_path: geosolver-core/src/verify/verify_support.rs::ComposedIdealMembershipSupportCertificate
  repair_action: implement
  notes: Machine-readable certificate with relation hashes, multipliers, exact identity hash, and certificate hash.

- item: verify/replay.rs::replay_run_certificate
  candidate_cover_required: true
  exact_image_later: false
  optimizer_optional: false
  implemented_status: implemented
  current_path: geosolver-core/src/verify/replay.rs::replay_run_certificate
  repair_action: keep
  notes: Recomputes DAG/messages/support certificate/root/candidate evidence for successful candidate-cover runs.

- item: roots/isolate.rs::isolate_real_roots
  candidate_cover_required: true
  exact_image_later: false
  optimizer_optional: false
  implemented_status: implemented
  current_path: geosolver-core/src/roots/isolate.rs::isolate_real_roots
  repair_action: keep
  notes: Candidate-cover requires exact real roots of squarefree support.

- item: roots/decode.rs::decode_candidates
  candidate_cover_required: true
  exact_image_later: false
  optimizer_optional: false
  implemented_status: implemented
  current_path: geosolver-core/src/roots/decode.rs::decode_candidates
  repair_action: keep
  notes: Binds target, support hash, root index, interval, and candidate hash.

- item: fiber/exact_image.rs::classify_real_target_image
  candidate_cover_required: false
  exact_image_later: true
  optimizer_optional: false
  implemented_status: implemented
  current_path: geosolver-core/src/fiber/exact_image.rs::classify_real_target_image
  repair_action: defer_exact_image
  notes: Retained for exact-image mode; candidate-cover mode must not require this classifier.

- item: algebra/f4.rs::*GroebnerBackedBatch*
  candidate_cover_required: false
  exact_image_later: false
  optimizer_optional: true
  implemented_status: not_applicable
  current_path: geosolver-core/src/algebra/f4.rs
  repair_action: quarantine
  notes: Non-production/test-only helper; no candidate-cover readiness claim relies on production F4.
```

## Source-Fidelity Boundary

`SOURCE_FAITHFUL_TO_V4_CANDIDATE_COVER_LAYER` may be claimed only for the candidate-cover layer
classified above. This file does not claim `SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC`,
`EXACT_IMAGE_CORE_READY`, `RGDTPK_Q_V4_ACCEPTANCE_COMPLETE`, benchmark superiority, or universal
finite-system completeness.
