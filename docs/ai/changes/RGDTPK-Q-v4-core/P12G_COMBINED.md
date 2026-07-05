# GeoSolver P12G Generality Remediation Instructions v1



---

# FILE: README_P12G.md

# P12G Generality Remediation Instruction Pack

Place these files under:

```text
docs/ai/changes/RGDTPK-Q-v4-core/
```

Then give `P12G_QUICK_GUARDIAN_PROMPT.md` to the implementation Agent.

Files:

```text
P12G_DIRECT_ALGORITHM_AUDIT_FINDINGS.md
P12G_BASE_SPEC_AMENDMENT.md
P12G_PLAN.md
P12G_REVIEWER_PROMPTS.md
P12G_ACCEPTANCE_CHECKLIST.yaml
P12G_QUICK_GUARDIAN_PROMPT.md
P12G_PATCH_NOTES.md
```

This is not a replacement for the existing v2.2 Base Spec. It is a mandatory hardening amendment after P12 and before P13/P14.


---

# FILE: P12G_DIRECT_ALGORITHM_AUDIT_FINDINGS.md

# P12G Direct Algorithm Audit Findings — Generality Remediation Before P13/P14

Spec ID: `RGDTPK-Q-v4-core-P12G`  
Status: mandatory corrective audit findings  
Insertion point: after P12, before P13 and before any P14 public-pipeline integration work.

This file records direct implementation findings from the current repository state after the claimed P12 push. It is not a replacement for `BASE_SPEC.md`; it is a remediation trigger. If any finding below is wrong because the code has since changed, the Agent must update this file with exact file/line evidence and rerun the P12G reviewer.

## A. Overall judgment

The current implementation contains real algorithmic work, especially:

- dense target relation search with exact membership verification;
- bounded Universal target/separator elimination;
- message composition over projection messages;
- kernel-certificate payload replay;
- squarefree/root isolation/candidate decode foundations.

However, the implementation is not yet safe to treat as a source-faithful, general R-GDTPK algorithm. The current code still has several places where a phase appears complete because the names, certificates, and tests exist, while the actual production mechanism is narrower than the specification intent.

The following issues block P13/P14 continuation until fixed or explicitly downgraded.

## B. Finding P12G-F1 — TargetActionKrylov is not yet a general quotient/action kernel

Observed code pattern:

```text
geosolver-core/src/kernels/action_krylov.rs
- plan_target_action_krylov_with_messages selects a relation through select_target_action_relation.
- select_target_action_relation calls polynomial_to_univariate(input.polynomial, target).
- polynomial_to_univariate rejects any polynomial containing variables outside {target}.
- build_quotient_handle_input_from_target_relation builds a companion quotient from that already target-only univariate relation.
```

This means the current P8c mechanism is essentially:

```text
if a target-only univariate relation S(T) is already present,
build the companion action matrix for S(T).
```

That is not the R-GDTPK target-action mechanism described in the specification. The intended mechanism is:

```text
from authorized local relations J in variables Y plus exported variables Z,
construct or verify a finite target-relevant quotient/action handle,
materialize multiplication-by-target without coordinate roots/RUR,
compute a verified characteristic support polynomial,
and export a target/separator relation.
```

The current code cannot, by this mechanism alone, solve a block such as:

```text
relations:
  x^2 - 2 = 0
  T - x = 0
exported:
  T
eliminated:
  x
```

unless another kernel has already produced `T^2 - 2`. Therefore P8c must not remain closed as a generic `TargetActionKrylovKernel` unless this is fixed.

## C. Finding P12G-F2 — Planning phases compute outputs too eagerly

Several kernels compute the actual candidate relation during plan construction in order to create hashes/template data:

```text
- SparseResultant plan calls build_sparse_resultant_trace, which computes resultants.
- SpecializationInterpolation plan calls build_specialization_interpolation_trace, which executes inner target-only search and exact local Groebner verification.
- TargetActionKrylov plan builds the quotient handle and characteristic relation.
- RegularChain and NormTrace planning build full projection traces.
```

