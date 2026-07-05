# Full Core Repair Base Spec — RGDTPK-Q Candidate-Cover Solver Completion

Spec ID: `RGDTPK-Q-v4-full-core-repair-v1`  
Status: mandatory corrective base spec overlay  
Insertion point: immediately after the current P12G worktree state and before P13/P14/P15/P16.  
Authority: this document hardens, narrows, and where necessary overrides all P1–P12G completion claims. It must be read together with the original R-GDTPK-Q v4 specification and the failure-cause summary.

This repair is not another small remediation phase. When this repair is complete, the repository must contain a working production solver core that takes the algebraic common representation produced from arbitrary geometry IR lowering and returns a finite certified target candidate cover whenever the target image is finite and the declared resources suffice. The implementation must not be a collection of narrow slices, demo cases, helper tests, or documentation-only claims.

---

## FCR-000 — Required final state

At the end of this repair, `api::solve_target(problem, options)` must execute the production candidate-cover pipeline:

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

For every well-formed Q-polynomial target system:

```text
- the input enters the same generic algebraic pipeline;
- no geometry name, problem id, fixture id, or expected answer dispatch exists;
- a TargetProjectionDAG is built;
- no-separator cases become one large target block;
- every block receives a deterministic KernelPlan whose ladder contains a generic target-direct path;
- successful finite target-image cases return S(T) != 0 in Q[T];
- S(T) is exact-Q verified as a candidate-cover support;
- squarefree support, exact real root isolation, and decoded candidates are produced;
- no coordinate solution list, full coordinate RUR, QE/CAD fallback, or hidden fallback is produced;
- failure uses FiniteResourceFailure, AlgorithmicHardCase, CertificateDesignGap, ImplementationBug, or InvalidInput only.
```

Completion of this repair must justify `CANDIDATE_COVER_CORE_READY` and no weaker "partial mechanism" label for the candidate-cover layer. It does not by itself justify exact-image completion unless P13 is also completed.

---

## FCR-001 — No partial-slice production implementation

A production implementation is invalid if its success path depends on a narrow algebraic shape being present.

Forbidden production claims include:

```text
- target-univariate-only solver;
- affine-only solver;
- bivariate-only solver;
- local-univariate plus linear-target-alias-only TargetActionKrylov;
- pairwise-binary-resultant-only SparseResultantProjection;
- explicit-tower-only NormTrace as generic projection coverage;
- triangular-pattern-only RegularChain as generic projection coverage;
- support-producing stress that passes only through helper functions instead of the planner/kernel/compose path;
- documenting a limitation instead of implementing the generic mechanism required by the original spec.
```

Specialized algebraic kernels may remain only as optimizations inside a generic pipeline. They must never be the repository's only reason for claiming the candidate-cover core is complete.

---

## FCR-002 — Current inappropriate implementations must be removed, isolated, or generalized

The Agent must inspect all existing P1–P12G code. Any production code that is one of the following must be corrected:

```text
- a test-only implementation exposed from production modules;
- a narrow shape recognizer used as a completion path;
- a hidden plan-time execution path;
- an unchecked or self-certifying certificate path;
- a synthetic replay substitute for real DAG/block authorization;
- a placeholder finalizer, temporary orchestrator, fake F4, or non-production kernel;
- a helper that constructs support without going through the declared production pipeline.
```

Permitted outcomes:

```text
Generalize:
    replace the implementation with the generic algorithm required by the v4 spec.

Delete from production:
    remove the path from production module exports and registry; move only harmless examples to #[cfg(test)] or test support.

Quarantine:
    keep code under an explicit non-production module that cannot be reached by api::solve_target, planner::all_kernels, UniversalTargetElimination, or final acceptance tests.
```

Not permitted:

```text
- merely documenting that the implementation is limited;
- keeping a limited implementation in production and claiming the generic mechanism is complete;
- letting a specialized optimizer be the only support-producing path for a required stress category.
```

---

## FCR-003 — Full public orchestrator is mandatory

`solver/orchestrator.rs` must not return `temporary_pipeline_not_connected`. The public API must run the full production candidate-cover pipeline.

Required behavior:

```text
solve_target(problem, options)
  -> TargetSolveResult with CertifiedCandidateCover / CertifiedNonFiniteTargetImage / allowed failure status
```

No test or review may pass this repair if `solve_with_context` remains a temporary failure wrapper.

---

## FCR-004 — Plan and execute must be separated

Planning must not compute final output relations.

Allowed planning work:

```text
- deterministic variable/relation ordering;
- kernel admission metadata;
- degree schedules;
- support schedules;
- template dimensions and support hashes;
- sample-point schedules;
- modular probe summaries for cost only;
- resource bounds;
- declared ladder and certificate routes.
```

