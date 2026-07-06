# Generic Planner Materialization Audit

Status: implementation evidence for `RGDTPK-Q-v4-generic-planner-success-route-v1`.

## Scope

This audit covers the dense TargetRelationSearch planning path, admission isolation, and Universal internal dense escalation guard. It does not claim exact-image completeness or full v4 acceptance.

## Findings

1. Dense monomial counts are estimated before support materialization.

   Evidence anchors:

   - `geosolver-core/src/planner/relation_schedule.rs:21` defines `SaturatingCount`.
   - `geosolver-core/src/planner/relation_schedule.rs:225` defines `estimate_dense_relation_search_schedule`.
   - `geosolver-core/src/planner/relation_schedule.rs:573` computes `#monomials(n,d)` as a closed-form binomial count.
   - `geosolver-core/src/planner/relation_schedule.rs:577` uses saturating `u128` arithmetic.

2. Dense support vectors are descriptor-first.

   Evidence anchors:

   - `geosolver-core/src/planner/relation_schedule.rs:119` defines `SupportDescriptor`.
   - `geosolver-core/src/planner/relation_schedule.rs:528` builds descriptor records from preflight estimates.
   - `geosolver-core/src/planner/relation_schedule.rs:341` builds a full schedule only after preflight.
   - `geosolver-core/src/planner/relation_schedule.rs:350` gates monomial materialization on `preflight.materialization_allowed`.

3. Admission does not enumerate dense supports before feasibility is known.

   Evidence anchors:

   - `geosolver-core/src/planner/admission.rs:167` estimates dense schedule feasibility.
   - `geosolver-core/src/planner/admission.rs:173` declines only `TargetRelationSearch` when dense materialization is prohibited.
   - `geosolver-core/src/planner/admission.rs:176` builds the materialized dense schedule only in the allowed branch.

4. Kernel execution and Universal escalation use the same guard.

   Evidence anchors:

   - `geosolver-core/src/kernels/target_relation_search.rs:152` preflights direct kernel admission.
   - `geosolver-core/src/kernels/target_relation_search.rs:256` preflights execution replay/recompute before materialization.
   - `geosolver-core/src/kernels/universal_elimination.rs:365` calls guarded `admit_target_relation_search`; a decline is a continuable hard-case stage.

5. Default planning caps are bounded.

   Evidence anchors:

   - `geosolver-core/src/planner/relation_schedule.rs:13` through `18` define default caps.
   - `geosolver-core/src/planner/relation_schedule.rs:90` derives planning caps from options. Explicit low execution limits do not lower planning safety caps; explicit larger limits can raise them.

## Static Scans

Commands run:

Source-specific forbidden-term scan over implementation, GPSR tests, and Active Context.

Result: no matches in the implementation, GPSR tests, or Active Context.

```text
rg -n "build_export_monomial_support|build_multiplier_supports|monomials_total_degree_leq|build_dense_relation_search_schedule" geosolver-core/src/planner/admission.rs geosolver-core/src/planner/planner.rs
```

Result: no direct dense support builders are reachable from planning/admission. The only planner admission schedule construction is after `materialization_allowed`.
