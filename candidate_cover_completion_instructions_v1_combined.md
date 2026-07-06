

<!-- FILE: CANDIDATE_COVER_PATCH_NOTES.md -->

# Patch Notes — From Source-Faithful Completion v1 to Candidate-Cover Completion v1

## What changed

The previous `SOURCE_FAITHFUL_COMPLETION_*` pack targeted full v4 source fidelity, including exact-image semantics. That was too broad for the current research layer.

This pack narrows the implementation target to the candidate-cover layer, as defined in v4:

```text
CertifiedCandidateCover:
    true target values ⊆ roots(S)
    spurious roots allowed
```

## Removed as candidate-cover blockers

```text
- mixed coordinate real fiber classifier;
- exact-image semantic filtering;
- removal of spurious roots;
- CertifiedExactTargetImage readiness;
- general nonfinite completeness for every nonfinite target image.
```

## Kept as blockers

```text
- missing exact proof that S contains all true target values;
- narrow slice completion;
- hidden full-coordinate fallback;
- expected answer / geometry dispatch;
- missing root isolation/decode;
- replay not binding DAG/messages/support/root/candidate;
- false nonfinite without positive proof.
```

## Important reviewer rule

Do not fail a candidate-cover implementation because it returns extra roots.

Fail only if the implementation cannot prove that all true target values are included in the finite candidate set.


<!-- FILE: CANDIDATE_COVER_COMPLETION_BASE_SPEC.md -->

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


<!-- FILE: CANDIDATE_COVER_COMPLETION_PLAN.md -->

# Candidate-Cover Source-Faithful Completion Plan v1

This plan replaces the previous `SOURCE_FAITHFUL_COMPLETION_PLAN.md` for the current work item.

The current work item is **not** exact-image completion. It is source-faithful completion of the candidate-cover layer.

## Phase CCC-P0 — Reset over-constraint and claim scope

### Required actions

1. Create `CANDIDATE_COVER_AGENT_RESET.md`.
2. Explicitly state:

```text
The goal is finite candidate cover, not exact target image.
Spurious roots are allowed.
Exact-image filtering is not a blocker.
The implementation must prove true target values are contained in roots(S).
The implementation must not shrink scope to a narrow algebraic slice.
```

3. Update `ACTIVE_CONTEXT.md` and `CLOSURE.md`:

```text
Current maximum claim after this repair:
CANDIDATE_COVER_CORE_READY

Not claimed by this repair alone:
EXACT_IMAGE_CORE_READY
SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC
RGDTPK_Q_V4_ACCEPTANCE_COMPLETE
```

4. Mark the previous `SOURCE_FAITHFUL_COMPLETION_*` pack as superseded for candidate-cover work.

### Reviewer must fail if

```text
- exact-image mixed fiber completion remains a blocker for candidate-cover;
- the agent says spurious roots are a correctness failure;
- final claim still implies exact target image equality.
```

## Phase CCC-P1 — Candidate-cover source map

### Required actions

Create `CANDIDATE_COVER_SOURCE_MAP.md`.

For every v4 section/file/function, classify:

```yaml
item: <section/file/function>
candidate_cover_required: true|false
exact_image_later: true|false
optimizer_optional: true|false
implemented_status: implemented|partial|missing|not_applicable
current_path: <file::function or MISSING>
repair_action: keep|implement|generalize|quarantine|defer_exact_image
notes: <why>
```

### Mandatory classification rules

```text
v4 sections 0-5: candidate-cover required except exact-image semantic filtering.
v4 sections 7-26: candidate-cover required where they participate in S(T), roots, candidates, replay.
v4 section 27: exact-image later, API/provenance must exist but mixed fiber classifier is not candidate-cover blocker.
v4 section 31: positive nonfinite proof safety required; general nonfinite completeness is exact/nonfinite extension, not candidate-cover blocker.
v4 section 33 condition 13: exact-image later, not candidate-cover blocker.
```

### Reviewer must fail if

