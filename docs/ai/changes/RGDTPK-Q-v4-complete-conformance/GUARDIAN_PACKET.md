# Guardian Packet: R-GDTPK-Q / ACCTP-Q v4 完全準拠修正

Status: original imported archive, superseded for execution by the finite candidate-cover scope
amendment.

Current executable contract:

1. `SCOPE_AMENDMENT_FINITE_CANDIDATE_COVER.md`
2. `BASE_SPEC.md`
3. `PLAN.md`
4. `SOURCE_MAP.md`
5. `REVIEWER_PROMPTS.md`

This combined archive preserves the zip contents for traceability. Its full-conformance wording is
not the current implementation target unless the active Base Spec and Plan are amended again.


---

# Change Base Spec: R-GDTPK-Q / ACCTP-Q v4 完全準拠修正

## Context Packet

Spec ID: `RGDTPK-Q-v4-complete-conformance`  
Type: Change Base Spec  
Status: Draft for user approval  
Parent: None for this packet. It may later be registered as an Area Base Spec for `geosolver-core`.  
Scope: `geosolver-core` solver core only.  
Applies To: `geosolver-core/src/**`, `geosolver-core/Cargo.toml`, `geosolver-core/README.md`, and conformance evidence under `docs/ai/changes/RGDTPK-Q-v4-complete-conformance/**`.  
Required Parent R-IDs: None.  
Original Source Authority: `docs/ai/sources/geosolver_core_r_gdtpk_q_algorithm_spec_v4.md`, blob sha `ef108f0dc95880d2e3030c96872b9073be995274`.  
Blocking Questions: None. If any ambiguity is found during implementation, dependent work must stop and this Base Spec must be amended before code changes continue.  
Non-blocking Debt: None admitted.  
Known Exceptions: None.  
Allowed Final Claims after full completion and reviewer PASS: `SOURCE_FAITHFUL`, `ACCEPTANCE_COMPLETE`, `VERIFIED` for this Change Base Spec only.  
Forbidden Claims before full completion: `PRODUCTION_SAFE`, `BENCHMARK_PROVEN`, `EXACT_IMAGE_COMPLETE` unless every relevant R-ID below has source-bound evidence and reviewer approval.  
Context Packet Authority: non-authoritative digest. The R-ID body below is the authority.

---

## 1. Source authority and zero-adaptation rule

### BS-R000 — Exact source fidelity

The implementation must be a faithful production implementation of **GeoSolver Core Algorithm Specification v4.0 / R-GDTPK-Q / ACCTP-Q** at `docs/ai/sources/geosolver_core_r_gdtpk_q_algorithm_spec_v4.md`, blob sha `ef108f0dc95880d2e3030c96872b9073be995274`.

Every normative statement in the source specification is incorporated into this Base Spec. The implementation must not reinterpret source sections for convenience. If this Base Spec and the original source differ, the original source wins and this Base Spec must be amended immediately before further implementation.

No requirement below is optional unless it is explicitly marked `OUT_OF_SCOPE`. There are no `OUT_OF_SCOPE` R-IDs in this Change Base Spec.

Acceptance:
- The final code must implement the file structure, public types, public functions, pipeline order, kernel list, certificate semantics, failure statuses, and completion conditions stated in the original source.
- The final review must compare source section by section against implementation, not just run tests.
- The final closure must include a source-to-code conformance matrix.

Forbidden:
- Passing only existing tests while leaving non-production shortcuts.
- Treating comments, old plans, previous generated specs, or current implementation behavior as authority over the source specification.
- Replacing a source algorithm with a smaller algorithm unless the source explicitly allows it.
- Claiming source fidelity when a source section is merely stubbed, aliased, or handled only for fixtures.

---

## 2. Overall solver semantics

### BS-R001 — Algebraic problem domain

Input is an arbitrary well-formed rational polynomial target system

```text
F = {f1,...,fm} ⊂ Q[x1,...,xn,T]
T = target variable
```

The solver core must treat all inputs as Q-polynomial systems. Variable roles are provenance only and must never control algorithmic dispatch.

Acceptance:
- `RationalTargetProblem` has the source fields: variables, target, equations, semantic encodings, variable roles, and input hash.
- `validate_input` rejects only malformed Q-polynomial input or invalid semantic references.
- It must not reject because the input lacks a geometry label, has branch/slack variables, has many variables, is not univariate, or is not affine.

Forbidden:
- `Unsupported` or equivalent status for well-formed Q-polynomial systems.
- Any branch on geometry name, problem name, fixture name, expected answer, variable role, or source DSL label.

### BS-R002 — Candidate cover is the mandatory core output

For finite candidate-cover mode, the solver must output a nonzero univariate support polynomial

```text
S(T) ∈ Q[T], S(T) != 0
```

such that every true target value is a root of `S`. Spurious roots are allowed in `CertifiedCandidateCover`.

Acceptance:
- On success without exact-image mode, status is `CertifiedCandidateCover`.
- `support_polynomial`, `squarefree_support_polynomial`, exact real root isolation records, decoded candidates, projection messages, run certificate, diagnostics, and cost trace are populated according to the source specification.
- `S(T)` must be exact over Q and must pass global support verification.

Forbidden:
- Returning numerical approximations instead of an exact support polynomial.
- Returning target values that are not bound to a support hash and root index.
- Returning coordinate solutions before or instead of target candidates.

### BS-R003 — Exact target image remains architecturally present

Exact image mode is a later classification stage, but the architecture, API, data structures, and specified real-fiber semantics must exist. If exact-image mode is requested, the solver must perform real fiber / guard / slack semantics handling as specified, or return `CertificateDesignGap` only for a certificate language limitation that is explicitly recorded and source-justified.

Acceptance:
- `fiber/exact_image.rs`, `fiber/hermite.rs`, `fiber/thom.rs`, and `fiber/slack_semantics.rs` implement the source APIs.
- Candidate-cover computation must not depend on exact-image filtering.
- Exact-image classification must not silently accept mixed coordinate fibers without a certificate.

Forbidden:
- Treating exact-image mode as a no-op.
- Returning `CertifiedExactTargetImage` without real-fiber evidence.

---

## 3. Non-negotiable invariants

### BS-R010 — Core invariant set I1-I10

Every pipeline step and kernel must preserve the source invariants:

```text
I1. Input is treated as a Q-polynomial system.
I2. No geometry-name dispatch.
I3. Production path does not construct coordinate solution lists.
I4. Each block exports only target/separator variables.
I5. Each exported relation has an exact Q certificate.
I6. Final S(T) is exactly verified as a candidate cover.
I7. Exact-image mode verifies real fiber / slack / guard semantics.
I8. Failure returns the blocking algebraic cost trace.
I9. Planner is deterministic.
I10. Hidden fallback is impossible.
```

Acceptance:
- `CoreInvariantFlags` in the run certificate must include and truthfully bind these invariants.
- Static audit scripts must scan production code for forbidden coordinate-root/RUR/QE/CAD/fixture-dispatch paths.
- Reviewer must inspect the controlling execution paths, not only tests.

Forbidden:
- Runtime fallback that is not in `KernelPlan.declared_ladder`.
- Test-only proof of an invariant.
- A function named as if it implements a source algorithm but internally calling a smaller unrelated method without diagnostic and source permission.

### BS-R011 — Failure statuses

The only permitted non-success statuses for well-formed input are:

```text
FiniteResourceFailure
AlgorithmicHardCase
CertificateDesignGap
ImplementationBug
InvalidInput
```

Acceptance:
- `FiniteResourceFailure` must include stage, block id when known, matrix rows/cols, density when known, quotient rank estimate when known, coefficient height bits when known, and memory/work evidence when known.
- `AlgorithmicHardCase` must include stage, algebraic reason, and minimal block hash.
- `CertificateDesignGap` must identify the constructed object hash and missing certificate kind.

Forbidden:
- `Unsupported`.
- A generic failure that hides matrix size, rank, degree, or coefficient height when those were known.

---

## 4. Required folder and module contract

### BS-R020 — Fixed folder structure

The repository must contain the source-specified `geosolver-core` structure:

```text
geosolver-core/
  Cargo.toml
  README.md
  src/
    lib.rs
    api.rs
    types/{mod.rs,ids.rs,rational.rs,monomial.rs,polynomial.rs,univariate.rs,matrix.rs,interval.rs,hash.rs}
    problem/{mod.rs,input.rs,semantic.rs,validate.rs,canonicalize.rs,context.rs}
    algebra/{mod.rs,monomial_order.rs,polynomial_ops.rs,modular.rs,crt.rs,rational_reconstruction.rs,sparse_matrix.rs,dense_matrix.rs,linear_solve.rs,normal_form.rs,groebner.rs,f4.rs,elimination.rs,resultant.rs,interpolation.rs,quotient.rs,krylov.rs,regular_chain.rs,norm_trace.rs,real_root.rs,sign.rs}
    preprocess/{mod.rs,compression.rs,linear_affine.rs,definitional.rs,binomial.rs,saturation.rs,independent.rs}
    graph/{mod.rs,hypergraph.rs,influence.rs,weighted_primal.rs,separators.rs,tree_decomposition.rs,projection_dag.rs,metrics.rs}
    planner/{mod.rs,cost_model.rs,probes.rs,admission.rs,kernel_plan.rs,ladder.rs,planner.rs}
    kernels/{mod.rs,traits.rs,target_univariate.rs,linear_affine.rs,target_relation_search.rs,sparse_resultant.rs,action_krylov.rs,universal_elimination.rs,regular_chain_projection.rs,norm_trace_projection.rs,specialization_interpolation.rs}
    compose/{mod.rs,message.rs,compose.rs,separator_elimination.rs,final_support.rs}
    verify/{mod.rs,certificates.rs,verify_message.rs,verify_support.rs,replay.rs,run_certificate.rs}
    roots/{mod.rs,squarefree.rs,isolate.rs,decode.rs,algebraic_number.rs}
    fiber/{mod.rs,exact_image.rs,hermite.rs,thom.rs,slack_semantics.rs}
    result/{mod.rs,status.rs,diagnostics.rs,cost_trace.rs,output.rs}
    solver/{mod.rs,options.rs,pipeline.rs,orchestrator.rs}
```

Acceptance:
- All listed modules exist in production, not only in tests.
- `lib.rs` exposes only the source module tree and must not contain solver logic.
- Missing files are blocking noncompliance unless the source itself is amended.

Forbidden:
- Moving source-required APIs into ad hoc test modules.
- Collapsing modules so that reviewer cannot map source sections to implementation.

---

## 5. Type and normalization requirements

### BS-R030 — Stable IDs and hashing

The ID and hash layer must expose and use stable `VariableId`, `RelationId`, `BlockId`, `PackageId`, `KernelPlanId`, and `Hash` types. All IDs after canonicalization must be deterministic.

Acceptance:
- `types/ids.rs` implements fresh ID generation and stable name-to-ID hashing.
- Every certificate-bearing structure has a reproducible hash.
- Hashes include enough binding data to prevent replay against a different source relation, block, plan, or support polynomial.

Forbidden:
- Hash placeholders.
- Omitted source relation hashes in message/certificate binding.

### BS-R031 — Rational, monomial, polynomial, and univariate canonical forms

Rationals must be normalized with positive denominator and zero as `0/1`. Monomials must be sorted by variable id with no zero exponents. Sparse polynomials must have sorted unique monomials and no zero coefficients. Univariate polynomials must be normalized and trimmed.

Acceptance:
- All arithmetic returns normalized values.
- `normalize_poly`, `clear_denominators_primitive`, `substitute_poly`, `degree_uni`, `gcd_uni`, `squarefree_part_uni`, and `eval_uni_q` match the source semantics.
- Constant nonzero canonicalization failure is handled by a specified status, not by panic.

Forbidden:
- Floating arithmetic in canonical algebra.
- Non-normalized terms crossing module boundaries.

### BS-R032 — Matrix and interval primitives

Sparse/dense matrices over Fp and Q vectors must support exact hashing, density, rank/nullspace, CRT reconstruction inputs, and rational interval operations.

Acceptance:
- `matrix_density` returns an exact rational or a deterministic exact representation in cost trace.
- `RationalInterval` constructors enforce `lo < hi`.
- Interval records used for candidates are exact rational intervals.

Forbidden:
- Float-only intervals or root approximations.

---

## 6. Problem, semantics, canonicalization, and context

### BS-R040 — Semantic encoding preservation

