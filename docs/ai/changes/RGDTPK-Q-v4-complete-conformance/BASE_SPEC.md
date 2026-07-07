# Change Base Spec: R-GDTPK-Q / ACCTP-Q v4 有限 Candidate-Cover 準拠修正

## Context Packet

Spec ID: `RGDTPK-Q-v4-finite-candidate-cover`  
Type: Change Base Spec  
Status: Approved for implementation by user on 2026-07-07 for finite candidate-cover scope  
Parent: None for this packet. It may later be registered as an Area Base Spec for `geosolver-core`.  
Scope: `geosolver-core` finite candidate-cover solver core only. Exact-image equality/classification is `OUT_OF_SCOPE` for this repair.  
Applies To: `geosolver-core/src/**`, `geosolver-core/Cargo.toml`, `geosolver-core/README.md`, and conformance evidence under `docs/ai/changes/RGDTPK-Q-v4-complete-conformance/**`.  
Required Parent R-IDs: None.  
Original Source Authority: `docs/ai/sources/geosolver_core_r_gdtpk_q_algorithm_spec_v4.md`, blob sha `ef108f0dc95880d2e3030c96872b9073be995274`.  
Blocking Questions: None. If any ambiguity is found during implementation, dependent work must stop and this Base Spec must be amended before code changes continue.  
Non-blocking Debt: None admitted.  
Known Exceptions: source sections that only prove exact target-image equality or real-fiber/slack/guard classification after candidate-cover construction are `OUT_OF_SCOPE`.  
Allowed Final Claims after scoped completion and reviewer PASS: `FINITE_CANDIDATE_COVER_COMPLETE`, `SOURCE_FAITHFUL_TO_V4_FINITE_CANDIDATE_COVER_LAYER`, `VERIFIED_FOR_FINITE_CANDIDATE_COVER` for this Change Base Spec only.  
Forbidden Claims: `SOURCE_FAITHFUL_TO_FULL_V4`, `ACCEPTANCE_COMPLETE_FOR_FULL_V4`, `EXACT_IMAGE_COMPLETE`, `CERTIFIED_EXACT_TARGET_IMAGE_COMPLETE`, `PRODUCTION_SAFE`, and `BENCHMARK_PROVEN`.  
Context Packet Authority: non-authoritative digest. The R-ID body below is the authority.

---

## 1. Source authority and zero-adaptation rule

### BS-R000 — Scoped source fidelity

The implementation must be a faithful production implementation of the finite candidate-cover layer of **GeoSolver Core Algorithm Specification v4.0 / R-GDTPK-Q / ACCTP-Q** at `docs/ai/sources/geosolver_core_r_gdtpk_q_algorithm_spec_v4.md`, blob sha `ef108f0dc95880d2e3030c96872b9073be995274`.

Every normative statement needed to produce and verify a finite candidate cover is incorporated into this Base Spec. The implementation must not reinterpret in-scope source sections for convenience. If this Base Spec and the original source differ within the finite candidate-cover layer, the original source wins and this Base Spec must be amended immediately before further implementation.

No requirement below is optional unless it is explicitly marked `OUT_OF_SCOPE`. Exact-image equality/classification requirements are `OUT_OF_SCOPE` except for the explicit API scope guard in BS-R003/BS-R122.

Acceptance:
- The final code must implement the file structure, public types, public functions, pipeline order, kernel list, certificate semantics, failure statuses, and completion conditions needed for finite candidate-cover output.
- The final review must compare every in-scope source section against implementation, not just run tests.
- The final closure must include a source-to-code conformance matrix with exact-image-only source sections marked `OUT_OF_SCOPE`.

Forbidden:
- Passing only existing tests while leaving non-production shortcuts.
- Treating comments, old plans, previous generated specs, or current implementation behavior as authority over the source specification.
- Replacing a source algorithm with a smaller algorithm unless the source explicitly allows it.
- Claiming full-v4 source fidelity or exact-image completion from this scoped repair.

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

### BS-R003 — Exact target image is out of scope and must be guarded

Exact image mode, exact target-image equality, real-fiber classification, Hermite/Thom/slack final filtering, and `CertifiedExactTargetImage` are `OUT_OF_SCOPE` for this repair. Candidate-cover computation must still preserve semantic provenance when it is part of well-formed input, but it does not need to classify real fibers.

If an exact-image option is requested through the API during this scoped repair, the solver must return an explicit evidence-backed scope diagnostic or allowed failure status such as `CertificateDesignGap` with reason `ExactImageOutOfScope`. It must never silently accept all candidates, silently reject candidates, or relabel candidate-cover output as exact-image output.

Acceptance:
- Candidate-cover computation must not depend on exact-image filtering.
- Exact-image-only source sections are marked `OUT_OF_SCOPE` in the source map and final conformance matrix.
- Any exposed exact-image request has a deterministic scope guard and cannot produce a success status.

