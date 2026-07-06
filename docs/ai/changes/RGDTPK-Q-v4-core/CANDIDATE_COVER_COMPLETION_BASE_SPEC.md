# Candidate-Cover Source-Faithful Completion Base Spec v1
Change ID: `RGDTPK-Q-v4-candidate-cover-source-faithful-v1`

## 0. Purpose

This amendment replaces the over-broad `SOURCE_FAITHFUL_COMPLETION_*` repair pack for the current implementation goal.

The current implementation target is the algebraic candidate-cover layer:

```text
Given a well-formed Q-polynomial target system F ⊂ Q[x1,...,xn,T],
produce a nonzero S(T) ∈ Q[T] such that every true target value is a root of S.
```

This layer is allowed to return extra target candidates.

```text
Required:
  true target values ⊆ roots(S)

Not required for candidate-cover:
  returned candidates = true target values
```

The implementation must not fail or be considered incomplete merely because spurious roots remain.

Exact-image filtering, mixed real fiber classification, and removal of spurious roots are not blockers for this repair. They remain the responsibility of `CertifiedExactTargetImage` / exact-image mode.

## 1. Source authority

The supplied v4 specification remains the source of truth. This amendment interprets it according to its own two-stage distinction:

```text
CertifiedCandidateCover:
    S(T) を返す。
    真の target 値は roots(S) に含まれる。
    spurious roots は含まれてよい.

CertifiedExactTargetImage:
    roots(S) の各実根について real fiber / guard / slack semantics を判定し、
    実際に実現可能な target 値だけを返す.
```

For this repair, the mandatory completion target is `CertifiedCandidateCover`.

## 2. Final allowed claim after this repair

If and only if every phase in `CANDIDATE_COVER_COMPLETION_PLAN.md` passes, the repository may claim:

```text
CANDIDATE_COVER_CORE_READY
```

It may also claim:

```text
SOURCE_FAITHFUL_TO_V4_CANDIDATE_COVER_LAYER
```

only if the candidate-cover obligations from the v4 spec are implemented source-faithfully.

It must not claim the following merely from this repair:

```text
CERTIFIED_EXACT_TARGET_IMAGE_COMPLETE
EXACT_IMAGE_CORE_READY
SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC
RGDTPK_Q_V4_ACCEPTANCE_COMPLETE
```

These stronger labels require the exact-image layer, including real fiber / guard / slack semantics, to be implemented source-faithfully.

## 3. Non-negotiable candidate-cover requirements

### CC-001: Generic algebraic input

Every well-formed `RationalTargetProblem` over Q-polynomials must enter the generic projection pipeline.

The solver must not reject an input because it is not one of the test shapes, not target-univariate, not affine, not bivariate, not a known geometric family, or not covered by a named specialized kernel.

Allowed non-success statuses:

```text
FiniteResourceFailure
AlgorithmicHardCase
CertificateDesignGap
ImplementationBug
InvalidInput
```

Forbidden production status or behavior:

```text
Unsupported
NotImplemented
Skipped
OutOfSlice
PatternNotHandled
```

### CC-002: Candidate-cover semantics

For a support-producing success:

```text
status = CertifiedCandidateCover
support_polynomial = Some(S)
S != 0
S ∈ Q[T]
true target values ⊆ roots(S)
```

The solver must not attempt to prove `roots(S) ⊆ true target values` in candidate-cover mode.

Spurious roots are allowed and must not be treated as a correctness failure.

### CC-003: Exact verification of containment

Success must be justified by exact Q algebra, not by numeric sampling, expected answers, geometry names, or fixture-specific dispatch.

At least one of the following exact proof routes must verify the support:

```text
1. Target-only root relation route:
   S is the squarefree product / lcm of verified target-only root relations in the composed projection.

2. Composed-ideal membership route:
   S(T) is converted to a sparse polynomial and verified as a member of the composed message ideal:
       S(T) = Σ_i q_i * r_i
   with exact Q membership certificate.

3. Kernel-specific route:
   A ProjectionMessage already contains target-only relation(s) with exact membership/resultant/action/norm/regular-chain/interpolation certificate,
   and the global support certificate binds those verified relations.
```

If route 1 is unavailable but route 2 is possible, the implementation must not return `CertificateDesignGap` merely because no explicit target-only root relation was present.

### CC-004: Pipeline shape

The public `api::solve_target(problem, options)` path must run:

```text
ValidateInput
CanonicalizeSystem
PreKernelAlgebraicCompression
BuildRelationVariableHypergraph
BuildTargetInfluenceGraph
BuildWeightedProjectionGraph
BuildTargetProjectionDAG
PlanProjectionMessages
ExecuteLocalProjectionKernels
ComposeProjectionMessages
BuildGlobalSupportPolynomial
VerifyGlobalSupport
SquarefreeSupport
ExactRealRootIsolation
DecodeTargetCandidates
FinalizeResultAndCertificate
```

`OptionalRealFiberClassification` is not required in candidate-cover mode and must not be used to reject spurious roots unless `exact_image_mode` is explicitly enabled.

### CC-005: TargetProjectionDAG and messages

The implementation must use real `TargetProjectionDAG` blocks and `ProjectionMessage`s.

DAGs, messages, certificates, and support must be replay-bound:

```text
- removing a message must make replay fail or change support;
- changing a plan hash must fail replay;
- changing a message package hash must fail replay;
- changing block authorization must fail replay;
- support/root/candidate hash tampering must fail replay.
```