`problem/semantic.rs` must model real constraint provenance for `A>=0`, `A>0`, `A!=0`, branch choices, and other real constraints. Slack encodings must be verified for valid relation and variable references.

Acceptance:
- `register_slack_encoding`, `semantic_relations`, and `verify_semantic_references` exist and are production functions.
- Canonicalization and compression preserve semantic provenance and relation-id mapping.
- Exact-image mode can recover the intended real semantics.

Forbidden:
- Dropping semantic provenance during denominator clearing or compression.
- Treating slack equations as ordinary complex equations when exact image classification is requested.

### BS-R041 — Canonicalization contract

`canonicalize_system` must:
1. Compute canonical variable order with target-aware deterministic ordering.
2. Clear denominators primitively.
3. Normalize relations.
4. Remove zero relations with trace.
5. Return a source-defined failure for nonzero constant contradiction or semantic inconsistency.
6. Preserve semantic encodings and stable relation order/hash.

Forbidden:
- Factor splitting without component semantics.
- Geometry-driven rewriting.
- Forgetting denominator-clearing provenance when it affects certificate or semantics.

### BS-R042 — Solver context and resource meter

`SolverContext` must carry options, id counter, hash config, resource meter, diagnostics, and active route budget. Resource checks must be cooperative and must produce `FiniteResourceFailure` with stage and cost details.

Forbidden:
- Panicking on resource overflow.
- Ignoring route budgets inside expensive algebra loops.

---

## 7. Algebra engine requirements

### BS-R050 — Monomial order and polynomial operations

`algebra/monomial_order.rs` and `algebra/polynomial_ops.rs` must implement lex, grevlex, block order, elimination order, leading term, S-polynomial, reduction, and primitive content operations exactly over Q.

Acceptance:
- `elimination_order(Y,Z)` orders all eliminated variables above all keep variables.
- `reduce_by_set` returns enough quotient evidence for membership certificates.

### BS-R051 — Modular arithmetic, CRT, rational reconstruction

`algebra/modular.rs`, `crt.rs`, and `rational_reconstruction.rs` must provide deterministic prime selection avoiding forbidden denominators/leading coefficients, reduction Q→Fp, CRT combination, and rational reconstruction with optional height bound.

Acceptance:
- Prime sequence is deterministic.
- Reconstruction is followed by exact Q verification by callers.
- Modular evidence is never the final proof by itself.

Forbidden:
- Accepting modular equality as final certificate.
- Random nondeterministic prime selection.

### BS-R052 — Sparse/dense exact linear algebra

`sparse_matrix.rs`, `dense_matrix.rs`, and `linear_solve.rs` must implement exact modular row echelon, rank, nullspace, homogeneous and inhomogeneous solve plans, CRT/rational reconstruction, and rank profile stabilization.

Acceptance:
- `solve_homogeneous_modular` returns modular traces, rank, reconstructed candidate basis, and deterministic hashes.
- Callers must verify reconstructed relations over Q.
- Matrix size and density flow into cost trace.

Forbidden:
- Returning nullspace candidates without exact Q check.
- Fixture-specific vectors.

### BS-R053 — Normal form and membership certificates

`normal_form.rs` must implement normal form and exact membership certificate verification:

```text
g = Σ multiplier_i * relation_i
```

Acceptance:
- Certificate relation IDs are bound to the authorized relation list.
- Verification recomputes the exact Q identity.

### BS-R054 — Groebner and F4 local elimination

`groebner.rs` must implement local elimination only, never global coordinate-first solving. `f4.rs` must be a production module implementing F4-style batch reduction and `f4_elimination_local`, not a test-only or placeholder module.

Acceptance:
- `f4_reduce_batch` builds symbolic preprocessing matrices for reducer/target batches, performs modular row reduction, reconstructs Q reducers, and returns exact normal-form certificates.
- `f4_elimination_local` returns `LocalEliminationResult` with generators in Q[keep], membership certificates, and matrix size trace.
- `eliminate_to_keep_variables` can choose `F4EliminationLocal` from production code.

Forbidden:
- `#[cfg(test)]` F4 as the only F4 implementation.
- A function named F4 that delegates to naive Groebner without F4 matrix batching and evidence.
- Using Groebner/F4 to produce full coordinate lex parametrization.

### BS-R055 — Elimination dispatcher

`algebra/elimination.rs` must dispatch among Groebner, F4, relation search, and resultant strategies as specified. It must validate disjoint keep/eliminate sets, validate every generator is in Q[keep], and attach exact certificates.

Acceptance:
- `EliminationStrategy` includes production variants for `EliminationGroebnerLocal`, `F4EliminationLocal`, `TargetRelationSearchEscalated`, `ResultantIfSquareOrOverdetermined`, and `SpecializeProjectInterpolateVerify`.
- Strategy choice is planner-declared, not hidden at runtime.
- Result includes cost trace and certificate for every generator.

### BS-R056 — Resultant, interpolation, quotient, Krylov, regular-chain, norm/trace, roots, sign

Each algebra module must implement the source APIs as production code:
- `resultant.rs`: support sets, sparse resultant templates, modular computation, exact verification.
- `interpolation.rs`: deterministic specialization points, specialization, sparse coefficient interpolation, exact interpolation verification.
- `quotient.rs`: target-relevant quotient handle with no coordinate-root/RUR export.
- `krylov.rs`: block Krylov sequence, recurrence recovery, coverage certificate, annihilator verification.
- `regular_chain.rs`: local regular-chain decomposition, projection, component-union/intersection combination with guard semantics.
- `norm_trace.rs`: algebraic tower detection, norm computation, exact norm verification.
- `real_root.rs`: exact Sturm and exact Descartes/Vincent isolation.
- `sign.rs`: sign at algebraic root and Thom encoding.

Forbidden:
- Descartes root isolation implemented as a silent call to Sturm.
- Regular-chain as only grouping by main variable with no guard/component verification.
- Norm/trace as only a fixture-shaped detector.
- Quotient handles that expose full coordinate bases, coordinate roots, or coordinate RUR.

---

## 8. Preprocessing requirements

### BS-R060 — Compression order and safety

`pre_kernel_compress` must run in the source order:

```text
1. definitional elimination
2. linear affine elimination
3. binomial / monomial simplification
4. safe saturation for explicitly nonzero encodings
5. target-independent component marking
6. coefficient height and monomial count trace
```

Acceptance:
- Every rewrite records a compression trace and preserves certificate-relevant provenance.
- Unsafe transformations return rejection diagnostics, not silent simplification.

Forbidden:
- Geometry-name rewrite.
- Expected-answer factor selection.
- Component splitting without union/guard semantics.

### BS-R061 — Definitional, affine, binomial, saturation, independent components

The modules `definitional.rs`, `linear_affine.rs`, `binomial.rs`, `saturation.rs`, and `independent.rs` must implement the source-detailed functions and constraints.

Acceptance:
- Definitional elimination only handles `y - p(X)=0` or `c*y - p(X)=0`, target excluded.
- Linear affine substitution requires constant nonzero denominator or recorded nonzero guard.
- Binomial simplification is reversible or guard/certificate recorded.
- Saturation is allowed only for explicit nonzero encodings.
- Target-independent components may be removed from candidate-cover construction only with feasibility obligations retained for exact-image mode.

---

## 9. Graph, decomposition, and projection DAG

### BS-R070 — Algebraic graph construction

The graph layer must build:
- relation-variable hypergraph,
- target influence graph,
- weighted primal/projection graph,
- separator candidates,
- target-rooted decomposition,
- projection DAG,
- structural metrics.

Acceptance:
- Target influence uses BFS in the bipartite graph from target.
- Weighted graph uses degree participation, monomial count, coefficient height, target distance, linear eliminability, and occurrence count.
- Decomposition tries articulation, min-fill, and bounded min-cut separators before falling back to one large block.
- No useful separator must not fail; it must produce one large target block.

### BS-R071 — Projection DAG authorization

Each relation belongs to exactly one block unless a duplication certificate exists. Each block can read only relations and child messages authorized by its hash. DAG validation must prove coverage, parent-child consistency, authorization hashes, duplication certificates, and root presence.

Forbidden:
- Omitting a compressed relation from the DAG.
- Duplicating relations without certificate.
- Allowing a kernel to read relations not bound to its block authorization hash.

---

## 10. Planner and ladder requirements

### BS-R080 — Deterministic cost model and probes

Planner must run cost probes for each block and estimate kernel costs deterministically. Probes can guide planning but cannot be correctness proof.

Acceptance:
- Cost estimates include matrix size, quotient rank, coefficient height, separator degree, certificate cost, and hash.
- Probe results are not used as final proof.

### BS-R081 — Kernel admission

`collect_kernel_admissions` must call every kernel's admission. UniversalTargetEliminationKernel must be admissible for any well-formed block with Q-polynomial relations.

Acceptance:
- Admission false is not a runtime failure.
- Every admission record carries exported variables, eliminated variables, initial bounds, estimated matrix/template size, execution plan when admitted, and admission hash.
- A block with relations and no admitted Universal kernel is `ImplementationBug`, not `AlgorithmicHardCase`.

### BS-R082 — Declared ladder, no hidden fallback

`KernelPlan.declared_ladder` must be fully determined before execution and must enter the run certificate. Only certificate-available, coordinate-free kernels can appear. UniversalTargetEliminationKernel appears last unless explicitly selected by user priority as still-declared plan.

Acceptance:
- Every route has resource, degree, matrix, coefficient-height, and failure behavior budgets.
- `execute_block_with_declared_ladder` may only try routes listed in the ladder.
- Fallback inside a kernel is allowed only if it is explicitly part of that kernel's declared support plan and certificate.

Forbidden:
- A kernel catching failure and silently invoking an undeclared different kernel.
- Test-only ladder entries.
- Planner selecting a route without a certificate route.

---

## 11. Kernel requirements

### BS-R090 — Common kernel contract

All kernels implement:

```rust
pub trait TargetProjectionKernel {
    fn kind(&self) -> KernelKind;
    fn admit(&self, block: &ProjectionBlock, ctx: &KernelContext) -> KernelAdmission;
    fn plan(&self, admission: &KernelAdmission, ctx: &KernelContext) -> Result<KernelExecutionPlan, SolverError>;
    fn execute(&self, plan: &KernelExecutionPlan, ctx: &mut KernelContext) -> Result<ProjectionMessage, SolverError>;
    fn replay(&self, message: &ProjectionMessage, ctx: &KernelContext) -> ReplayResult;
}
```

Acceptance:
- `all_kernels()` returns exactly:
  1. TargetUnivariateKernel
  2. LinearAffineKernel
  3. TargetRelationSearchKernel
  4. SparseResultantProjectionKernel
  5. TargetActionKrylovKernel
  6. NormTraceProjectionKernel
  7. RegularChainProjectionKernel
  8. SpecializationInterpolationKernel
  9. UniversalTargetEliminationKernel
- Every `execute` validates plan binding, block authorization, source relations, child messages, exported-variable subset, and certificate route.
- Every message contains exact Q certificate and cost trace.

Forbidden:
- Kernel returns coordinate roots or full coordinate RUR.
- Kernel creates support relation without certificate.

### BS-R091 — TargetUnivariateKernel

Admission occurs when a block or child message has a nonzero relation in Q[T]. Execution must compute primitive LCM or squarefree-compatible support from all target-only relations and return `PrincipalSupport` with `CandidateCoverStrong`.

Acceptance:
- It must consider child message relations as source allows.
- Certificate is source membership bound to relation IDs/message hashes.

### BS-R092 — LinearAffineKernel

LinearAffineKernel must eliminate triangular affine variables only with safe pivots. Nonconstant denominators require recorded nonzero guards.

Acceptance:
- Incomplete affine elimination returns `AlgorithmicHardCase`.
- Unsafe pivot in plan is `ImplementationBug`.

Forbidden:
- Dividing by variable-dependent denominator without guard.
- Dropping denominator guard.

### BS-R093 — TargetRelationSearchKernel

This is the generic target-direct workhorse. For local ideal `J=<f_i>⊂Q[Y,Z]`, it must find nonzero `g(Z)∈J∩Q[Z]` by constructing the linear identity

```text
g(Z) = Σ_i q_i(Y,Z) f_i(Y,Z)
```

over declared export and multiplier supports, solving homogeneous modular systems, reconstructing candidates, and exact Q verifying membership.