```text
- exact-image functions are silently deleted;
- exact-image deficiencies block candidate-cover;
- a candidate-cover function is misclassified as later;
- a specialized kernel limitation is marked implemented without generic fallback.
```

## Phase CCC-P2 — Support containment proof boundary

### Goal

Make `CertifiedCandidateCover` mean exactly:

```text
true target values ⊆ roots(S)
```

not equality.

### Required implementation

1. Audit `compose/final_support.rs`, `verify/verify_support.rs`, `solver/orchestrator.rs`, and `result/output.rs`.
2. Ensure candidate-cover mode does not call exact-image filtering.
3. Ensure spurious roots from slack/guard semantics remain in `CertifiedCandidateCover`.
4. Add diagnostics that distinguish:

```text
CandidateCoverMayContainSpuriousRoots
ExactImageFilteringNotRequested
```

without treating them as failure.

### Required tests

At least one public `solve_target` test where:

```text
support S(T) has more real roots than the real-semantic target image,
exact_image_mode = false,
status = CertifiedCandidateCover,
all roots remain decoded candidates.
```

At least one test with `exact_image_mode = true` may show filtering, but it must be outside candidate-cover acceptance.

### Reviewer must fail if

```text
- candidate-cover mode filters by guard/slack semantics;
- spurious roots cause failure;
- tests only check exact-image mode.
```

## Phase CCC-P3 — Global support verification completion

### Goal

Complete the candidate-cover proof that `S(T)` vanishes on all true target values.

### Required implementation

`verify_global_support` must support both routes:

```text
Route A: target-only root relation product/lcm.
Route B: composed message ideal membership certificate.
```

### Route B required structure

Add or complete:

```rust
ComposedIdealMembershipSupportCertificate {
    target: VariableId,
    support_hash: Hash,
    composed_hash: Hash,
    relation_hashes: Vec<Hash>,
    multipliers: Vec<SparsePolynomialQ>,
    exact_identity_hash: Hash,
    certificate_hash: Hash,
}
```

Verification:

```text
support_sparse(T) - Σ_i multiplier_i * composed_relation_i == 0 over Q
```

### Required tests

1. A support-producing case where support verification uses Route A.
2. A support-producing case where Route B is used or forced.
3. Tamper one multiplier and replay must reject.
4. Remove one composed relation and replay must reject.

### Reviewer must fail if

```text
- Route B is documented but never production reachable;
- membership is checked by hash only;
- support is hand-built from expected answer;
- verifier only checks target-only relation product.
```

## Phase CCC-P4 — Generic projection integrity audit

### Goal

Ensure support production is not a narrow-slice portfolio.

### Required audit targets

```text
TargetRelationSearchKernel
UniversalTargetEliminationKernel
TargetActionKrylovKernel
SparseResultantProjectionKernel
RegularChainProjectionKernel
NormTraceProjectionKernel
SpecializationInterpolationKernel
LinearAffineKernel
TargetUnivariateKernel
```

For each kernel, create a table:

```yaml
kernel: <name>
role: generic_workhorse|generic_last_resort|specialized_optimizer
production_reachable: true|false
declines_allowed: true|false
generic_fallback_after_decline: <TargetRelationSearch|Universal|none>
known_shape_limits: []
proof_route: <membership|resultant|action|norm|regular_chain|interpolation>
not_completion_by_itself: true|false
```

### Reviewer must fail if

```text
- a specialized optimizer is the only path for a required support-producing structure;
- TargetActionKrylov is target-only/alias-only;
- Universal is hidden full-coordinate fallback;
- TargetRelationSearch schedule is discretionary or heuristic-only.
```

## Phase CCC-P5 — TargetRelationSearch / Universal hardening

### TargetRelationSearch requirements

1. Dense total-degree schedule remains deterministic.
2. Sparse heuristic is optional only.
3. Exact Q membership certificate is mandatory.
4. Failure within declared bounds is not nonfinite.
5. Cost trace includes matrix dimensions and degree bounds.

### Universal requirements

