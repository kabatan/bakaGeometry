# P12G Reviewer Prompts — Direct Algorithm Generality Review

Use these prompts for Guardian review archives under:

```text
docs/ai/changes/RGDTPK-Q-v4-core/reviews/P12G-*/<timestamp>/
```

The reviewer must inspect code and tests. A PASS based only on evidence formatting is invalid.

---

## General P12G reviewer rule

Return exactly one of:

```text
PASS
FAIL_FIXABLE
FAIL_BLOCKING
USER_DECISION_REQUIRED
```

A PASS is invalid if:

```text
- the reviewed issue is only documented but unsafe code remains;
- the implementation passes by target-only, univariate, or fixture-shaped shortcuts;
- a support-producing generality stress case is routed to AlgorithmicHardCase, CertificateDesignGap, CertifiedNonFiniteTargetImage, or any non-spec escape;
- review_summary.yaml says PASS while response.md contains blockers;
- current HEAD is not bound to evidence;
- the reviewer did not inspect actual code.
```

Every review must include:

```text
- files inspected;
- exact code paths used by production;
- tests run;
- whether the behavior is general algebraic behavior or a narrow pattern;
- whether the claim ceiling must be downgraded.
```

---

## P12G-a Reviewer — Inventory and closure consistency

Check:

```text
- CLOSURE.md no longer says P5R is the current implementation state.
- ACTIVE_CONTEXT.md, CLOSURE.md, and P12G inventory agree on claim ceiling.
- Inventory directly describes every P6–P12 kernel's real production input/output class.
- Inventory identifies target-univariate-only, binary-only, tower-only, and probe-execution paths honestly.
```

Fail if CLOSURE remains stale or if the inventory is merely a list of files/functions.

---

## P12G-b Reviewer — TargetActionKrylov generality

Check Route A or Route B.

### If Route A

Inspect `action_krylov.rs`, `quotient.rs`, `krylov.rs`, and verification code.

Required PASS condition:

```text
TargetActionKrylov production path solves the non-target-only quotient case:
  x^2 - 2 = 0
  T - x = 0
without pre-existing target-only relation and without calling TargetRelationSearch/Universal as the producing path.
```

Fail if it still selects only `poly_variables(poly) ⊆ {T}` relations.

### If Route B

PASS only if P8c/MECH-014 is explicitly reopened/demoted in all active docs and final claims are blocked accordingly.

Fail if generic TargetActionKrylov is still claimed.

---

## P12G-c Reviewer — Plan/execute separation

For every kernel, classify plan-time work.

Fail if:

```text
- admission or plan computes the final relation invisibly;
- a plan hash hides an output relation without a typed probe object;
- SpecializationInterpolation uses undeclared local Groebner verification;
- SparseResultant/Specialization/Action/RegularChain/NormTrace planning performs execution but no CertifiedProbePlan exists.
```

PASS requires either PurePlan refactor or explicit CertifiedProbePlan with tamper tests.

---

## P12G-d Reviewer — No-real-root candidate cover

Check `finalize_candidate_cover_result` and integration tests.

PASS requires `T^2+1` to produce:

```text
CertifiedCandidateCover
Some(nonzero support)
Some(squarefree support)
root_isolation == []
decoded_candidates == []
```

Fail if no-real-root support is hard-case, dropped, or treated as placeholder.

---

## P12G-e Reviewer — Truthful invariant flags

Inspect `run_certificate.rs`, `replay.rs`, and closure docs.

Fail if:

```text
no_geometry_dispatch,
no_problem_id_dispatch,
no_expected_answer_dispatch,
no_qe_cad
```

remain hard-coded false while final-claim paths can ignore them.

PASS requires either implemented evidence-backed truth or explicit P14/P16 blocker that cannot be bypassed.

---

## P12G-f Reviewer — DAG replay

Inspect replay and run-certificate code.

Fail if final replay still verifies messages using synthetic all-relations blocks without an actual DAG authorization blocker.

PASS requires either:

```text
- actual DAG/block replay implemented and tested; or
- P14/P16 explicitly blocked until it is implemented, with a failing regression demonstrating the gap.
```

---

## P12G-g Reviewer — Nonfinite proof discipline

Inspect `compose/final_support.rs`.

Fail if any of the following can produce `CertifiedNonFiniteTargetImage`:

```text
no relation found
TargetRelationSearch exhaustion
Universal stage failure
composition failure
bounded rational witness failure
```

PASS requires proof-kind tagging and tests.

---

## P12G-h Reviewer — General algebraic stress battery

Run and inspect `geosolver-core/tests/p12g_generality_stress.rs`.

PASS requires all required G1–G8 cases.

Fail if:

```text
- tests use geometry names or problem IDs;
- production code sees expected answers;
- a non-target-only case is made target-only by adding a helper relation;
- relation/variable permutation variants are missing for G3/G4/G5 where applicable;
- TargetActionKrylov G2 passes through another producing kernel while P8c is still claimed generic.
```

The reviewer must explicitly answer whether the current implementation is a general algebraic target-direct algorithm, a collection of narrow kernels, or a mixed state with downgraded claims.