Acceptance:
- Admission is possible for all blocks with relations; planner may delay or cost-prohibit but must report algebraic cost.
- DenseTotalDegree, SparseFromProjectionFootprint, and SpecializedInterpolationFootprint export support strategies exist.
- Multiplier supports are deterministic and include all monomials required by coefficient comparison.
- `TargetRelationSearchCertificate` includes exported/eliminated variable hashes, support hashes, membership matrix hash, primes used, rational reconstruction hash, relation hash, multipliers hash, and exact identity hash.
- Exact Q membership is the only success oracle.

Forbidden:
- Returning candidate from modular nullspace without exact identity check.
- Refusing all non-sparse or non-small blocks as unsupported.
- Producing only first relation from a fixture-known matrix.

### BS-R094 — SparseResultantProjectionKernel

SparseResultantProjectionKernel must build support/template estimates and compute target/separator eliminants without coordinate roots. It must accept only when exact verification method is available; otherwise admission is false and another kernel handles the block.

Acceptance:
- Template input can represent the source-specified support-set resultant/eliminant route, not only hard-coded fixture cases.
- Resultant computation can use modular templates, determinant/null relation reconstruction, and exact verification.
- Output relation variables are subset of exported variables.
- Certificate records template, input supports, modular traces, relation hash, and exact verification hash.

Forbidden:
- `sparse enough` as unsupported.
- Resultant relation accepted without exact verification.
- A two-polynomial-only implementation being presented as complete if the source calls for general support template estimates.

### BS-R095 — TargetActionKrylovKernel

When finite-rank target-relevant quotient/action is cheap, compute an annihilating polynomial of multiplication by target without coordinate roots/RUR.

Acceptance:
- Quotient handle must not expose coordinate roots, coordinate solution list, full coordinate RUR, or target-unrelated full quotient basis.
- Coverage proof must use one of the source-permitted coverage methods.
- If coverage cannot be proved, no candidate polynomial may be returned.
- Annihilator is verified by `verify_annihilator`.

Forbidden:
- Single Krylov sequence without coverage certificate.
- Treating characteristic support estimate as proof.

### BS-R096 — UniversalTargetEliminationKernel

UniversalTargetEliminationKernel is a planned generic target/separator projection kernel, not an undeclared fallback.

Admission:
```text
block has Q-polynomial relations -> admitted
```

Execution:
- Build `Y = local variables - exported variables`, `Z = exported variables`, `J = local relations plus child messages`.
- Use planner-declared `EliminationStrategy`.
- Extract all nonzero primitive generators in Q[Z].
- Verify every generator exactly against J.
- Return `GeneratorSet` with `ExactProjectionIdeal` or `CandidateCoverStrong`.

Allowed internal strategies, exactly as source section 20.4:
```text
- EliminationGroebnerLocal
- F4EliminationLocal
- TargetRelationSearchEscalated
- ResultantIfSquareOrOverdetermined
- SpecializeProjectInterpolateVerify
```

Acceptance:
- The internal strategy list is planner-declared and certificate-bound.
- F4 is production, not test-only.
- `gens.empty` attempts certified nonfinite projection only if nonfinite certificate exists; otherwise `AlgorithmicHardCase`.

Forbidden:
- Adding extra undeclared inner strategies because they are convenient.
- Calling regular-chain, norm/trace, or target-action internally unless the Base Spec is amended; those are separate ladder kernels in the source.
- Hidden fallback.

### BS-R097 — RegularChainProjectionKernel

RegularChainProjectionKernel must preserve triangular component, guard, and projection semantics.

Acceptance:
- `local_regular_chain_decomposition` constructs a ComponentDAG with source relation hashes, guards, component semantics, regularity evidence, and projection certificates.
- `project_chain_to_variables` proves the projected generators are valid for keep variables.
- Component union/intersection combination is certified.

Forbidden:
- Merely grouping by main variable and calling it regular-chain complete.
- Ignoring guards.
- Returning combined relations without per-component projection certificates.

### BS-R098 — NormTraceProjectionKernel

NormTraceProjectionKernel detects explicit finite algebraic tower by algebraic form and computes norm relation.

Acceptance:
- Detection never uses geometry labels.
- Norm computation uses exact resultant/norm algorithms and verifies with `verify_norm_relation`.
- Output is target/separator-only.

Forbidden:
- Fixture-shaped tower detection only.
- Unverified norm relation.

### BS-R099 — SpecializationInterpolationKernel

For non-target separators, specialization/interpolation may generate a candidate relation, but success is allowed only after exact Q verification.

Acceptance:
- Sample points deterministic and certificate-bound.
- Inner kernel plans are declared, not ad hoc hidden fallback.
- Interpolation support is declared and hashed.
- Final `verify_interpolated_relation_by_membership_or_elimination` is exact over Q.

Forbidden:
- Accepting interpolation from samples alone.
- Choosing sample points based on expected answer.

---

## 12. Composition, final support, and verification

### BS-R110 — Projection message composition

Composition must merge projection messages along `TargetProjectionDAG` and eliminate remaining separators using message relations only.

Acceptance:
- `compose/message.rs` implements `MessageIdeal`, `message_to_relations`, and `merge_messages`.
- `compose_projection_messages` works postorder to root.
- `eliminate_remaining_separators` creates a pseudo block from message relations only and uses target-direct kernels.
- No original coordinate variables are reintroduced during separator elimination.

Forbidden:
- Rebuilding and globally solving the original coordinate system during composition.
- Bounded heuristic that says no target eliminant merely because the matrix is large without returning resource evidence.

### BS-R111 — Final support polynomial

`build_global_support_polynomial` must:
1. Collect target-only root relations.
2. Convert to univariate.
3. Compute primitive LCM, not unnecessary product when LCM is available.
4. Return normalized nonzero S(T).
5. If target-only is empty, either certify nonfinite target image or return `AlgorithmicHardCase`.

Acceptance:
- `S(T)` is exact Q[T].
- Target-only relations are source-bound to verified projection messages.

Forbidden:
- Producing support from unverified relation.
- Returning zero support.

### BS-R112 — Support verification

`verify_global_support` proves that `S(T)` vanishes on all true target fibers.

Acceptance:
- Target-only root relation product/LCM route is verified.
- Composed ideal membership route has exact multipliers and exact identity.
- No certificate means `CertificateDesignGap`, not success.

### BS-R113 — Run certificate and replay

Run certificate must bind input hash, canonical system hash, target variable, compression hash, hypergraph hash, DAG hash, kernel plan hashes, projection message hashes, support hash, squarefree support hash, root isolation hash, decoded candidate hash, optional fiber classification hash, and invariant flags.

Acceptance:
- `verify/replay.rs` implements replay against the original problem.
- `replay_run_certificate` must recompute hashes and replay messages/support/root isolation.

Forbidden:
- Certificate that only records hashes without enough replay data.
- Review passing without replay evidence.

---

## 13. Roots, candidate decode, and exact-image

### BS-R120 — Squarefree support and exact root isolation

`roots/squarefree.rs` must reject zero support and compute squarefree part via derivative and gcd. `roots/isolate.rs` must implement exact isolation. `real_root.rs` must provide both Sturm and Descartes/Vincent algorithms as distinct exact methods.

Acceptance:
- Root records contain support hash, root index, and rational isolating interval.
- Descartes option must not silently call Sturm.
- Floating-only root finding is forbidden.

### BS-R121 — Candidate decode

`decode_candidates` binds every candidate to target variable, support hash, root index, isolating interval, and candidate hash.

Acceptance:
- Candidate order is deterministic.
- Candidate count equals root isolation count.

### BS-R122 — Real fiber classification

Exact image mode must add algebraic target condition, attach slack/guard semantics, decide real fiber nonempty, classify signs at algebraic target roots, and return certificates.

Acceptance:
- Hermite real root count is implemented for zero-dimensional fibers.
- Thom sign classification is implemented for guard/sign decisions.
- Slack semantics for `A>=0`, `A>0`, `A!=0`, and branch choice are interpreted from provenance.

---

## 14. Result, diagnostics, cost trace

### BS-R130 — Result status and output

`TargetSolveResult` must contain exactly the source fields and must not lose failure details.

Acceptance:
- Success and failure finalizers preserve diagnostics and cost trace.
- Candidate-cover mode explicitly diagnoses possible spurious roots.

### BS-R131 — Algebraic cost compression trace

Every run must record:
- total `n,m,d,s,h`,
- maximum block width `w`,
- maximum separator width `τ`,
- each block local variable count,
- each block relation count,
- each block estimated quotient rank,
- each block matrix size,
- each block coefficient height before/after,
- final support degree `δ`,
- certificate size `κ`.

Acceptance:
- These values are in `GlobalCostTrace`.
- They are certificate-bound where required.

---

## 15. Top-level solver pipeline

### BS-R140 — Pipeline order

`solve_target(problem, options)` must run exactly:

```text
0. ValidateInput
1. CanonicalizeSystem
2. PreKernelAlgebraicCompression
3. BuildRelationVariableHypergraph
4. BuildTargetInfluenceGraph
5. BuildWeightedProjectionGraph
6. BuildTargetProjectionDAG
7. PlanProjectionMessages
8. ExecuteLocalProjectionKernels
9. ComposeProjectionMessages
10. BuildGlobalSupportPolynomial
11. VerifyGlobalSupport
12. SquarefreeSupport
13. ExactRealRootIsolation
14. DecodeTargetCandidates
15. OptionalRealFiberClassification
16. FinalizeResultAndCertificate
```

Acceptance:
- `solver/pipeline.rs` exposes step functions for each stage.
- `solver/orchestrator.rs` calls them in order.
- Every step boundary has failure handling that returns `TargetSolveResult` status, not panic.

Forbidden:
- Reordering support verification after candidate decode.
- Skipping message verification before composition.
- Any direct global coordinate solve.

---

## 16. Completion criteria

### BS-R150 — Source-specified completion conditions

The implementation is complete only when all 17 source completion conditions hold:

```text
1. Every well-formed Q-polynomial target system enters the generic pipeline.
2. No geometry-name dispatch exists.
3. No problem-id / fixture-id / expected-answer dispatch exists.
4. TargetProjectionDAG is built for every valid input.
5. If no useful separator exists, one large block is sent to a generic target-direct kernel.
6. Every block receives a deterministic KernelPlan.
7. UniversalTargetEliminationKernel exists and returns target/separator-only output.
8. Production path does not construct full coordinate solution list.
9. Production path does not construct full coordinate RUR.
10. On success, S(T) is in Q[T] and passes exact Q verification.
11. Root isolation is exact.
12. Decoded candidates are bound to support hash and root index.
13. Exact image mode handles real fiber / guard / slack semantics.
14. Failures return evidence-backed status, not Unsupported.
15. Cost trace records every algebraic-cost-compression parameter.
16. Hidden fallback is impossible at the API level.
17. Narrow slice completion is impossible at the API level.
```

Acceptance:
- Reviewer must PASS each item individually with code evidence.
- Any single failed item blocks `ACCEPTANCE_COMPLETE`.

---

## 17. Explicit forbidden simplifications

These are blocking failures even if tests pass:

1. `algebra/f4.rs` exists only under `#[cfg(test)]`.
2. `isolate_real_roots_descartes` silently calls Sturm.
3. `UniversalTargetEliminationKernel` uses undeclared inner routes.
4. `UniversalTargetEliminationKernel` lacks production `F4EliminationLocal`.
5. `TargetRelationSearchKernel` accepts modular candidates without exact Q membership verification.
6. `SparseResultantProjectionKernel` is only two-polynomial fixture support but is claimed complete for source section 18.
7. `RegularChainProjectionKernel` is only grouping plus Groebner without regularity/guard/component proof.
8. `NonFiniteCertificate` is based only on small rational witness search when the source requires elimination/dimension/algebraic-dependence certificate.
9. `compose` uses fixed small heuristic bounds to decide no eliminant exists without returning resource-bound evidence.
10. Any production path constructs coordinate solution list, full coordinate RUR, full lex parametrization, QE/CAD fallback, or geometry-specific solver.
11. Any required public function panics or returns placeholder.
12. Tests check for strings or snapshots instead of algorithmic evidence and are used as the sole acceptance basis.
13. `CertificateDesignGap` is used where the source requires an implemented production certificate.
14. Any module or API is implemented only for named fixtures.

---

## 18. Required MECH entries

### MECH-01 — TargetRelationSearch exact membership mechanism

