# P12G Base Spec Amendment — Generality Remediation Before P13/P14

Spec ID: `RGDTPK-Q-v4-core-P12G`  
Parent: `docs/ai/changes/RGDTPK-Q-v4-core/BASE_SPEC.md` v2.2 plus `P5R_BASE_SPEC_AMENDMENT.md`  
Status: Mandatory hardening amendment  
Insertion point: after P12, before P13 exact-image and before P14 public orchestration  
Scope: direct algorithmic generality audit and remediation for the P6–P12 implementation.

P12G exists because P6–P12 can pass local phase reviews while still failing the central requirement of R-GDTPK:

```text
all well-formed rational polynomial target systems enter the same generic target-direct pipeline,
and support-producing success comes from algebraic projection mechanisms,
not from narrow patterns, hidden coordinate solving, or documentation/review evidence.
```

P12G must be applied before P13/P14 begins.

---

## P12G-RGQ-073 — P12G is a mandatory generality barrier

P13 and P14 must not begin until P12G-a through P12G-h are closed.

P12G closure requires:

```text
- code changes where the unsafe path is in code;
- active documentation updates;
- fresh current-HEAD command evidence;
- direct algorithmic generality tests;
- reviewer prompt/response archives;
- schema-valid review_summary.yaml and evidence_manifest.yaml;
- an explicit P13/P14 readiness statement.
```

P12G cannot be satisfied by saying that P13/P14 will handle the issue later, except where this amendment explicitly allows a claim downgrade.

---

## P12G-RGQ-074 — Generality means algebraic generality, not more named slices

The implementation must be judged by algebraic structure, not by file names, phase labels, evidence folders, or a small set of toy examples.

Forbidden completion evidence:

```text
- target-only relation already present, therefore candidate cover works;
- unary tower only, therefore NormTrace is generic;
- binary pairwise resultant only, therefore SparseResultantProjection is generic;
- target-univariate companion matrix only, therefore TargetActionKrylov is generic;
- local Groebner verification used everywhere, therefore target-direct projection is generic;
- static scans and review PASS, therefore the algorithm is source-faithful.
```

Required generality evidence must include structure-renamed, variable-renamed, relation-order-permuted algebraic tests with no geometry names or expected-answer dispatch.

---

## P12G-RGQ-075 — TargetActionKrylov must be generic or the P8c/MECH-014 claim must be reopened

`TargetActionKrylovKernel` must not be considered closed if its production path only accepts an already target-only univariate relation.

One of the following must be true before P12G closes.

### Option A — implement generic production target-action quotient construction

Production TargetActionKrylov must accept authorized local relations where no target-only relation is initially present, build or verify a finite target-relevant quotient/action handle from those relations, and output a target support polynomial by verified characteristic support coverage.

Minimum required production case:

```text
variables: T, x
relations:
  x^2 - 2 = 0
  T - x = 0
exported variables:
  T
eliminated variables:
  x

Expected behavior:
  TargetActionKrylovKernel can produce a certified support equivalent to T^2 - 2
  without using TargetRelationSearch, TargetUnivariate, SparseResultant, Universal local Groebner, full coordinate roots, or full coordinate RUR.
```

Required construction properties:

```text
- quotient basis is built or verified from authorized local relations;
- normal-form basis certificate is independent of injected action columns;
- multiplication-by-target columns are exact membership-certified against authorized relations;
- characteristic polynomial is computed exactly;
- Cayley-Hamilton or equivalent matrix certificate is exact;
- output relation contains only exported variables;
- no coordinate roots or full coordinate RUR are exported;
- failure maps to AlgorithmicHardCase / FiniteResourceFailure / CertificateDesignGap, never Unsupported or CertifiedNonFiniteTargetImage.
```

### Option B — reopen and demote P8c/MECH-014

If Option A is not implemented now, then the Agent must:

```text
- rename the current path as TargetUnivariateCompanionAction or equivalent;
- remove the claim that P8c closes generic TargetActionKrylov;
- update ACTIVE_CONTEXT.md, CLOSURE.md, PRIMITIVE_SCOPE_LEDGER.md, and reviews to say MECH-014 is not closed;
- block P15/P16 candidate-cover/core-complete claims until generic TargetActionKrylov is implemented or explicitly excluded from the final claim with user approval.
```

Option B is allowed only as an honest downgrade. It is not a way to continue claiming the current algorithm is source-faithful.

---

## P12G-RGQ-076 — Planning must not hide execution

Kernel planning must not be a hidden execution phase.

For every P6–P12 kernel, the Agent must classify the plan-time work as one of:

```text
- PurePlan: computes schedules, bounds, supports, hashes, resource caps, but no final output relation.
- CertifiedProbePlan: computes candidate algebraic objects during planning, with explicit cost trace, authorization binding, and proof that execute replays exactly the same certified probe.
- InvalidHiddenExecution: computes final output relation during plan/admission without modeling it as execution or certified probe.
```

`InvalidHiddenExecution` is forbidden.

Preferred correction:

```text
Plan stores schedules/templates/bounds/sample points/inner kernel plan ids.
Execute computes candidate relations and certificates.
Replay verifies execute outputs.
```

If `CertifiedProbePlan` is used, the probe object must be named and stored explicitly in the plan/certificate language. It must not be an invisible call to resultant/interpolation/Groebner/action construction during admission.

---

## P12G-RGQ-077 — SparseResultant and SpecializationInterpolation must not be overclaimed

Current binary resultant chains and specialization-interpolation helpers may remain, but they cannot close generic sparse/resultant/interpolation capability unless the following is true.

SparseResultant generic closure requires:

```text
- declared support/template construction for the intended arity;
- exact reconstruction route;
- exact Q verification;
- failure semantics that never become Unsupported;
- tests where the relation is not target-only initially and cannot be solved by the target-univariate path.
```

SpecializationInterpolation generic closure requires:

```text
- declared separator variables and sample schedule;
- declared inner kernel plan(s);
- declared interpolation support;
- sample agreement is not proof;
- exact Q verification is mandatory;
- local Groebner verification, if used, is declared, resource-capped, and certificate-bound.
```

If only the current narrower variants are implemented, the ledger must say so and P15 must not use them as generic proof.

---

## P12G-RGQ-078 — Candidate-cover with no real roots is valid and must not be hard-case

For a nonzero support polynomial `S(T)`, exact real root isolation may return zero real roots. This is not an algorithmic hard case.

Required behavior:

```text
candidate-cover mode:
  return CertifiedCandidateCover with nonzero support,
  squarefree support,
  empty real-root list,
  empty decoded-candidate list,
  and a diagnostic saying the real candidate cover is empty.

exact-image mode:
  may return CertifiedEmptyRealTargetImage if exact-image semantics also confirm no real target values,
  or may route to the P13 classifier.
```

Forbidden behavior:

```text
- treating S(T)=T^2+1 as AlgorithmicHardCase solely because it has no real roots;
- returning placeholder candidates;
- dropping the support polynomial because there are no real roots.
```

---

## P12G-RGQ-079 — Run certificate invariants must be truthful and evidence-bound

`CoreInvariantFlags` must not contain false fixed values for invariants required by final claims.

Before P12G closes, either:

```text
A. implement evidence-backed invariant derivation for:
   no_geometry_dispatch,
   no_problem_id_dispatch,
   no_expected_answer_dispatch,
   no_qe_cad,
   no_full_coordinate_solution_set,
   no_full_coordinate_rur,
   exact_q_verification,
   no_hidden_fallback;
```

or:

```text
B. keep final strong claims blocked and update docs so P11/P12 replay is explicitly not final invariant proof.
```

P14/P16 may not pass unless option A is fully implemented.

---

## P12G-RGQ-080 — Replay must verify real DAG/block authorization before final claims

Synthetic all-relations replay is not sufficient for final source-faithful claims.

P12G must prepare the run certificate and replay API for real DAG replay:

```text
CoreRunCertificate must carry or bind:
  target_projection_dag_hash
  per-block authorization hashes
  per-message block id
  per-message plan hash
  per-message source relation ids/hashes
  per-message child dependency hashes
```

Replay must eventually verify messages against:

```text
authorized relations of that block
plus declared child messages
not arbitrary relations from the compressed system.
```

If full DAG replay is deferred to P14, P12G must add explicit blockers and tests proving P14 cannot close without it.

---

## P12G-RGQ-081 — Nonfinite certification must be positive and not overclaimed

The bounded rational witness helper may remain only as a limited positive certificate.

Generic certified nonfinite target image requires one of:

```text
- exact dimension certificate;
- algebraic-dependence certificate;
- regular-chain dimension/projection certificate;
- Groebner/elimination certificate proving I ∩ Q[T] = {0};
- another explicitly specified positive nonfinite proof.
```

The following must not route to `CertifiedNonFiniteTargetImage`:

```text
- no relation found;
- relation search bounds exhausted;
- sparse/resultant/interpolation failure;
- Universal local stage failure;
- composition failed to produce target-only support;
- bounded rational witness search failed.
```

---

## P12G-RGQ-082 — Active closure documents must match the current implementation