A deterministic support-producing plan may perform bounded probes and declare templates, schedules, and certificate routes. It must not silently become "execution during planning" in a way that hides algorithmic failure behind admission, duplicates heavy work without cost accounting, or makes the declared ladder decorative.

P12G must either:

1. refactor planning so it declares enough to execute later without constructing final output relations; or
2. explicitly model these as `PreExecutionCertificateProbe` objects with resource/cost traces and prove that execution replays the exact probe, the probe is authorization-bound, and the probe cannot be used as a shortcut to bypass declared execution.

Option 1 is preferred.

## D. Finding P12G-F3 — SparseResultantProjection remains a binary pairwise resultant chain

The current sparse resultant kernel selects pairs of polynomials involving a chosen eliminated variable and calls `ResultantInput { polynomials: vec![left, right], ... }`. This is useful, but it is not a generic sparse resultant projection kernel over arbitrary arity/support templates.

This is acceptable only if:

```text
- the ledger and reviewers continue to treat it as a binary/chain sparse resultant kernel;
- P8b/P15 do not claim generic sparse resultant completion from it alone;
- Universal and TargetRelationSearch remain responsible for broader target/separator relation search.
```

If the Agent wants P8b to close a generic sparse resultant mechanism, it must implement multi-polynomial/multivariate sparse resultant or an explicitly certified eliminant-template route beyond pairwise binary resultants.

## E. Finding P12G-F4 — SpecializationInterpolation exact verification currently falls back to local Groebner inside trace building

The current specialization-interpolation trace construction does the following:

```text
1. specialize separators;
2. run inner target-only TargetRelationSearch;
3. interpolate;
4. run local Groebner elimination to verify the interpolated relation.
```

Local exact verification is allowed when block-local, authorization-bound, target/separator-only, and resource-capped. But it must not become a hidden heavy fallback or a substitute for a declared inner-kernel schedule.

P12G must make the inner schedule explicit:

```text
SpecializationInterpolationPlan:
  separator variables
  sample points
  inner kernel kind(s)
  inner plan hashes
  interpolation support
  final exact verification route
  resource bounds for each phase
```

The verifier must reject sample-only proof and must also reject undeclared local Groebner verification.

## F. Finding P12G-F5 — Run certificate invariants are not truthful enough for final claims

`CoreInvariantFlags` contains no-geometry-dispatch, no-problem-id-dispatch, no-expected-answer-dispatch, and no-QE/CAD flags, but the current derived flags set several of these to `false` by construction.

This may be tolerable before final orchestration, but it is not acceptable for P14/P16 final claims. P12G must make these flags evidence-backed and truthful:

```text
- scan source code for forbidden terms and dispatch paths;
- bind the scan evidence into the run certificate or final closure evidence;
- set flags to true only when evidence proves them;
- require final replay/closure to fail if required invariant flags are false.
```

## G. Finding P12G-F6 — Replay reconstructs synthetic blocks rather than the original TargetProjectionDAG

The current replay path verifies messages using a synthetic block built from the compressed system and message exports. This checks many useful conditions, but it is weaker than replaying the actual TargetProjectionDAG and each block authorization.

P12G must prepare or implement:

```text
CoreRunCertificate:
  target_projection_dag_hash
  per-block authorization hashes
  per-message block id
  per-message plan hash
  per-message child dependencies
  per-message source relation ids/hashes

Replay:
  reconstruct or verify the actual DAG
  replay messages in DAG order
  verify each message only against its authorized block relations plus declared child messages
  fail if a message could only verify by reading unrelated relations from another block
```

Synthetic all-relations replay must not be the final certificate semantics.

## H. Finding P12G-F7 — Candidate-cover finalizer treats "no real roots" as hard-case