Forbidden planning work:

```text
- computing the resultant relation;
- running inner TargetRelationSearch to produce final support;
- running Groebner/F4/regular-chain/norm-trace to produce final projection generators;
- building final target support before execute;
- treating a heavy plan-time computation as PurePlan.
```

Any existing `plan_*` function that calls a `build_*_trace` function which constructs final relations must be refactored. The relation-producing computation belongs in `execute_*`.

`CertifiedProbePlan` is allowed only for non-final cost probes. It must not be used to legitimize final output relation construction during planning.

---

## FCR-005 — Generic TargetRelationSearch remains the central workhorse

`TargetRelationSearchKernel` must be production-grade and generic over authorized local relations:

```text
J = <f1,...,fr> in Q[Y,Z]
Z = exported target/separator variables
Y = eliminated local variables
```

It must search for nonzero `g(Z) in J ∩ Q[Z]` using deterministic dense total-degree schedules and exact membership verification:

```text
g(Z) = Σ_i q_i(Y,Z) f_i(Y,Z)
```

Modular linear algebra may be used for candidate generation, but no relation may be returned until the exact Q identity is verified.

This kernel may return AlgorithmicHardCase or FiniteResourceFailure when declared resources are insufficient. It must not become geometry-specific, fixture-specific, or target-univariate-only.

---

## FCR-006 — Generic TargetActionKrylov must be implemented, not slice-patched

`TargetActionKrylovKernel` must not be limited to:

```text
- already target-only univariate support;
- local-univariate relation plus linear target alias relation.
```

Production TargetActionKrylov must implement the original generic algorithm:

```text
1. From all authorized local block relations, construct or verify a finite target-relevant quotient/action handle.
2. Build a normal-form basis from the authorized ideal, not from injected action columns.
3. Certify normal forms of multiplication-by-target on every basis element by exact membership identities against authorized relations.
4. Build the target action matrix with no coordinate roots and no full coordinate RUR.
5. Compute characteristic/minimal support with verified coverage; single-vector undercoverage is forbidden.
6. Export only target/separator relations.
7. Replay must reject tampered basis, action columns, source relations, authorization hash, and output support.
```

Minimum acceptance for this kernel must include multivariate finite quotient ideals whose quotient basis is not a single local univariate chain and whose target relation is not initially present.

If this cannot be implemented, `TargetActionKrylovKernel` must be removed from the production generic completion claim, P8c/MECH-014 must be reopened, and this full-core repair must fail. It is not acceptable to continue with only the alias-univariate pattern.

---

## FCR-007 — SparseResultantProjection must be generic or non-production

A pairwise binary resultant chain may remain as an internal optimization but must not be claimed as the generic SparseResultantProjection kernel.

Production SparseResultantProjection must:

```text
- declare support/template construction for the intended arity and elimination variables;
- execute template construction and relation recovery during execute, not plan;
- verify the produced relation over Q;
- export only target/separator variables;
- include exact certificate/replay;
- fail by allowed statuses, not Unsupported.
```

If only binary pairwise chains are available, the code must be clearly non-generic and cannot satisfy the v4 SparseResultantProjection requirement or final full-core completion.

---

## FCR-008 — SpecializationInterpolation must be a declared execution algorithm

SpecializationInterpolation must not run inner kernels during planning.

Required production structure:

```text
Plan:
    exported separator variables;
    sample schedule;
    interpolation support bound;
    inner kernel plan template or declared inner ladder;
    exact verification route;
    resource bounds.

Execute:
    run declared inner target-only computations on each specialization;
    interpolate coefficients;
    verify the interpolated relation by exact Q membership/elimination;
    export a relation in Q[target,separators].

Replay:
    verify sample schedule, inner message certificates, interpolation certificate, and exact Q verification.
```

Sample agreement alone is never proof.

---

## FCR-009 — UniversalTargetElimination is mandatory and must not be a fallback

UniversalTargetElimination is a planned generic target/separator projection kernel. It is not a hidden runtime fallback.

Required properties:

```text
- admissible for every well-formed local block with authorized relations;
- appears in the deterministic declared ladder;
- reads only authorized local relations and declared child messages;
- keeps only exported target/separator variables;
- never exports coordinate roots or full coordinate RUR;
- never routes local exhaustion to CertifiedNonFiniteTargetImage;
- returns verified target/separator relations or an allowed failure with cost trace.
```

The implementation may use local Groebner/F4, TargetRelationSearch escalation, sparse resultant, or specialization-interpolation, but only as declared strategies with resource bounds and exact certificates.

---

## FCR-010 — Real DAG/block replay is mandatory for core readiness

