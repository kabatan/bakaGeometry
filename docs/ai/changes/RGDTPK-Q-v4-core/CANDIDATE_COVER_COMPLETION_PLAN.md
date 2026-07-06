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