1. Always admissible for well-formed relation-bearing block.
2. Declared strategy sequence is fixed or hash-bound.
3. No coordinate roots / full RUR / QE/CAD.
4. Local block only; exports only target/separator relations.
5. No nonfinite status without positive proof.
6. If local Groebner is used, it must be certified and keep-only.

### Required tests

```text
- one large block with no useful separator returns candidate cover;
- relation-search exhaustion returns AlgorithmicHardCase/FiniteResourceFailure/CertificateDesignGap, not nonfinite;
- tampering strategy sequence fails replay;
- exported relation containing local variable is rejected.
```

## Phase CCC-P6 — TargetActionKrylov source-fidelity

### Required behavior

If production reachable, TargetActionKrylov must build or verify target-relevant finite quotient/action handles from authorized local relations.

It must not rely on:

```text
- target-only relation already present;
- local-univariate plus linear alias only;
- externally injected action matrix self-consistency;
- single Krylov probe without coverage proof.
```

### Required tests

1. Multivariate quotient/action case not reducible to local-univariate alias.
2. Renamed/permuted/scaled version of same algebraic structure.
3. Undercoverage trap rejected.
4. Action column membership tamper rejected.
5. No coordinate roots or full RUR in handle.

## Phase CCC-P7 — Specialized optimizer cleanup

### Goal

Keep specialized kernels useful, but prevent completion-by-slice.

### SparseResultant

Required:

```text
- if claimed generic sparse resultant, implement multi-polynomial sparse/Macaulay template route;
- if only pairwise chain, label it as optimizer and ensure generic pipeline succeeds without relying on it.
```

### RegularChain

Required:

```text
- projection certificate must preserve component/union semantics;
- component projection cannot select a component without certificate;
- if limited recognizer, it must be optimizer only.
```

### NormTrace

Required:

```text
- multi-step tower support if claimed;
- norm_of_target_minus_expression for T - r(alpha,Z);
- exact verification of norm relation.
```

### SpecializationInterpolation

Required:

```text
- plan defines sample/support schedule only;
- execute runs declared inner kernel;
- final relation must be verified over Q by membership or elimination;
- interpolation samples alone are not proof.
```

### Reviewer must fail if

```text
- any optimizer's limitation becomes an unsupported/slice boundary;
- module-only tests are the only evidence;
- planner cannot continue to generic workhorse after optimizer decline.
```

## Phase CCC-P8 — F4 route decision

### Required decision

Choose exactly one:

```text
A. Implement production F4/F5-like sparse local algebra.
B. Keep F4 non-production and remove all production claims/dependencies.
```

### If A

Implement:

```text
f4_reduce_batch
f4_elimination_local
F4Options
F4BatchReductionResult
matrix trace
exact Q verification
membership certificates
```

### If B

Required:

```text
- all NotProductionF4 helpers remain test-only or explicitly non-production;
- Universal and elimination strategies do not require F4 to produce candidate covers;
- closure states "F4 route not claimed" and does not use F4 evidence for readiness.
```

### Reviewer must fail if

```text
- Groebner-backed helper is named or claimed as production F4;
- F4 absence is hidden while claiming source-faithful full v4;
- candidate-cover readiness relies on unimplemented F4.
```

## Phase CCC-P9 — Nonfinite safety, not completeness

### Required behavior

Candidate-cover completion requires safe nonfinite handling, not general nonfinite completeness.

Implement/verify:

```text
- CertifiedNonFiniteTargetImage only with positive certificate.
- Relation-search failure, resource exhaustion, sparse heuristic failure, and Universal exhaustion never route to nonfinite.
- Nonfinite replay verifies certificate against composed projection.
- If positive proof is unavailable, return AlgorithmicHardCase or CertificateDesignGap.
```

### Required tests

1. Positive nonfinite case with replay-bound certificate.
2. Similar no-positive-proof case returns non-nonfinite failure.
3. Tamper nonfinite certificate hash and replay rejects.
4. Inject nonfinite certificate into finite result and replay rejects.

### Reviewer must fail if

```text
- nonfinite is used to hide inability to produce support;
- rational witness-only proof is claimed as general nonfinite completeness;
- nonfinite completeness is required for candidate-cover readiness.
```

