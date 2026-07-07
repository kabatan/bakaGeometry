# ACR-P10 Static Scan Results

## Forbidden Fixture Scan

Command:

```text
rg -n -i "mixtilinear|mixtilinear_candidate_cover_problem|circumcircle|incircle|tangent_solver|expected_cos|expected target value|expected support polynomial|known_support_polynomial|diagnostic_problem|problem hash|geometry name" geosolver-core\src geosolver-core\tests
```

Result: PASS, no matches.

## CoreInvariantFlags Scan

Command:

```text
rg -n "CoreInvariantFlags|invariants|no_qe_cad|no_full_coordinate_rur|no_dispatch|scan_hash" geosolver-core\src\verify\run_certificate.rs geosolver-core\tests\p14_full_pipeline_integration.rs geosolver-core\tests\p15_acceptance_stress.rs
```

Result: expected references only. The scan shows runtime flags are conservative and final claims
require scan-hash-bound evidence.

## Route Budget Scan

Command:

```text
rg -n "struct RouteBudget|RouteBudget|route_budget|budget_hash|max_work_units|max_elapsed_steps|route_failure|KernelRouteTrace" geosolver-core\src\planner geosolver-core\src\solver geosolver-core\src\kernels
```

Result: expected implementation and tests for budget binding, enforcement, diagnostics, and kernel
guards.

## Decomposition Scan

Command:

```text
rg -n "separator|Separator|cost_aware|algebraic.*cost|large_block|decomposition" geosolver-core\src\graph geosolver-core\tests\acr_p9_large_footprint_stress.rs
```

Result: expected implementation and tests for algebraic-cost-aware separator scoring, diagnostics,
projection DAG construction, and P9 S7/S8 stress coverage.

