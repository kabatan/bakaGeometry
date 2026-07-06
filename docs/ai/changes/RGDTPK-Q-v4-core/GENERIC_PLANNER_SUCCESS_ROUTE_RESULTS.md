# Generic Planner Success Route Results

Status: GPSR implementation result summary.

## Implemented Changes

- Added closed-form dense TargetRelationSearch preflight with saturating counts for export columns, multiplier columns, matrix columns, row monomial upper bounds, and estimated support memory.
- Added descriptor-first dense support representation before materialized monomial vectors.
- Changed dense TargetRelationSearch admission so cost-prohibited dense routes decline locally instead of blocking later kernel admissions.
- Recorded `CostProhibitedDenseRoute` diagnostics with machine-readable details during planning.
- Reused the guarded dense schedule implementation from the kernel-facing public helper path to avoid duplicate planner/kernel semantics.
- Preserved Universal ladder behavior: internal dense escalation decline is a continuable stage failure, allowing later Universal strategies to run.

## Generic Stress Tests

New test file:

`geosolver-core/tests/gpsr_generic_planner_success_route.rs`

Coverage:

1. `gpsr_dense_trs_large_footprint_preflight_is_descriptor_only`
   - Large generic algebraic footprint.
   - Dense route is preflight-prohibited.
   - Schedule retains descriptors and does not materialize stages.

2. `gpsr_admission_isolation_keeps_later_kernels_after_dense_decline`
   - Dense TargetRelationSearch declines locally.
   - Later admissions are still collected.
   - Declared ladder excludes the declined dense route and retains feasible later routes.

3. `gpsr_large_footprint_action_route_still_produces_candidate_cover`
   - Public `solve_target`.
   - Generic large-footprint input.
   - Support-producing success through `TargetActionKrylov`.

4. `gpsr_large_footprint_sparse_route_still_produces_candidate_cover`
   - Public `solve_target`.
   - Generic large-footprint input.
   - Support-producing success through `SparseResultantProjection`.

5. `gpsr_universal_ladder_survives_internal_dense_decline`
   - Public `solve_target`.
   - Universal is preferred.
   - Dense escalation decline does not prevent Universal from producing a candidate-cover message.

The stress inputs use parameterized variable IDs and rational scaling. They are abstract algebraic inputs, not a replay of the prior timeout input.

## Verification Commands

```text
cargo test --manifest-path geosolver-core\Cargo.toml --test gpsr_generic_planner_success_route -- --nocapture
```

Result:

```text
5 passed; 0 failed
```

```text
cargo test --manifest-path geosolver-core\Cargo.toml
```

Result:

```text
216 unit tests passed
2 ccc_candidate_cover_completion tests passed
2 fcr_final_nonfinite_semantics tests passed
12 fcr_p10_acceptance_suite tests passed
10 fcr_p11_red_team_suite tests passed
7 fcr_p4_pure_planning tests passed
5 gpsr_generic_planner_success_route tests passed
1 p12_roots_decode_integration test passed
1 p12g_generality_stress test passed
7 p13_exact_image_semantics tests passed
10 p14_full_pipeline_integration tests passed
6 p15_acceptance_stress tests passed
2 p3_public_pipeline_integration tests passed
0 doctests failed
```