Current candidate-cover finalization returns an error when a nonzero support has no real roots. This is not generally correct.

If `S(T)` is nonzero and exact root isolation proves it has zero real roots, then the real target candidate list is empty. In candidate-cover mode this can be a valid `CertifiedCandidateCover` with empty decoded candidates. In exact-image mode it is a strong path toward `CertifiedEmptyRealTargetImage`.

P12G must change this behavior and add tests for support such as:

```text
S(T) = T^2 + 1
```

No real roots must not be reported as `AlgorithmicHardCase`.

## I. Finding P12G-F8 — Nonfinite certification is currently positive but too narrow for a generic claim

Current nonfinite certification searches for a small rational consistency witness after observing that target does not occur in root relations. This is safer than treating "no relation found" as nonfinite, but it is not a general certificate for `I ∩ Q[T] = {0}`.

P12G must explicitly prevent overclaim:

```text
- current bounded rational witness certificate is a limited positive certificate only;
- P15 certified-nonfinite suite may not pass solely because of this helper unless the suite case is exactly within the helper's certified semantics;
- generic nonfinite certification requires dimension/algebraic-dependence/regular-chain/Groebner-dimension evidence as described in the spec.
```

## J. Finding P12G-F9 — CLOSURE.md is stale and contradicts the current phase claim

`CLOSURE.md` still describes P5R as the current closure state, while `ACTIVE_CONTEXT.md` says P12 has PASS archives and the ceiling is `PARTIAL_MECHANISM_READY:MECH-011`.

P12G must update closure and all active navigation docs so they agree on:

```text
current latest phase: P12 + P12G pending
maximum claim before P12G closure: no stronger than PARTIAL_MECHANISM_READY:MECH-011
not complete: P13 exact image, P14 public orchestration, P15 acceptance suites, P16 final closure, performance claim, source-faithful claim, acceptance-complete claim
```

## K. Finding P12G-F10 — General algebraic structure coverage is still not strong enough

Current tests include real mechanisms, but the support-producing stress remains too close to small hand-built cases. Before P14/P15, the implementation must prove that the same generic pipeline handles algebraic structures representative of geometry-derived IR without geometry names:

```text
- no target-only relation initially, support obtained by projection;
- coordinate-signature variables but no role dispatch;
- bilinear determinant/oriented-area-like relations;
- dot/Gram-like quadratic relations;
- guarded rational affine substitutions;
- multi-separator projection and composition;
- finite quotient/action where the target support is not already present;
- algebraic tower/norm trace, including at least one two-step tower or explicitly documented limitation;
- relation order and variable renaming invariance.
```

The Agent must not add problem names, expected answers, or fixture dispatch. These tests must be algebraic structure tests, not geometry problem tests.


---

# FILE: P12G_BASE_SPEC_AMENDMENT.md

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


---

# FILE: P12G_PLAN.md

# P12G Plan — Generality Remediation Before P13/P14

This plan is inserted after P12 and before P13. It is a mandatory remediation phase group. P13 and P14 are blocked until P12G-a through P12G-h have PASS reviews.

The goal is not to add more documentation. The goal is to verify and repair whether the current implementation is a real R-GDTPK general algebraic target-direct algorithm rather than a collection of narrow kernel-shaped slices.

---

## P12G-a — Current HEAD rebind, closure repair, and direct algorithm inventory

**Supports:** P12G-RGQ-073, P12G-RGQ-074, P12G-RGQ-082, P12G-RGQ-085.

### Files to inspect

```text
docs/ai/ACTIVE_CONTEXT.md
docs/ai/changes/RGDTPK-Q-v4-core/CLOSURE.md
docs/ai/changes/RGDTPK-Q-v4-core/PRIMITIVE_SCOPE_LEDGER.md
docs/ai/changes/RGDTPK-Q-v4-core/reviews/P6 through P12
geosolver-core/src/kernels/*
geosolver-core/src/planner/*
geosolver-core/src/compose/*
geosolver-core/src/verify/*
geosolver-core/src/roots/*
geosolver-core/src/solver/*
```

