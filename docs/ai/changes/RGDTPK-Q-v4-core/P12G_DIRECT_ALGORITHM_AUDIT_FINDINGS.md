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