Supports: BS-R052, BS-R053, BS-R093, BS-R112  
Domain: algorithm, verification path  
Required: yes  
Semantics: Construct `g(Z)=Σq_i f_i` by exact modular linear algebra and Q verification.  
Inputs: local relations plus child messages, eliminated variables Y, exported variables Z, degree/support plan.  
Outputs: `ProjectionMessage` with relation generators and membership certificate.  
Oracle: `verify_membership_exact(g, qs, J)` over Q.  
Failure behavior: resource overflow -> `FiniteResourceFailure`; no relation within declared bounds -> `AlgorithmicHardCase`; missing certificate -> `CertificateDesignGap`.  
Required evidence: matrix construction trace, modular traces, reconstructed candidate trace, exact identity proof.

### MECH-02 — F4 local elimination mechanism

Supports: BS-R054, BS-R055, BS-R096  
Domain: algorithm, exactification path  
Required: yes  
Semantics: Production F4 batch reduction for local elimination to Q[keep].  
Inputs: relations, elimination order, F4 options.  
Outputs: local elimination generators, membership certificates, matrix traces.  
Oracle: normal-form/membership verification over Q.  
Failure behavior: resource overflow -> `FiniteResourceFailure`; no generator -> `AlgorithmicHardCase`.  
Required evidence: production code path not under `#[cfg(test)]`, matrix construction/reduction evidence, exact certificate verification.

### MECH-03 — Declared ladder and no hidden fallback mechanism

Supports: BS-R010, BS-R080, BS-R081, BS-R082, BS-R096, BS-R150  
Domain: planning, selection, verification path  
Required: yes  
Semantics: Every route tried during execution is declared before execution and certificate-bound.  
Inputs: kernel admissions, cost estimates, user priority.  
Outputs: `KernelPlan.declared_ladder`.  
Oracle: execution trace route hashes are subset of declared ladder hashes.  
Failure behavior: missing Universal route for relation block -> `ImplementationBug`.  
Required evidence: route plan hashes in run certificate and diagnostics.

### MECH-04 — Projection DAG and composition mechanism

Supports: BS-R070, BS-R071, BS-R110, BS-R111, BS-R112  
Domain: algorithm, verification path  
Required: yes  
Semantics: Local target/separator messages are composed along DAG; remaining separators are eliminated using message-only pseudo blocks.  
Inputs: `TargetProjectionDAG`, projection messages.  
Outputs: `ComposedProjection`, final support.  
Oracle: `verify_projection_message`, `verify_global_support`, replay.  
Failure behavior: unverified message -> `ImplementationBug` or certificate failure; no support -> nonfinite or `AlgorithmicHardCase`.

### MECH-05 — Exact root isolation and candidate binding mechanism

Supports: BS-R120, BS-R121, BS-R140, BS-R150  
Domain: exactification path  
Required: yes  
Semantics: Squarefree support and exact real root isolation by Sturm or Descartes/Vincent, then deterministic candidate decode.  
Inputs: nonzero `S(T)`.  
Outputs: squarefree support, isolating intervals, target candidates.  
Oracle: exact root-count/isolation certificates and support hash binding.  
Failure behavior: zero support -> `AlgorithmicHardCase`; isolation resource issue -> `FiniteResourceFailure`.

### MECH-06 — Real fiber / slack / guard semantics mechanism

Supports: BS-R003, BS-R040, BS-R122, BS-R150  
Domain: exactification path, semantics  
Required: yes  
Semantics: For exact-image mode, classify candidate roots by real fiber nonemptiness and encoded guard/slack semantics.  
Inputs: compressed system, support, candidates, semantic encodings.  
Outputs: fiber classification result and certificates.  
Oracle: Hermite real root count, Thom sign classification, slack consistency verification.  
Failure behavior: missing certificate -> `CertificateDesignGap`.

### MECH-07 — Algebraic cost trace mechanism

Supports: BS-R011, BS-R080, BS-R131, BS-R150  
Domain: diagnostics, verification path  
Required: yes  
Semantics: Bind algebraic cost compression parameters to result/certificate.  
Inputs: system metrics, block traces, matrix traces, support degree, certificate size.  
Outputs: `GlobalCostTrace`.  
Oracle: recomputation from compressed system, DAG, messages, certificates.  
Failure behavior: missing trace field for known value -> `ImplementationBug`.

---

## 19. Acceptance and verification requirements

### Acceptance A1 — Source-to-code conformance matrix

Final evidence must include a table mapping:
- Source section,
- Base Spec R-ID,
- implementation file/function,
- code evidence,
- test/evidence command,
- reviewer decision.

### Acceptance A2 — Static forbidden-path audit

A production audit script must scan all non-test Rust source for forbidden tokens and patterns:
- geometry dispatch tokens,
- fixture/expected answer dispatch,
- coordinate solution list/RUR construction in production,
- QE/CAD fallback,
- `todo!`, `unimplemented!`, placeholder returns,
- `#[cfg(test)]` being the only implementation of source-required algorithm,
- Descartes alias to Sturm,
- hidden fallback routes not present in plan.

Static audit is not sufficient for PASS, but failure blocks PASS.

### Acceptance A3 — Algorithmic behavior suites

Tests must include small but source-representative systems for:
- target-univariate,
- affine elimination with and without guard,
- dense relation search,
- sparse footprint relation search,
- sparse resultant,
- action Krylov with coverage,
- universal F4 local elimination,
- specialization-interpolation with exact verification,
- regular-chain with guard/component semantics,
- norm/trace tower,
- nonfinite target image certificate,
- Sturm and Descartes isolation,
- slack/guard exact-image mode.

Tests are supplementary. Reviewer must still inspect algorithms.

### Acceptance A4 — Fresh evidence

Every final claim must cite fresh commands, git state, changed files, and reviewer reports. Reviewer PASS alone is not evidence.

---

## 20. Reviewer blocking rules

A reviewer must return FAIL if:
- Any Base Spec R-ID is unimplemented, stubbed, test-only, or simplified.
- A Plan task claims closure by tests only without code inspection.
- A source-required algorithm is replaced by a smaller algorithm without explicit approved exception.
- A required certificate is only a hash with no replay or exact identity.
- A production route can return candidate cover without exact Q verification.
- Hidden fallback remains possible.
- Descartes is an alias to Sturm.
- F4 remains test-only.
- Universal internal strategy list differs from source section 20.4.
- Exact image mode silently accepts or rejects without the source-required semantics.



---

# Plan: R-GDTPK-Q / ACCTP-Q v4 完全準拠修正

## Context Packet

Spec ID: `RGDTPK-Q-v4-complete-conformance`  
Type: Plan Contract  
Status: Draft for user approval  
Base Spec: `BASE_SPEC.md` in this packet  
Original Source: `docs/ai/sources/geosolver_core_r_gdtpk_q_algorithm_spec_v4.md`, blob sha `ef108f0dc95880d2e3030c96872b9073be995274`  
Scope: full production conformance of `geosolver-core` to the v4 source specification  
Allowed Claim During Execution: `PARTIALLY_IMPLEMENTED` per completed phase only  
Allowed Final Claim After All Reviews: `SOURCE_FAITHFUL`, `ACCEPTANCE_COMPLETE`, `VERIFIED` for this Change Base Spec  
Implementation Permission: Not granted by this file. User must explicitly approve implementation after reviewing this Plan.  
Context Packet Authority: non-authoritative digest.

---

## 0. Plan rules

### P-RULE-1 — Base Spec supremacy

If this Plan conflicts with the Base Spec, the Base Spec wins. Stop and amend the Plan or Base Spec before proceeding.

### P-RULE-2 — No gate-only completion

A phase is not complete merely because tests pass. Each phase requires:
1. code implementation evidence,
2. source-to-code mapping,
3. negative shortcut audit,
4. behavior evidence,
5. phase reviewer PASS using the prompt in `REVIEWER_PROMPTS.md`.

### P-RULE-3 — No simplification by omission

A phase must not skip source-required APIs because the current code does not use them yet. If a source-required function is not needed by the current tests, it still must be implemented or explicitly blocked.

### P-RULE-4 — Test-first where practical

For behavior-changing algorithm work, create or update tests before or with implementation. Tests must check mathematical behavior, certificates, and failure evidence, not just names or snapshots.

### P-RULE-5 — Fresh evidence per phase

Each phase must write evidence under:

```text
docs/ai/changes/RGDTPK-Q-v4-complete-conformance/evidence/P<phase>/
```

Required evidence files per phase:
- `changed_files.txt`
- `source_to_code_map.md`
- `commands.log`
- `algorithm_evidence.md`
- `static_audit.log`
- `review_request.md`
- `review_result.md`

---

## Phase 0 — Source lock, current-gap audit, and conformance harness

Supports: BS-R000, BS-R010, BS-R150  
MECHs: MECH-03, MECH-07  
Reviewer Prompt: RP-P0

### Tasks

1. Create change directory:
   ```text
   docs/ai/changes/RGDTPK-Q-v4-complete-conformance/
     BASE_SPEC.md
     PLAN.md
     SOURCE_MAP.md
     REVIEWER_PROMPTS.md
     evidence/
     reviews/
   ```

2. Copy this Base Spec, Plan, Source Map, and Reviewer Prompts into that directory.

3. Add `docs/ai/changes/RGDTPK-Q-v4-complete-conformance/evidence/P0/source_lock.md` with:
   - source path,
   - source blob sha,
   - current git commit,
   - `git status --short`,
   - exact command used to read source.

4. Add a conformance-audit script:
   ```text
   geosolver-core/scripts/audit_v4_conformance.py
   ```
   The script must:
   - scan production Rust files, excluding tests only when checking production implementation,
   - report missing required files,
   - report `todo!`, `unimplemented!`, `panic!("TODO")`, placeholder strings,
   - report `#[cfg(test)]`-only source-required APIs,
   - report `isolate_real_roots_descartes` delegating to Sturm,
   - report forbidden coordinate/RUR/QE/CAD/geometry-dispatch tokens,
   - report `Unsupported` status or diagnostic in solver core,
   - report any `kernel_not_ready_error` in production route.

5. Write `current_gap_inventory.md` listing every known mismatch:
   - F4 test-only or non-production.
   - Descartes alias to Sturm.
   - Universal inner strategy list mismatch with source section 20.4.
   - Sparse resultant two-polynomial-only limitation if still present.
   - Regular-chain simplification if still present.
   - Nonfinite small witness limitation if still present.
   - composition bounded heuristic limitation if still present.
   - exact-image mixed fiber gaps if still present.

### Completion evidence

- `python geosolver-core/scripts/audit_v4_conformance.py --strict` runs and prints current failures.
- This phase can pass with failures reported, because it is an audit setup phase.
- Reviewer must verify the audit script is not written to ignore known failures.

---

## Phase 1 — Public API, type layer, hash binding, and normalization

Supports: BS-R020, BS-R030, BS-R031, BS-R032, BS-R130  
MECHs: MECH-07  
Reviewer Prompt: RP-P1

### Files

```text
geosolver-core/src/lib.rs
geosolver-core/src/api.rs
geosolver-core/src/types/ids.rs
geosolver-core/src/types/rational.rs
geosolver-core/src/types/monomial.rs
geosolver-core/src/types/polynomial.rs
geosolver-core/src/types/univariate.rs
geosolver-core/src/types/matrix.rs
geosolver-core/src/types/interval.rs
geosolver-core/src/types/hash.rs
geosolver-core/src/result/status.rs
geosolver-core/src/result/diagnostics.rs
geosolver-core/src/result/cost_trace.rs
geosolver-core/src/result/output.rs
```

### Implementation steps

1. Ensure `lib.rs` exports only the source modules. Remove solver logic from `lib.rs`.

2. Ensure `api::solve_target(problem, options)`:
   - constructs `SolverContext`,
   - calls `solver::orchestrator::solve_with_context`,
   - catches solver errors into `TargetSolveResult`,
   - does not panic for normal failures.

3. Extend `ids.rs`:
   - define all required ID types,
   - implement `IdCounter`,
   - implement fresh variable/relation/package/plan ID functions,
   - implement stable name-to-ID hashing with namespace.

4. Normalize rational arithmetic:
   - denominator always positive,
   - gcd normalized,
   - zero as `0/1`,
   - checked division.

5. Normalize monomials and polynomials:
   - sorted terms,
   - no zero coefficients,
   - no duplicate monomial,
   - deterministic hash after normalization.

