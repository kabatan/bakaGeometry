# ACR-P9 Review Prompt

Scope: ACR-P9 generic large-footprint support-producing stress suite only.

Do not review or close ACR-P10/final readiness. Do not make exact-image, source-fidelity, full-acceptance, or `CANDIDATE_COVER_CORE_READY` claims from this review.

Review the current working tree after the S6 executed-failure repair. Earlier ACR-P9 review attempts are historical only.

Required source and prompt files:

- `docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_COMPLETION_REPAIR_BASE_SPEC.md`
- `docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_COMPLETION_REPAIR_PLAN.md`, ACR-P9 section
- `docs/ai/changes/RGDTPK-Q-v4-core/ALG_COST_COMPLETION_REVIEWER_PROMPTS.md`, ACR-P9 prompt and meta-protocol
- `docs/ai/changes/RGDTPK-Q-v4-core/evidence/ACR-P9/MECH_EVIDENCE.md`
- `docs/ai/changes/RGDTPK-Q-v4-core/evidence/ACR-P9/stress_matrix.yaml`
- `geosolver-core/tests/acr_p9_large_footprint_stress.rs`

Implementation areas:

- `geosolver-core/src/verify/certificates.rs`
- `geosolver-core/src/kernels/universal_elimination.rs`
- `geosolver-core/src/verify/verify_message.rs`
- `geosolver-core/src/verify/replay.rs`
- `geosolver-core/src/solver/pipeline.rs`
- `geosolver-core/src/algebra/resultant.rs`
- `geosolver-core/src/problem/canonicalize.rs`
- `geosolver-core/src/planner/admission.rs`
- `geosolver-core/src/planner/cost_model.rs`
- `geosolver-core/src/planner/kernel_plan.rs`
- `geosolver-core/src/planner/ladder.rs`
- `geosolver-core/src/planner/planner.rs`
- `geosolver-core/src/kernels/sparse_resultant.rs`

Fresh evidence after repair:

- `cargo fmt --manifest-path geosolver-core\Cargo.toml --check`: PASS.
- Forbidden diagnostic/name/hash/expected-answer `rg` scan over `geosolver-core/src` and relevant tests: no matches.
- `cargo test --manifest-path geosolver-core\Cargo.toml --test acr_p9_large_footprint_stress acr_p9_s6_universal_after_internal_failures -- --test-threads=1 --nocapture`: PASS, 1 test / 3 variants, 46.99s.
- `cargo test --manifest-path geosolver-core\Cargo.toml --test acr_p9_large_footprint_stress -- --test-threads=1 --nocapture`: PASS, 8 tests / 24 variants, 69.13s.
- `cargo test --manifest-path geosolver-core\Cargo.toml -- --test-threads=1`: PASS, lib 253 passed plus all integration/doc tests, including ACR-P9 8 passed in 68.63s.

Required checks:

1. At least 8 required generated algebraic stress families exist and are support-producing.
2. Each family runs deterministic anti-overfit variants covering baseline, non-base variable-id renaming, relation-order permutation, and coefficient scaling by nonzero rationals.
3. No diagnostic fixture/name/imported file/problem hash/expected answer/hardcoded support polynomial is used.
4. Tests use public or near-public solver/pipeline paths, not only helper-level APIs.
5. Route trace/cost trace proves the successful route for each family.
6. Projection messages are verified, support is exactly verified, and `replay_run_certificate` accepts.
7. S3 uses an exact replayable backend; current implementation uses `ResultantBackendKind::QuadraticSubresultant` and exact Q verification.
8. S5 proves `SpecializationInterpolation` succeeds after dense/sparse prohibitions, including dense TRS materialization false and a bounded sparse-resultant failure/prohibition probe.
9. S6 must prove at least two enabled, non-skipped, non-`CostProhibited` Universal stages actually executed and failed before the chosen stage, using `executed_failed_strategy_hashes`; do not count `failed_strategy_hashes`, `skipped_cost_prohibited_strategy_hashes`, or disabled records.
10. Claim ceiling remains P9-only.

