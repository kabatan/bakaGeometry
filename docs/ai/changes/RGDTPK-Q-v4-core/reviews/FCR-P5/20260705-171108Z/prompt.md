Guardian boundary review for FCR-P5 closure.

Workspace: C:\Users\bakat\OneDrive\ドキュメント\bakaGeometry
Review target: FCR-P5 only.

Source requirements to inspect:
- docs/ai/changes/RGDTPK-Q-v4-core/FULL_CORE_REPAIR_BASE_SPEC.md FCR-006.
- docs/ai/changes/RGDTPK-Q-v4-core/FULL_CORE_REPAIR_PLAN.md FCR-P5.
- docs/ai/changes/RGDTPK-Q-v4-core/FULL_CORE_REPAIR_REVIEWER_PROMPTS.md FCR-P5.
- docs/ai/changes/RGDTPK-Q-v4-core/FULL_CORE_SOURCE_SPEC_COMPLIANCE_MAP.md TargetAction rows and current claim limits.

Code files to inspect:
- geosolver-core/src/algebra/quotient.rs
- geosolver-core/src/algebra/krylov.rs
- geosolver-core/src/algebra/normal_form.rs
- geosolver-core/src/algebra/groebner.rs
- geosolver-core/src/algebra/f4.rs
- geosolver-core/src/kernels/action_krylov.rs
- geosolver-core/src/verify/verify_message.rs
- geosolver-core/src/verify/certificates.rs

Evidence to inspect:
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/FCR-P5/commands.txt
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/FCR-P5/command_outputs.txt
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/FCR-P5/generic_action_static_scan.txt

Required checks:
1. PASS only if TargetActionKrylov now has a production GenericQuotient path beyond TargetOnly and AliasUnivariate.
2. Verify quotient basis construction is discovered or certified from authorized relations, not hard-coded to tests.
3. Verify each multiplication-by-target action column has an exact membership certificate against authorized relations.
4. Verify coverage uses deterministic full-basis/block coverage and undercoverage is rejected.
5. Verify no coordinate roots or full coordinate RUR are produced.
6. Verify the required FCR-P5 tests exist and pass through planner/admission/execute/message verification/replay.
7. Fail if expected supports are used by production code, if basis/action columns are hard-coded for the test examples, or if the complete production selection space is still only TargetOnly/AliasUnivariate.

Report PASS / FAIL_FIXABLE / FAIL_BLOCKING. Include file/line references for any finding. If PASS, state that FCR-P5 does not by itself authorize P13, exact-image readiness, candidate-cover readiness, source fidelity, or acceptance completion, and no R-ID is VERIFIED by this review.