6. Implement missing univariate operations:
   - exact derivative,
   - exact Euclidean gcd,
   - exact squarefree part,
   - exact division helper used by squarefree.

7. Ensure `matrix.rs` supports sparse/dense Q/Fp types, vector hashes, exact density, shape, and hash.

8. Ensure `interval.rs` validates rational intervals and includes disjoint/contains.

9. Ensure result types have every source-required field plus any already-existing backward-compatible fields. Backward-compatible extra fields are allowed only if they do not weaken source semantics.

10. Add unit/property tests:
    - rational normalization,
    - polynomial normalization idempotence,
    - clear denominators primitive,
    - univariate gcd/squarefree,
    - interval validation.

### Forbidden shortcuts

- No float arithmetic.
- No hash placeholders.
- No unimplemented source-required function.
- No panics for invalid rational denominator except through source-specified error conversion.

### Completion evidence

- `cargo test -p geosolver-core types result`
- `python geosolver-core/scripts/audit_v4_conformance.py --phase P1`
- Reviewer checks source-to-code map for every file in this phase.

---

## Phase 2 — Problem input, semantic provenance, canonicalization, context

Supports: BS-R001, BS-R040, BS-R041, BS-R042  
MECHs: MECH-06, MECH-07  
Reviewer Prompt: RP-P2

### Files

```text
problem/input.rs
problem/semantic.rs
problem/validate.rs
problem/canonicalize.rs
problem/context.rs
preprocess/compression.rs only for trace compatibility
```

### Implementation steps

1. `RationalTargetProblem`:
   - include `variables`, `target`, `equations`, `semantic_encodings`, `variable_roles`, `input_hash`.
   - `make_problem` stores raw input and computes hash; it must not canonicalize.

2. `semantic.rs`:
   - implement `RealConstraintKind::{NonNegative,Positive,NonZero,BranchChoice,Other}`.
   - implement `RealConstraintEncoding`.
   - implement `register_slack_encoding`, `semantic_relations`, `verify_semantic_references`.
   - bind semantic hash to kind, relation IDs, slack variables.

3. `validate_input`:
   - rejects undeclared target,
   - rejects undeclared polynomial variables,
   - rejects non-normalized zero terms,
   - rejects invalid semantic references,
   - does not reject branch/slack/unknown geometry.

4. `canonicalize_system`:
   - deterministic target-aware variable order,
   - denominator clearing primitive normalization,
   - zero relation removal with diagnostic,
   - nonzero constant contradiction handling,
   - semantic relation ID preservation,
   - canonical hash.

5. `context.rs`:
   - route budget,
   - cooperative work accounting,
   - `FiniteResourceFailure` with stage/block/matrix/height/memory when known,
   - diagnostics API.

### Tests

- variable roles do not affect validation.
- invalid semantic references fail.
- canonicalization preserves semantic relation IDs.
- constant contradiction produces source status, not panic.
- route budget failure includes evidence.

### Forbidden shortcuts

- Dropping semantic encodings.
- Using variable roles for dispatch.
- Treating slack equations as unsupported.

---

## Phase 3 — Exact algebra foundation: modular, CRT, rational reconstruction, linear algebra, normal form

Supports: BS-R050, BS-R051, BS-R052, BS-R053  
MECHs: MECH-01, MECH-02  
Reviewer Prompt: RP-P3

### Files

```text
algebra/monomial_order.rs
algebra/polynomial_ops.rs
algebra/modular.rs
algebra/crt.rs
algebra/rational_reconstruction.rs
algebra/sparse_matrix.rs
algebra/dense_matrix.rs
algebra/linear_solve.rs
algebra/normal_form.rs
types/matrix.rs
```

### Implementation steps

1. Implement lex, grevlex, block order, and elimination order. Add tests proving eliminated variables compare above keep variables.

2. Implement leading term, S-polynomial, reduction by set returning quotients.

3. Implement deterministic prime stream:
   - avoid all denominators,
   - avoid forbidden leading coefficient denominators,
   - record prime choices.

4. Implement Q→Fp reduction and vector/matrix reduction.

5. Implement CRT for scalars and vectors.

6. Implement rational reconstruction with height bounds:
   - return `None` if no rational under bound exists,
   - never fabricate values.

7. Implement sparse and dense row echelon over Fp.

8. Implement modular rank and nullspace:
   - deterministic pivoting,
   - stable rank profile,
   - reconstructed basis candidates,
   - modular traces.

9. Implement `solve_homogeneous_modular` and `solve_inhomogeneous_modular`:
   - use multiple primes,
   - CRT combine,
   - rational reconstruction,
   - return candidates but not final proof.

10. Implement membership certificate verification by exact Q identity.

### Tests

- modular rank against small exact matrices.
- nullspace vectors satisfy Q matrix equations after reconstruction.
- rational reconstruction rejects out-of-bound wrong candidates.
- membership certificate rejects tampering.

### Forbidden shortcuts

- Accepting modular result without exact verification.
- Random nondeterminism.
- Faking rank stability.

---

## Phase 4 — Production Groebner, production F4, and elimination dispatcher

Supports: BS-R054, BS-R055, BS-R096  
MECHs: MECH-02  
Reviewer Prompt: RP-P4

### Files

```text
algebra/groebner.rs
algebra/f4.rs
algebra/elimination.rs
algebra/normal_form.rs
result/cost_trace.rs
```

### Implementation steps

1. Keep existing local Groebner only if it returns exact membership certificates and resource traces.

2. Replace any test-only F4 with production F4:
   - remove `#[cfg(test)]` around required module exports,
   - implement symbolic preprocessing,
   - construct sparse F4 matrices,
   - reduce batches modulo deterministic primes,
   - reconstruct Q reducers,
   - verify normal-form/membership identities.

3. Implement:
   ```rust
   pub fn f4_reduce_batch(...)
   pub fn f4_elimination_local(...)
   ```

4. Extend `EliminationStrategy` with source-required production variants:
   - `EliminationGroebnerLocal`
   - `F4EliminationLocal`
   - `TargetRelationSearchEscalated`
   - `ResultantIfSquareOrOverdetermined`
   - `SpecializeProjectInterpolateVerify`

5. Implement `eliminate_to_keep_variables` dispatcher:
   - validate disjoint sets,
   - call only declared strategy,
   - return generators in Q[keep],
   - attach membership certificates,
   - return matrix/cost traces.

6. Add audit rule: no source-required F4 public function is `#[cfg(test)]`.

### Tests

- F4 and Groebner produce same elimination generator on small systems.
- F4 certificate verifies exact Q identity.
- Dispatcher rejects keep/eliminate overlap.
- Dispatcher cannot silently use another strategy.

### Forbidden shortcuts

- F4 wrapper calling Groebner only.
- Test-only F4.
- Hidden dispatcher fallback.

---

## Phase 5 — Pre-kernel compression

Supports: BS-R060, BS-R061  
MECHs: MECH-07  
Reviewer Prompt: RP-P5

### Files

```text
preprocess/compression.rs
preprocess/definitional.rs
preprocess/linear_affine.rs
preprocess/binomial.rs
preprocess/saturation.rs
preprocess/independent.rs
problem/semantic.rs
result/diagnostics.rs
```

### Implementation steps

1. Implement `CompressionState` with:
   - variables,
   - relations,
   - semantic encodings,
   - substitutions,
   - guards,
   - saturations,
   - feasibility obligations,
   - trace,
   - diagnostics.

2. Implement definitional elimination exactly:
   - `y - p(X)` or `c*y - p(X)`,
   - `y != target`,
   - deterministic cost ordering,
   - substitution certificates.

3. Implement linear affine elimination:
   - detect `a(X)*y+b(X)=0`,
   - constant nonzero denominator direct,
   - nonconstant denominator only with recorded nonzero guard,
   - reject unsafe candidates.

4. Implement binomial simplification:
   - reversible monomial/binomial cases,
   - no union semantics factor split,
   - guards/certificates for irreversible steps.

5. Implement explicit saturation:
   - only from encoded `A*s-1=0`,
   - record saturation provenance.

6. Implement target-independent component marking:
   - use target influence component,
   - remove from candidate-cover construction,
   - retain feasibility obligations for exact image.

7. Ensure compression order is exactly source order.

### Tests

- Each compression kind with certificate.
- Unsafe affine pivot is not used.
- Target-independent hard component cannot be dropped without obligation.
- Semantics survive compression.

### Forbidden shortcuts

- Geometry rewrite.
- Factor split without semantics.
- Dropping guards.

---

## Phase 6 — Graph metrics, decomposition, and Projection DAG

Supports: BS-R070, BS-R071  
MECHs: MECH-04, MECH-07  
Reviewer Prompt: RP-P6

### Files

```text
graph/hypergraph.rs
graph/influence.rs
graph/weighted_primal.rs
graph/separators.rs
graph/tree_decomposition.rs
graph/projection_dag.rs
graph/metrics.rs
planner/cost_model.rs for shared estimates
```

### Implementation steps

1. Hypergraph:
   - add every relation,
   - add every variable,
   - add every incidence,
   - connected components.

2. Influence:
   - BFS from target in bipartite graph,
   - target component and independent components.

3. Weighted primal:
   - variable/edge weights using source factors,
   - target distance,
   - linear eliminability,
   - coefficient/monomial cost.

4. Separator search:
   - articulation candidates,
   - min-fill candidates,
   - bounded min-cut candidates,
   - deterministic `score_separator`.

5. Decomposition:
   - recursive target-rooted decomposition,
   - use separator if it improves estimated cost,
   - otherwise leaf block, never unsupported.

6. Projection DAG:
   - relation assignment exactly once,
   - duplication certificates when needed,
   - block authorization hash,
   - DAG validation.

7. Metrics:
   - structural metrics,
   - quotient rank estimates,
   - sparse template size estimates,
   - coefficient growth estimates.

### Tests

- no useful separator -> one large block.
- relation coverage exactly once.
- duplicated relation without certificate fails validation.
- authorization hash prevents unauthorized relation access.

### Forbidden shortcuts

- Omitting relations from DAG.
- Failing large block as unsupported.
- Non-deterministic separator choice.

---

## Phase 7 — Planner, admissions, ladder, and route budget

Supports: BS-R080, BS-R081, BS-R082  
MECHs: MECH-03, MECH-07  
Reviewer Prompt: RP-P7

### Files

```text
planner/cost_model.rs
planner/probes.rs
planner/admission.rs
planner/kernel_plan.rs
planner/ladder.rs
planner/planner.rs
kernels/mod.rs
kernels/traits.rs
solver/options.rs
```

### Implementation steps

1. Define `KernelKind` exactly in source order.

2. `all_kernels()` and planner kernel list return exactly the nine source kernels.

3. Cost model:
   - matrix size,
   - quotient rank,
   - coefficient height,
   - separator degree,
   - certificate cost,
   - deterministic comparison.

4. Probes:
   - modular rank probe,
   - local Macaulay size probe,
   - mixed support probe,
   - probe hash,
   - diagnostics that probes are not proofs.

5. Admission:
   - call every kernel,
   - Universal admitted for every well-formed relation block,
   - admission records source data and hash.

6. KernelExecutionPlan:
   - plan ID,
   - block ID,
   - kernel kind,
   - source relation IDs/hashes,
   - child block IDs/message hashes,
   - exported/eliminated variables,
   - support plan,
   - resource bounds,
   - certificate route,
   - failure behavior,
   - algebraic work estimate,
   - route budget,
   - plan hash.

7. Ladder:
   - only certificate-available routes,
   - no coordinate-first kernel,
   - Universal last generic route,
   - all route budgets enforceable.

8. Planner:
   - postorder over DAG,
   - all relation blocks get a `KernelPlan`,
   - empty relation blocks get explicit no-projection record or are documented as structural with certificate evidence,
   - missing Universal is `ImplementationBug`.

### Tests

- all nine admissions run.
- Universal present for arbitrary block.
- declared ladder contains no hidden route.
- plan hashes change if source relation hash changes.
- route budget preflight/postflight.

### Forbidden shortcuts

- Treating admission failure as runtime failure.
- Cost estimates based on tests only.
- Hidden fallback.

---

## Phase 8 — TargetUnivariateKernel and LinearAffineKernel

Supports: BS-R091, BS-R092  
Reviewer Prompt: RP-P8

### Files

```text
kernels/target_univariate.rs
kernels/linear_affine.rs
verify/certificates.rs
verify/verify_message.rs
```

### Implementation steps