### Required tasks

1. Record current `git rev-parse --verify HEAD`.
2. Rerun:
   ```text
   cargo fmt --manifest-path geosolver-core/Cargo.toml -- --check
   cargo test --manifest-path geosolver-core/Cargo.toml -- --nocapture
   git diff --check
   ```
3. Update `CLOSURE.md` so it no longer describes P5R as the current state.
4. Create `docs/ai/changes/RGDTPK-Q-v4-core/P12G_DIRECT_ALGORITHM_INVENTORY.md`.
5. For each kernel and solver stage, record:
   ```text
   - actual production input class
   - actual production output class
   - whether it can produce target/separator support without pre-existing target-only relation
   - whether it uses local Groebner/resultant/interpolation/action during plan/admission
   - exact certificate route
   - limitation and whether limitation is honest or overclaimed
   ```
6. Mark any previously closed phase claim that depends on an overclaimed kernel as `REQUIRES_P12G_RECHECK`.

### Forbidden shortcuts

```text
- Do not rely on review_summary.yaml alone.
- Do not say "P13/P14 will fix this" unless the Base Spec amendment explicitly allows deferral.
- Do not hide stale CLOSURE wording as historical.
```

### Required evidence

```text
evidence/P12G-a/commands.txt
evidence/P12G-a/command_outputs.txt
evidence/P12G-a/direct_algorithm_inventory.md
evidence/P12G-a/claim_consistency_matrix.yaml
reviews/P12G-a/<timestamp>/{prompt.md,response.md,review_summary.yaml,evidence_manifest.yaml}
```

---

## P12G-b — TargetActionKrylov generalization or honest demotion

**Supports:** P12G-RGQ-075.

### Files to modify

```text
geosolver-core/src/kernels/action_krylov.rs
geosolver-core/src/algebra/quotient.rs
geosolver-core/src/algebra/krylov.rs
geosolver-core/src/verify/verify_message.rs
docs/ai/changes/RGDTPK-Q-v4-core/PRIMITIVE_SCOPE_LEDGER.md
docs/ai/changes/RGDTPK-Q-v4-core/ACTIVE_CONTEXT.md or docs/ai/ACTIVE_CONTEXT.md
docs/ai/changes/RGDTPK-Q-v4-core/CLOSURE.md
```

### Required decision

Choose exactly one route.

### Route A — implement generic finite quotient/action

Implement production TargetActionKrylov for at least the minimum non-target-only quotient case:

```text
x^2 - 2 = 0
T - x = 0
```

Required implementation outline:

1. Build a quotient basis from authorized local relations without computing coordinate roots.
2. For the minimum case, a valid basis is `[1, x]`; more general code must not hard-code variable IDs or expected relation strings.
3. Compute normal forms of `T * basis_i` using authorized relations:
   ```text
   T*1 - x ∈ J
   T*x - 2 ∈ J
   ```
4. Produce action columns with independent membership certificates.
5. Build a production provenanced quotient handle from those certificates.
6. Compute verified characteristic support coverage.
7. Export `T^2 - 2` as target support.
8. Add replay tests that tampering with:
   ```text
   - relation T - x
   - relation x^2 - 2
   - action column certificate
   - quotient authorization hash
   - output relation
   ```
   fails.

Required tests:

```text
p12g_action_krylov_non_target_only_quotient_produces_support
p12g_action_krylov_does_not_use_target_univariate_relation
p12g_action_krylov_rejects_tampered_authorized_relation
p12g_action_krylov_rejects_injected_action_columns
```

### Route B — demote and reopen

If Route A is not implemented:

