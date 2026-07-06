# Generic Success-Route Core Repair Reviewer Prompts v2

## Reviewer Meta-Protocol

For every phase, the reviewer must answer:

```text
1. Did the implementation add or use a concrete investigated problem as a test, gate, fixture, benchmark, route selector, or expected answer?
2. Did it add geometry-family dispatch?
3. Did it turn timeout into faster failure instead of support-producing success on generic algebraic stress?
4. Did it confuse admission/planning with execution success?
5. Did it leave a path where dense TargetRelationSearch materializes huge monomial supports during admission?
6. Did it preserve a single unified TargetProjectionDAG pipeline?
7. Did it preserve declared ladders and avoid hidden fallback?
8. Did it verify final support exactly and replay-bind messages/support/roots/candidates?
```

If any answer is bad, the phase must be FAIL.

A reviewer PASS is invalid if it only says tests passed. The reviewer must inspect the named source files and cite exact functions.

---

## GSR-REVIEW-P0

Review `GENERIC_SUCCESS_ROUTE_AGENT_RESET.md` and `GENERIC_SUCCESS_ROUTE_CURRENT_AUDIT.md`.

Reject if:
- the concrete investigated geometry problem or family name appears as a required regression;
- exact variable/relation counts from the investigated case are used as gate constants;
- the agent frames the repair as fast failure;
- the audit omits any required planner/DAG/kernel file.

Required response sections:
```yaml
phase: GSR-P0
status: PASS|FAIL
concrete_case_overfit_found: true|false
fast_failure_framing_found: true|false
missing_files: []
blocking_findings: []
required_fixes: []
```

---

## GSR-REVIEW-P1

Review dense TRS preflight and lazy schedule.

Must inspect:
```text
planner/relation_schedule.rs
solver/options.rs
planner/admission.rs
kernels/target_relation_search.rs
```

Reject if:
- `build_dense_relation_search_schedule` can be called in admission before preflight passes;
- preflight allocates monomial vectors;
- no default planner caps exist;
- execution does not recheck caps before materializing supports;
- tests use the investigated concrete problem;
- estimated counts can overflow silently.

Required adversarial check:
Explain how the code handles a hypothetical 30+ variable, degree >= 7 local block without materializing monomial supports.

---

## GSR-REVIEW-P2

Review admission isolation.

Must inspect:
```text
planner/admission.rs
planner/kernel_plan.rs
planner/planner.rs
result/cost_trace.rs
result/diagnostics.rs
```

Reject if:
- one route-local failure prevents later kernel admissions;
- admissions vector does not contain records for all kernel kinds;
- cost-prohibited route is converted into solve-level failure during planning;
- Universal is missing for relation-bearing blocks;
- route diagnostics are not machine-readable.

Required adversarial check:
Describe the behavior when TargetRelationSearch is cost-prohibited but TargetActionKrylov and Universal are admissible.

---

## GSR-REVIEW-P3

Review declared ladder and execution isolation.

Must inspect:
```text
planner/ladder.rs
planner/kernel_plan.rs
planner/planner.rs
solver/pipeline.rs
kernels/traits.rs
```

Reject if:
- the ladder can be empty for relation-bearing blocks;
- Universal is not last/present for relation-bearing blocks;
- `execute_block_with_declared_ladder` stops after the first failed route;
- failure traces from failed routes are lost;
- any executed route was not declared in the ladder;
- a "planned" route is counted as success without executing and verifying a `ProjectionMessage`.

Required adversarial check:
Describe a generated case where route 1 fails and route 2 succeeds. Verify code supports it.

---

## GSR-REVIEW-P4

Review UniversalTargetElimination.

Must inspect:
```text
kernels/universal_elimination.rs
algebra/elimination.rs
kernels/sparse_resultant.rs
kernels/action_krylov.rs
kernels/specialization_interpolation.rs
verify/verify_message.rs
```

Reject if:
- Universal is a placeholder or only returns hardcase for large blocks;
- Universal invokes dense TRS without preflight;
- Universal does not try later strategies after earlier strategy failure;
- Universal returns nonfinite from exhaustion;
- Universal output contains non-exported variables;
- Universal certificate does not record chosen and failed internal strategies.

Required adversarial check:
Explain how Universal handles a large block where dense TRS is cost-prohibited but a compact target-action or sparse route exists.

---

## GSR-REVIEW-P5

Review decomposition repair.

Must inspect:
```text
graph/weighted_primal.rs
graph/separators.rs
graph/tree_decomposition.rs
graph/projection_dag.rs
graph/metrics.rs
```

Reject if:
- separator scoring ignores algebraic projection cost;
- large relation-heavy blocks are not penalized;
- geometry role/name affects decomposition;
- relation assignment loses authorization;
- relation duplication, if present, lacks certificate;
- no-useful-separator case is treated as unsupported.

Required adversarial check:
Provide a generic algebraic graph where separator-rich structure should split. Explain what the implementation does.

---

## GSR-REVIEW-P6

Review generic success-route stress suite.

Must inspect:
```text
tests/generic_success_route_planner.rs
planner/*
kernels/*
solver/*
compose/*
verify/*
```

Reject if:
- tests include the concrete investigated problem or family name;
- stress is helper-only and does not call `api::solve_target`;
- support-producing cases return failure statuses;
- dense TRS is not cost-prohibited in large-footprint cases;
- successful route is geometry-specific or expected-answer based;
- replay is not checked;
- cost trace / route trace is absent.

Required response must list every generated stress family:
```yaml
families:
  - id:
    generated_parameters:
    dense_trs_status:
    successful_kernel:
    public_status:
    replay_accepted:
    route_trace_present:
```

---

## GSR-REVIEW-P7

Final anti-overfit and closure review.

Must inspect:
```text
GENERIC_SUCCESS_ROUTE_CLOSURE.md
GENERIC_SUCCESS_ROUTE_STATIC_SCAN.md
GENERIC_SUCCESS_ROUTE_ACCEPTANCE_RESULTS.md
all modified production source files
all new tests
```

Reject if:
- closure claims benchmark superiority;
- closure claims exact-image completion;
- closure claims full v4 source-fidelity beyond candidate-cover/routing readiness;
- concrete investigated case appears in tests or production dispatch;
- static scan hits are unclassified;
- failures are accepted as completion for support-producing generic stress;
- reviewer cannot trace a generic large-footprint input through public pipeline to `CertifiedCandidateCover`.

Required final answer:
```yaml
phase: GSR-P7
status: PASS|FAIL
allowed_claims:
  - GENERIC_SUCCESS_ROUTE_PLANNER_READY
  - CANDIDATE_COVER_CORE_READY_if_existing_candidate_cover_closure_remains_valid
forbidden_claims:
  - EXACT_IMAGE_CORE_READY
  - RGDTPK_Q_V4_ACCEPTANCE_COMPLETE
blocking_findings: []
required_fixes: []
```
