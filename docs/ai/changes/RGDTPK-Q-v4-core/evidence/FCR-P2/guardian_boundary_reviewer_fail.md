RESULT: FAIL_FIXABLE

Blocker:
- Partial high-risk kernels were still public production modules in `geosolver-core/src/kernels/mod.rs`: `action_krylov`, `sparse_resultant`, `norm_trace_projection`, `regular_chain_projection`, `specialization_interpolation`, `target_relation_search`, and `universal_elimination` were exported without `#[cfg(test)]`. Registry/admission quarantine was not enough for FCR-P2 because the plan fails code that remains in a public production path.

What passed:
- `algebra::f4` is `#[cfg(test)]`.
- `NonProductionGroebnerBatchForTests` is test-only.
- `synthetic_for_tests` is test-only.
- `verify::replay` and its re-export are test-only.
- `all_kernels()` and production `all_planner_kernel_kinds()` only expose `TargetUnivariate` and `LinearAffine`.
- Production admission explicitly declines advanced kinds if reached.

Required fix:
- Move the partial kernel modules/functions behind `#[cfg(test)]` or a test-support module, or actually generalize them before exposing them as production modules. Keep only production-safe kernel modules publicly reachable.

Evidence issue:
- FCR-P2 evidence proved registry/admission quarantine, but its `PASS_BY_QUARANTINE` was too narrow because it did not reject the still-public production module exports.