1. Rename current production path to a non-generic companion path.
2. Ensure `TargetActionKrylovKernel` is not admitted as generic P8c.
3. Update `PRIMITIVE_SCOPE_LEDGER.md`:
   ```text
   Current implementation: target-univariate companion action only.
   Generic TargetActionKrylov: not implemented.
   MECH-014: not closed.
   ```
4. Update active context and closure to reopen P8c/MECH-014.
5. Add a failing/ignored-with-blocker test showing the non-target-only quotient case is not solved by current TargetActionKrylov.

Route B may pass P12G only as a claim downgrade. It does not permit final source-faithful completion.

### Forbidden shortcuts

```text
- Do not pass by adding a target-only relation to the test input.
- Do not call TargetRelationSearch and then feed its output into TargetActionKrylov as proof that ActionKrylov is generic.
- Do not use coordinate roots, coordinate RUR, or expected answer dispatch.
```

---

## P12G-c — Plan/execute separation and declared probe model

**Supports:** P12G-RGQ-076, P12G-RGQ-077.

### Files to inspect/modify

```text
geosolver-core/src/planner/*
geosolver-core/src/kernels/sparse_resultant.rs
geosolver-core/src/kernels/specialization_interpolation.rs
geosolver-core/src/kernels/action_krylov.rs
geosolver-core/src/kernels/regular_chain_projection.rs
geosolver-core/src/kernels/norm_trace_projection.rs
geosolver-core/src/kernels/universal_elimination.rs
geosolver-core/src/result/cost_trace.rs
docs/ai/changes/RGDTPK-Q-v4-core/PRIMITIVE_SCOPE_LEDGER.md
```

### Required tasks

1. Create a `PlanWorkClassification` table for every kernel:
   ```text
   PurePlan
   CertifiedProbePlan
   InvalidHiddenExecution
   ```
2. Any `InvalidHiddenExecution` must be fixed.
3. If a kernel computes a relation during plan:
   - define a concrete `CertifiedProbePlan` object;
   - store probe hash, resource trace, authorization hash, source relation hashes;
   - require execute to replay the exact probe or recompute under the same declared probe;
   - require verifier to reject plan/execute mismatch.
4. Prefer refactoring so plan only declares:
   ```text
   schedules, bounds, template dimensions, sample points, inner kernel plans, resource caps
   ```
   and execute does the algebraic construction.

### Required tests

```text
p12g_plan_does_not_silently_execute_final_relation
p12g_certified_probe_plan_hash_tamper_fails
p12g_specialization_interpolation_inner_schedule_is_declared
p12g_sparse_resultant_template_plan_does_not_overclaim_binary_chain
```

### Forbidden shortcuts

```text
- Do not merely document that planning is heavy.
- Do not hide a final output relation in a plan hash without a typed probe object.
- Do not allow admission false to mean unsupported solver input.
```

---

## P12G-d — Candidate-cover no-real-root semantics

**Supports:** P12G-RGQ-078.

### Files to modify

```text
geosolver-core/src/compose/final_support.rs
geosolver-core/src/roots/isolate.rs
geosolver-core/src/roots/decode.rs
geosolver-core/src/result/output.rs
geosolver-core/tests/p12_roots_decode_integration.rs
```

### Required behavior

For support `S(T)=T^2+1`:

```text
finalize_candidate_cover_result(...)
  status == CertifiedCandidateCover
  support_polynomial == Some(S)
  squarefree_support_polynomial == Some(S)
  root_isolation == []
  decoded_candidates == []
  no AlgorithmicHardCase solely due to zero real roots
```

If exact-image mode later classifies this as empty, that is P13 responsibility.

### Required tests

```text
p12g_candidate_cover_no_real_roots_keeps_support_and_returns_empty_candidates
p12g_no_real_roots_replay_accepts_empty_root_and_candidate_lists_when_support_hashes_match
p12g_nonzero_support_with_no_real_roots_is_not_placeholder
```

---

## P12G-e — Truthful invariant flags and final-claim blockers

