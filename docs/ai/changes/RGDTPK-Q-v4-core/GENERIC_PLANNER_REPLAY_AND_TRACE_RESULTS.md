# Generic Planner Replay And Trace Results

Status: trace/replay evidence for GPSR closure.

## Trace Evidence

The public planner now emits a `CostProhibitedDenseRoute` diagnostic when dense TargetRelationSearch is not materializable.

Machine-readable fields asserted by tests:

- `kernel = TargetRelationSearch`
- `route = DenseTotalDegree`
- `decision = CostProhibitedDenseRoute`
- `block_id`
- `stage_count`
- `materialized_stage_cap`
- `matrix_col_cap`
- `matrix_row_cap`
- `memory_cap_bytes`
- `first_export_degree`
- `estimated_matrix_cols`
- `estimated_rows`
- `estimated_memory_bytes`
- `first_prohibited_stage` when present

Evidence anchors:

- `geosolver-core/src/planner/planner.rs:58` records the diagnostic from declined dense admission.
- `geosolver-core/tests/gpsr_generic_planner_success_route.rs:150` asserts the machine-readable diagnostic fields.
- `geosolver-core/tests/gpsr_generic_planner_success_route.rs:245` asserts planning retains later kernels after dense decline.

## Replay Evidence

The support-producing GPSR stress cases call `replay_run_certificate` and require acceptance.

Evidence anchors:

- `geosolver-core/tests/gpsr_generic_planner_success_route.rs:170` defines the support success assertion.
- `geosolver-core/tests/gpsr_generic_planner_success_route.rs:198` requires replay acceptance.
- `geosolver-core/tests/gpsr_generic_planner_success_route.rs:286` covers the action route.
- `geosolver-core/tests/gpsr_generic_planner_success_route.rs:301` covers the sparse route.
- `geosolver-core/tests/gpsr_generic_planner_success_route.rs:316` covers the Universal route.

## Universal Behavior

Universal dense escalation uses the same guarded `admit_target_relation_search` path as direct dense TargetRelationSearch admission. When that route is prohibited, the stage returns a continuable hard case and the Universal strategy loop proceeds.

Evidence anchors:

- `geosolver-core/src/kernels/universal_elimination.rs:365` calls guarded dense admission.
- `geosolver-core/src/kernels/universal_elimination.rs:321` treats hard-case stage failures as continuable.
- `geosolver-core/tests/gpsr_generic_planner_success_route.rs:316` confirms Universal still produces a candidate-cover result.