Synthetic all-relations replay is insufficient. Core run replay must verify each `ProjectionMessage` against its actual `ProjectionBlock` from the `TargetProjectionDAG`.

Replay must check:

```text
- DAG hash;
- block authorization hash;
- relation ids and relation hashes authorized for that block;
- child message dependencies matching DAG edges;
- plan hash and kernel kind;
- exported variable subset;
- relation generators in Q[exports];
- variant-specific exact certificate;
- support verification;
- squarefree/root/candidate binding.
```

A message that verifies only because replay reconstructed a synthetic block containing all relations must fail.

---

## FCR-011 — Candidate cover and finite candidate narrowing

Successful candidate-cover mode must return:

```text
status = CertifiedCandidateCover
support_polynomial = Some(S(T)), S != 0
squarefree_support_polynomial = Some(squarefree(S))
root_isolation = exact rational isolating intervals for real roots, possibly empty
decoded_candidates = one per isolated real root, possibly empty
projection_messages = nonempty verified production messages
certificate = Some(CoreRunCertificate)
cost_trace = complete algebraic cost trace
```

If `S(T)` has no real roots, the result is an empty real candidate cover, not AlgorithmicHardCase.

---

## FCR-012 — Nonfinite certification discipline

`CertifiedNonFiniteTargetImage` requires a positive proof:

```text
- exact dimension certificate;
- algebraic-dependence certificate;
- regular-chain dimension/projection certificate;
- Groebner/elimination certificate proving I ∩ Q[T] = {0};
- another explicitly implemented positive proof approved by this spec.
```

The following must never route to `CertifiedNonFiniteTargetImage`:

```text
- no relation found;
- degree/resource exhaustion;
- sparse/resultant/interpolation failure;
- Universal local stage failure;
- composition failed to produce target-only support;
- bounded rational witness search failure.
```

---

## FCR-013 — Acceptance is public-pipeline based

Final repair acceptance cannot be based on module-only helper tests. Every required support-producing stress case must exercise at least:

```text
RationalTargetProblem
-> validate/canonicalize/compress
-> graph/DAG or explicit valid block from the graph path
-> deterministic planner/admission
-> declared KernelPlan
-> kernel execution
-> ProjectionMessage verification
-> composition or direct root support assembly
-> global support verification
-> squarefree/root/decode
-> TargetSolveResult through public or near-public pipeline
```

For final `CANDIDATE_COVER_CORE_READY`, the public `api::solve_target` path must be used.

---

## FCR-014 — Required general algebraic coverage

The repair must demonstrate generic algebraic coverage, not named toy examples. The required suite must include at least:

```text
1. no initial target-only relation;
2. multivariate finite quotient/action basis, not local-univariate plus alias;
3. nonlinear target expressions in quotient variables;
4. multiple eliminated variables and multiple exported separators;
5. sparse resultant/eliminant where target support is not initially present;
6. specialization-interpolation with declared sample schedule and exact verification;
7. guarded rational affine preprocessing preserved through support construction;
8. target-independent components preserved as feasibility obligations;
9. no useful separator one-large-block execution;
10. non-real support with empty real candidate cover;
11. certified nonfinite only with positive proof;
12. resource-bounded hard cases that do not become Unsupported or NonFinite.
```

Variable names, relation order, and ids must be permuted across cases. Production code must not inspect case ids or expected answers.

---

## FCR-015 — Completion claim

This repair may pass only if all of the following are true:

```text
- no inappropriate limited production implementation remains;
- the public pipeline is connected;
- every mandatory kernel either implements the v4 generic contract or is not used for completion, with final completion failing if the v4 kernel is mandatory;
- UniversalTargetElimination can handle no-separator one-large-block input target-directly;
- actual DAG/block replay is the main replay path;
- stress suite A support-producing cases all return CertifiedCandidateCover with nonzero support;
- no support-producing acceptance case returns AlgorithmicHardCase, CertificateDesignGap, FiniteResourceFailure, CertifiedNonFiniteTargetImage, placeholder roots, or placeholder candidates;
- exact Q certificate/replay verifies all produced supports;
- root isolation and candidate decode are exact;
- final CLOSURE.md states CANDIDATE_COVER_CORE_READY and explicitly keeps exact-image completion separate unless P13 is complete.
```


---

## FCR-016 — Agent operating doctrine; phase gates are not the goal

The implementing Agent must treat this repair as a scientific-algorithm implementation task, not as a paperwork or gate-satisfaction task.

The Agent must keep the following invariant active in every implementation decision:

```text
The repository is acceptable only when the production solver core implements the R-GDTPK-Q candidate-cover algorithm described by the v4 specification.
A phase review, YAML schema, test count, file presence, or limited success case is never sufficient evidence by itself.
```

