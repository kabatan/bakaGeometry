# ACR-P9 Mechanical Evidence

Scope: ACR-P9 generic large-footprint support-producing stress suite only.

Claim ceiling: this closes ACR-P9 evidence only after reviewer PASS. It does not claim final candidate-cover readiness, source-fidelity, exact-image acceptance, or full v4 acceptance; those remain ACR-P10 closure scope.

## Implemented Coverage

- Added and repaired `geosolver-core/tests/acr_p9_large_footprint_stress.rs`.
- The suite defines eight generated algebraic stress families S1-S8 from the alg-cost plan.
- Each family runs three deterministic isomorphic variants:
  - baseline with nonzero rational coefficient scaling,
  - non-base variable-id renaming (`offset * 3 + index + 1`) with coefficient scaling,
  - relation-order permutation with coefficient scaling.
- The suite uses public or near-public solver entry points and does not import or name the diagnostic geometry problem, its builder, its variable IDs, a known answer, or a known support polynomial.
- Every support-producing run asserts `CertifiedCandidateCover`, nonzero support, squarefree support, projection messages, route/cost trace presence, `verify_global_support`, `replay_run_certificate`, and projection-message verification.

## Reviewer-Finding Repairs

- S3 now proves an exact non-determinant feasible sparse resultant backend through `ResultantBackendKind::QuadraticSubresultant`; the old "small symbolic determinant" ambiguity is removed for that stress.
- S5 still proves `SpecializationInterpolation` success, but its inner sample relation search now uses `max_relation_search_export_degree=1` so the route is not a cap artifact. The same family also runs a separate bounded near-public `SparseResultantProjection` probe with `max_matrix_rows=1` and asserts `PlanProbeFailed` or `CostProhibited`.
- S6 now asserts `executed_failed_strategy_hashes.len() >= 2` from the Universal certificate. This field is replay-checked from enabled, non-`CostProhibited` strategy records before the chosen Universal stage, so cost-prohibited/disabled skips cannot satisfy the S6 internal-failure requirement.
- Semantic-free canonicalization now content-sorts relations and reassigns relation IDs after sorting, so relation-order variants are not accidentally input-order fixtures. Existing relation-ID-sensitive tests were adjusted to locate canonical relations by polynomial content.
- The resultant P4 guard test was moved to a cubic/cubic input so it still proves symbolic determinant resource rejection after the new quadratic subresultant backend.

## Stress Matrix

See `stress_matrix.yaml` in this directory. Highlights:

- S1: large block, dense TRS prohibited, TargetAction succeeds.
- S2: large block, dense TRS prohibited, sparse/lazy TRS succeeds.
- S3: sparse resultant feasible, `QuadraticSubresultant`, exact certificate replay.
- S4: sparse resultant expression-swell prohibited, later route succeeds.
- S5: specialization-interpolation succeeds after dense-route prohibition and bounded sparse-resultant failure probe.
- S6: Universal succeeds after at least two certificate-recorded executed internal strategy failures.
- S7: graph decomposition reduces a high-cost block through algebraic separators.
- S8: no useful separator, one large block, bounded Universal succeeds.

## Commands And Results

```text
cargo fmt --manifest-path geosolver-core\Cargo.toml --check
PASS

rg -n -i "diagnostic problem|imported file|mixtilinear|\bmixt\b|circumcircle|incircle|tangent_solver|expected_cos|expected target value|expected support polynomial|known_support_polynomial|problem hash|diagnostic_problem|geometry name" geosolver-core\src geosolver-core\tests\acr_p9_large_footprint_stress.rs geosolver-core\tests\generic_success_route_planner.rs geosolver-core\tests\gpsr_generic_planner_success_route.rs geosolver-core\tests\p14_full_pipeline_integration.rs geosolver-core\tests\fcr_p10_acceptance_suite.rs geosolver-core\tests\ccc_candidate_cover_completion.rs
PASS: no matches

cargo test --manifest-path geosolver-core\Cargo.toml --test acr_p9_large_footprint_stress -- --test-threads=1 --nocapture
PASS: 8 passed; 0 failed; 24 variants executed; finished in 69.13s standalone after S6 executed-failure certificate repair

cargo test --manifest-path geosolver-core\Cargo.toml algebra::resultant::tests:: -- --test-threads=1
PASS: 10 passed; 0 failed

cargo test --manifest-path geosolver-core\Cargo.toml --test generic_success_route_planner -- --test-threads=1
PASS: 3 passed; 0 failed

cargo test --manifest-path geosolver-core\Cargo.toml --test gpsr_generic_planner_success_route -- --test-threads=1
PASS: 5 passed; 0 failed

cargo test --manifest-path geosolver-core\Cargo.toml --test fcr_p10_acceptance_suite -- --test-threads=1
PASS: 12 passed; 0 failed

cargo test --manifest-path geosolver-core\Cargo.toml --test ccc_candidate_cover_completion -- --test-threads=1
PASS: 2 passed; 0 failed

cargo test --manifest-path geosolver-core\Cargo.toml --test p14_full_pipeline_integration -- --test-threads=1
PASS: 10 passed; 0 failed

cargo test --manifest-path geosolver-core\Cargo.toml -- --test-threads=1
PASS:
  lib: 253 passed; 0 failed
  acr_p9_large_footprint_stress: 8 passed; 0 failed; finished in 68.63s within full run
  ccc_candidate_cover_completion: 2 passed; 0 failed
  fcr_final_nonfinite_semantics: 2 passed; 0 failed
  fcr_p10_acceptance_suite: 12 passed; 0 failed
  fcr_p11_red_team_suite: 10 passed; 0 failed
  fcr_p4_pure_planning: 7 passed; 0 failed
  generic_success_route_planner: 3 passed; 0 failed
  gpsr_generic_planner_success_route: 5 passed; 0 failed
  p12_roots_decode_integration: 1 passed; 0 failed
  p12g_generality_stress: 1 passed; 0 failed
  p13_exact_image_semantics: 7 passed; 0 failed
  p14_full_pipeline_integration: 10 passed; 0 failed
  p15_acceptance_stress: 6 passed; 0 failed
  p3_public_pipeline_integration: 2 passed; 0 failed
  doc tests: 0 passed
```
