# Quick Guardian Prompt — Full Core Repair

You must stop the current P13/P14 continuation and implement `RGDTPK-Q-v4-full-core-repair-v1`.

Read in order:

```text
1. FULL_CORE_REPAIR_BASE_SPEC.md
2. FULL_CORE_REPAIR_PLAN.md
3. FULL_CORE_REPAIR_CLEANUP_POLICY.md
4. FULL_CORE_REPAIR_CURRENT_DEFECT_AUDIT.md
5. FULL_CORE_REPAIR_ACCEPTANCE_MATRIX.yaml
6. FULL_CORE_REPAIR_REVIEWER_PROMPTS.md
7. Original BASE_SPEC.md
8. GeoSolver Core Algorithm Specification v4.0 if available in the repo/context
9. GeoSolver failure-cause summary if available in the repo/context
```

Non-negotiable goal:

```text
When this repair is complete, api::solve_target must implement the full production R-GDTPK-Q candidate-cover solver core for algebraic common representations derived from geometry IR. It must not be a narrow slice solver, a helper test suite, or a documented partial implementation.
```

Do not proceed to P13/P14/P15/P16 until all FCR phases pass.

Immediate actions:

```text
1. Reset active claim documents so P13/P14 are blocked.
2. Audit P1–P12G production code and identify every partial or hidden-execution path.
3. Delete/quarantine/generalize inappropriate production code.
4. Connect the full public candidate-cover pipeline.
5. Enforce pure planning and move relation construction to execute.
6. Implement generic TargetActionKrylov, not only target-univariate or local-univariate alias.
7. Implement or remove fake/non-generic production claims for F4, SparseResultant, SpecializationInterpolation, RegularChain, NormTrace.
8. Make actual DAG/block replay the main replay path.
9. Run public-pipeline acceptance suites and tamper suites.
10. Produce CANDIDATE_COVER_CORE_READY only if all required evidence passes.
```

Reviewer rule:

```text
Any phase must fail if it passes only by documenting limitations, keeping partial production code, using module-only helper tests, or relying on a previous Guardian PASS.
```


## v2 additions to follow immediately

Before implementing, read and obey:

```text
FULL_CORE_AGENT_FAILURE_MODE_RESET_TEMPLATE.md
FULL_CORE_REPAIR_SPEC_ALIGNMENT_AUDIT.md
FULL_CORE_REPAIR_BASE_SPEC.md sections FCR-016 through FCR-020
FULL_CORE_REPAIR_PLAN.md sections FCR-P0A, FCR-P1A, and FCR-P12
FULL_CORE_REPAIR_REVIEWER_PROMPTS.md Reviewer Meta-Protocol v2
```

Do not treat this as a documentation cleanup. Current narrow implementations must be generalized, deleted from production, or quarantined. If a v4-required algorithm cannot be implemented without narrowing scope, raise AlgorithmDefect.