`CLOSURE.md`, `ACTIVE_CONTEXT.md`, `P6_READINESS.md` if still active, and all new P12G notes must agree.

Before P12G closes, they must state:

```text
current latest phase: P12 plus P12G remediation
maximum claim: no stronger than PARTIAL_MECHANISM_READY:MECH-011
not complete:
  P13 exact-image semantics
  P14 public orchestration
  P15 acceptance suites
  P16 final closure
  performance claim
  SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC
  RGDTPK_Q_V4_ACCEPTANCE_COMPLETE
```

If P12G chooses `P12G-RGQ-075 Option B`, they must also state that generic TargetActionKrylov / MECH-014 is reopened.

---

## P12G-RGQ-083 — General algebraic stress battery is required before P13/P14

P12G must add a direct algorithmic stress battery. These are not final P15 acceptance suites, but they are required to prevent P13/P14 from building on a narrow solver.

At minimum, all support-producing cases below must pass at the direct module/pipeline-fragment level. They must not use geometry names, problem IDs, expected answers, or fixture-specific dispatch.

### Case G1 — projection without initial target-only relation

```text
relations:
  x - y = 0
  y - T = 0
  x^2 - 2 = 0
target:
  T
expected support:
  T^2 - 2
```

Allowed producing kernels: TargetRelationSearch, Universal, composition.  
Forbidden producing path: pre-existing target-only relation.

### Case G2 — finite quotient/action without initial target-only relation

```text
relations:
  x^2 - 2 = 0
  T - x = 0
target:
  T
expected support:
  T^2 - 2
```

Required producing kernel if P8c remains closed: TargetActionKrylov.  
If TargetActionKrylov is demoted, this case must be marked as blocking generic P8c closure.

### Case G3 — bilinear / determinant-like algebraic structure

Use variable names with no geometry meaning.

```text
relations:
  a*u - b*v - T = 0
  a - 1 = 0
  b - 2 = 0
  u - 3 = 0
  v - 5 = 0
target:
  T
expected support:
  T + 7
```

The test must be relation-order and variable-id permuted.

### Case G4 — dot/Gram-like quadratic structure

```text
relations:
  p^2 + q^2 - r = 0
  p - T = 0
  q - 1 = 0
  r - 5 = 0
target:
  T
expected support:
  T^2 - 4
```

### Case G5 — guarded rational affine preprocessing

```text
relations:
  (x + 1) * s - 1 = 0
  (x + 1) * y - (T + x) = 0
  y^2 - 2 = 0
  x - 1 = 0
target:
  T
expected support:
  (T + 1)^2 - 8
```

The test must prove the guard is recorded and no polynomial-quotient-only narrowing occurs.

### Case G6 — multi-separator projection and composition

A child message must export a separator relation and a root message must compose it into a target-only support. Removing the child message must fail support verification or change the support.

### Case G7 — algebraic tower/norm trace

```text
relations:
  a^2 - 2 = 0
  T - a = 0
target:
  T
expected support:
  T^2 - 2
```

If two-step towers are claimed, also include:

```text
a^2 - 2 = 0
b^2 - a = 0
T - b = 0
expected support:
  T^4 - 2
```

### Case G8 — non-real support

```text
support:
  T^2 + 1
expected:
  nonzero support retained,
  zero real roots,
  zero decoded candidates,
  no AlgorithmicHardCase solely from no real roots.
```

---

## P12G-RGQ-084 — Public pipeline remains blocked until P14

P12G must not implement a half-connected public solver and call it complete.

Before P14, `solve_target` may still return the documented temporary failure, but active docs must say:

```text
public solve_target candidate-cover path is not connected until P14.
```

When P14 begins, it must connect the complete pipeline in the exact order from the base spec and cannot skip any stage by directly calling finalizers or hand-built messages.

---

## P12G-RGQ-085 — P12G reviewer must judge the actual algorithm, not the evidence packet

The P12G reviewer must inspect code and must fail if any of the following remains true:

```text
- TargetActionKrylov is only a target-univariate companion path but P8c is still claimed as generic.
- Planning computes final relations invisibly and execution just recomputes them without a declared probe/proof model.
- Candidate-cover no-real-root support is treated as hard-case.
- Run certificate invariant flags are false fixed values while final-claim paths rely on them.
- Replay relies only on synthetic all-relations blocks for final claims.
- CLOSURE.md contradicts ACTIVE_CONTEXT.md.
- General algebraic stress cases are missing, named, or fixture-dispatched.
- Any support-producing stress case routes to Unsupported, CertifiedNonFiniteTargetImage, or an unrelated hard-case.
```