## Phase CCC-P10 — Run invariants and anti-dispatch evidence

### Required implementation

Bind evidence for:

```text
no_geometry_dispatch
no_problem_id_dispatch
no_expected_answer_dispatch
no_full_coordinate_solution_set
no_full_coordinate_rur
no_qe_cad
no_hidden_fallback
exact_q_verification
```

Permitted approach:

```text
- runtime certificate for algebraic invariants;
- hash-bound static scan evidence for repository-wide dispatch/fallback absence;
- closure artifact tying scan hashes to final claim.
```

### Reviewer must fail if

```text
- static scan evidence is unhashbound;
- run certificate claims false runtime flags;
- geometry/fixture strings appear in production dispatch;
- expected-answer or problem-id path exists.
```

## Phase CCC-P11 — Candidate-cover acceptance suite

### Required support-producing public cases

All support-producing cases must use `api::solve_target`.

Include at least:

```text
A1. no initial target-only relation, one block
A2. multivariate finite quotient/action, target nonlinear in quotient variables
A3. multiple eliminated variables and two or more separators
A4. sparse resultant/eliminant with no initial target-only input
A5. specialization/interpolation with exact Q verification
A6. guarded rational affine preprocessing with nonconstant denominator
A7. target-independent component with feasibility obligation retained for exact-image later
A8. one-large-block no useful separator via Universal
A9. regular-chain-style projection as optimizer
A10. norm/trace two-step tower as optimizer
A11. support with no real roots returns empty candidate cover, not hardcase
A12. slack/guard semantic case where candidate-cover keeps spurious roots
A13. bounded hard/resource/certificate failure has trace and is not nonfinite
```

### Mandatory transformations

Every support-producing case must use at least two:

```text
- permuted variable ids
- permuted relation order
- nonzero rational scaling of relations
- target variable id not zero
- syntactic polynomial rearrangement
```

### Support-producing success must assert

```text
status = CertifiedCandidateCover
support_polynomial is nonzero
squarefree_support_polynomial exists
projection_messages nonempty
run certificate exists
root/candidate hashes valid
replay accepts
cost trace populated
```

It must not assert:

```text
decoded_candidates are exactly the true target image
spurious roots are absent
exact_image_certificate exists
```

## Phase CCC-P12 — Adversarial red-team

Reviewer must create at least 12 new algebraic inputs not used in P11.

At least 4 cases must intentionally allow spurious roots in candidate-cover mode.

The reviewer must answer for each:

```text
- Why does true target values ⊆ roots(S) hold?
- Which exact certificate proves containment?
- Which part of the generic pipeline produced the support?
- Why is this not expected-answer dispatch?
- Why is this not a narrow slice?
```

Reviewer must fail if:

```text
- any support-producing case fails only because of spurious roots;
- any success lacks containment proof;
- reviewer relies on previous suite labels instead of new inputs;
- exact-image filtering is used to make candidate-cover pass.
```

## Phase CCC-P13 — Final candidate-cover closure

Required artifacts:

```text
CANDIDATE_COVER_ACCEPTANCE_RESULTS.md
CANDIDATE_COVER_REPLAY_TAMPER_RESULTS.md
CANDIDATE_COVER_COST_TRACE_SUMMARY.md
CANDIDATE_COVER_SOURCE_MAP.md
CANDIDATE_COVER_CLOSURE.md
```

Allowed final claim:

```text
CANDIDATE_COVER_CORE_READY
SOURCE_FAITHFUL_TO_V4_CANDIDATE_COVER_LAYER
```

Forbidden claim:

```text
SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC
RGDTPK_Q_V4_ACCEPTANCE_COMPLETE
EXACT_IMAGE_CORE_READY
```

unless exact-image completion is separately implemented and reviewed.

Reviewer must fail if:

```text
- closure says or implies candidate set equals exact target image;
- closure uses exact-image tests to justify candidate-cover containment;
- closure omits a clear statement that spurious roots are allowed;
- any support-producing suite success lacks replay-bound exact containment evidence.
```


