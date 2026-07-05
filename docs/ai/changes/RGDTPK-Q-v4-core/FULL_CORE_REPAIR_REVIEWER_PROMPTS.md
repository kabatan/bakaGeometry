# Full Core Repair Reviewer Prompts

These prompts are mandatory. A reviewer must inspect actual code, not only evidence files. If the code implements only a narrow algebraic case, the phase fails even when the claim is carefully limited.

---

## Global reviewer instruction

You are reviewing whether the repository now implements the full R-GDTPK-Q candidate-cover solver core, not whether a phase generated plausible evidence.

Fail if any of the following is true:

```text
- a production path handles only a narrow algebraic slice and the phase treats it as generic;
- a kernel plan computes final output relations during planning;
- a helper bypasses planner/kernel/message/compose and is used as acceptance evidence;
- public solve_target is not connected to the full pipeline;
- replay succeeds using synthetic all-relations blocks instead of actual DAG/block authorization;
- UniversalTargetElimination is absent, non-generic, hidden, or coordinate-first;
- TargetActionKrylov is still target-only/local-univariate-alias only;
- SparseResultant is still pairwise-binary only while claimed generic;
- F4 is fake or test-only while claimed production;
- support-producing acceptance cases return hard-case, resource failure, certificate gap, nonfinite, or empty support;
- tests are named slices rather than algebraic-structure stress through the production pipeline.
```

The correct question is always:

```text
Can the current production solver take the algebraic common representation from arbitrary geometry IR lowering and directly narrow the target to a finite certified candidate cover using the generic R-GDTPK pipeline?
```

If the answer is not yes, fail.

---

## FCR-P0 reviewer prompt

Inspect `ACTIVE_CONTEXT.md`, `CLOSURE.md`, `P12G_READINESS.md`, and current git state.

PASS only if:

```text
- P13/P14/P15/P16 are blocked;
- old P12G PASS is not treated as generality proof;
- P8c/MECH-014 is reopened or explicitly blocked until generic TargetActionKrylov is implemented;
- current claim ceiling does not exceed PARTIAL_MECHANISM_READY:MECH-011;
- commands are fresh at current HEAD.
```

Fail if continuation to P13 is still allowed.

---

## FCR-P1 reviewer prompt

Inspect every production module listed in the plan. Read code, not only the audit table.

PASS only if `FULL_CORE_PRODUCTION_AUDIT.md` accurately lists:

```text
- every production-reachable function;
- actual input/output class;
- whether it is generic or partial;
- whether it constructs final relations during plan;
- certificate strength;
- required action.
```

You must directly inspect the current implementations of:

```text
solver/orchestrator.rs
algebra/f4.rs
kernels/action_krylov.rs
kernels/sparse_resultant.rs
kernels/specialization_interpolation.rs
kernels/regular_chain_projection.rs
kernels/norm_trace_projection.rs
verify/replay.rs
planner/kernel_plan.rs
```

Fail if any known partial path is omitted or softened.

---

## FCR-P2 reviewer prompt

Verify that inappropriate implementations were actually removed, quarantined, or generalized.

PASS only if:

```text
- production registry cannot reach non-production F4 or *_for_tests code;
- alias-univariate TargetActionKrylov is not the full production kernel;
- pairwise binary resultant is not claimed as generic SparseResultantProjection;
- explicit tower detection is not claimed as generic NormTrace coverage;
- synthetic all-relations replay is not the final replay path;
- no helper-only support construction is used for acceptance.
```

Fail if comments say "not generic" but the code remains the success path.

---

## FCR-P3 reviewer prompt

Review public pipeline integration.

PASS only if `api::solve_target` and `solver::orchestrator::solve_with_context` execute the entire candidate-cover pipeline from validation to final certificate.

Required code evidence:

```text
- no temporary_pipeline_not_connected;
- step_compress exists and is called;
- graph/DAG construction exists and is called;
- plan_all_blocks is called;
- projection kernels execute in DAG order with child messages;
- projection messages are verified;
- compose and support construction are called;
- global support is verified;
- roots and candidates are produced;
- CoreRunCertificate is included in success result.
```

Fail if orchestrator hand-builds a support or message for tests.

---

## FCR-P4 reviewer prompt

Review plan/execute separation across all kernels.

You must inspect every `plan_*` function in:

```text
kernels/sparse_resultant.rs
kernels/specialization_interpolation.rs
kernels/regular_chain_projection.rs
kernels/norm_trace_projection.rs
kernels/action_krylov.rs
kernels/universal_elimination.rs
```

Fail if a plan function calls or indirectly calls any function that constructs final relation generators, including:

```text
compute_resultant_relation
execute_target_relation_search
eliminate_to_keep_variables
local_regular_chain_decomposition
project_chain_to_variables
norm_relation_for_tower_plan
build_*_trace that returns output relation
```

PASS only if final relation construction happens in execute and replay verifies it.

---

