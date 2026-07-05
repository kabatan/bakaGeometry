RESULT: PASS

**Blocking Findings**
None for PLAN.md P7 scope.

**Review Notes**
- Kernel registry lists all nine kernels in Appendix order: [mod.rs](C:/Users/bakat/OneDrive/ドキュメント/bakaGeometry/geosolver-core/src/kernels/mod.rs:24).
- TargetUnivariate now admits child-message target-only relations and binds child package hashes in the plan: [target_univariate.rs](C:/Users/bakat/OneDrive/ドキュメント/bakaGeometry/geosolver-core/src/kernels/target_univariate.rs:106), [target_univariate.rs](C:/Users/bakat/OneDrive/ドキュメント/bakaGeometry/geosolver-core/src/kernels/target_univariate.rs:125).
- TargetUnivariate execution checks plan hash, block authorization, child message presence, and planned source hashes before support construction: [target_univariate.rs](C:/Users/bakat/OneDrive/ドキュメント/bakaGeometry/geosolver-core/src/kernels/target_univariate.rs:199), [target_univariate.rs](C:/Users/bakat/OneDrive/ドキュメント/bakaGeometry/geosolver-core/src/kernels/target_univariate.rs:417), [target_univariate.rs](C:/Users/bakat/OneDrive/ドキュメント/bakaGeometry/geosolver-core/src/kernels/target_univariate.rs:446).
- Planner TargetUnivariate admission is now target-only, not separator/exported-variable-only: [admission.rs](C:/Users/bakat/OneDrive/ドキュメント/bakaGeometry/geosolver-core/src/planner/admission.rs:116).
- LinearAffine checks authorization/source hashes, safe pivots/guards, clears denominators, and exports only planned exported variables: [linear_affine.rs](C:/Users/bakat/OneDrive/ドキュメント/bakaGeometry/geosolver-core/src/kernels/linear_affine.rs:262), [linear_affine.rs](C:/Users/bakat/OneDrive/ドキュメント/bakaGeometry/geosolver-core/src/kernels/linear_affine.rs:286), [linear_affine.rs](C:/Users/bakat/OneDrive/ドキュメント/bakaGeometry/geosolver-core/src/kernels/linear_affine.rs:305), [linear_affine.rs](C:/Users/bakat/OneDrive/ドキュメント/bakaGeometry/geosolver-core/src/kernels/linear_affine.rs:467).
- P7 replay paths now reject stale package identity and wrong block/export context at the P7 structural replay level: [target_univariate.rs](C:/Users/bakat/OneDrive/ドキュメント/bakaGeometry/geosolver-core/src/kernels/target_univariate.rs:63), [linear_affine.rs](C:/Users/bakat/OneDrive/ドキュメント/bakaGeometry/geosolver-core/src/kernels/linear_affine.rs:59).

**Evidence Inspected**
- Base Spec RGQ-016/RGQ-017/RGQ-018/RGQ-038 and Appendix §14-§16.
- PLAN.md P7 and REVIEWER_PROMPTS.md#P7.
- P7 evidence files under `docs/ai/changes/RGDTPK-Q-v4-core/evidence/P7/*`.
- Remediated source files requested.
- I reran: `cargo test --manifest-path geosolver-core/Cargo.toml p7_ -- --nocapture` -> 9 passed.

**Residual Risks**
- P7 replay is still structural kernel-level replay, not final run replay or full certificate replay closure. That is acceptable for this P7 scope and remains bounded by later replay/certificate phases.
- P8/P9-owned kernels remain registered placeholders and are not covered by this PASS.

**Exact PASS Scope**
PASS applies only to PLAN.md P7: kernel trait/registry, TargetUnivariate, LinearAffine, P7 admission/execute consistency, P7 replay presence, and P6 regression risk from changed shared planning/admission structs. Claim ceiling is no higher than MECH-007 started. No later generic kernel readiness, final replay closure, candidate-cover completion, exact-image completion, or source-faithful/acceptance-complete claim is supported.