<!-- FILE: CANDIDATE_COVER_COMPLETION_REVIEWER_PROMPTS.md -->

# Candidate-Cover Completion Reviewer Prompts v1

## Meta-protocol for every reviewer

Before issuing PASS, answer the following in the review response:

```text
1. What is the largest algebraic input class actually handled by the changed production code?
2. Does the reviewed phase prove true target values ⊆ roots(S), or only produce a plausible polynomial?
3. Are spurious roots being incorrectly treated as failures?
4. Does any success depend on exact-image filtering?
5. Does any success depend on geometry name, fixture, expected answer, or problem id?
6. Does any production path build full coordinate roots or full coordinate RUR?
7. Is every successful support replay-bound through actual DAG/messages/certificates?
8. Is this a generic candidate-cover mechanism, or a narrow slice with documentation?
```

A reviewer must not PASS merely because tests, YAML, hashes, or phase gates are present.

## Global pass/fail principles

### Must not fail

Do not fail candidate-cover merely because:

```text
- extra roots remain;
- a root has no real fiber under guard/slack semantics;
- exact_image_certificate is absent in candidate-cover mode;
- CertifiedExactTargetImage is not implemented.
```

### Must fail

Fail if:

```text
- true target values may be missing from returned candidates;
- S(T) is not exact Q verified;
- support is constructed from expected answer;
- support-producing case is helper-only and bypasses public pipeline;
- exact-image filtering is used to hide candidate-cover weakness;
- support-producing case returns AlgorithmicHardCase / FiniteResourceFailure / CertificateDesignGap / CertifiedNonFiniteTargetImage;
- narrow target-univariate/affine/alias-only implementation is treated as generic completion;
- nonfinite is returned without positive proof.
```

## Prompt: CCC-P0 reset review

```
Review CCC-P0. Confirm the Agent reset correctly states that candidate-cover means true target values are contained in roots(S), not equality. Confirm exact-image mixed fiber classification is no longer a blocker for candidate-cover readiness. Fail if the Agent still treats spurious roots as a defect, or if it tries to claim full exact-image/source-fidelity completion from this repair.
```

## Prompt: CCC-P1 source map review

```
Review CANDIDATE_COVER_SOURCE_MAP.md against v4. Confirm candidate-cover required functions are not deferred to exact-image later. Confirm section 27 exact-image functions are retained as later API/provenance, not deleted. Fail if support verification, root isolation, decode, DAG, planner, kernels, or replay are misclassified as later.
```

## Prompt: CCC-P2 candidate-cover semantics review

```
Inspect solver/orchestrator.rs, compose/final_support.rs, result/output.rs, and tests. Confirm candidate-cover mode does not filter roots by guard/slack semantics. Confirm at least one test intentionally permits spurious roots and still passes as CertifiedCandidateCover. Fail if exact-image mode is used to justify candidate-cover correctness.
```

## Prompt: CCC-P3 support verification review

```
Inspect verify/verify_support.rs, compose/final_support.rs, verify/replay.rs. Confirm both target-only product/lcm route and composed-ideal membership route are implemented when applicable. For membership route, require exact identity S(T)-Σq_i*r_i=0 over Q. Fail if membership is hash-only, sampled, numeric, or helper-only.
```

## Prompt: CCC-P4 generic projection integrity review

```
Inspect all kernels and planner/admission/ladder. Confirm specialized kernels are optimizers, not whole-solver slices. Confirm TargetRelationSearch and Universal remain generic routes. Fail if a declined optimizer can cause unsupported/slice failure when a generic route should continue.
```

## Prompt: CCC-P5 TRS/Universal review

```
Inspect target_relation_search.rs, universal_elimination.rs, planner/relation_schedule.rs, certificates. Confirm TRS uses deterministic schedule and exact membership. Confirm Universal is declared, local, resource-bounded, export-only, and never maps exhaustion to nonfinite. Fail if any hidden coordinate fallback exists.
```

## Prompt: CCC-P6 ActionKrylov review