## FCR-P5 reviewer prompt

Review generic TargetActionKrylov.

PASS only if TargetActionKrylov constructs a quotient/action handle for nontrivial multivariate authorized ideals, not just target-only or local-univariate-plus-linear-alias cases.

Required algorithm evidence:

```text
- quotient basis is discovered or certified from authorized relations;
- basis is not hard-coded to test examples;
- multiplication-by-target columns are exact membership-certified;
- characteristic support coverage uses deterministic full-basis/block coverage;
- output support is verified and replayable;
- no coordinate roots or full RUR appear;
- multivariate quotient stress passes through planner/admission/execute/message verification.
```

Mandatory failure if the code still chooses only:

```text
TargetOnly
AliasUnivariate
```

as the complete production selection space.

---

## FCR-P6 reviewer prompt

Review production F4/Groebner/local elimination.

PASS only if:

```text
- fake/test F4 is not production reachable;
- if F4 is claimed, it is a real sparse matrix F4/F5-like implementation with exact certificates;
- if F4 is not implemented, all F4 claims are removed and Universal uses certified local Groebner/TargetRelationSearch;
- local elimination returns only keep variables;
- every output generator has exact membership certificate;
- no coordinate roots/RUR are produced.
```

Fail if any `NotProductionF4`, `for_tests`, or `NonProductionGroebnerBatch` path is reachable from production execution.

---

## FCR-P7 reviewer prompt

Review SparseResultant, SpecializationInterpolation, RegularChain, and NormTrace.

PASS only if each kernel implements its v4 contract or is removed from completion claims.

Fail if:

```text
- SparseResultant is only a binary pair chain while claimed generic;
- SpecializationInterpolation computes samples or inner supports during plan;
- RegularChain decomposes/projects during plan;
- NormTrace constructs norm relation during plan;
- exact Q verification is missing;
- output relation is not exported-only;
- module-only tests are used as proof of production readiness.
```

---

## FCR-P8 reviewer prompt

Review actual DAG/block replay and certificate finalization.

PASS only if `replay_run_certificate` verifies messages against the actual TargetProjectionDAG blocks and not synthetic all-relations blocks.

Mandatory tamper tests:

```text
- remove a block relation authorization -> replay fails;
- message uses relation outside block -> replay fails;
- child message not on DAG edge -> replay fails;
- plan hash tamper -> replay fails;
- package hash tamper -> replay fails;
- support tamper -> replay fails;
- root/candidate omission/duplication -> replay fails.
```

Fail if actual DAG replay is only a helper or final-claim blocker.

---

## FCR-P9 reviewer prompt

Review support, roots, candidates, and result construction.

PASS only if successful public candidate-cover results contain:

```text
CertifiedCandidateCover
nonzero support_polynomial
squarefree_support_polynomial
exact root_isolation, possibly empty
exact decoded_candidates matching roots, possibly empty
projection_messages
CoreRunCertificate
GlobalCostTrace
```

Fail if support is hand-built, if root decode is skipped, or if no-real-root support becomes hard-case.

---

## FCR-P10 reviewer prompt

Review the acceptance suite.

PASS only if every required support-producing category uses public or near-public production
pipeline and returns CertifiedCandidateCover with exact verified support. Certified nonfinite is not
part of P10 support-producing acceptance unless the public result carries a machine-readable,
replay-bound nonfinite certificate.

Fail if:

```text
- any required support-producing case is module-only;
- any support-producing case returns hard-case/resource/certificate/nonfinite;
- expected answers, case ids, or fixture names appear in production code;
- variable/relation permutations are absent;
- TargetActionKrylov stress is still univariate/alias-only;
- no one-large-block Universal case exists;
- no multiseparator composition case exists.
- P10 evidence is used to claim final nonfinite readiness.
```

---

## FCR-P11 red-team and final nonfinite reviewer prompt

Do not read previous PASS summaries until after code review. First inspect the current source code
and run or specify new algebraic counterexamples.

PASS only if:

```text
- the reviewer creates at least 10 new non-fixture algebraic inputs not used in FCR-P10;
- each new input runs through public or near-public pipeline;
- the set includes multivariate quotient/action, two-separator composition, sparse resultant,
  guarded rational affine, one-large-block no-separator, target-independent feasibility, positive
  nonfinite, and similar no-positive-proof cases;
- failures are resource/certificate failures with traces, not partial-slice failures;
- final nonfinite is either backed by a machine-readable replay-bound positive certificate or kept
  out of CANDIDATE_COVER_CORE_READY;
- CoreInvariantFlags are tied to fresh static scans and replay/tamper evidence;
- no-dispatch and no-QE/CAD/no-full-coordinate claims are backed by explicit scan outputs;
- exact-image/source-fidelity/full-acceptance claims remain blocked unless P13 exact-image passes.
```

Fail if the review would have passed the old P12G implementation, if fewer than 10 fresh algebraic
inputs are constructed, or if it relies on FCR-P10 examples as the red-team input set.

