Guardian boundary review for FCR-P4 closure.

Workspace: C:\Users\bakat\OneDrive\ドキュメント\bakaGeometry
Review target: FCR-P4 only.

Source requirements to inspect:
- docs/ai/changes/RGDTPK-Q-v4-core/FULL_CORE_REPAIR_BASE_SPEC.md, especially forbidden planning work: no plan-time resultant relation, inner TargetRelationSearch support, Groebner/F4/regular-chain/norm-trace projection generators, final target support, or heavy final-output computation mislabeled PurePlan.
- docs/ai/changes/RGDTPK-Q-v4-core/FULL_CORE_REPAIR_PLAN.md FCR-P4.
- docs/ai/changes/RGDTPK-Q-v4-core/FULL_CORE_REPAIR_REVIEWER_PROMPTS.md FCR-P4.

Code files to inspect:
- geosolver-core/src/planner/kernel_plan.rs
- geosolver-core/src/kernels/sparse_resultant.rs
- geosolver-core/src/kernels/specialization_interpolation.rs
- geosolver-core/src/kernels/regular_chain_projection.rs
- geosolver-core/src/kernels/norm_trace_projection.rs
- geosolver-core/src/kernels/action_krylov.rs
- geosolver-core/src/kernels/universal_elimination.rs
- geosolver-core/tests/fcr_p4_pure_planning.rs

Evidence to inspect:
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/FCR-P4/commands.txt
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/FCR-P4/command_outputs.txt
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/FCR-P4/pure_planning_static_scan.txt

Required checks:
1. Inspect every listed plan_* function and any probe helper it calls.
2. Fail if a plan/probe path calls or indirectly calls compute_resultant_relation, execute_target_relation_search, eliminate_to_keep_variables, local_regular_chain_decomposition, project_chain_to_variables, norm_relation_for_tower_plan, or any build_*_trace that returns final relation generators.
3. Fail if CertifiedProbePlan is used to hide final output construction during planning.
4. Verify relation-producing logic now occurs in execute_* and is bound into certificates/replay-relevant message construction.
5. Verify the required FCR-P4 tests exist and passed.

Report PASS / FAIL_FIXABLE / FAIL_BLOCKING. Include file/line references for any finding. If PASS, state that the claim ceiling remains PARTIAL_MECHANISM_READY:MECH-011, no R-ID is VERIFIED by this review, and this does not authorize P13, exact-image readiness, candidate-cover readiness, source fidelity, or acceptance completion.