```
Inspect action_krylov.rs, algebra/quotient.rs, algebra/krylov.rs. Confirm production TargetActionKrylov is not target-only or alias-univariate only. Confirm action columns are authorized-relation membership certified. Confirm coverage proof prevents missed eigenvalues. Fail if external action matrix self-consistency is accepted as proof.
```

## Prompt: CCC-P7 specialized optimizer review

```
Inspect sparse_resultant.rs, regular_chain_projection.rs, norm_trace_projection.rs, specialization_interpolation.rs. Confirm each optimizer has exact verification and does not define solver scope. If a kernel claims generic capability, verify the generic algorithm exists. Fail if limitations are used as completion boundaries.
```

## Prompt: CCC-P8 F4 decision review

```
Inspect algebra/f4.rs and all production references. Confirm either real production F4 exists with exact verification, or all F4 helpers are non-production and no readiness claim relies on F4. Fail if NotProductionF4 is used as production evidence.
```

## Prompt: CCC-P9 nonfinite safety review

```
Inspect final_support.rs, replay.rs, nonfinite tests. Confirm CertifiedNonFiniteTargetImage requires positive certificate and replay verifies it. Confirm relation-search exhaustion, sparse heuristic failure, Universal exhaustion, and composition failure do not become nonfinite. Fail if general nonfinite completeness is required for candidate-cover readiness.
```

## Prompt: CCC-P10 invariant review

```
Inspect run_certificate.rs, replay.rs, static scans, closure evidence. Confirm no-geometry/no-problem-id/no-expected-answer/no-QE/no-RUR/no-coordinate-solution evidence is hash-bound and reviewed. Fail if runtime flags claim more than they prove, or if static scan evidence is unbound.
```

## Prompt: CCC-P11 acceptance suite review

```
Run/inspect the public candidate-cover acceptance suite. Every support-producing case must use api::solve_target and return CertifiedCandidateCover with support, squarefree support, candidates, certificate, replay, and cost trace. Do not require exact target image equality. Fail if spurious roots are removed or treated as defects.
```

## Prompt: CCC-P12 red-team review

```
Construct at least 12 fresh algebraic inputs. At least 4 must permit spurious roots in candidate-cover mode. For each success, identify the proof of true target containment. Fail if support is expected-answer-based, if exact-image filtering is required for success, or if any success lacks exact containment proof.
```

## Prompt: CCC-P13 final closure review

```
Review CANDIDATE_COVER_CLOSURE.md and all evidence. Confirm final claim is only CANDIDATE_COVER_CORE_READY / SOURCE_FAITHFUL_TO_V4_CANDIDATE_COVER_LAYER. Confirm closure clearly states spurious roots are allowed. Fail if it claims SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC or RGDTPK_Q_V4_ACCEPTANCE_COMPLETE without separate exact-image completion.
```


<!-- FILE: CANDIDATE_COVER_ACCEPTANCE_MATRIX.yaml -->

schema_version: candidate-cover-acceptance-v1
claim_target:
  allowed:
    - CANDIDATE_COVER_CORE_READY
    - SOURCE_FAITHFUL_TO_V4_CANDIDATE_COVER_LAYER
  forbidden_without_exact_image_completion:
    - EXACT_IMAGE_CORE_READY
    - SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC
    - RGDTPK_Q_V4_ACCEPTANCE_COMPLETE

semantic_rule:
  candidate_cover:
    statement: "true_target_values_subset_of_roots_S"
    spurious_roots_allowed: true
    exact_target_image_required: false
  reviewer_must_not_fail_for:
    - extra_roots
    - absent_exact_image_certificate
    - real_semantic_filtering_not_performed
  reviewer_must_fail_for:
    - missing_true_target_value
    - no_exact_Q_support_verification
    - helper_only_support
    - expected_answer_dispatch
    - hidden_coordinate_solve