Forbidden:
- Treating exact-image mode as a no-op.
- Returning `CertifiedExactTargetImage` without real-fiber evidence.
- Claiming exact-image or full-v4 acceptance from finite candidate-cover evidence.

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
I7. Exact-image mode is out of scope and cannot be reported as successful.
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

The repository must contain the source-specified candidate-cover `geosolver-core` structure:

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
    fiber/{mod.rs,exact_image.rs,hermite.rs,thom.rs,slack_semantics.rs} (OUT_OF_SCOPE except explicit scope guard if already exposed)
    result/{mod.rs,status.rs,diagnostics.rs,cost_trace.rs,output.rs}
    solver/{mod.rs,options.rs,pipeline.rs,orchestrator.rs}
```

Acceptance:
- All listed non-`OUT_OF_SCOPE` modules exist in production, not only in tests.
- `lib.rs` exposes only the source module tree and must not contain solver logic.
- Missing exact-image-only files are not blocking for finite candidate-cover completion if no candidate-cover production path depends on them.

Forbidden:
- Moving in-scope source-required APIs into ad hoc test modules.
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
- Candidate-cover certificates preserve semantic provenance hashes when those encodings are present.

Forbidden:
- Dropping semantic provenance during denominator clearing or compression.
- Using slack/guard provenance for geometry dispatch or target-value filtering in candidate-cover mode.

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
- `sign.rs`: sign at algebraic root and Thom encoding only when used by candidate-cover code; exact-image-only sign classification is `OUT_OF_SCOPE`.

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
- Target-independent components may be removed from candidate-cover construction only when this cannot remove any finite target value from the support cover; exact-image feasibility obligations remain out of scope.

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

## 13. Roots, candidate decode, and scoped-out exact-image

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

### BS-R122 — Real fiber classification scope guard

Real fiber classification is `OUT_OF_SCOPE` for this finite candidate-cover repair. The required behavior is a scope guard: exact-image requests must not return exact-image success, and candidate-cover success must not depend on real-fiber filtering.

Acceptance:
- Exact-image-only requests are rejected or diagnosed with `ExactImageOutOfScope` or an equivalent allowed, evidence-backed status.
- Candidate hashes and support verification are unchanged by exact-image scope guard code.
- Semantic provenance remains available for future exact-image work but is not required to classify candidates now.

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
15. FinalizeCandidateCoverResultAndCertificate
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

### BS-R150 — Finite candidate-cover completion conditions

The implementation is complete for this scoped repair only when all 16 finite candidate-cover conditions hold:

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
13. Exact-image/full-image requests cannot return silent success and are explicitly out of scope.
14. Failures return evidence-backed status, not Unsupported.
15. Cost trace records every algebraic-cost-compression parameter.
16. Hidden fallback is impossible at the API level.
```

Acceptance:
- Reviewer must PASS each in-scope item individually with code evidence.
- Any single failed item blocks `FINITE_CANDIDATE_COVER_COMPLETE`.

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

### MECH-06 — Exact-image scope guard mechanism

Supports: BS-R003, BS-R040, BS-R122, BS-R150  
Domain: scoped API boundary, semantics provenance  
Required: yes  
Semantics: For exact-image mode, return explicit out-of-scope evidence and do not classify candidate roots.  
Inputs: compressed system, support, candidates, semantic encodings.  
Outputs: candidate-cover result or explicit exact-image-out-of-scope failure/diagnostic; never `CertifiedExactTargetImage`.  
Oracle: replayable option/status/certificate evidence showing no exact-image success path is reachable.  
Failure behavior: exact-image request -> `CertificateDesignGap` or equivalent allowed status with `ExactImageOutOfScope`.

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
- exact-image request scope guard that cannot be mistaken for candidate-cover success.

Tests are supplementary. Reviewer must still inspect algorithms.

### Acceptance A4 — Fresh evidence

Every final claim must cite fresh commands, git state, changed files, and reviewer reports. Reviewer PASS alone is not evidence.

---

## 20. Reviewer blocking rules

A reviewer must return FAIL if:
- Any Base Spec R-ID is unimplemented, stubbed, test-only, or simplified.
- A Plan task claims closure by tests only without code inspection.
- An in-scope source-required algorithm is replaced by a smaller algorithm without explicit approved exception.
- A required certificate is only a hash with no replay or exact identity.
- A production route can return candidate cover without exact Q verification.
- Hidden fallback remains possible.
- Descartes is an alias to Sturm.
- F4 remains test-only.
- Universal internal strategy list differs from source section 20.4.
- Exact-image request silently accepts/rejects candidates, or returns exact-image success, instead of the scoped out-of-scope status.