1. TargetUnivariate:
   - admission scans block and child relations for nonzero Q[T].
   - execute converts all target-only relations to univariate.
   - computes primitive LCM/squarefree-compatible support.
   - builds source membership certificate.
   - message representation `PrincipalSupport`.

2. LinearAffine:
   - detects triangular affine order,
   - plan stores order and pivot guards,
   - execute performs substitutions,
   - denominator guard handling,
   - exported relation extraction,
   - exact certificate.

3. Message verification:
   - verify exported variable subset,
   - verify certificate route,
   - reject relation outside exported variables.

### Tests

- target-only relation from child message works.
- multiple target-only relations use LCM not overlarge product when possible.
- affine with constant pivot succeeds.
- affine with unsafe nonconstant pivot is rejected.

### Forbidden shortcuts

- ignoring child messages,
- unsafe denominators,
- unsupported for slack/guards.

---

## Phase 9 — TargetRelationSearchKernel production workhorse

Supports: BS-R093  
MECHs: MECH-01, MECH-03, MECH-07  
Reviewer Prompt: RP-P9

### Files

```text
kernels/target_relation_search.rs
planner/relation_schedule.rs if kept, otherwise integrate into planner/probes.rs and target_relation_search.rs
algebra/linear_solve.rs
verify/certificates.rs
verify/verify_message.rs
```

### Implementation steps

1. Define support strategies:
   - DenseTotalDegree,
   - SparseFromProjectionFootprint,
   - SpecializedInterpolationFootprint.

2. Define multiplier support strategies.

3. Admission:
   - possible for all blocks with local or child relations,
   - returns initial degree bounds and estimated matrix size,
   - if cost-prohibited, records estimate and lets other kernels handle.

4. Execute:
   - gather local relations and child message relations.
   - for each declared degree/support bound:
     - build export support A,
     - build multiplier supports B_i,
     - build membership matrix for `g - Σq_i f_i = 0`,
     - solve homogeneous modular system,
     - reconstruct candidate relations and multipliers,
     - deterministic candidate order,
     - exact `verify_membership_exact`,
     - build certificate with all source fields.

5. Resource handling:
   - preflight estimate is planning, not runtime failure.
   - runtime matrix/work overflow -> `FiniteResourceFailure`.

6. Cost trace:
   - matrix rows/cols/density,
   - monomial counts,
   - coefficient height before/after,
   - primes used,
   - rank estimate.

### Tests

- dense relation search finds eliminant in simple multivariate system.
- sparse footprint route finds relation with fewer columns than dense route.
- tampered multiplier fails exact verification.
- modular candidate without exact identity is rejected.
- no relation within declared bounds returns AlgorithmicHardCase with matrix trace.

### Forbidden shortcuts

- modular proof only.
- fixture-specific nullspace vector.
- returning zero relation.
- claiming unsupported.

---

## Phase 10 — SparseResultantProjectionKernel

Supports: BS-R094  
Reviewer Prompt: RP-P10

### Files

```text
algebra/resultant.rs
kernels/sparse_resultant.rs
verify/certificates.rs
verify/verify_message.rs
```

### Implementation steps

1. Expand `ResultantInput` and template structures so they represent source support-set resultant/eliminant route:
   - input polynomials,
   - eliminated variables,
   - keep variables,
   - support sets,
   - template options,
   - matrix/resource bounds.

2. Implement finite template estimation:
   - pairwise resultant chains allowed only as one declared template kind,
   - multivariate support template estimates must be represented when source permits,
   - unsupported sparse shape -> admission false or AlgorithmicHardCase with evidence, never generic Unsupported.

3. Compute relation:
   - modular determinant or null relation,
   - CRT/reconstruction,
   - primitive Q relation.

4. Verify:
   - resultant certificate or exact membership/elimination verification,
   - source relation and support hashes.

5. Kernel execution:
   - output target/separator-only relation,
   - `SparseResultantMatrix` representation,
   - `CandidateCoverStrong`.

### Tests

- linear resultant,
- quadratic resultant,
- chain elimination,
- template too large -> FiniteResourceFailure with matrix dimensions,
- tampered certificate fails.

### Forbidden shortcuts

- two-polynomial-only complete claim.
- symbolic determinant cap as silent unsupported.
- unverified relation.

---

## Phase 11 — TargetActionKrylovKernel and quotient/action handle

Supports: BS-R095  
MECHs: MECH-03  
Reviewer Prompt: RP-P11

### Files

```text
algebra/quotient.rs
algebra/krylov.rs
kernels/action_krylov.rs
verify/certificates.rs
verify/verify_message.rs
```

### Implementation steps

1. Build target-relevant quotient handle:
   - finite rank proof,
   - basis scope limited to target-relevant variables,
   - normal form certificates,
   - action column certificates,
   - no coordinate-root/RUR APIs.

2. Krylov:
   - block Krylov sequence from deterministic basis probes,
   - recurrence recovery,
   - coverage certificate using source-permitted method,
   - verified annihilator.

3. Kernel admission:
   - finite rank estimate,
   - normal form availability,
   - coverage certificate feasibility,
   - handle no-coordinate export.

4. Execute:
   - build handle,
   - prove no coordinate export,
   - compute coverage,
   - verify annihilator,
   - output annihilator relation as `QuotientAction`.

### Tests

- finite univariate quotient yields correct target support.
- multi-vector coverage catches missing eigenvalue case.
- single uncovered Krylov sequence is rejected.
- quotient handle cannot expose coordinate roots/RUR.

### Forbidden shortcuts

- characteristic polynomial without coverage.
- target-unrelated full quotient basis export.
- coordinate solution enumeration.

---

## Phase 12 — UniversalTargetEliminationKernel exactly as source section 20

Supports: BS-R096  
MECHs: MECH-02, MECH-03  
Reviewer Prompt: RP-P12

### Files

```text
kernels/universal_elimination.rs
algebra/elimination.rs
algebra/f4.rs
kernels/target_relation_search.rs
kernels/sparse_resultant.rs
kernels/specialization_interpolation.rs
verify/certificates.rs
```

### Implementation steps

1. Rewrite Universal support plan to use exactly source section 20.4 internal strategies:
   - EliminationGroebnerLocal,
   - F4EliminationLocal,
   - TargetRelationSearchEscalated,
   - ResultantIfSquareOrOverdetermined,
   - SpecializeProjectInterpolateVerify.

2. Remove or stop using NormTrace, RegularChain, and ActionKrylov as Universal internal stages. They remain separate ladder kernels.

3. Admission:
   - true for any block with Q-polynomial relations,
   - if no relations, structural no-projection.

4. Plan:
   - deterministic strategy selection based on algebraic cost and available certificate.
   - every internal strategy has resource bounds and certificate route.
   - selected strategy stored as `plan.elimination_strategy`.

5. Execute:
   - `J = local relations + child messages`,
   - `Y = local - exported`,
   - `Z = exported`,
   - call `eliminate_to_keep_variables(J,Y,Z,plan.elimination_strategy)`,
   - extract nonzero primitive generators in Q[Z],
   - verify every generator exactly,
   - return message.

6. Empty generators:
   - if nonfinite projection certificate available, return certified nonfinite path as specified,
   - otherwise AlgorithmicHardCase.

7. Certificate:
   - records strategy,
   - source relation hashes,
   - child message hashes,
   - generator memberships,
   - cost trace.

### Tests

- arbitrary small multivariate block handled by Universal.
- strategy hash changes if source relation changes.
- F4 strategy used when declared.
- hidden fallback attempt fails audit.
- NormTrace/RegularChain/ActionKrylov are not Universal internal strategies.

### Forbidden shortcuts

- current broad Universal internal stage ladder if it conflicts with source.
- hidden retry with different strategy.
- coordinate solve.

---

## Phase 13 — RegularChainProjection, NormTraceProjection, SpecializationInterpolation

Supports: BS-R097, BS-R098, BS-R099  
MECHs: MECH-03  
Reviewer Prompt: RP-P13

### Files

```text
algebra/regular_chain.rs
kernels/regular_chain_projection.rs
algebra/norm_trace.rs
kernels/norm_trace_projection.rs
algebra/interpolation.rs
kernels/specialization_interpolation.rs
verify/certificates.rs
verify/verify_message.rs
```

### Implementation steps

1. Regular-chain:
   - implement ComponentDAG with guard and component semantics,
   - check triangular pattern,
   - verify regularity/guard conditions,
   - project each chain to keep variables,
   - certify projection,
   - combine union/intersection with certificates.

2. Norm/trace:
   - detect explicit finite algebraic tower by polynomial form,
   - compute norm of target minus expression exactly,
   - verify norm relation by recomputation/certificate,
   - no geometry labels.

3. Specialization/interpolation:
   - deterministic specialization points,
   - declared inner target-only kernel plans,
   - collect samples,
   - interpolate sparse coefficients,
   - exact Q verification by membership or elimination,
   - certificate binds samples and support.

### Tests

- regular-chain guard/component semantics case.
- norm/trace tower with target expression.
- specialization/interpolation with two separators.
- tampered samples fail verification.
- exact Q verification required.

### Forbidden shortcuts

- regular-chain grouping only.
- norm fixture detector.
- sample-only interpolation proof.

---

## Phase 14 — Projection message verification, composition, final support, nonfinite certificate

Supports: BS-R110, BS-R111, BS-R112, BS-R113  
MECHs: MECH-04, MECH-07  
Reviewer Prompt: RP-P14

### Files

```text
compose/message.rs
compose/compose.rs
compose/separator_elimination.rs
compose/final_support.rs
verify/certificates.rs
verify/verify_message.rs
verify/verify_support.rs
verify/replay.rs
verify/run_certificate.rs
result/cost_trace.rs
```

### Implementation steps

1. `compose/message.rs`:
   - implement `MessageIdeal`,
   - `message_to_relations`,
   - `merge_messages`.

2. `verify_message`:
   - verify block authorization,
   - exported variables,
   - certificate route per kernel,
   - no coordinate objects.

3. Composition:
   - postorder to root,
   - merge incoming child/local messages,
   - eliminate separators when needed,
   - use message-only pseudo block,
   - never reintroduce original coordinate system.

4. Replace bounded `message_relations_have_target_eliminant` heuristic:
   - if a resource-bound attempt is needed, it must return `FiniteResourceFailure` with evidence,
   - if proof not available, return `CertificateDesignGap` or `AlgorithmicHardCase`,
   - do not silently conclude no eliminant due to fixed small thresholds.

5. Final support:
   - compute primitive LCM of target-only root relations,
   - verify membership/product route,
   - avoid overlarge product when LCM/gcd reduction applies.

6. Nonfinite:
   - implement dimension/algebraic-dependence certificate using Groebner/regular-chain dimension information,
   - small rational witness alone is not enough for complete nonfinite proof,
   - if not certified, return AlgorithmicHardCase with source reason.

7. Run certificate:
   - build all source fields,
   - implement replay of input/canonicalization/DAG/messages/support/root isolation.

### Tests

- multi-block separator composition.
- message-only separator elimination never reads original relations.
- target-only LCM support.
- composed ideal membership support.
- no target relation + nonfinite certified.
- nonfinite unproved -> AlgorithmicHardCase.
- replay detects tampering.

### Forbidden shortcuts

- global coordinate solve during composition.
- fixed small heuristic as proof.
- small witness nonfinite proof only.

---

## Phase 15 — Exact root isolation, Descartes/Vincent, candidate decode

Supports: BS-R120, BS-R121  
MECHs: MECH-05  
Reviewer Prompt: RP-P15

### Files

```text
algebra/real_root.rs
algebra/sign.rs
roots/squarefree.rs
roots/isolate.rs
roots/decode.rs
roots/algebraic_number.rs
```

### Implementation steps

1. Squarefree:
   - reject zero support,
   - exact derivative/gcd division,
   - normalized output.

2. Sturm:
   - exact Sturm sequence,
   - exact Cauchy bound,
   - exact root count,
   - rational interval bisection or deterministic subdivision without fixed insufficient cap,
   - record isolation certificate.

3. Descartes/Vincent:
   - implement distinct exact Descartes/Vincent isolation,
   - no alias to Sturm,
   - certificate/trace records method.

4. Decode:
   - bind support hash and root index,
   - deterministic candidate order,
   - candidate hash.

5. Algebraic root:
   - refinement,
   - comparison,
   - hash binding.

6. Sign:
   - sign at algebraic root,
   - Thom encoding.

### Tests

