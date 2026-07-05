# P6 Reviewer FAIL_FIXABLE Remediation

Initial Guardian P6 review result: `FAIL_FIXABLE`.

Blockers and fixes:

1. Incomplete ladder hash binding.
   - Root cause: top-level plan hash used stored child `KernelExecutionPlan.plan_hash`, so public execution-plan fields could be mutated without recomputing the top-level hash.
   - Fix: deep hash recomputation for child execution plans, support plans, resource bounds, failure behavior, relation-search schedules, Universal strategy steps, and rank plans.
   - Regression: `p6_declared_ladder_rejects_execution_plan_field_tampering`.

2. Insufficient RGQ-055 schedule evidence.
   - Root cause: only one RGQ-042 schedule reproducibility test existed.
   - Fix: added three-shape RGQ-055 test with different `|Y|`, `|Z|`, and degrees.
   - Regression: `rgq055_schedule_reproducibility_covers_three_local_shapes`.

3. RGQ-042 `e_cap` drift.
   - Root cause: builder used `.max(z_seed)`, silently widening an explicit low cap.
   - Fix: builder now uses `options.max_relation_search_export_degree.unwrap_or(e_cap_default)` exactly.
   - Regression: `rgq042_option_cap_is_not_silently_widened`.

Fresh verification after fixes:

- `cargo fmt --manifest-path geosolver-core/Cargo.toml -- --check`: pass.
- `cargo test --manifest-path geosolver-core/Cargo.toml p6_ -- --nocapture`: 3 passed.
- `cargo test --manifest-path geosolver-core/Cargo.toml rgq -- --nocapture`: 3 passed.
- `cargo test --manifest-path geosolver-core/Cargo.toml`: 112 passed.