support_producing_public_cases:
  - id: A1
    name: no_initial_target_only_relation_one_block
    required_status: CertifiedCandidateCover
    required_path: api::solve_target
    required_evidence:
      - nonzero_support
      - exact_Q_verification
      - replay_accepts
      - cost_trace
  - id: A2
    name: multivariate_quotient_action_nonlinear_target
    required_kernel: TargetActionKrylov
    must_not_be:
      - target_only_relation_already_present
      - local_univariate_alias_only
  - id: A3
    name: multiple_eliminated_variables_multiple_separators
    required_evidence:
      - projection_messages_at_least_2
      - separator_composition_essential
  - id: A4
    name: sparse_resultant_or_generic_route_no_initial_target_only
    required_evidence:
      - exact_resultant_or_membership_verification
  - id: A5
    name: specialization_interpolation_exact_Q_verification
    required_evidence:
      - sample_schedule_declared
      - final_relation_exactly_verified
  - id: A6
    name: guarded_rational_affine_nonconstant_denominator
    required_evidence:
      - denominator_guard_recorded
      - support_contains_true_values
  - id: A7
    name: target_independent_component_feasibility_obligation
    required_evidence:
      - candidate_cover_not_blocked_by_target_independent_component
      - feasibility_obligation_retained_for_exact_image_later
  - id: A8
    name: one_large_block_universal
    required_kernel: UniversalTargetElimination
    required_evidence:
      - no_useful_separator
      - declared_universal_ladder
  - id: A9
    name: regular_chain_optimizer
    required_evidence:
      - component_semantics_not_overclaimed
      - generic_pipeline_continues_if_declined
  - id: A10
    name: norm_trace_optimizer
    required_evidence:
      - norm_relation_exactly_verified
      - generic_pipeline_continues_if_declined
  - id: A11
    name: non_real_support_empty_candidate_cover
    required_status: CertifiedCandidateCover
    allow_empty_candidates: true
  - id: A12
    name: slack_guard_spurious_roots_allowed
    required_status: CertifiedCandidateCover
    required_behavior:
      - exact_image_mode_false
      - spurious_roots_remain
      - replay_accepts
  - id: A13
    name: bounded_hard_failure_not_nonfinite
    required_status_one_of:
      - AlgorithmicHardCase
      - FiniteResourceFailure
      - CertificateDesignGap
    forbidden_status:
      - CertifiedNonFiniteTargetImage
    required_evidence:
      - nonempty_cost_trace

red_team:
  fresh_inputs_minimum: 12
  spurious_root_cases_minimum: 4
  must_include:
    - multivariate_action_not_alias
    - two_separator_composition
    - higher_degree_sparse_eliminant
    - guarded_affine_nonconstant_denominator
    - one_large_block_universal
    - target_independent_component
    - positive_nonfinite_certificate
    - no_positive_proof_not_nonfinite
    - regular_chain_fresh
    - norm_trace_fresh
    - support_membership_route
    - semantic_spurious_roots


<!-- FILE: CANDIDATE_COVER_AGENT_RESET_TEMPLATE.md -->

# Candidate-Cover Agent Reset Template

The Agent must copy this file to `CANDIDATE_COVER_AGENT_RESET.md` and fill every item before editing code.

## 1. Current goal

Write in your own words:

```text
The current goal is to implement the v4 candidate-cover layer.
The solver must return finite candidates containing every true target value.
It is allowed to return extra candidates.
```

## 2. What must not be done

Acknowledge:

```text
- Do not treat spurious roots as a failure.
- Do not use exact-image filtering to make candidate-cover pass.
- Do not shrink to target-univariate / affine / alias / bivariate / fixture-specific slices.
- Do not use full coordinate solution, full RUR, or hidden QE/CAD fallback.
- Do not route failure to nonfinite without positive proof.
```

## 3. Prior failure modes to avoid

List at least five concrete prior failure modes from this project, generalized.

Required inclusions:

```text
- gate/review evidence treated as algorithmic proof;
- documented limitation treated as acceptable completion;
- no-fallback confused with narrow scope;
- expected-answer or fixture-like stress;
- exact-image requirements confused with candidate-cover requirements.
```

## 4. Phase handoff pledge

