# FCR-P9 Guardian Boundary Review Request

Review the FCR-P9 implementation for source/spec fidelity and phase closure.

Scope:
- Phase: FCR-P9, "Full support composition, root isolation, and result construction".
- Claim ceiling requested: `PARTIAL_MECHANISM_READY:MECH-009` and `PARTIAL_MECHANISM_READY:MECH-011` evidence only. Do not approve P13, final source fidelity, exact-image readiness, acceptance completion, full production readiness, or any R-ID VERIFIED status.
- Changed files to inspect:
  - `geosolver-core/src/compose/compose.rs`
  - `geosolver-core/src/compose/final_support.rs`
  - `geosolver-core/src/compose/mod.rs`
  - `geosolver-core/src/compose/separator_elimination.rs`
  - `geosolver-core/src/kernels/mod.rs`
  - `geosolver-core/src/solver/orchestrator.rs`
  - `geosolver-core/src/verify/replay.rs`
  - `geosolver-core/src/verify/verify_support.rs`
  - `geosolver-core/tests/p3_public_pipeline_integration.rs`

Controlling source sections:
- `docs/ai/changes/RGDTPK-Q-v4-core/FULL_CORE_REPAIR_PLAN.md` FCR-P9:
  - `compose_projection_messages` must use message ideals and separator elimination without reconstructing the full coordinate system.
  - `build_global_support_polynomial` must build nonzero `S(T)` from verified target-only root relations or verified membership in the composed message ideal.
  - `verify_global_support` must replay exact proof.
  - `squarefree_support`, root isolation, and decode must run in public success path.
  - Empty real root set remains `CertifiedCandidateCover` with empty candidates.
  - Cost trace must include parameters required by the v4 spec.
  - Fail if support is hand-built for tests; support verification only multiplies target-only helper relations while other support routes are unverified; roots/candidates absent on success; or no-real-root support becomes hard-case.
- `docs/ai/changes/RGDTPK-Q-v4-core/FULL_CORE_REPAIR_REVIEWER_PROMPTS.md` FCR-P9 reviewer prompt.
- `docs/ai/changes/RGDTPK-Q-v4-core/FULL_CORE_REPAIR_BASE_SPEC.md` FCR-011 candidate cover and root/candidate outputs.
- `docs/ai/changes/RGDTPK-Q-v4-core/BASE_SPEC.md` MECH-009 composition/final support and MECH-011 exact root isolation/decode.

Evidence to inspect:
- `docs/ai/changes/RGDTPK-Q-v4-core/evidence/FCR-P9/commands.txt`
- `docs/ai/changes/RGDTPK-Q-v4-core/evidence/FCR-P9/command_outputs.txt`
- `docs/ai/changes/RGDTPK-Q-v4-core/evidence/FCR-P9/support_roots_static_scan.txt`

Fresh command evidence after final code change:
- `cargo fmt`: pass
- `cargo check`: pass
- `cargo test fcr_p9_support_verifier_replays_separator_elimination_certificate`: pass
- `cargo test --test p3_public_pipeline_integration`: pass
- `cargo test`: pass
- `git diff --check`: pass

Specific review questions:
1. Does production composition now use message relations and separator elimination without reconstructing the full coordinate system?
2. Does global support verification replay the separator membership proof instead of accepting a hand-built support route?
3. Does production replay recompute support using the actual DAG composition path and avoid the removed synthetic all-relations support helper?
4. Does the public candidate-cover path run squarefree support, exact root isolation, and decode before result construction?
5. Does a no-real-root support return `CertifiedCandidateCover` with empty roots/candidates, certificate, and accepted replay?
6. Did promoting `target_relation_search` to production compilation accidentally add it to the production completion registry or planner list?

Please return PASS only if there are no blocking findings or required fixes for FCR-P9. Include inspected files, evidence considered, residual risks, and claim ceiling.
