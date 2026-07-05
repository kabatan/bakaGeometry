# Full Core Repair Cleanup Policy

This policy tells the Agent how to handle current inappropriate implementations.

---

## 1. Production reachability rule

A function is production-reachable if it can be called from any of:

```text
api::solve_target
solver::orchestrator::solve_with_context
planner::all_kernels / kernels::all_kernels
UniversalTargetEliminationKernel
any execute_* kernel
verify/replay production path
result finalizers
```

Production-reachable code must satisfy the v4 spec. Limited, fake, placeholder, or test-only code must not be production-reachable.

---

## 2. Delete or quarantine rules

Move to `#[cfg(test)]` or delete if the function:

```text
- has `_for_tests` in its name;
- exposes NotProductionF4;
- creates support directly for tests;
- uses synthetic certificates outside tests;
- bypasses planner/kernel/message/compose;
- recognizes a single algebraic slice and returns support as if generic;
- is only useful as a debug oracle.
```

---

## 3. Generalize rules

Generalize rather than delete if the function is a mandatory v4 production mechanism:

```text
TargetRelationSearchKernel
SparseResultantProjectionKernel
TargetActionKrylovKernel
UniversalTargetEliminationKernel
RegularChainProjectionKernel
NormTraceProjectionKernel
SpecializationInterpolationKernel
LinearAffineKernel
TargetUnivariateKernel
```

Generalization means implementing the algorithm contract from the v4 spec, not adding another recognized shape.

---

## 4. Forbidden cleanup patterns

Do not do any of the following:

```text
- rename a partial implementation and leave it production-reachable;
- write "not generic" in documentation while using the path in acceptance;
- leave a fake kernel in all_kernels;
- keep plan-time output construction but call it CertifiedProbePlan;
- delete a mandatory kernel and still claim CANDIDATE_COVER_CORE_READY;
- replace a narrow slice with a different narrow slice;
- rely on Universal as a hidden full-coordinate Groebner fallback.
```

---

## 5. Required cleanup report

Create `FULL_CORE_CLEANUP_REPORT.md`:

```yaml
removed_from_production:
  - path:
    reason:
    replacement:
quarantined_to_tests:
  - path:
    reason:
production_generalized:
  - path:
    previous_limitation:
    new_generic_contract:
still_blocking:
  - path:
    reason:
```

The `still_blocking` list must be empty before final repair closure.

