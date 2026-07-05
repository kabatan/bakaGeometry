# P6 Evidence Notes

Status: evidence.
Authority: non-normative verification notes for Plan P6; does not weaken `BASE_SPEC.md`, `PLAN.md`, or `P5R_BASE_SPEC_AMENDMENT.md`.

P6 implements deterministic planning and declared ladders:

1. `planner/` now contains cost probes, cost estimates, admission collection, execution-plan records, ladder construction, and `plan_all_blocks`.
2. `collect_kernel_admissions` considers all nine Appendix A kernel kinds in the required order.
3. Generic P6-admitted plans are `TargetRelationSearch` and `UniversalTargetElimination`; `TargetUnivariate` is admitted only when an authorized target/separator-only relation exists.
4. P7/P8/P9 kernel families are explicitly declined until their owning phases provide generic support-producing plans, preventing P5R primitive overclaim.
5. TargetRelationSearch plans include a deterministic RGQ-042 dense schedule with support hashes, row monomial hashes, matrix dimensions, and stage hashes.
6. Universal plans include the RGQ-041 fixed local strategy sequence and `NoLocalCertifiedNonFinite`.
7. Every `KernelPlan` hash binds admissions, cost estimates, and every declared `KernelExecutionPlan`; `require_declared_kernel_plan` rejects tampered or missing ladder entries.
8. Hash binding recomputes nested execution-plan, support-plan, resource-bound, failure-behavior, relation-search schedule, Universal strategy, and rank hashes from current contents rather than trusting stored child hashes.
9. RGQ-055 schedule reproducibility is covered by three local ideal shapes with different eliminated-variable count, exported-variable count, and total degree.

Safety boundaries:

- P6 does not execute projection kernels, construct projection messages, build candidate covers, isolate roots, classify exact images, or connect public orchestration.
- Sparse resultant, specialization-interpolation, target-action Krylov, regular-chain, norm/trace, and guarded affine kernel execution remain later phases.
- Cost probes are stored as deterministic planning/cost data only; they are not proof of correctness.
- Universal local failure is planned to route to hard/resource/certificate outcomes and cannot certify nonfiniteness locally.

Reviewer remediation:

- Initial P6 Guardian review returned `FAIL_FIXABLE`.
- The implementation now includes deep execution-plan tamper rejection, exact RGQ-042 `e_cap` semantics, and RGQ-055 three-shape schedule reproducibility tests.

Claim boundary:

- P6 can support a `PARTIAL_MECHANISM_READY:MECH-005` review claim only after Guardian review passes.
- P6 starts but does not close MECH-013 or any candidate-cover/exact-image claim.