### CC-006: No coordinate-first fallback

Production path must not produce:

```text
- full coordinate solution list
- full coordinate RUR
- full coordinate lex parametrization
- hidden global QE/CAD
- geometry-specific solver dispatch
```

Local elimination is allowed only inside authorized local blocks and may export only target/separator relations.

### CC-007: UniversalTargetElimination

`UniversalTargetEliminationKernel` is not a hidden fallback. It is the declared generic target/separator projection kernel.

For every well-formed block with relations, it must be admissible and appear in the declared ladder unless a stronger declared kernel already gives a verified message.

Universal may use declared internal strategies, but each strategy must remain local, resource-bounded, target/separator-export-only, and certificate-bound.

If no relation is found, Universal must not route to `CertifiedNonFiniteTargetImage` unless positive nonfinite proof exists.

### CC-008: TargetRelationSearch

`TargetRelationSearchKernel` remains the central generic candidate-cover workhorse.

It must construct relation candidates by exact membership search:

```text
g(Z) = Σ_i q_i(Y,Z) f_i(Y,Z)
```

and return a message only after exact Q verification of the identity.

The degree/support schedule must be deterministic and replay-bound. Sparse heuristics may accelerate but may not be the only correctness basis.

### CC-009: TargetActionKrylov

`TargetActionKrylovKernel` must not be target-only or alias-univariate only.

If used in production, it must build or verify a target-relevant finite quotient/action handle from authorized local relations, provide multiplication-by-target action, and certify coverage of all target-relevant eigenvalues by a verified characteristic/support/annihilator certificate.

A single weak Krylov probe without coverage proof must not produce candidate support.

### CC-010: Specialized kernels as optimizers, not whole solver

The following kernels may be production optimizers:

```text
SparseResultantProjection
RegularChainProjection
NormTraceProjection
SpecializationInterpolation
LinearAffine
TargetUnivariate
```

But none of them may be treated as the whole solver or as an excuse to reject other algebraic forms.

If a specialized kernel declines, planner must continue through the declared ladder. The generic pipeline must not become a collection of narrow slices.

### CC-011: SparseResultant source-fidelity for candidate-cover

If `SparseResultantProjectionKernel` is production-reachable, it must either:

```text
A. implement the v4 sparse/Macaulay template route with exact certificate; or
B. clearly mark itself as a pairwise resultant optimizer and never use that limitation as a reason for pipeline failure.
```

In either case, Universal/TargetRelationSearch must still cover the generic candidate-cover route.

### CC-012: F4/F5-like sparse linear algebra

If the implementation or documentation claims production F4/F5-like local sparse linear algebra, `algebra/f4.rs` must provide production `f4_reduce_batch` and `f4_elimination_local` functions with matrix trace and exact Q verification.

If production F4 is not implemented, all NotProductionF4 / for-tests helpers must remain non-production, and no completion claim may rely on them.

Candidate-cover readiness may proceed without production F4 only if the declared generic pipeline does not require F4 for support-producing success and does not claim F4 readiness.

### CC-013: Nonfinite safety

Candidate-cover completion does not require proving every nonfinite target image.

However:

```text
CertifiedNonFiniteTargetImage may be returned only with positive proof.
No-relation-found, degree-bound exhaustion, sparse heuristic failure,
Universal stage exhaustion, or composition failure must not route to nonfinite.
```

If nonfinite cannot be positively proven, return `AlgorithmicHardCase`, `FiniteResourceFailure`, or `CertificateDesignGap` with cost trace.

### CC-014: Root isolation and decoded candidates

Candidate-cover success must include:

```text
- squarefree_support_polynomial
- exact real root isolation
- decoded_candidates
- each candidate bound to target, support hash, root index, isolating interval, and candidate hash
```

If `S` has no real roots, success is still allowed:

```text
status = CertifiedCandidateCover
decoded_candidates = []
root_isolation = []
support_polynomial = Some(S)
```

### CC-015: Guard/slack semantics in candidate-cover mode

Guard/slack provenance must be preserved for exact-image mode, but candidate-cover mode must not filter candidates by real semantics.

A candidate-cover test may intentionally include spurious roots caused by guard/slack semantics. The correct candidate-cover behavior is to keep them, as long as all true values are included.

### CC-016: Cost trace

Every success and resource/hard/certificate failure must carry useful cost trace:

```text
n, m, d, monomial count, coefficient height
max block width, separator width
per block local variable/relation/monomial count
quotient rank estimate where applicable
matrix rows/cols/density where applicable
final support degree δ where applicable
certificate size κ where applicable
```

### CC-017: Run-level invariants

Run certificates or final closure evidence must bind:

```text
no_geometry_dispatch
no_problem_id_dispatch
no_expected_answer_dispatch
no_full_coordinate_solution_set
no_full_coordinate_rur
no_qe_cad
exact_q_verification
no_hidden_fallback
```

For candidate-cover readiness, static-scan evidence may be used if it is hash-bound to the closure and reviewed. Runtime `CoreRunCertificate` should not falsely claim unavailable per-run scans.

### CC-018: Reviewer interpretation rule

Reviewer must not fail a support-producing candidate-cover case because extra roots remain.

Reviewer must fail if:

```text
- support does not contain all true target values;
- proof of containment is missing;
- success uses expected answers, geometry names, or fixture dispatch;
- support is produced by hidden full-coordinate solve;
- a narrow pattern is presented as generic completion.
```