The Agent must stop and raise `PlanDefect` or `AlgorithmDefect` instead of proceeding when it finds any of the following:

```text
- the current plan would permit a narrow algebraic shape as the only support-producing path;
- the current plan would keep a non-production implementation in a production route;
- a kernel's admission or planner path is acting as an entrance ticket for a fixed pattern rather than a generic algebraic mechanism;
- a reviewer prompt asks mainly for evidence consistency and not algorithmic sufficiency;
- a proposed shortcut would make tests pass without implementing the v4 algorithm;
- the Agent cannot explain how a support-producing success is obtained from the v4 pipeline rather than from a helper or example-specific route.
```

The Agent must not interpret this repair as permission to implement only the listed stress cases. The stress cases are lower-bound witnesses against slice behavior, not the full scope.

---

## FCR-017 — Source-spec fidelity and scope boundary

This repair is source-bound to the supplied v4 specification. The v4 specification defines a solver core over Q-polynomial target systems, not a geometry-family handler. It requires the top-level pipeline from validation through final certificate, exact-Q projection messages, deterministic planning, no hidden fallback, no coordinate solution list, no full coordinate RUR, no geometry/problem/expected-answer dispatch, and no narrow-slice completion.

The repair's completion target is the finite-candidate narrowing layer:

```text
CertifiedCandidateCover over the generic R-GDTPK-Q production pipeline.
```

This repair does not by itself close exact-image semantics. However, it must not break or fake the exact-image API/data structures required by v4. `EXACT_IMAGE_CORE_READY`, `SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC`, and `RGDTPK_Q_V4_ACCEPTANCE_COMPLETE` remain forbidden until the real-fiber/slack/guard semantics phase is implemented and reviewed.

If any text in this repair pack conflicts with the v4 specification, the stricter source-faithful interpretation wins. A claim may be lowered; a required v4 algorithmic obligation may not be dropped.

---

## FCR-018 — Complete v4 function and module compliance map is mandatory

The Agent must produce and satisfy a source-to-code compliance map covering all modules and functions in v4 sections 6 through 29.

For every required file/function, the map must state:

```yaml
spec_section: <v4 section id>
required_path: <file::function/type>
current_path: <implemented path or MISSING>
production_required_for_candidate_cover: true|false
exact_image_only_or_later_phase: true|false
implementation_status: implemented|incorrect|missing|quarantined|test_only
algorithmic_notes: <what it actually does>
certificate_or_replay: <exact|missing|not_applicable>
repair_action: keep|implement|replace|remove_from_production|quarantine|defer_to_P13_exact_image
```

A file existing with a similarly named function is not sufficient. The implemented function must match the v4 input, processing, output, and invariant. Missing candidate-cover-path functions block this repair. Exact-image-only functions may be deferred only when the map explicitly marks them as P13 responsibilities and verifies that no exact-image final claim is made.

---

## FCR-019 — No specialized kernel may stand in for the generic pipeline

The v4 kernel list is a set of production mechanisms selected by a deterministic planner. Specialized kernels may have algebraic admission conditions, but they are not a license to make the whole solver a collection of admitted slices.

The final candidate-cover core must satisfy both of the following:

```text
1. Every specialized kernel used in production implements its v4 contract for its admitted algebraic class.
2. Every well-formed target block still has a generic target-direct path through TargetRelationSearch and/or UniversalTargetElimination, with declared bounds, exact certificates, and allowed failure semantics.
```

The following are explicit failures:

```text
- TargetActionKrylov admits only target-only or local-univariate-plus-linear-alias systems and is treated as closed.
- SparseResultant is only binary pair chaining and is treated as the v4 sparse resultant kernel.
- NormTrace is only one or two hand-detected towers and is treated as generic norm/trace coverage.
- RegularChain is only a triangular-shape recognizer and is treated as generic regular-chain projection.
- UniversalTargetElimination delegates to an unbounded global elimination or to a narrow subkernel and calls that generic.
```

---

## FCR-020 — Reviewer failure is a specification failure, not an excuse

If a reviewer would have passed a phase despite a production narrow-slice implementation, the reviewer prompt is defective. The Agent must repair the prompt and rerun the review. Review PASS is not evidence when the prompt failed to ask the right algorithmic question.

Each reviewer must be forced to answer:

```text
What algebraic input class does this production code actually handle?
What nontrivial algebraic input class from the v4 contract would fail?
Is that failure an allowed resource/certificate failure, or a partial-implementation defect?
Does this phase make the final R-GDTPK candidate-cover core more complete, or merely more documented?
```

A reviewer response that does not answer these questions is invalid even if `review_status: PASS` appears in YAML.