**Supports:** P12G-RGQ-079.

### Files to modify

```text
geosolver-core/src/verify/run_certificate.rs
geosolver-core/src/verify/replay.rs
docs/ai/changes/RGDTPK-Q-v4-core/CLOSURE.md
docs/ai/changes/RGDTPK-Q-v4-core/REVIEWER_PROMPTS.md
```

### Required tasks

1. Remove false fixed invariant derivation for final-claim flags.
2. Add an `InvariantEvidence` structure or explicit final-closure evidence hook for:
   ```text
   no_geometry_dispatch
   no_problem_id_dispatch
   no_expected_answer_dispatch
   no_qe_cad
   no_full_coordinate_solution_set
   no_full_coordinate_rur
   exact_q_verification
   no_hidden_fallback
   ```
3. Before P14/P16, replay/closure must fail if required invariant flags are false.
4. If runtime scan evidence is not implemented yet, mark P14/P16 as blocked until it is.

### Required tests

```text
p12g_run_certificate_cannot_claim_final_invariants_with_false_flags
p12g_replay_or_closure_rejects_false_no_geometry_dispatch_for_final_claim
p12g_invariant_flags_are_not_ignored_by_p11_replay_enforced
```

---

## P12G-f — DAG/block authorization replay preparation

**Supports:** P12G-RGQ-080.

### Files to modify

```text
geosolver-core/src/verify/run_certificate.rs
geosolver-core/src/verify/replay.rs
geosolver-core/src/graph/projection_dag.rs
geosolver-core/src/compose/message.rs
docs/ai/changes/RGDTPK-Q-v4-core/PLAN.md
```

### Required tasks

1. Add certificate fields or a typed structure for per-block authorization binding.
2. Add replay support for verifying messages against actual DAG block authorization.
3. Add a regression where a message can verify against synthetic all-relations replay but must fail under actual block authorization.
4. If full DAG replay is deferred to P14, add a P14 blocking TODO with schema-visible status:
   ```text
   P14 cannot close until actual DAG replay replaces synthetic all-relations replay for final claims.
   ```

### Required tests

```text
p12g_replay_rejects_message_using_relation_outside_original_block
p12g_replay_rejects_child_message_not_on_declared_dag_edge
p12g_dag_authorization_hash_bound_into_run_certificate
```

---

## P12G-g — Nonfinite claim tightening

**Supports:** P12G-RGQ-081.

### Files to modify

```text
geosolver-core/src/compose/final_support.rs
geosolver-core/src/result/status.rs
docs/ai/changes/RGDTPK-Q-v4-core/PRIMITIVE_SCOPE_LEDGER.md
docs/ai/changes/RGDTPK-Q-v4-core/PLAN.md
```

### Required tasks

1. Rename or document current bounded rational witness certificate as limited.
2. Add explicit certificate enum/field that identifies proof kind:
   ```text
   BoundedRationalWitness
   DimensionCertificate
   AlgebraicDependenceCertificate
   RegularChainDimensionCertificate
   GroebnerEliminationZeroCertificate
   ```
3. Only proof kinds with sufficient semantics may satisfy P15 generic nonfinite cases.
4. Add tests that failure to find a bounded witness returns `AlgorithmicHardCase` or `CertificateDesignGap`, not `CertifiedNonFiniteTargetImage`.

### Required tests

```text
p12g_nonfinite_no_relation_search_failure_is_not_nonfinite
p12g_nonfinite_bounded_witness_failure_is_not_nonfinite
p12g_nonfinite_certificate_declares_proof_kind
```

---

## P12G-h — General algebraic stress battery and P13/P14 readiness

**Supports:** P12G-RGQ-074, P12G-RGQ-083, P12G-RGQ-084, P12G-RGQ-085.

### Files to create/modify

