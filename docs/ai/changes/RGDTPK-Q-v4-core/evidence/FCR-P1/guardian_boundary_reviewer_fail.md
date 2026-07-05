# FCR-P1 Guardian Boundary Reviewer Result

Reviewer: guardian_boundary_reviewer

Result: FAIL_FIXABLE

Blockers:

- `FULL_CORE_PRODUCTION_AUDIT.md` does not consistently provide actual input/output classes. Many rows use boilerplate like "General Q-polynomial/data input accepted by the function signature" and "Declared return type..." instead of the real algorithmic class.
- Replay is softened: the mandatory row says synthetic all-relations replay is `missing|replace`, but the public row for `verify/replay.rs::replay_run_certificate` marks `certificate_binding: exact`. Code reconstructs replay blocks from `compressed.relation_order`, not actual DAG blocks, so `exact` is not supported.
- Final support/nonfinite rows are softened: `certify_nonfinite_target_image` and related rows mark local certificates as `exact`, while code can emit `ZeroTargetEliminationOverQ` and final results with `certificate: None::<CoreRunCertificate>`.
- Public method identity is ambiguous: `planner/kernel_plan.rs::new` appears twice without disambiguating `KernelPlan::new` vs `KernelExecutionPlan::new`.
- The command evidence is only mechanical row-count/value checking and cannot establish semantic accuracy required by FCR-P1.

Required fixes:

- Rewrite `FULL_CORE_PRODUCTION_AUDIT.md` rows with exact function-qualified paths and real input/output classes.
- Downgrade synthetic replay and final-support certificate rows to the actual strength, with repair actions.
- Re-run semantic review against the corrected audit; mechanical checks alone are insufficient.

Forbidden claims:

- Do not claim FCR-P1 PASS.
- Do not claim the production audit is complete or source-faithful.
- Do not claim certificate/replay binding is exact for replay or final support paths.
