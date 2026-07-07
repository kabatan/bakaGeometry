# P7 Review Request

Reviewer prompt: RP-P7 from `REVIEWER_PROMPTS.md`.

Relevant R-IDs:

- BS-R080
- BS-R081
- BS-R082

Files to inspect:

- `geosolver-core/src/planner/cost_model.rs`
- `geosolver-core/src/planner/probes.rs`
- `geosolver-core/src/planner/admission.rs`
- `geosolver-core/src/planner/kernel_plan.rs`
- `geosolver-core/src/planner/ladder.rs`
- `geosolver-core/src/planner/planner.rs`
- `geosolver-core/src/kernels/mod.rs`
- `geosolver-core/src/kernels/traits.rs`
- `geosolver-core/src/solver/options.rs`
- `geosolver-core/scripts/audit_v4_conformance.py`

Evidence:

- `source_to_code_map.md`
- `algorithm_evidence.md`
- `commands.log`
- `static_audit.log`
- `changed_files.txt`

Requested checks:

- All nine kernels are registered in source order in enum, runtime registry, and planner list.
- `collect_kernel_admissions` calls every kernel and records source data/hash.
- UniversalTargetElimination is admitted for every relation block, and missing Universal is
  `ImplementationBug`.
- Cost estimates/probes are deterministic planning aids, not proof.
- Declared ladder contains only budgeted certificate-capable routes; Universal is last unless
  explicitly prioritized.
- Execution cannot call a route absent from the declared ladder.

Remediation focus after prior P7 FAIL_FIXABLE:

- `collect_kernel_admissions` now calls the runtime registry `all_kernels()` and every
  `kernel.admit(block, &kernel_context)` path.
- The old local planner-dispatch admission authority was removed from `planner/admission.rs`.
- `KernelAdmission` now carries `KernelAdmissionEvidence`: runtime admission hash, source relation
  IDs/hashes, initial bounds, matrix/template estimates, and evidence hash.
- Admission evidence hash is included in admission hash and emitted in route diagnostics.
- TargetRelationSearch uses the runtime trait `plan` path with the active solver context so its
  declared schedule is reproducible from the active options.

Requested decision: PASS / FAIL_FIXABLE / FAIL_BLOCKING / NEEDS_MORE_EVIDENCE for P7 only.