- Sturm and Descartes both isolate same roots on test polynomials but use distinct code paths.
- repeated roots handled after squarefree.
- zero support failure.
- candidate hash changes when interval changes.
- sign classification at algebraic root.

### Forbidden shortcuts

- Descartes alias to Sturm.
- floating approximations.
- fixed split search failure where deterministic exact refinement should continue.

---

## Phase 16 — Exact image mode and real semantics

Supports: BS-R003, BS-R040, BS-R122  
MECHs: MECH-06  
Reviewer Prompt: RP-P16

### Files

```text
fiber/exact_image.rs
fiber/hermite.rs
fiber/thom.rs
fiber/slack_semantics.rs
problem/semantic.rs
preprocess/saturation.rs
```

### Implementation steps

1. `classify_real_target_image`:
   - for each candidate, add algebraic target condition,
   - build fiber problem,
   - attach semantics,
   - decide real fiber nonempty,
   - record accept/reject disposition.

2. Hermite:
   - implement real root count for zero-dimensional fiber,
   - certificate binds support, root, equality relations, semantic factors.

3. Thom:
   - sign classification of guard polynomial at algebraic target root,
   - derivative/Thom evidence.

4. Slack semantics:
   - interpret NonNegative, Positive, NonZero, BranchChoice,
   - verify slack encoding consistency,
   - handle guards and saturations.

5. Mixed fibers:
   - implement required certificate path or return source-specific `CertificateDesignGap`.
   - no silent accept/reject.

### Tests

- `A>=0` via `A-s^2=0`,
- `A>0` via `A*s^2-1=0`,
- `A!=0` via `A*s-1=0`,
- branch choice semantics,
- guard rejects vanishing target root,
- mixed fiber gap is explicit.

### Forbidden shortcuts

- exact image no-op.
- accepting with no Hermite/Thom/slack evidence.
- target-independent witness small search as the only proof when source requires full semantics.

---

## Phase 17 — Orchestrator, pipeline, result finalization, cost trace

Supports: BS-R130, BS-R131, BS-R140  
MECHs: MECH-03, MECH-04, MECH-05, MECH-07  
Reviewer Prompt: RP-P17

### Files

```text
solver/options.rs
solver/pipeline.rs
solver/orchestrator.rs
result/output.rs
result/cost_trace.rs
verify/run_certificate.rs
verify/replay.rs
```

### Implementation steps

1. `SolverOptions`:
   - include exact-image mode,
   - memory/matrix/coefficient limits,
   - root isolation method,
   - certificate level,
   - any additional source-compatible options only if non-weakening.

2. `pipeline.rs`:
   - expose step functions for each source stage,
   - do no hidden work outside named stages.

3. `orchestrator.rs`:
   - call stages in exact source order,
   - verify messages before composition,
   - verify support before roots,
   - squarefree/root isolation/decode,
   - exact-image branch if requested,
   - build certificate,
   - finalize result.

4. Failure handling:
   - every error converted to `TargetSolveResult`,
   - include cost trace from completed steps and failure step.

5. Cost trace:
   - all source fields,
   - final support degree,
   - certificate size,
   - route attempts.

### Tests

- pipeline stage order observable through diagnostics.
- support verification failure stops before root decode.
- candidate-cover result includes spurious-root diagnostic.
- failure result includes cost trace.
- exact-image mode changes status only after classification.

### Forbidden shortcuts

- skipping verify support.
- root isolation before support verification.
- panic on solver error.

---

## Phase 18 — Full conformance suite, static audit, reviewers, closure

Supports: BS-R000 through BS-R150  
MECHs: all  
Reviewer Prompt: RP-P18

### Tasks

1. Add `geosolver-core/tests/v4_conformance.rs` with integration tests covering all acceptance categories.

2. Add property-style tests for normalization, membership, root isolation, and hash binding.

3. Add adversarial tests:
   - geometry label absent,
   - variable roles misleading,
   - fixture names unavailable,
   - same polynomial structure with different variable IDs,
   - relation order permutations.

4. Run:
   ```text
   cargo fmt --check
   cargo clippy -p geosolver-core --all-targets -- -D warnings
   cargo test -p geosolver-core
   python geosolver-core/scripts/audit_v4_conformance.py --strict
   ```

5. Generate final:
   ```text
   docs/ai/changes/RGDTPK-Q-v4-complete-conformance/CLOSURE.md
   docs/ai/changes/RGDTPK-Q-v4-complete-conformance/evidence/final/source_to_code_conformance_matrix.md
   docs/ai/changes/RGDTPK-Q-v4-complete-conformance/evidence/final/final_commands.log
   ```

6. Run reviewers in this order:
   - guardian boundary reviewer,
   - spec verifier,
   - quality reviewer.

7. If any reviewer FAILs, create a new plan amendment. Do not patch only tests.

### Final claim conditions

The final answer may claim `SOURCE_FAITHFUL` and `ACCEPTANCE_COMPLETE` only if:
- all Base Spec R-IDs have code evidence,
- all required tests/audits pass,
- all reviewers PASS,
- no blocking QuestionDebt remains,
- git status is recorded,
- closure cites fresh evidence.



---

# Reviewer Prompts: R-GDTPK-Q / ACCTP-Q v4 完全準拠修正

## Global reviewer instructions

You are reviewing a Guardian Lane implementation of `RGDTPK-Q-v4-complete-conformance`.

Source authority:
- Original source: `docs/ai/sources/geosolver_core_r_gdtpk_q_algorithm_spec_v4.md`, blob sha `ef108f0dc95880d2e3030c96872b9073be995274`.
- Base Spec: `docs/ai/changes/RGDTPK-Q-v4-complete-conformance/BASE_SPEC.md`.
- Plan: `docs/ai/changes/RGDTPK-Q-v4-complete-conformance/PLAN.md`.

Return exactly one of:
```text
PASS
FAIL
NEEDS_MORE_EVIDENCE
```

Do not PASS from tests alone. You must inspect implementation code and compare it with Base Spec R-IDs and source sections. Tests, CI, and audit scripts are supporting evidence only.

A reviewer PASS means:
- no visible source drift,
- no hidden simplification,
- no test-only implementation,
- no forbidden fallback,
- required certificates are real and replayable,
- failure statuses carry source-required evidence.

Return FAIL immediately if any of these are found:
- `todo!`, `unimplemented!`, placeholder production function, or fake certificate in a required path.
- Source-required algorithm implemented only under `#[cfg(test)]`.
- Descartes root isolation silently delegates to Sturm.
- F4 implementation is only a wrapper around Groebner.
- UniversalTargetElimination internal strategy list differs from source section 20.4 without approved Base Spec amendment.
- A production path returns a candidate relation without exact Q verification.
- A route fallback is not declared in the plan/ladder/certificate.
- Geometry-name, fixture-id, expected-answer, or variable-role dispatch.
- Coordinate solution list, full coordinate RUR, global lex parametrization, QE/CAD fallback.
- Nonfinite certificate based only on tiny rational witness search while claiming source completeness.
- Regular-chain implementation lacks component/guard/projection semantics but claims complete.
- Exact-image mode silently accepts/rejects without fiber/guard/slack evidence.

For every review, include:
1. Scope reviewed.
2. Files inspected.
3. R-IDs checked.
4. Source sections checked.
5. Evidence accepted.
6. Evidence missing.
7. Blocking findings.
8. Decision.

---

## RP-P0 — Source lock and audit harness reviewer

Review phase: P0  
Relevant R-IDs: BS-R000, BS-R010, BS-R150  
Files to inspect:
- `docs/ai/changes/RGDTPK-Q-v4-complete-conformance/**`
- `geosolver-core/scripts/audit_v4_conformance.py`
- `docs/ai/sources/geosolver_core_r_gdtpk_q_algorithm_spec_v4.md`

Checks:
1. Confirm the Base Spec and Plan are copied into the repo.
2. Confirm source path and blob sha are recorded.
3. Confirm audit script scans production files and is not written to suppress known gaps.
4. Confirm current gap inventory honestly lists all known noncompliance.
5. Confirm implementation has not started without explicit user approval if this is only planning.

PASS only if the harness makes failures visible. P0 may PASS even when audit reports implementation failures, but only if the audit is strict and honest.

---

## RP-P1 — Type/public API reviewer

Review phase: P1  
Relevant R-IDs: BS-R020, BS-R030, BS-R031, BS-R032, BS-R130

Inspect:
- public API,
- types,
- normalization,
- result/status/cost trace.

Checks:
1. `lib.rs` is only module exports.
2. `api::solve_target` returns `TargetSolveResult` and does not panic for normal solver errors.
3. Rational normalization is exact and total for valid inputs.
4. Polynomial normalization removes zero coefficients and duplicate monomials.
5. Hashes are deterministic and structure-bound.
6. Matrix density/shape/hash are exact.
7. Intervals are rational and validated.

FAIL if any source-required type/function is missing, placeholder, or float-based.

---

## RP-P2 — Problem/semantic/canonicalization reviewer

Review phase: P2  
Relevant R-IDs: BS-R001, BS-R040, BS-R041, BS-R042

Checks:
1. Variable roles never affect validation or dispatch.
2. Slack/branch semantic references are validated and preserved.
3. Canonicalization clears denominators primitively and preserves relation provenance.
4. Constant contradictions and invalid semantics return source statuses.
5. Resource context produces evidence-bearing `FiniteResourceFailure`.

FAIL if semantic provenance is dropped or geometry labels influence behavior.

---

## RP-P3 — Modular algebra and linear solve reviewer

Review phase: P3  
Relevant R-IDs: BS-R050, BS-R051, BS-R052, BS-R053  
MECHs: MECH-01, MECH-02

Checks:
1. Prime selection deterministic and avoids denominators.
2. CRT and rational reconstruction are exact and reject impossible reconstructions.
3. Sparse/dense row reduction has deterministic pivots.
4. Nullspace candidates are not treated as proof.
5. Membership certificates recompute exact Q identities.

FAIL if modular equality is accepted as final proof.

---

## RP-P4 — Groebner/F4/elimination reviewer

Review phase: P4  
Relevant R-IDs: BS-R054, BS-R055, BS-R096  
MECH: MECH-02

Checks:
1. `algebra/f4.rs` is production, not `#[cfg(test)]` only.
2. `f4_reduce_batch` constructs and reduces actual batch matrices.
3. `f4_elimination_local` returns Q[keep] generators with exact certificates.
4. `eliminate_to_keep_variables` dispatches only the declared strategy.
5. No hidden Groebner fallback inside F4 unless the plan explicitly declares Groebner.

FAIL if F4 is just a renamed Groebner call or if certificate verification is missing.

---

## RP-P5 — Preprocessing reviewer

Review phase: P5  
Relevant R-IDs: BS-R060, BS-R061

Checks:
1. Compression order exactly matches source.
2. Definitional elimination excludes target and records substitutions.
3. Affine elimination uses only safe pivots.
4. Binomial simplification is reversible or guarded.
5. Saturation only from explicit nonzero encoding.
6. Target-independent components retain exact-image feasibility obligations.

FAIL if any rewrite loses semantic/certificate provenance.

---

## RP-P6 — Graph/DAG reviewer

Review phase: P6  
Relevant R-IDs: BS-R070, BS-R071

Checks:
1. Hypergraph contains every relation-variable incidence.
2. Influence graph BFS starts from target.
3. Weighted graph uses all source cost signals.
4. Decomposition tries source-specified separator families.
5. No useful separator produces one large block, not failure.
6. DAG validates relation coverage, authorization, duplication certificates, and root.

FAIL if any relation can be omitted, duplicated without certificate, or read by an unauthorized kernel.

---

## RP-P7 — Planner/ladder reviewer

Review phase: P7  
Relevant R-IDs: BS-R080, BS-R081, BS-R082  
MECH: MECH-03

Checks:
1. All nine kernels are registered in source order.
2. Admissions call every kernel.
3. Universal is admitted for every relation block.
4. Cost estimates are deterministic and not proof.
5. Declared ladder contains only certificate-capable coordinate-free kernels.
6. Every route has enforceable bounds and failure behavior.
7. Execution cannot call a route absent from declared ladder.

FAIL if hidden fallback remains possible.

---

## RP-P8 — TargetUnivariate/LinearAffine reviewer

Review phase: P8  
Relevant R-IDs: BS-R091, BS-R092

