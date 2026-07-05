Read-only Guardian re-review for P6 remediation in repository C:\Users\bakat\OneDrive\ドキュメント\bakaGeometry. Do not edit files.

Context: An earlier P6 Guardian review returned FAIL_FIXABLE with 3 blockers:
1. top KernelPlan hash trusted stored child KernelExecutionPlan.plan_hash, so child execution-plan field tampering could bypass hash binding;
2. RGQ-055 needed at least three local ideals with different |Y|, |Z|, and degrees;
3. RGQ-042 e_cap used .max(z_seed), silently widening explicit option caps.

Please re-review the current working tree against:
- PLAN.md P6
- BASE_SPEC.md RGQ-015, RGQ-039, RGQ-041, RGQ-042, RGQ-047, RGQ-055, RGQ-062, MECH-005 and starts MECH-013/MECH-016
- P6_READINESS.md
- PRIMITIVE_SCOPE_LEDGER.md
- REVIEWER_PROMPTS.md section P6

Changed implementation files:
- geosolver-core/src/planner/cost_model.rs
- geosolver-core/src/planner/probes.rs
- geosolver-core/src/planner/admission.rs
- geosolver-core/src/planner/kernel_plan.rs
- geosolver-core/src/planner/ladder.rs
- geosolver-core/src/planner/planner.rs
- geosolver-core/src/kernels/target_relation_search.rs
- geosolver-core/src/solver/options.rs

Evidence folder:
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P6

Focus on actual code paths and the remediation:
- `hash_kernel_plan`/`hash_kernel_execution_plan`/support-plan hashes must recompute nested current contents and reject tampering before execution.
- `p6_declared_ladder_rejects_execution_plan_field_tampering` should be meaningful.
- RGQ-055 test should cover at least three local ideals with different |Y|, |Z|, and degrees, independently recomputing support hashes, row monomial hashes, matrix dimensions, and stage order.
- RGQ-042 e_cap should follow exact formula, with no silent widening.
- The original P6 checks still apply: all kernels considered, Universal admitted for well-formed Q-polynomial blocks and final in ladder, selected plans contain concrete support/rank/template/resource/certificate/failure data, Universal fixed sequence and NoLocalCertifiedNonFinite, no overclaim of P5R narrow primitives.

Return exact PASS or FAIL with concise findings, R-IDs/MECHs inspected, files/evidence inspected, and residual risks.
