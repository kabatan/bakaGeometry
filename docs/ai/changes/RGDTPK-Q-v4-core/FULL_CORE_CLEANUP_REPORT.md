# Full Core Cleanup Report

Purpose: active FCR cleanup ledger. Status is scoped to the current pre-P13 repair work and does not authorize final source-fidelity or acceptance completion claims.

```yaml
removed_from_production:
  - path: geosolver-core/src/kernels/mod.rs::all_kernels
    reason: Advanced projection kernels still have narrow or test-only coverage and must not be claimed as generic production implementations.
    replacement: TargetUnivariateKernel and LinearAffineKernel remain the only production registry entries until each advanced kernel is generalized or explicitly excluded from final scope.
  - path: geosolver-core/src/planner/admission.rs::all_planner_kernel_kinds
    reason: Planner admission must not declare incomplete advanced kernels as production completion paths.
    replacement: cfg(not(test)) planner kernel list returns TargetUnivariate and LinearAffine only.

quarantined_to_tests:
  - path: geosolver-core/src/kernels/sparse_resultant.rs
    reason: SparseResultantProjection remains available for test/support evidence but is not a production generic SparseResultant completion claim.
  - path: geosolver-core/src/kernels/specialization_interpolation.rs
    reason: SpecializationInterpolation remains available for test/support evidence but is not a production generic completion claim.
  - path: geosolver-core/src/kernels/regular_chain_projection.rs
    reason: RegularChainProjection remains available for test/support evidence but is not a production generic completion claim.
  - path: geosolver-core/src/kernels/norm_trace_projection.rs
    reason: NormTraceProjection remains available for test/support evidence but is not a production generic completion claim.

production_generalized:
  - path: geosolver-core/src/kernels/action_krylov.rs
    previous_limitation: target-only or local-univariate alias action path
    new_generic_contract: FCR-P5 added a GenericQuotient path from authorized relations with certified Groebner basis/action columns and full-basis coverage.
  - path: geosolver-core/src/kernels/universal_elimination.rs
    previous_limitation: local Groebner output/certificate mismatch risk and fake F4 ambiguity
    new_generic_contract: FCR-P6 routes Universal local elimination through certified LocalGroebner with keep-only outputs and exact output_membership replay; fake F4 remains test-only.

still_blocking:
  - path: final full-core completion claim
    reason: Candidate-cover readiness, source fidelity, and acceptance completion remain blocked until later FCR phases finish actual DAG replay, support/root/candidate finalization, acceptance partitions, and final claim audit.
```