Checks:
1. TargetUnivariate scans local and child message relations.
2. Support uses primitive LCM/squarefree-compatible construction.
3. Source membership certificate binds source IDs/hashes.
4. LinearAffine pivots are safe and guard-bound.
5. Incomplete affine elimination returns correct failure.

FAIL if unsafe denominator division is possible.

---

## RP-P9 — TargetRelationSearch reviewer

Review phase: P9  
Relevant R-IDs: BS-R093  
MECH: MECH-01

Checks:
1. Kernel implements `g(Z)=Σq_i f_i` coefficient comparison.
2. Dense, sparse, and specialized support strategies exist or source-amended reason is present.
3. Modular solve returns candidates only.
4. Every success is exact Q membership verified.
5. Certificate contains all source fields.
6. AlgorithmicHardCase includes accumulated matrix trace.

FAIL if candidate relation can return without exact identity.

---

## RP-P10 — SparseResultant reviewer

Review phase: P10  
Relevant R-IDs: BS-R094

Checks:
1. Template represents source support-set resultant/eliminant semantics.
2. Two-polynomial special cases are not claimed as the full implementation unless source amended.
3. Resource caps return `FiniteResourceFailure`.
4. Resultant relation variables are subset of exported variables.
5. Certificate recomputes exact relation or proves exact membership.

FAIL if a resultant is accepted only by modular trace or small determinant hash.

---

## RP-P11 — ActionKrylov reviewer

Review phase: P11  
Relevant R-IDs: BS-R095

Checks:
1. Quotient handle does not expose coordinate roots, coordinate solution list, full coordinate RUR, or target-unrelated full quotient basis.
2. Krylov sequence uses deterministic coverage probes.
3. Coverage certificate cannot miss target-relevant eigenvalues.
4. `verify_annihilator` is exact.
5. No candidate polynomial returns without coverage.

FAIL if single Krylov recurrence is used without coverage proof.

---

## RP-P12 — Universal reviewer

Review phase: P12  
Relevant R-IDs: BS-R096  
MECHs: MECH-02, MECH-03

Checks:
1. Universal admission true for Q-polynomial relation blocks.
2. Internal strategies exactly match source section 20.4:
   - EliminationGroebnerLocal
   - F4EliminationLocal
   - TargetRelationSearchEscalated
   - ResultantIfSquareOrOverdetermined
   - SpecializeProjectInterpolateVerify
3. NormTrace, RegularChain, ActionKrylov are not Universal internal stages.
4. Strategy is declared before execution and certificate-bound.
5. Every exported generator is in Q[Z] and exactly verified.
6. Empty generator path either certifies nonfinite or returns AlgorithmicHardCase.

FAIL on any hidden strategy, undeclared fallback, or unverified generator.

---

## RP-P13 — RegularChain/NormTrace/Specialization reviewer

Review phase: P13  
Relevant R-IDs: BS-R097, BS-R098, BS-R099

Checks:
1. RegularChain has component/guard/projection semantics and certificates.
2. NormTrace detects algebraic tower by form, not geometry label.
3. Norm relation is exactly verified.
4. Specialization sample points are deterministic and certificate-bound.
5. Interpolated relation is accepted only after exact Q verification.

FAIL if any of these kernels are fixture-shaped wrappers.

---

## RP-P14 — Composition/support/nonfinite/replay reviewer

Review phase: P14  
Relevant R-IDs: BS-R110, BS-R111, BS-R112, BS-R113  
MECH: MECH-04

Checks:
1. Composition follows TargetProjectionDAG.
2. Separator elimination uses message-only pseudo blocks.
3. Original coordinate system is not globally reconstructed.
4. Fixed heuristic thresholds are not used as proof of no eliminant.
5. Support uses primitive LCM where applicable.
6. `verify_global_support` proves target-only or composed-ideal membership.
7. Nonfinite certificate uses elimination/dimension/algebraic-dependence evidence.
8. Replay recomputes input/canonical/DAG/message/support/root hashes.

FAIL if nonfinite proof is only small rational witness search or if replay is only hash comparison.

---

## RP-P15 — Roots/candidates reviewer

Review phase: P15  
Relevant R-IDs: BS-R120, BS-R121  
MECH: MECH-05

Checks:
1. Squarefree rejects zero support.
2. Sturm is exact.
3. Descartes/Vincent is distinct exact implementation, not alias.
4. Isolation intervals are rational and isolate one root each.
5. Candidate hash binds target, support hash, root index, interval.
6. Sign/Thom routines are exact.

FAIL if any root isolation uses float-only approximation.

---

## RP-P16 — Exact-image reviewer

Review phase: P16  
Relevant R-IDs: BS-R003, BS-R040, BS-R122  
MECH: MECH-06

Checks:
1. `classify_real_target_image` actually classifies each candidate.
2. Hermite real root count is implemented for zero-dimensional fibers.
3. Thom sign classification is used for guards.
4. Slack semantics for >=, >, !=, branch choice are interpreted.
5. Mixed fiber cases return explicit certificate gap if not supported.
6. No silent exact-image no-op.

FAIL if exact-image mode only filters target-only equations or accepts all candidates.

---

## RP-P17 — Orchestrator/result/cost reviewer

Review phase: P17  
Relevant R-IDs: BS-R130, BS-R131, BS-R140  
MECH: MECH-07

Checks:
1. Pipeline order exactly matches source.
2. Message verification occurs before composition.
3. Support verification occurs before roots.
4. Failure results preserve stage/cost evidence.
5. Cost trace contains every source parameter.
6. Final certificate binds all source fields.

FAIL if any pipeline stage is skipped or reordered for convenience.

---

## RP-P18 — Final conformance reviewer

Review phase: P18  
Relevant R-IDs: all  
MECHs: all

Checks:
1. Source-to-code matrix covers every source section and every Base Spec R-ID.
2. Audit script passes strict mode.
3. Cargo fmt/clippy/test evidence is fresh.
4. All prior reviewer findings are resolved.
5. No blocking QuestionDebt remains.
6. All 17 source completion conditions are individually PASS.
7. Final claim is no stronger than evidence.

Return PASS only if the implementation is truly source-faithful, not merely test-passing.



---

# Source Map: R-GDTPK-Q / ACCTP-Q v4 完全準拠修正

Original Source:
- Path: `docs/ai/sources/geosolver_core_r_gdtpk_q_algorithm_spec_v4.md`
- Blob sha: `ef108f0dc95880d2e3030c96872b9073be995274`

This file maps source sections to Base Spec R-IDs. It is an index for review. The Base Spec body and the original source are the normative authorities.

| Source Section | Source Topic | Base Spec R-IDs | Implementation Area |
|---|---|---|---|
| 0 | self-contained fixed implementation spec | BS-R000 | all files |
| 1.1-1.3 | research strength, target-direct, genericity | BS-R000, BS-R001, BS-R010, BS-R150 | solver, planner, kernels |
| 2.1 | input Q-polynomial system | BS-R001, BS-R040, BS-R041 | problem/* |
| 2.2 | S(T), squarefree, root isolation | BS-R002, BS-R111, BS-R120, BS-R121 | compose/final_support.rs, roots/* |
| 2.3 | candidate cover vs exact image | BS-R002, BS-R003, BS-R122 | result/*, fiber/* |
| 2.4 | slack/guard encodings | BS-R003, BS-R040, BS-R122 | problem/semantic.rs, fiber/* |
| 3.1 | prohibited coordinate/RUR/QE/CAD/geometry fallback | BS-R010, BS-R150, section 17 forbidden simplifications | all production paths |
| 3.2 | allowed heavy local algebra | BS-R054-BS-R056, BS-R093-BS-R099 | algebra/*, kernels/* |
| 3.3 | failure statuses | BS-R011, BS-R130 | result/status.rs, result/output.rs |
| 4.1 | top-level pipeline | BS-R140 | solver/pipeline.rs, solver/orchestrator.rs |
| 4.2 | invariants I1-I10 | BS-R010 | run certificate and all stages |
| 5.1-5.3 | IDs, RationalQ, Monomial/Polynomial | BS-R030, BS-R031 | types/* |
| 5.4 | RationalTargetProblem and semantics | BS-R001, BS-R040 | problem/input.rs, problem/semantic.rs |
| 5.5 | CanonicalSystemQ | BS-R041 | problem/canonicalize.rs |
| 5.6 | ProjectionMessage | BS-R090, BS-R110 | compose/message.rs, kernels/* |
| 5.7 | TargetSolveResult and SolverStatus | BS-R011, BS-R130 | result/* |
| 6 | folder layout | BS-R020 | geosolver-core/src/** |
| 7.1 | lib.rs | BS-R020 | src/lib.rs |
| 7.2 | api.rs | BS-R130, BS-R140 | src/api.rs |
| 8.1-8.7 | types functions | BS-R030-BS-R032 | types/* |
| 9.1-9.5 | problem functions | BS-R040-BS-R042 | problem/* |
| 10.1-10.8 | algebra foundations | BS-R050-BS-R053 | algebra/monomial_order.rs through normal_form.rs |
| 10.9 | Groebner | BS-R054 | algebra/groebner.rs |
| 10.10 | F4 | BS-R054 | algebra/f4.rs |
| 10.11 | elimination dispatcher | BS-R055 | algebra/elimination.rs |
| 10.12 | resultant | BS-R056, BS-R094 | algebra/resultant.rs, kernels/sparse_resultant.rs |
| 10.13 | interpolation | BS-R056, BS-R099 | algebra/interpolation.rs |
| 10.14 | quotient handle | BS-R056, BS-R095 | algebra/quotient.rs |
| 10.15 | Krylov | BS-R056, BS-R095 | algebra/krylov.rs |
| 10.16 | regular chain | BS-R056, BS-R097 | algebra/regular_chain.rs |
| 10.17 | norm/trace | BS-R056, BS-R098 | algebra/norm_trace.rs |
| 10.18 | real roots | BS-R120 | algebra/real_root.rs |
| 10.19 | sign | BS-R122 | algebra/sign.rs, fiber/thom.rs |
| 11.1-11.6 | preprocessing | BS-R060, BS-R061 | preprocess/* |
| 12.1-12.7 | graph and DAG | BS-R070, BS-R071 | graph/* |
| 13.1-13.6 | planner | BS-R080-BS-R082 | planner/* |
| 14.1-14.2 | kernel trait and registry | BS-R090 | kernels/traits.rs, kernels/mod.rs |
| 15 | TargetUnivariate | BS-R091 | kernels/target_univariate.rs |
| 16 | LinearAffine | BS-R092 | kernels/linear_affine.rs |
| 17 | TargetRelationSearch | BS-R093 | kernels/target_relation_search.rs |
| 18 | SparseResultant | BS-R094 | kernels/sparse_resultant.rs |
| 19 | TargetActionKrylov | BS-R095 | kernels/action_krylov.rs |
| 20 | UniversalTargetElimination | BS-R096 | kernels/universal_elimination.rs |
| 21 | RegularChainProjection | BS-R097 | kernels/regular_chain_projection.rs |
| 22 | NormTraceProjection | BS-R098 | kernels/norm_trace_projection.rs |
| 23 | SpecializationInterpolation | BS-R099 | kernels/specialization_interpolation.rs |
| 24.1-24.4 | message composition and final support | BS-R110, BS-R111 | compose/* |
| 25.1-25.5 | certificates, verify, replay | BS-R112, BS-R113 | verify/* |
| 26.1-26.3 | squarefree, roots, decode | BS-R120, BS-R121 | roots/*, algebra/real_root.rs |
| 27.1-27.4 | exact image / hermite / thom / slack | BS-R003, BS-R122 | fiber/* |
| 28.1-28.3 | result and cost trace | BS-R011, BS-R130, BS-R131 | result/* |
| 29.1-29.3 | solver options/pipeline/orchestrator | BS-R140 | solver/* |
| 30.1-30.3 | algebraic cost design | BS-R131 | result/cost_trace.rs, planner/* |
| 31 | nonfinite target image | BS-R111, BS-R112, BS-R113 | compose/final_support.rs, verify/* |
| 32 | algebraic footprint not geometry dispatch | BS-R001, BS-R010, BS-R150 | all production code |
| 33 | completion conditions | BS-R150 | final conformance review |
| 34 | final summary | BS-R000, BS-R150 | closure |

Reviewer use:
- For every implementation phase, reviewers must use this map to select source sections and code files.
- If a source section has no implementation file evidence, the phase fails.
- If a source section is implemented only by tests or by a placeholder, the phase fails.
