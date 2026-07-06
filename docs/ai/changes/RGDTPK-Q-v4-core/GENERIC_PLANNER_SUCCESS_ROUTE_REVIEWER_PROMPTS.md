# Generic Planner Success-Route Reviewer Prompts v1

## Reviewer meta-protocol

For every phase, reviewer must answer:

```text
1. Did the patch avoid adding the investigated geometry problem as a test/gate/fixture?
2. Does the patch use only algebraic footprint information?
3. Does it preserve the single unified candidate-cover pipeline?
4. Does it prevent dense TargetRelationSearch from materializing huge supports in admission?
5. Does it route toward feasible declared kernels instead of fast failure?
6. Does it avoid hidden fallback, coordinate solving, expected-answer dispatch, and geometry-name dispatch?
```

A PASS is forbidden if any answer is missing or unsupported by code evidence.

## GPSR-RP0: Agent reset review

PASS only if `GENERIC_PLANNER_AGENT_RESET.md` explicitly rejects:

```text
- case-specific optimization;
- adding the investigated problem as a regression;
- fast failure as success;
- separate solvers;
- hidden fallback;
- dense route blocking the whole planner.
```

FAIL if the reset language implies that returning `FiniteResourceFailure` for hard large blocks is an acceptable closure for this repair.

## GPSR-RP1: Materialization audit review

Reviewer must inspect the listed files and verify the audit includes every planning/admission site that can allocate monomial supports or matrices.

Mandatory challenge:

```text
Point to the exact code that previously called dense schedule construction during TargetRelationSearch admission.
Point to the exact row in the audit requiring it to be preflighted or descriptorized.
```

FAIL if any relevant site is omitted.

## GPSR-RP2: Preflight review

PASS only if closed-form preflight computes estimates without enumerating monomials.

Reviewer must check:

```text
- combinatorial count uses saturating arithmetic;
- overflow cannot panic;
- default caps exist;
- explicit options cannot mean unbounded admission allocation;
- estimated rows/cols are hash-bound or traceable.
```

FAIL if `Vec<Monomial>` is allocated inside preflight.

## GPSR-RP3: Descriptor-first schedule review

PASS only if plan hashes can be deterministic without full support materialization.

FAIL if dense support lists are built during admission just to compute support hashes.

Reviewer must run or inspect a synthetic high-dimensional preflight test and confirm no materialized support count is proportional to `C(n+d,d)`.

## GPSR-RP4: TargetRelationSearch admission review

PASS only if dense infeasibility declines/cost-prohibits only the dense route, not the whole solve.

Reviewer must inspect `admission.rs` and confirm:

```text
build_dense_relation_search_schedule is not called before preflight passes;
TargetRelationSearch returns quickly on high-dimensional dense-infeasible footprint;
other kernels are still eligible after it declines.
```

FAIL if admission still materializes dense supports before preflight.

## GPSR-RP5: Admission isolation review

PASS only if one kernel's infeasibility cannot stop later kernels from being collected.

Mandatory code challenge:

```text
Show the loop over all kernel kinds.
Show how a cost-prohibited/declined TargetRelationSearch does not break or return early.
Show UniversalTargetElimination is still considered for relation-bearing blocks.
```

FAIL if the implementation silently skips later kernels after a dense route problem.

## GPSR-RP6: Declared ladder / cost policy review

PASS only if the ladder contains feasible declared target-direct plans and excludes cost-prohibited dense routes.

Reviewer must fail if the code:

```text
- ranks infeasible dense TRS above feasible compact routes;
- leaves the ladder empty for a well-formed relation-bearing block;
- uses geometry names, problem IDs, or expected answers in ranking.
```

## GPSR-RP7: Universal safeguard review

PASS only if Universal uses the same dense preflight and does not blindly call TargetRelationSearch escalation.

Mandatory challenge:

```text
Show what Universal does when dense escalation is infeasible.
Show that it tries other declared local target/separator strategies.
Show that it does not convert dense exhaustion to CertifiedNonFiniteTargetImage.
```

## GPSR-RP8: Generic stress review

PASS only if tests are generic algebraic footprint tests, not the investigated problem.

Reviewer must scan tests/docs for forbidden names and exact fixture imports.

Support-producing tests must prove:

```text
- status = CertifiedCandidateCover;
- support is verified by exact Q certificate;
- replay accepts;
- dense TRS infeasibility did not cause solve failure;
- produced route is through declared kernel ladder.
```

FAIL if tests only check “planning returns quickly” without at least two generic support-producing successes.

## GPSR-RP9: Trace review

PASS only if cost-prohibited dense route decisions are observable in diagnostics or cost trace and do not become solve-level failures when other routes succeed.

FAIL if route decisions disappear from evidence.

## GPSR-RP10: Closure review

PASS only if closure claims are limited to planner success-route readiness and dense admission safety, unless the full candidate-cover suite is rerun.

FAIL if closure claims:

```text
- benchmark superiority;
- exact-image completion;
- full v4 source-fidelity;
- success on the investigated problem as a gated result.
```
