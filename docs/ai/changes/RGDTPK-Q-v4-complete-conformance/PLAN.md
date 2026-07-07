# Plan: R-GDTPK-Q / ACCTP-Q v4 有限 Candidate-Cover 準拠修正

## Context Packet

Spec ID: `RGDTPK-Q-v4-finite-candidate-cover`  
Type: Plan Contract  
Status: Approved for implementation by user on 2026-07-07 for finite candidate-cover scope  
Base Spec: `BASE_SPEC.md` in this packet  
Original Source: `docs/ai/sources/geosolver_core_r_gdtpk_q_algorithm_spec_v4.md`, blob sha `ef108f0dc95880d2e3030c96872b9073be995274`  
Scope: production conformance of `geosolver-core` to the v4 finite candidate-cover layer; exact-image equality/classification is out of scope.  
Allowed Claim During Execution: `PARTIALLY_IMPLEMENTED` per completed phase only  
Allowed Final Claim After All Reviews: `FINITE_CANDIDATE_COVER_COMPLETE`, `SOURCE_FAITHFUL_TO_V4_FINITE_CANDIDATE_COVER_LAYER`, `VERIFIED_FOR_FINITE_CANDIDATE_COVER` for this Change Base Spec  
Implementation Permission: Granted by user message on 2026-07-07 after finite candidate-cover scope amendment. Evidence: `evidence/P0/implementation_authority.md`.  
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

### P-RULE-3 — No in-scope simplification by omission

A phase must not skip in-scope source-required APIs because the current code does not use them yet. Exact-image-only APIs are out of scope, but any exposed exact-image option must have the explicit scope guard required by the Base Spec.

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
   - report `#[cfg(test)]`-only in-scope source-required APIs,
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
   - exact-image success paths or no-op filters if still present; these must become explicit out-of-scope guards.

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
   - prove the removal cannot exclude any finite target value from the candidate-cover support.

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
   - sign at algebraic root only if used by candidate-cover code,
   - Thom encoding only if used by candidate-cover code.

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

## Phase 16 — Exact-image scope guard and semantic boundary

Supports: BS-R003, BS-R040, BS-R122  
MECHs: MECH-06  
Reviewer Prompt: RP-P16

### Files

```text
solver/options.rs
solver/pipeline.rs
solver/orchestrator.rs
result/status.rs
result/output.rs
problem/semantic.rs
verify/run_certificate.rs
```

### Implementation steps

1. Audit every exact-image option, status, finalizer, and classification hook.

2. Replace any exact-image success/no-op/filtering path with an explicit scoped response:
   - allowed failure status such as `CertificateDesignGap`,
   - diagnostic reason `ExactImageOutOfScope`,
   - input hash, support hash when available, candidate hash when available,
   - no `CertifiedExactTargetImage` result.

3. Ensure candidate-cover mode:
   - never calls exact-image classification,
   - never filters candidates by real-fiber/slack/guard semantics,
   - preserves semantic provenance hashes in certificates where present.

4. Ensure replay/certificate code binds the exact-image scope guard if that path is requested.

5. Add static-audit checks for exact-image success/no-op strings if those paths remain reachable.

### Tests

- exact-image requested -> explicit `ExactImageOutOfScope` diagnostic or allowed failure status.
- exact-image requested -> no `CertifiedExactTargetImage`.
- candidate-cover requested -> decoded candidates are unchanged by semantic encodings except for provenance hashes.
- exact-image scope guard is replay/certificate-bound.

### Forbidden shortcuts

- exact image no-op.
- accepting or rejecting candidates through partial exact-image filtering.
- returning exact-image success from finite candidate-cover evidence.

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
   - include finite candidate-cover mode and explicit exact-image-out-of-scope guard if the option is exposed,
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
   - exact-image scope guard if requested,
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
- exact-image mode returns only the explicit out-of-scope guard/status.

### Forbidden shortcuts

- skipping verify support.
- root isolation before support verification.
- panic on solver error.

---

## Phase 18 — Finite candidate-cover conformance suite, static audit, reviewers, closure

Supports: BS-R000 through BS-R150  
MECHs: all  
Reviewer Prompt: RP-P18

### Tasks

1. Add `geosolver-core/tests/v4_candidate_cover_conformance.rs` with integration tests covering all in-scope candidate-cover acceptance categories.

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

The final answer may claim `FINITE_CANDIDATE_COVER_COMPLETE` and `SOURCE_FAITHFUL_TO_V4_FINITE_CANDIDATE_COVER_LAYER` only if:
- all in-scope Base Spec R-IDs have code evidence,
- all required tests/audits pass,
- all reviewers PASS,
- no blocking QuestionDebt remains,
- git status is recorded,
- closure cites fresh evidence.