```text
geosolver-core/tests/p12g_generality_stress.rs
docs/ai/changes/RGDTPK-Q-v4-core/P13_P14_READINESS_AFTER_P12G.md
docs/ai/changes/RGDTPK-Q-v4-core/CLOSURE.md
docs/ai/ACTIVE_CONTEXT.md
```

### Required stress cases

Implement all cases from `P12G-RGQ-083`:

```text
G1 projection without initial target-only relation
G2 finite quotient/action without initial target-only relation
G3 bilinear determinant-like structure
G4 dot/Gram-like quadratic structure
G5 guarded rational affine preprocessing
G6 multi-separator projection and composition
G7 algebraic tower/norm trace
G8 non-real support
```

Each support-producing case must prove:

```text
- no geometry names;
- no expected-answer dispatch;
- production code receives only algebraic polynomials;
- output support is nonzero;
- support is exact-verified;
- squarefree/root/decode behavior is correct;
- relation-order and variable-renaming variants still pass where applicable;
- failures, if any, are treated as blockers unless the case is explicitly tied to a downgraded claim.
```

### Required readiness file

`P13_P14_READINESS_AFTER_P12G.md` must answer:

```text
1. Is P8c generic TargetActionKrylov genuinely implemented? If not, which claim was reopened?
2. Are plan-time computations classified and safe?
3. Does candidate-cover handle non-real support correctly?
4. Are run certificate invariant flags truthful or still blocking final claims?
5. Is actual DAG replay implemented or explicitly blocking P14/P16?
6. Are nonfinite claims limited to positive proof kinds?
7. Did all P12G generality stress cases pass without geometry/problem/expected-answer dispatch?
8. May P13 begin?
9. May P14 begin?
```

### Forbidden shortcuts

```text
- Do not mark a stress case as hard-case and still pass P12G.
- Do not add target-only relations to make a non-target-only case pass.
- Do not use helper functions that bypass planner/kernel/compose where the case is meant to test them.
- Do not claim P14 readiness if solve_with_context is still temporary without an explicit P14 plan blocker.
```


---

# FILE: P12G_REVIEWER_PROMPTS.md

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


---

# FILE: P12G_ACCEPTANCE_CHECKLIST.yaml

schema_version: rgdtpk-p12g-acceptance-v1
phase_group: P12G
must_close_before:
- P13
- P14
- P15
- P16
claim_ceiling_before_closure: PARTIAL_MECHANISM_READY:MECH-011
required_subphases:
- id: P12G-a
  name: HEAD rebind, closure repair, direct algorithm inventory
  must_pass: true
- id: P12G-b
  name: TargetActionKrylov generic implementation or honest demotion
  must_pass: true
- id: P12G-c
  name: Plan/execute separation or CertifiedProbePlan model
  must_pass: true
- id: P12G-d
  name: No-real-root candidate-cover semantics
  must_pass: true
- id: P12G-e
  name: Truthful invariant flags or hard P14/P16 blocker
  must_pass: true
- id: P12G-f
  name: Actual DAG replay implementation or hard P14/P16 blocker
  must_pass: true
- id: P12G-g
  name: Positive nonfinite proof discipline
  must_pass: true
- id: P12G-h
  name: General algebraic stress battery and P13/P14 readiness
  must_pass: true
mandatory_general_stress_cases:
  G1_projection_without_initial_target_only_relation: must produce support by target-direct
    projection
  G2_non_target_only_quotient_action: must pass via TargetActionKrylov if P8c remains
    closed
  G3_bilinear_determinant_like: must pass with variable/relation permutations
  G4_dot_gram_quadratic: must pass without geometry names
  G5_guarded_rational_affine: must preserve guard and produce support
  G6_multi_separator_composition: must fail on child-message deletion and pass when
    intact
  G7_algebraic_tower_norm_trace: single tower required; two-step tower required if
    claimed
  G8_non_real_support: must not be hard-case solely because no real roots exist
