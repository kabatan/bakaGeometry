RESULT: PASS

FCR-P1 audit is admissible for the repair plan.

Checked:
- Wildcard `geosolver-core/src/kernels/*::...` trait rows are gone.
- All 45 `TargetProjectionKernel` trait-dispatch rows use concrete file paths and source_line values.
- Actual input/output classes are semantic, not the prior boilerplate.
- No high/fatal rows are assigned `keep`.
- `replay_run_certificate` remains downgraded to `certificate_binding: missing`.
- `final_support` finalizers remain downgraded; no overclaim of exact final certificate binding.
- `KernelPlan::new` and `KernelExecutionPlan::new` are disambiguated.
- Evidence records semantic checks, concrete trait-row validation, mandatory defect markers, and downgrade checks.

Residual risks:
- PASS here only admits FCR-P1 as a repair-driving audit. It does not verify any implementation repair, candidate-cover readiness, replay correctness, or production safety. No files edited.