Every phase handoff must answer:

```text
1. Did this phase prove true values ⊆ roots(S)?
2. Did this phase accidentally require roots(S) ⊆ true values?
3. Did any support-producing case depend on exact-image filtering?
4. Did any partial slice become a completion claim?
5. Did any hidden fallback or unsupported path appear?
```


<!-- FILE: CANDIDATE_COVER_QUICK_GUARDIAN_PROMPT.md -->

# Quick Guardian Prompt — Candidate-Cover Source-Faithful Completion

You are implementing the candidate-cover layer of R-GDTPK-Q / ACCTP-Q.

Do not continue from the previous source-faithful exact-image repair pack as-is. That pack over-constrained the current work by making exact-image mixed real fiber classification a blocker.

Use these files as controlling instructions:

```text
CANDIDATE_COVER_COMPLETION_BASE_SPEC.md
CANDIDATE_COVER_COMPLETION_PLAN.md
CANDIDATE_COVER_COMPLETION_REVIEWER_PROMPTS.md
CANDIDATE_COVER_ACCEPTANCE_MATRIX.yaml
CANDIDATE_COVER_AGENT_RESET_TEMPLATE.md
```

Core semantic rule:

```text
CertifiedCandidateCover means:
    true target values ⊆ roots(S)

It does not mean:
    roots(S) = true target values
```

Spurious roots are allowed.

Your final candidate-cover repair may claim only:

```text
CANDIDATE_COVER_CORE_READY
SOURCE_FAITHFUL_TO_V4_CANDIDATE_COVER_LAYER
```

Do not claim:

```text
EXACT_IMAGE_CORE_READY
SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC
RGDTPK_Q_V4_ACCEPTANCE_COMPLETE
```

unless exact-image source-faithful completion is separately implemented and reviewed.

Start with:

```text
CCC-P0
CCC-P1
```

Do not edit code before completing the reset and source map.


<!-- FILE: CANDIDATE_COVER_CURRENT_DEFECT_AUDIT.md -->

# Candidate-Cover Current Defect Audit

This audit replaces the over-broad exact-image-focused defect audit for the current work.

## Corrected interpretation

The current algorithm is intended to return a finite candidate cover, not an exact target image.

Therefore, the following are not candidate-cover defects:

```text
- spurious roots remain;
- exact_image_certificate is absent in candidate-cover mode;
- mixed coordinate real fiber classification is incomplete;
- guard/slack semantics are not used to filter candidate-cover candidates.
```

They are exact-image-layer defects only.

## Still candidate-cover defects

The following remain defects for the current layer:

```text
1. support S(T) is not exact-Q verified;
2. support may omit a true target value;
3. success depends on expected answer or fixture dispatch;
4. production path uses full coordinate solutions or full RUR;
5. generic pipeline is actually a narrow slice portfolio;
6. TargetActionKrylov is target-only or alias-only;
7. support verification only supports target-only product route when composed-ideal membership route is required;
8. Universal exhaustion maps to nonfinite without positive proof;
9. root isolation or candidate decode is placeholder or absent;
10. replay does not bind DAG/messages/support/roots/candidates.
```

## Current known likely work items

Based on prior review, inspect and either fix or explicitly clear:

```text
- verify/verify_support.rs:
    Add composed-ideal membership route if absent.

- compose/final_support.rs:
    Ensure S(T) proof is containment proof, not exact-image proof.

- kernels/action_krylov.rs and algebra/quotient.rs:
    Verify generic quotient/action path is not alias-only.

- kernels/universal_elimination.rs:
    Verify declared local generic route, no hidden coordinate fallback.

- algebra/f4.rs:
    Either production F4 or non-production-only with no readiness dependency.

- compose/final_support.rs nonfinite:
    Positive proof only; no general nonfinite completeness required for candidate-cover.

- result/output.rs and solver/orchestrator.rs:
    Candidate-cover mode must not filter spurious roots.

- verify/replay.rs:
    Replay must bind actual DAG/messages/support/root/candidate and nonfinite certificate if present.
```