---

## FCR-P12 reviewer prompt

Final closure review.

PASS only if all previous FCR phase reviews pass, including FCR-P11 red-team/final-nonfinite gate,
and current HEAD has fresh command evidence.

The final reviewer must independently inspect:

```text
solve_with_context
plan_all_blocks
all all_kernels entries
TargetActionKrylov generic quotient/action path
UniversalTargetElimination path
replay_run_certificate actual DAG path
acceptance suite source
FULL_CORE_RED_TEAM_RESULTS.md
FULL_CORE_NONFINITE_RESULTS.md
FULL_CORE_INVARIANT_SCAN_BINDING.md
CLOSURE.md
```

Fail if any final claim exceeds evidence.

Allowed final claim after this repair:

```text
CANDIDATE_COVER_CORE_READY
```

Forbidden unless later exact-image phases pass:

```text
EXACT_IMAGE_CORE_READY
RGDTPK_Q_V4_ACCEPTANCE_COMPLETE
SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC
```


---

## Reviewer Meta-Protocol v2 — Algorithmic sufficiency is mandatory

Every reviewer must apply this protocol before phase-specific checks.

### Required reviewer posture

The reviewer is not checking whether the Agent followed instructions superficially. The reviewer is trying to falsify the claim that the repository implements the v4 R-GDTPK-Q candidate-cover algorithm.

The reviewer must assume that an Agent may accidentally or strategically:

```text
- implement only the easiest algebraic slice;
- move heavy computation into planning;
- keep a limited helper in a production path;
- use a test fixture as an acceptance surrogate;
- rely on previous PASS archives;
- write a limitation honestly but still keep the invalid completion claim;
- pass static scans while missing the actual algorithm.
```

### Mandatory negative tests in every algorithmic phase

The reviewer must ask and answer:

```text
1. What is the broadest algebraic input class that this code actually handles?
2. What v4-required input class does it still fail to handle?
3. Is that failure due to legitimate resource/certificate bounds or due to partial implementation?
4. Does the production path go through the declared v4 pipeline?
5. Would permuting variable ids, relation order, and polynomial syntax change behavior?
6. Does any phase evidence rely on a helper not reachable from api::solve_target or the production pipeline?
7. Does planning compute final output relations?
8. Does replay bind actual DAG/block authorization, not a synthetic superset?
```

If any answer is missing, the review must fail.

### PASS is forbidden when scope is narrowed

The reviewer must fail, not downgrade, if a mandatory v4 algorithm is replaced by a narrowed production implementation. Examples:

```text
- TargetActionKrylov is not generic because it only handles target-only or local-univariate alias structures.
- SparseResultant is not generic because it only handles binary pair chains.
- NormTrace is not v4-complete because it only handles one hard-coded tower shape.
- RegularChain is not v4-complete because it only recognizes triangular examples.
- The orchestrator is not complete because support is hand-built outside DAG/planner/kernel/compose.
```

Honest documentation of these limitations is not sufficient. The phase fails unless the limited path is removed from the completion claim and the final candidate-cover core still satisfies the v4 production pipeline.

---

## FCR-P0A reviewer prompt — Agent failure-mode reset

PASS only if the Agent's `FULL_CORE_AGENT_FAILURE_MODE_RESET.md` is operationally binding:

```text
- it lists the prior failure modes;
- it states concrete stop conditions;
- it requires AlgorithmDefect/PlanDefect rather than scope shrinking;
- later phase handoffs reference whether these triggers fired.
```

Fail if it is merely a motivational note.

---

## FCR-P1A reviewer prompt — Source-spec compliance map

Inspect the supplied v4 specification and the implementation. PASS only if the compliance map covers every v4 file/function/type and correctly separates candidate-cover obligations from exact-image later-phase obligations.

Fail if:

```text
- a required v4 function is missing from the map;
- a function is marked implemented because a similarly named function exists;
- exact-image obligations are forgotten rather than explicitly deferred to P13;
- candidate-cover obligations are deferred without a source-backed reason;
- SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC is claimed before exact-image is complete.
```

---

## Historical FCR-P12 red-team reviewer prompt

Superseded by the FCR-P11 red-team and final nonfinite reviewer prompt above. This historical text
is retained only to prevent ambiguity in older evidence packets.

Do not read previous PASS summaries until after code review. First inspect the current source code and run or specify new algebraic counterexamples.

PASS only if:

```text
- the public or near-public pipeline handles non-fixture algebraic systems beyond the acceptance examples;
- failures are resource/certificate failures with traces, not partial-slice failures;
- no stress case relies on target-univariate-only, alias-only, binary-only, tower-only, or helper-only behavior;
- all mandatory v4 candidate-cover modules are production-reachable and replay-bound;
- exact-image claims remain blocked unless P13 is complete.
```

Fail if the review would have passed the old P12G implementation.