forbidden_pass_conditions:
- review_summary PASS with blockers
- documentation-only fix where code path remains unsafe
- target-only shortcut for non-target-only tests
- geometry/problem/expected-answer dispatch
- CertifiedNonFiniteTargetImage without positive proof
- P8c generic claim when TargetActionKrylov only handles target-univariate companion
  relation
- P14/P16 readiness while invariant flags or DAG replay are unresolved without blockers
required_evidence_per_subphase:
- commands.txt
- command_outputs.txt
- static_scans.txt
- function_implementation_table.yaml
- prompt.md
- response.md
- review_summary.yaml
- evidence_manifest.yaml


---

# FILE: P12G_QUICK_GUARDIAN_PROMPT.md

# P12G Quick Guardian Prompt

You are implementing the mandatory P12G remediation in `kabatan/bakaGeometry`.

Read first:

```text
docs/ai/changes/RGDTPK-Q-v4-core/BASE_SPEC.md
docs/ai/changes/RGDTPK-Q-v4-core/P5R_BASE_SPEC_AMENDMENT.md
docs/ai/changes/RGDTPK-Q-v4-core/P12G_DIRECT_ALGORITHM_AUDIT_FINDINGS.md
docs/ai/changes/RGDTPK-Q-v4-core/P12G_BASE_SPEC_AMENDMENT.md
docs/ai/changes/RGDTPK-Q-v4-core/P12G_PLAN.md
docs/ai/changes/RGDTPK-Q-v4-core/P12G_REVIEWER_PROMPTS.md
docs/ai/changes/RGDTPK-Q-v4-core/P12G_ACCEPTANCE_CHECKLIST.yaml
```

Do not begin P13 or P14. P12G is a mandatory barrier.

Your goal is to determine and repair whether the current P6–P12 implementation is truly a general R-GDTPK target-direct algebraic algorithm, not merely a set of narrow kernels with good evidence.

Implement P12G-a through P12G-h in order.

The most important blockers are:

```text
1. TargetActionKrylov currently appears to require an already target-only univariate relation.
   Either implement generic quotient/action construction from authorized local relations, or reopen/demote P8c/MECH-014.

2. Several kernels appear to compute final output relations during planning.
   Refactor to pure plans or introduce typed CertifiedProbePlan objects with evidence and replay.

3. Candidate-cover finalizer must not treat nonzero support with zero real roots as AlgorithmicHardCase.

4. CoreRunCertificate invariant flags must become truthful/evidence-bound or hard-block P14/P16.

5. Replay must eventually use actual TargetProjectionDAG/block authorization, not synthetic all-relations replay.

6. Add the P12G general algebraic stress battery G1–G8.
```

Reviewer PASS alone is not enough. Each subphase requires code/evidence/review archive. P12G-h must produce `P13_P14_READINESS_AFTER_P12G.md`.

Do not claim:

```text
CANDIDATE_COVER_CORE_READY
EXACT_IMAGE_CORE_READY
RGDTPK_Q_V4_ACCEPTANCE_COMPLETE
SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC
```

P12G can only preserve or downgrade the current partial mechanism claim. If generic TargetActionKrylov is not fixed, reopen the relevant claim instead of hiding it.


---

# FILE: P12G_PATCH_NOTES.md

# P12G Patch Notes

This pack was created after direct inspection of the P6–P12 implementation.

The key conclusion is:

```text
P6–P12 contain real mechanisms, but the current implementation is not yet safe to treat as a complete general R-GDTPK algorithm.
```

The most important direct finding is that `TargetActionKrylovKernel` currently behaves like a companion-matrix wrapper around an already target-only univariate relation. That is not the intended generic quotient/action kernel.

The second most important finding is that planning and execution are blurred in several kernels. This must be resolved before public orchestration and final claims.

P12G is intentionally inserted before P13/P14 so that exact-image and public-pipeline work does not build on overclaimed P8/P11/P12 assumptions.
