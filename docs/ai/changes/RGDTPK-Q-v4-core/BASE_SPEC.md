# Change Base Spec: R-GDTPK-Q / ACCTP-Q solver core implementation

## Context Packet

Spec ID: `RGDTPK-Q-v4-core`  
Type: Change Base Spec  
Status: DRAFT FOR USER APPROVAL — v2.2 consistency-audited  
Parent: none in an empty repo; if a repo-level Base Spec later exists, this Change Base Spec must be checked for conflict before implementation continues.  
Scope: implement `geosolver-core` solver core from an empty repository, exactly following the supplied algorithm specification.  
Applies To: `geosolver-core/` Rust crate and Guardian evidence/review artifacts under `docs/ai/changes/RGDTPK-Q-v4-core/`.  
Required Parent R-IDs: none at draft time. v2/v2.2 hardening adds RGQ-041 through RGQ-064 as normative requirements.  
Blocking Questions: none. The uploaded v4 algorithm spec is treated as the full source of technical requirements, with hardening amendments `RGQ-041` through `RGQ-064` in this file.  
Non-blocking Debt: none accepted for final strong claim. Any unimplemented file, function, algorithm path, certificate path, root isolation path, or exact-image API is blocking.  
Known Exceptions: none.  
Read-First R-IDs: `RGQ-000`, `RGQ-001`, `RGQ-004`, `RGQ-011`, `RGQ-015`, `RGQ-018`, `RGQ-019`, `RGQ-022`, `RGQ-023`, `RGQ-024`, `RGQ-031`, `RGQ-032`, `RGQ-041` through `RGQ-064`.  
Last Reviewed: not yet reviewed.  
Read full file only when: admitting the Base Spec, admitting the Plan, closing any MECH, making a source-fidelity claim, resolving a suspected drift, or making a final strong claim.  
Context Packet Authority: non-authoritative digest. The body below is authoritative.

---

## 1. Source authority

### RGQ-000 — Source hierarchy and fidelity

**Requirement.** The normative algorithm source is the uploaded file `geosolver_core_r_gdtpk_q_algorithm_spec_v4_algebraic_cost_compression(3).md`, copied into this pack as `docs/ai/sources/geosolver_core_r_gdtpk_q_algorithm_spec_v4.md` with SHA-256:

```text
2dc2f950896ff3e60858b17bf3f1867667564ae773e0a71d6db8c0953143caed
```

The normative failure-prevention source is the uploaded file `geosolver_failure_causes_generalized_2026_07_04(1).md`, copied into this pack as `docs/ai/sources/geosolver_failure_causes_generalized_2026_07_04.md` with SHA-256:

```text
df0d9d525a022f1851fe8021c70fea97d10408425e7b2670bf991858723ae14e
```

Appendix A of this file contains the algorithm specification verbatim and is part of this Base Spec. Appendix B contains the failure-prevention document verbatim and is part of this Base Spec for process, review, and anti-drift requirements.

**Conflict rule.** Appendix A is the base algorithm source. However, the hardening requirements `RGQ-041` through `RGQ-064` are explicit normative tightening amendments. They override weaker or unsafe wording in Appendix A, earlier summary R-IDs, the Plan, reviewer prompts, or implementation notes, but only in the stricter direction: they must never permit behavior that Appendix A forbids. If any artifact conflicts with Appendix A plus these hardening R-IDs, Appendix A plus the hardening R-IDs win. If Appendix B conflicts with Appendix A on algorithm semantics, Appendix A wins except where `RGQ-041` through `RGQ-064` explicitly tighten a failure-prevention rule. The Agent must stop and record `PlanDefect` or `BaseSpecConflict`; it must not silently choose an easier interpretation.

**No narrowing rule.** The Base Spec cannot be satisfied by a declared slice, a small stress subset, a preflight protocol, a scaffold, or a package/certificate shape without the actual algorithmic behavior.

---

## 2. Project scope

### RGQ-001 — Implement the algebraic solver core, not the geometry DSL

The repository must implement the solver core whose input is already a rational polynomial target problem. It must not implement geometry parsing, diagram recognition, natural language lowering, fixture-specific geometry formulas, or geometry-family handlers. The solver core must see polynomial equations, guards, branches, target relations, algebraic dependency, finite target-candidate structure, and projection structure, not geometry family names.

### RGQ-002 — Mathematical problem and outputs

The solver must accept a well-formed sparse rational polynomial system

```text
F = {f1, ..., fm} ⊂ Q[x1, ..., xn, T]
```

with target variable `T`. In candidate-cover mode it must return a nonzero univariate support polynomial `S(T) ∈ Q[T]` when the finite target image can be covered, such that every true target value is a root of `S`. Spurious roots are allowed in `CertifiedCandidateCover`. The solver must then squarefree `S`, isolate real roots exactly, and decode candidates bound to the support hash and root index.

### RGQ-003 — Candidate cover and exact image are separate modes

The implementation must distinguish:

```text
CertifiedCandidateCover: S(T) covers all true target values; spurious roots may remain.
CertifiedExactTargetImage: real fiber, guard, slack, and branch semantics have been checked and only realizable target values remain.
```

Exact-image APIs and data structures must exist from the first complete implementation. Exact-image status may only be returned after the RealFiberClassifier has checked the relevant semantics. Candidate-cover status must not pretend to have performed exact real-fiber classification.

### RGQ-004 — Non-negotiable forbidden paths

The production path must not:

1. build a full coordinate solution list;
2. build a full coordinate RUR;
3. build a full coordinate lex parametrization and then read `T`;
4. use generic QE/CAD or generic RCF as hidden fallback;
5. use an external CAS answer as the certified production proof;
6. call any planner-unrecorded hidden fallback after a kernel fails;
7. branch on geometry names, problem names, fixture names, expected answers, or official solutions;
8. support only a narrow formula shape and return ordinary `Unsupported` for the rest;
9. treat preflight, review PASS, evidence files, or scaffolding as algorithm completion;
10. let DAGs, certificates, feature records, or coverage matrices become decorative artifacts not controlling execution.

### RGQ-005 — Allowed heavy algebra must remain target/separator-direct

The implementation may use local block target/separator elimination, exact modular F4/F5-like sparse linear algebra, target/separator-only resultants or eliminants, multiplication-by-target operators, target-relevant quotient/action handles, sparse resultant templates, regular-chain projection, norm/trace computation, and specialization-interpolation. These are allowed only when chosen by a deterministic pre-execution plan, exporting only target/separator relations, never exporting coordinate roots or full coordinate RURs, and carrying exact Q verification, certificate, and cost trace.

### RGQ-006 — Failure status semantics

For well-formed Q-polynomial systems, ordinary `Unsupported` is not a valid solver outcome. The allowed failure statuses are exactly:

- `FiniteResourceFailure`, with stage, block, matrix/rank/height/memory evidence where applicable;
- `AlgorithmicHardCase`, with minimal obstruction, matrix size, rank estimate, degree bound, and algebraic reason;
- `CertificateDesignGap`, when an object was built but cannot be certified in the current certificate language;
- `ImplementationBug`, when a specified invariant is violated;
- `InvalidInput`, only for inputs that are not well-formed Q-polynomial target systems.

---

## 3. Required architecture and file layout

### RGQ-007 — Exact folder layout

The crate root must be `geosolver-core/` and must contain exactly the module families required by Appendix A section 6:

```text
geosolver-core/
  Cargo.toml
  README.md
  src/
    lib.rs
    api.rs
    types/
    problem/
    algebra/
    preprocess/
    graph/
    planner/
    kernels/
    compose/
    verify/
    roots/
    fiber/
    result/
    solver/
```

Every file listed in Appendix A section 6 must exist. The function, type, and trait names listed in Appendix A sections 7 through 29 must exist with the specified semantic role. The Agent may add helper files only when they do not replace, skip, weaken, or rename required files and functions.

### RGQ-008 — Public API

`src/lib.rs` must expose the modules in Appendix A section 7.1. `src/api.rs` must expose exactly the top-level call:

```rust
pub fn solve_target(
    problem: RationalTargetProblem,
    options: SolverOptions,
) -> TargetSolveResult
```

The API must initialize a `SolverContext`, call `solver::orchestrator::solve_with_context`, and return a `TargetSolveResult` with status rather than panicking for solver errors.

### RGQ-009 — Core algebraic data model

The implementation must provide stable ID types, canonical rational numbers, monomials, sparse multivariate polynomials, univariate polynomials, matrices, intervals, hashes, problem input records, canonical systems, projection messages, target result records, statuses, diagnostics, and cost traces as specified in Appendix A sections 5, 8, and 28. All invariants listed there are mandatory. `variable_roles` are provenance only and must not control algorithmic dispatch.

### RGQ-010 — Exact rational polynomial arithmetic

The crate must implement exact rational normalization, polynomial normalization, monomial arithmetic, univariate arithmetic, gcd, squarefree part, derivative, substitution, clear-denominator primitive normalization, matrix hashing, and interval operations. Floating-only logic is forbidden for exact algebra, certification, and root isolation.

---

## 4. Pipeline requirements

### RGQ-011 — Top-level pipeline

`solve_target` must execute the Appendix A section 4.1 pipeline in order:

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
OptionalRealFiberClassification
FinalizeResultAndCertificate
```

The implementation must enforce the Appendix A section 4.2 invariants: Q-polynomial input, no geometry dispatch, no coordinate solution list, only target/separator exports from blocks, exact Q certificates for exports, exact candidate-cover verification for final `S(T)`, exact-image semantic validation when enabled, cost trace on failure, deterministic planner, and no hidden fallback.

### RGQ-012 — Validation and canonicalization

`problem/validate.rs` and `problem/canonicalize.rs` must implement Appendix A section 9. Validation must reject only invalid target declarations, non-finite/non-rational polynomials, or invalid semantic references. It must not reject because a variable is a coordinate, because branches/slacks exist, or because geometry labels are absent. Canonicalization must clear denominators, normalize polynomials, remove zero relations with diagnostics, handle nonzero constants exactly, and preserve semantic/certificate facts.

### RGQ-013 — Pre-kernel algebraic compression

`preprocess/` must implement definitional elimination, safe linear-affine elimination, binomial simplification, explicit saturation, and target-independent component marking exactly as Appendix A section 11 specifies. Unsafe affine substitutions and component/factor choices without guard/certificate records are forbidden.

### RGQ-014 — Graph construction and TargetProjectionDAG

`graph/` must implement relation-variable hypergraph, target influence graph, weighted primal graph, separator candidates, target-rooted decomposition, target projection DAG, authorization hashes, DAG validation, and metrics exactly as Appendix A section 12 specifies. If no useful separator exists, the implementation must build one large target block and still feed it into the generic target-direct planner; it must not declare the input unsupported.

### RGQ-015 — Deterministic planner and declared ladder

`planner/` must implement cost probes, admissions, cost estimates, declared ladders, and `plan_all_blocks` as Appendix A section 13 specifies. Every block must receive a deterministic `KernelPlan`. The ladder is not hidden fallback: every candidate kernel execution plan must be declared before execution and included in the certificate. `UniversalTargetEliminationKernel` must be present as the final generic target-direct plan for well-formed blocks.

### RGQ-016 — Kernel trait and registry

`kernels/traits.rs` must define `TargetProjectionKernel` with `kind`, `admit`, `plan`, `execute`, and `replay`. `kernels/mod.rs` must return all nine kernels in Appendix A section 14.2:

1. TargetUnivariateKernel
2. LinearAffineKernel
3. TargetRelationSearchKernel
4. SparseResultantProjectionKernel
5. TargetActionKrylovKernel
6. NormTraceProjectionKernel
7. RegularChainProjectionKernel
8. SpecializationInterpolationKernel
9. UniversalTargetEliminationKernel

A kernel execute path must construct target/separator relations, produce exact Q certificates, and not output local coordinate solutions.

---

## 5. Kernel requirements

### RGQ-017 — TargetUnivariateKernel

Must implement Appendix A section 15. It finds nonzero relations whose variables are contained in `{T}`, computes primitive LCM squarefree support, and returns a `PrincipalSupport` message with source membership certificate. Admission and execute must agree; invalid admission is `ImplementationBug`.

### RGQ-018 — LinearAffineKernel

Must implement Appendix A section 16. It eliminates triangular affine variables only through constant nonzero pivots or recorded nonzero guards, records denominator guards, clears denominators, returns only exported-variable relations, and reports `AlgorithmicHardCase` when a nonzero local variable remains after planned elimination.

### RGQ-019 — TargetRelationSearchKernel

Must implement Appendix A section 17 and is a central generic target-direct workhorse, not an optional toy. For a local ideal `J = <f_i> ⊂ Q[Y,Z]`, it must search for a nonzero `g(Z) ∈ J ∩ Q[Z]` using unknown export coefficients and multiplier coefficients, coefficient comparison, modular homogeneous solving, rational reconstruction, and exact membership verification of

```text
g - Σ_i q_i f_i = 0.
```

It must build deterministic export monomial supports and multiplier supports, record matrix traces, and return `AlgorithmicHardCase` with evidence only after exhausting declared bounds. It must not return a candidate relation without exact membership proof.

### RGQ-020 — SparseResultantProjectionKernel

Must implement Appendix A section 18. It uses support sets and sparse resultant templates to compute a target/separator relation, reconstructs it exactly, verifies the resultant or membership certificate, and returns only exported-variable relations. “Not sparse enough” is only admission false for this kernel, not solver unsupported.

### RGQ-021 — TargetActionKrylovKernel

Must implement Appendix A section 19. It may return an annihilator only when the target-relevant quotient handle exposes no coordinate roots/RUR and Krylov coverage is certified. A single Krylov sequence without coverage proof cannot produce a candidate polynomial.

### RGQ-022 — UniversalTargetEliminationKernel

Must implement Appendix A section 20. It is a planned generic target/separator projection kernel, not a hidden fallback. Admission is true for blocks with Q-polynomial relations. Execute must eliminate local variables to exported variables using a planned local strategy, gather only exported-variable generators, verify every generator exactly, and return a `ProjectionMessage` only when at least one exported generator is certified. When no exported relation exists, it must return `AlgorithmicHardCase`, `FiniteResourceFailure`, or `CertificateDesignGap` according to the exhausted planned step; it must not return `CertifiedNonFiniteTargetImage` from a local block. It must never enumerate coordinate solutions, produce full coordinate RUR, invoke global QE/CAD, branch on geometry, or run a runtime hidden fallback.

### RGQ-023 — RegularChainProjectionKernel

Must implement Appendix A section 21, preserving component/guard/projection semantics through compact component DAGs and verified chain projections.

### RGQ-024 — NormTraceProjectionKernel

Must implement Appendix A section 22 based on algebraic tower detection, norm of target-minus-expression, and exact norm relation verification. Detection must be by algebraic form, not geometry name.

### RGQ-025 — SpecializationInterpolationKernel

Must implement Appendix A section 23. It may specialize separators, compute target-only relations through a declared inner kernel, interpolate sparse coefficients, and accept the result only after exact Q verification by membership or elimination. Specialization/interpolation is candidate generation, not proof.

---

## 6. Composition, verification, roots, fiber, and result requirements

### RGQ-026 — Projection message composition and final support

`compose/` must implement Appendix A section 24. Messages must merge into message ideals, compose bottom-up through the DAG, eliminate remaining separators using only message relations and target-direct kernels, never reconstruct the original full coordinate system, and build final `S(T)` from target-only root relations or return certified nonfinite/algorithmic-hard-case status.

### RGQ-027 — Certificates and verification

`verify/` must implement Appendix A section 25. Every projection message must verify exported variables and generator variables, then verify the certificate variant. `verify_global_support` must prove that the support polynomial vanishes on all true target fibers. Replay must re-check input hash, canonicalization hash, DAG hash, projection messages, support verification, root isolation hash, and candidate hash. A candidate object that cannot be verified must produce `CertificateDesignGap`, not success.

### RGQ-028 — Exact root isolation and candidate decode

`roots/` must implement Appendix A section 26. `squarefree_support` must compute `p / gcd(p,p')` exactly. `isolate_real_roots` must use exact Sturm, Descartes-Vincent, or deterministic exact isolation. Floating-only roots or approximate roots without rational isolating intervals are forbidden. `decode_candidates` must bind each candidate to target, support hash, root index, isolating interval, and candidate hash.

### RGQ-029 — Real fiber classification API

`fiber/` must implement Appendix A section 27. Exact image mode must build algebraic target fiber problems, attach slack/guard semantics, decide real fiber nonemptiness with certificates, and use Hermite/Thom/sign/slack semantics helpers. Exact image status is forbidden unless these semantics have actually been evaluated for the candidate set.

### RGQ-030 — Result, diagnostics, cost trace, and orchestrator

`result/` and `solver/` must implement Appendix A sections 28 and 29. `SolverOptions`, stage functions, and `solve_with_context` must exist and match the specified pipeline. Success and failure results must include status, support, squarefree support, root isolation, candidates, messages, certificate, diagnostics, and global cost trace as applicable.

---

## 7. Algebraic-cost compression and performance claim limits

### RGQ-031 — Cost trace is required, benchmark claims are not allowed by this Base Spec

Every run must record the algebraic-cost compression parameters in Appendix A section 30: total variables, equations, degrees, monomial count, coefficient height, maximum block width, separator width, local quotient/action ranks, local matrix sizes, coefficient height growth, final support degree, and certificate size. This Base Spec does not authorize claims such as “faster than Groebner/homotopy/CAS” without a separate benchmark spec. The allowed claim is only that the implementation follows the target/separator local projection architecture and records the cost trace.

### RGQ-032 — Nonfinite target image handling

The implementation must handle `I ∩ Q[T] = {0}` as Appendix A section 31 specifies. If nonfiniteness can be certified, return `CertifiedNonFiniteTargetImage`; otherwise return `AlgorithmicHardCase` with reason. It must not turn nonfinite or not-yet-certified cases into ordinary unsupported or empty candidate lists.

### RGQ-033 — Geometry-derived footprint, not geometry dispatch

The solver may use algebraic footprint derived from sparse incidence graphs, low degree, affine eliminability, small separators, slack/guard encodings, algebraic towers, and target-independent components. It must not branch on terms like circle, triangle, tangent, distance, area, incircle, circumcircle, orthic, mixtilinear, fixture, official solution, or expected answer.

### RGQ-034 — Completion criteria

The implementation is complete only when all Appendix A section 33 conditions hold:

1. any well-formed Q-polynomial target system enters the generic pipeline;
2. no geometry-name dispatch exists;
3. no problem-id, fixture-id, or expected-answer dispatch exists;
4. `TargetProjectionDAG` is built for all valid input;
5. no useful separator still produces one large generic target block;
6. every block gets a deterministic `KernelPlan`;
7. `UniversalTargetEliminationKernel` exists and returns target/separator-only output;
8. production path makes no full coordinate solution list;
9. production path makes no full coordinate RUR;
10. success creates `S(T) ∈ Q[T]` and exact Q verification passes;
11. root isolation is exact;
12. decoded candidates are bound to support hash and root index;
13. exact image mode handles real fiber, guard, and slack semantics;
14. failure is an evidence-bearing status, not `Unsupported`;
15. cost trace records all algebraic-cost-compression parameters;
16. hidden fallback is API-impossible;
17. narrow-slice completion is API-impossible.

---

## 8. Failure-prevention requirements from the generalized failure document

### RGQ-035 — No phase/gate substitution for the real objective

The Agent must always evaluate progress by whether the implementation directly, generically, and exactly enumerates target candidate covers from algebraic IR. Phase closure, reviewer PASS, evidence presence, review summary files, scaffolded certificates, or support packages on tiny examples are not evidence of algorithmic completion.

### RGQ-036 — Heavy fallback ban and narrow-scope ban must both hold

The implementation must not choose between “generic but full-coordinate heavy fallback” and “no fallback but narrow slice.” It must satisfy both: no heavy fallback and no narrow-scope completion. If the current algorithm cannot build a support relation for a well-formed case within resources, the status must name the algebraic obstruction; it must not declare the case outside the solver’s supported slice.

### RGQ-037 — Generalized real-problem algebraic structures must be success targets

The verification suite and reviewer checks must include generalized algebraic structures derived from real failures without using problem names, expected answers, or fixture IDs.

The following are **support-producing success obligations**. Each must be represented by at least one case that returns `CertifiedCandidateCover` or `CertifiedExactTargetImage`, with nonzero support when real target roots exist, exact support verification, exact squarefree/root isolation, and non-placeholder decoded candidates:

- coordinate-role variables treated only as algebraic variables;
- multi-variable projection with separators;
- guard and branch encodings with provenance while still producing candidate-cover support;
- determinant/oriented-bilinear expressions as ordinary polynomials;
- dot/Gram-like bilinear/quadratic structures as ordinary polynomials;
- explicit algebraic towers with norm/trace projection or a verified equivalent target projection;
- target action/quotient cases requiring `VerifiedCharacteristicSupportCoverage`;
- separator specialization/interpolation with exact final verification;
- one-large-block generic target-direct projection when separators are not useful;
- exact root isolation and non-placeholder decoded candidates.

The following are **failure-semantics obligations**, not substitutes for the support-producing obligations above:

- certified nonfinite target image through the positive proof required by `RGQ-045`;
- relation-search exhaustion returning `AlgorithmicHardCase` or `FiniteResourceFailure`, never `CertifiedNonFiniteTargetImage`;
- certificate-gap cases returning `CertificateDesignGap` with the constructed-object evidence.

These stress cases must not become the solver’s implementation dispatch table. They are verification probes for a generic algorithm. Any earlier wording that grouped “nonfinite target image certification or hard-case evidence” with support-producing success targets is superseded by this split and by `RGQ-048`.

### RGQ-038 — DAGs, plans, admissions, and certificates must be operational

DAG nodes, authorization hashes, admissions, kernel plans, declared ladders, certificates, and replay data must control execution. If a DAG, plan, authorization hash, or certificate is removed, mismatched, or altered, the run or replay must fail. These artifacts may not be decorative evidence objects.

### RGQ-039 — Admission must imply a support-producing execution plan, not just a label

For every kernel selected in a ladder, admission and `plan()` must provide enough concrete information to attempt a support-producing projection: exported variables, eliminated variables, degree/support/template/rank plans, resource bounds, certificate route, and failure behavior. A kernel may decline admission. A selected kernel may fail with evidence. It may not be selected based only on a feature label.

### RGQ-040 — Reviewer prompts must check algorithmic sufficiency

Reviewers must judge whether the actual implementation is a coherent R-GDTPK-Q/ACCTP-Q target candidate cover algorithm, not merely whether names, files, evidence, and gates are present. A reviewer must fail the phase if it sees partial support being presented as solver completion, unsupported hiding algorithmic gaps, preflight being treated as proof, placeholder candidates, empty root isolation after support, hidden fallback, or geometry dispatch.

---

## 8A. v2 hardening requirements for the eight identified gaps

The requirements in this section are normative. They do not replace Appendix A. They remove implementation choices that were still too broad in the first draft. If any earlier summary sentence appears to permit a weaker implementation, this section wins unless Appendix A explicitly requires the opposite. Appendix A remains the source of the mathematical algorithm; this section fixes the execution policy, certificate policy, phase closure policy, and final-claim policy.

### RGQ-041 — UniversalTargetEliminationKernel must be bounded local target/separator projection, not heavy fallback

`UniversalTargetEliminationKernel` is not a permissive fallback and must not be implemented as “try arbitrary elimination until something works.” It is a planned, bounded, local target/separator projection kernel.

A valid `UniversalEliminationExecutionPlan` must contain all fields below and must be hash-bound before execution:

```text
block_id
input_block_authorization_hash
source_relation_hashes
child_message_hashes
exported_variables Z
eliminated_variables Y
fixed_strategy_sequence
relation_search_schedule_hash
local_elimination_budget
specialization_interpolation_budget
resource_budget
certificate_route
local_nonfinite_policy = NoLocalCertifiedNonFinite
forbidden_coordinate_export_flags
```

The fixed strategy sequence is exactly:

```text
1. TargetRelationSearchEscalated using RGQ-042 dense schedule.
2. SparseResultantIfSquareOrOverdetermined only when the planned sparse template has exact verification route.
3. SpecializeProjectInterpolateVerify only when Z contains non-target separators and only with final exact Q membership/elimination verification.
4. LocalF4OrGroebnerEliminationToKeepZ only inside the authorized block and only with exported variables Z as keep variables.
```

No other internal strategy may run unless a later approved Base Spec amendment names it. A listed step may be skipped only when its algebraic precondition is false, and the skip reason must be stored in the plan/cost trace. The kernel may return `ProjectionMessage` only when every returned generator lies in `Q[Z]` and every generator has an exact Q certificate. It must return `FiniteResourceFailure`, `AlgorithmicHardCase`, or `CertificateDesignGap` when the planned sequence cannot produce a certified exported relation. It must not return `CertifiedNonFiniteTargetImage` from a local block.

Forbidden Universal behavior:

```text
- reading relations outside the block authorization hash;
- reconstructing the full original coordinate system during execution;
- unbounded global Groebner/F4 as a last resort;
- coordinate solution enumeration;
- full coordinate RUR;
- target-unrelated quotient basis export;
- generic QE/CAD/RCF;
- runtime selection of a strategy not present in the predeclared plan;
- treating “no relation found” as certified nonfinite.
```

Required functions in `kernels/universal_elimination.rs`:

```rust
pub fn admit_universal_elimination(block: &ProjectionBlock, ctx: &KernelContext) -> KernelAdmission;
pub fn build_universal_elimination_plan(admission: &KernelAdmission, ctx: &KernelContext) -> Result<KernelExecutionPlan, SolverError>;
pub fn execute_universal_elimination(plan: &KernelExecutionPlan, ctx: &mut KernelContext) -> Result<ProjectionMessage, SolverError>;
pub fn execute_universal_stage(stage: &UniversalStagePlan, ctx: &mut KernelContext) -> Result<ProjectionMessage, SolverError>;
pub fn verify_universal_no_coordinate_fallback(plan: &KernelExecutionPlan, trace: &ProjectionCostTrace) -> Result<(), SolverError>;
pub fn extract_verified_export_generators(result: &LocalEliminationResult, exported: &[VariableId]) -> Result<Vec<SparsePolynomialQ>, SolverError>;
```

Closure oracle: a semantic deletion test must alter the block authorization hash or remove a child message and make Universal execution or replay fail. A one-large-block case must pass through this kernel without any geometry dispatch or full-coordinate solve.

### RGQ-042 — TargetRelationSearch degree and support schedule is fixed

`TargetRelationSearchKernel` must implement the following deterministic dense total-degree membership schedule. Implementers may add sparse accelerated attempts only before this schedule, and only if the dense schedule remains implemented and used as the required correctness baseline.

Definitions for a local search problem `J = <f_1,...,f_r> ⊂ Q[Y,Z]`:

```text
Z = sorted exported variables = target plus separators
Y = sorted local variables not in Z
d_i = total_degree(f_i)
d_max = max_i d_i
zdeg_i = total degree of f_i after ignoring exponents of variables in Y
z_seed = max(1, max_i zdeg_i)
e_cap_default = max(z_seed, 2*d_max + |Y| + |Z|)
e_cap = options.max_relation_search_export_degree.unwrap_or(e_cap_default)
```

The declared bounds are exactly:

```text
for e in z_seed..=e_cap:
    L_e = e + d_max
    A_e = all monomials in Q[Z] of total degree <= e, sorted by canonical block order
    B_i,e = all monomials in Q[Y,Z] of total degree <= max(0, L_e - d_i), sorted by canonical block order
    C_e = union(A_e, all products b * monomial(f_i) for b in B_i,e and monomial(f_i) in support(f_i))
```

The membership matrix for stage `e` is the coefficient comparison matrix of:

```text
g(Z) - Σ_i q_i(Y,Z) f_i(Y,Z) = 0
```

where `support(g)=A_e` and `support(q_i)=B_i,e`. `C_e` is the exact row monomial set. A candidate is valid only if:

```text
g != 0
variables(g) ⊆ Z
g - Σ_i q_i f_i == 0 over Q exactly
```

If all stages are exhausted without a valid relation, the only allowed outcomes are `AlgorithmicHardCase` with the full stage trace or `FiniteResourceFailure` if an explicit resource bound was exceeded. This condition is never evidence of nonfiniteness.

Required functions in `kernels/target_relation_search.rs`:

```rust
pub fn relation_search_default_export_degree_cap(j: &[SparsePolynomialQ], eliminated: &[VariableId], exported: &[VariableId]) -> usize;
pub fn build_dense_relation_search_schedule(j: &[SparsePolynomialQ], eliminated: &[VariableId], exported: &[VariableId], options: &SolverOptions) -> Vec<RelationSearchBound>;
pub fn build_export_monomial_support(exported: &[VariableId], bound: &RelationSearchBound) -> Vec<Monomial>;
pub fn build_multiplier_supports(relations: &[SparsePolynomialQ], eliminated: &[VariableId], exported: &[VariableId], bound: &RelationSearchBound) -> Vec<Vec<Monomial>>;
pub fn build_membership_matrix_builder(j: &[SparsePolynomialQ], bound: &RelationSearchBound) -> MembershipMatrixBuilder;
pub fn reconstruct_and_verify_relation(ns: &ModularNullspaceResult, bound: &RelationSearchBound, j: &[SparsePolynomialQ]) -> Result<Vec<VerifiedRelationSearchCandidate>, SolverError>;
```

Closure oracle: reviewer must be able to recompute the schedule from `J,Y,Z,options` and obtain the same support hashes as the implementation. Hard-coded stress supports, hand-picked degree caps, or sparse-only schedules fail this R-ID.

### RGQ-043 — Sparse relation-search support is optimization only

`SparseFromProjectionFootprint`, `SpecializedInterpolationFootprint`, mixed-support heuristics, and any future sparse relation-search support may reduce cost, but they are not allowed to replace RGQ-042. A phase or final claim may not pass because a sparse heuristic succeeds on a small stress case while the dense deterministic schedule is absent, unreachable, or untested.

### RGQ-044 — TargetActionKrylov has exactly one accepted coverage proof

`TargetActionKrylovKernel` may return a support polynomial only through `VerifiedCharacteristicSupportCoverage`. No other coverage proof is accepted in this Base Spec.

Mandatory method:

```text
1. Build a target-relevant quotient/action handle with finite rank r.
2. Materialize the multiplication-by-target matrix M_T column by column using handle.multiply_by_variable on the handle basis.
3. For every column, verify by exact normal form that it equals multiplication by T in the quotient handle.
4. Compute χ_T(λ) = det(λI - M_T) exactly, using deterministic exact Q or modular reconstruction with exact Q verification.
5. Verify Cayley-Hamilton exactly by checking χ_T(M_T)=0 as a matrix over the quotient/action representation.
6. Return χ_T(T) as the candidate-cover relation, followed by normal support verification and root isolation.
```

The certificate must contain:

```yaml
TargetActionKrylovCertificate:
  coverage_kind: VerifiedCharacteristicSupportCoverage
  quotient_handle_hash: ""
  basis_hash: ""
  quotient_rank: 0
  target_action_matrix_hash: ""
  column_normal_form_certificate_hashes: []
  characteristic_polynomial_hash: ""
  cayley_hamilton_verification_hash: ""
  no_coordinate_roots_exported: true
  no_full_coordinate_rur_exported: true
```

A single Krylov sequence, random vector, block Wiedemann claim, trace-power claim, or `S(M_T)=0` without characteristic support coverage is insufficient. If the matrix cannot be certified, the kernel must return `CertificateDesignGap` or `FiniteResourceFailure`, not a candidate polynomial.

Closure oracle: include a quotient/action test where a single starting vector misses one target eigenvalue. The kernel must either construct the characteristic coverage and include the missing eigenvalue in the support, or refuse to return a relation.

### RGQ-045 — CertifiedNonFiniteTargetImage requires positive nonfiniteness proof

`CertifiedNonFiniteTargetImage` must never mean “relation search failed.” It may be returned only by final-support/nonfinite certification, not by local block Universal execution.

A valid `NonFiniteTargetImageCertificate` must prove all items below:

```text
1. The composed target ideal is consistent over the relevant algebraic semantics, or exact-image mode supplies the stronger real certificate in item 5.
2. An exact elimination basis or exact regular-chain dimension certificate has been built for the composed root ideal with keep variable {T}.
3. The certificate proves I ∩ Q[T] = {0}; for a Groebner certificate this means the elimination basis for Q[T] is exactly empty and all basis reductions and memberships are certified over Q.
4. The certificate proves that the projection to T is algebraically nonfinite, not merely that the implemented search did not find a relation within bounds.
5. In exact-image mode, a `RealNonFiniteTargetCertificate` must also prove nonfinite real target image under guard/slack/branch semantics. Without this real certificate, exact-image mode must return `AlgorithmicHardCase` or `CertificateDesignGap`.
```

Required functions:

```rust
pub fn certify_nonfinite_target_image(composed: &ComposedProjection, ctx: &mut SolverContext) -> Result<NonFiniteCertificate, SolverError>;
pub fn certify_zero_target_elimination_ideal(composed: &ComposedProjection, ctx: &mut SolverContext) -> Result<TargetEliminationZeroCertificate, SolverError>;
pub fn verify_nonfinite_certificate(cert: &NonFiniteCertificate, composed: &ComposedProjection) -> Result<(), SolverError>;
pub fn certify_real_nonfinite_target_image(input: RealNonFiniteInput, ctx: &mut SolverContext) -> Result<RealNonFiniteTargetCertificate, SolverError>;
```

Closure oracle: one test must show relation-search exhaustion returns `AlgorithmicHardCase`; a separate constructed nonfinite system must return `CertifiedNonFiniteTargetImage` with the certificate above.

### RGQ-046 — Phase closure cannot be won with stubs, wrappers, or broad mixed work packages

Each Plan phase or subphase may close only the R-IDs and MECHs for which controlling functions are semantically implemented and behavior-tested. File creation, type names, thin wrappers, empty vectors, `Ok(())`, unconditional `true`, static hashes, fake certificates, and compile-only tests are not implementation evidence.

For every algorithmic phase after P2, the Agent must produce:

```text
docs/ai/changes/RGDTPK-Q-v4-core/evidence/<phase>/function_implementation_table.yaml
```

with one entry per public function touched by the phase:

```yaml
- function: "module::function_name"
  controlling_requirement: "RGQ-... or MECH-..."
  semantic_role: "what mathematical work it performs"
  implementation_status: "implemented|helper_only|deferred_by_plan"
  non_stub_evidence: "test name, code path, or exact certificate identity"
  forbidden_shortcut_scan: "command/evidence path"
  reviewer_must_inspect: true
```

A phase must fail if any public function required by Appendix A or by `RGQ-041` through `RGQ-064` is marked `deferred_by_plan` after that function's owning Plan phase. `helper_only` is permitted only for private or data-only helper code that is not a controlling algorithm path and is not used to close an R-ID or MECH. Phrases such as "as needed", "hook", "foundation", or "skeleton" do not reduce the required function list.

### RGQ-047 — Review prompt, response, summary schema, and evidence manifest are mandatory and claim-bound

`REVIEW_SUMMARY_SCHEMA.yaml` and `schemas/review_summary.schema.yaml` are mandatory companion schemas for every `review_summary.yaml`. They must be byte-identical mirrors; if they differ, no phase may close until both are updated to the stricter schema. `schemas/evidence_manifest.schema.yaml` is mandatory for every `evidence_manifest.yaml`.

Every reviewer invocation must be archived exactly under:

```text
docs/ai/changes/RGDTPK-Q-v4-core/reviews/<phase-id>/<YYYYMMDD-HHMMSSZ>/
  prompt.md
  response.md
  review_summary.yaml
  evidence_manifest.yaml
```

`prompt.md` is the exact prompt sent to the reviewer. `response.md` is the exact reviewer response. The Agent must not paraphrase or rewrite the reviewer response as the only record.

`review_summary.yaml` must validate against the machine-readable schema and must include the exact required fields listed in `REVIEW_ARCHIVE_SCHEMA.md` and `REVIEW_SUMMARY_SCHEMA.yaml`. The schema is closed: fields not listed there are invalid. Required semantic groups include:

```text
- artifact hashes, source document hashes, schema hashes, and archive paths;
- `algorithmic_sufficiency`;
- `forbidden_pattern_scan`, including non-spec status, required-function-deferred, Appendix-override-bypass, and stress-fixture-dispatch checks;
- `semantic_deletion_challenges` and `tamper_challenges`;
- `appendix_override_checks`;
- `status_mapping_checks`;
- `suite_partition_checks`;
- `blocking_findings` and `required_fixes`;
- `pass_conditions` with all v2.2 boolean checks;
- `claim_ceiling_after_phase`.
```

The markdown example in `REVIEW_ARCHIVE_SCHEMA.md` and the YAML schema must stay aligned. If they differ, the stricter rule wins and the Agent must stop for `BaseSpecConflict`.

A `PASS` is invalid unless all conditions below hold:

1. `review_status=PASS` and `phase_closable=true`.
2. Every hash in the summary matches the current file or archived file it names, including `evidence_manifest_sha256`.
3. `prompt_path`, `response_path`, and `evidence_manifest_path` point to archived files in the exact review directory.
4. All relevant `pass_conditions` are true.
5. `blocking_findings` and `required_fixes` are empty.
6. For every phase except pure setup P0, `algorithmic_sufficiency.verdict=sufficient`.
7. `not_applicable` is forbidden for P15 and P16 forbidden-pattern scans. For earlier phases it is allowed only when the phase prompt explicitly makes the scan irrelevant and `not_applicable_justifications` names the field with a nonempty rationale.
8. The raw `response.md` must not contain unresolved blockers, required fixes, or caveats that the summary overrides.
9. The reviewer must have inspected the controlling code files for the phase, not only evidence files.
10. For P5, P8d, P11, P15, and P16, semantic deletion and tamper challenges must be present and must fail in the intended way.

A non-PASS review must set `phase_closable=false`. The Agent must not close a phase from `review_summary.yaml` alone; the prompt, response, evidence manifest, function implementation table, command outputs, and code inspection are required evidence.

### RGQ-048 — Acceptance suites are separated by claim level

P15 acceptance is divided into three suites. The suites are not interchangeable:

```text
A. support_producing_candidate_cover_suite
B. exact_image_semantics_suite
C. failure_and_nonfinite_semantics_suite
```

Suite A is required for `CANDIDATE_COVER_CORE_READY`. Every Suite A case must run in candidate-cover mode or a stronger exact-image mode and must return `CertifiedCandidateCover` or `CertifiedExactTargetImage`, with nonzero `S(T)`, exact global support verification, exact squarefree support, exact real root isolation, and decoded candidates when real roots exist. `AlgorithmicHardCase`, `FiniteResourceFailure`, `CertificateDesignGap`, `CertifiedEmptyRealTargetImage`, `CertifiedNonFiniteTargetImage`, empty support, empty placeholder roots, and empty placeholder candidates are failures for Suite A.

Suite A must include at least:

```text
- coordinate-role variables treated purely algebraically;
- multi-variable separator projection;
- guard/branch/slack provenance with candidate-cover support still produced;
- determinant/oriented-bilinear polynomial structure;
- dot/Gram-like bilinear/quadratic structure;
- explicit algebraic tower handled by norm/trace or verified equivalent target projection;
- target-action quotient requiring VerifiedCharacteristicSupportCoverage;
- specialization/interpolation with final exact Q verification;
- one-large-block Universal path;
- exact root isolation and non-placeholder candidate decode.
```

Suite B is required for `EXACT_IMAGE_CORE_READY` and for `RGDTPK_Q_V4_ACCEPTANCE_COMPLETE`. Suite B must run exact-image mode and must prove real fiber, guard, slack, and branch semantics. It must include at least:

```text
- nonempty exact image after fiber classification;
- spurious candidate-cover roots removed by real semantics;
- empty real target image distinguished from candidate-cover support;
- guard/slack/branch semantics changing target feasibility;
- algebraic target roots classified with Hermite/Thom/sign/slack certificates;
- exact-image nonfinite returning `CertifiedNonFiniteTargetImage` only with `RealNonFiniteTargetCertificate`.
```

Suite B cannot be passed by candidate-cover evidence alone. If exact-image classification is missing, incomplete, or merely stubbed, Suite B fails and the maximum final claim is below `EXACT_IMAGE_CORE_READY`.

Suite C separately checks invalid input, finite resource failure, algorithmic hard case, certificate gap, relation-search exhaustion, and certified nonfinite target image. Passing Suite C does not compensate for any Suite A or Suite B failure.

### RGQ-049 — Final claim ladder prevents exact-image overclaim and candidate-cover/exact-image suite confusion

The only allowed closure claim labels are:

```text
SCAFFOLD_READY
PARTIAL_MECHANISM_READY:<MECH-ID>
CANDIDATE_COVER_CORE_READY
EXACT_IMAGE_CORE_READY
RGDTPK_Q_V4_ACCEPTANCE_COMPLETE
SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC
```

`CANDIDATE_COVER_CORE_READY` requires all candidate-cover pipeline requirements, Suite A from `RGQ-048`, exact root isolation, decode, global support verification, replay, and anti-fallback checks. It explicitly does not imply Suite B or exact-image completion.

`EXACT_IMAGE_CORE_READY` requires `CANDIDATE_COVER_CORE_READY` plus Suite B from `RGQ-048` and exact-image real fiber/guard/slack/branch classification evidence.

`RGDTPK_Q_V4_ACCEPTANCE_COMPLETE` is forbidden unless `EXACT_IMAGE_CORE_READY` is also valid. If exact image is incomplete, the final closure must stop at `CANDIDATE_COVER_CORE_READY` or below and must state that full acceptance is not complete.

`SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC` is a separate source-fidelity label. It is forbidden if any required Appendix A file, function, algorithm path, certificate path, root-isolation path, or exact-image API/semantics path is missing, renamed away, stubbed, or weaker than Appendix A plus the hardening R-IDs. It does not by itself make any benchmark or universal-completeness claim.

### RGQ-050 — Exact-image statuses require executed fiber semantics

`CertifiedExactTargetImage`, `CertifiedEmptyRealTargetImage`, and exact-image-mode `CertifiedNonFiniteTargetImage` may be returned only after the fiber layer has executed and verified the relevant real semantics. Candidate-cover evidence, complex elimination, or target support verification alone is insufficient.

### RGQ-051 — Relation-search failure and nonfinite proof are disjoint

No relation found within RGQ-042 bounds, sparse heuristic failure, Universal stage failure, or composition failure to produce target-only support must not route to `CertifiedNonFiniteTargetImage`. Such cases must route to `AlgorithmicHardCase`, `FiniteResourceFailure`, or `CertificateDesignGap` according to the available evidence. The only route to `CertifiedNonFiniteTargetImage` is the positive proof required by RGQ-045. This rule must be enforced in code and tested.

### RGQ-052 — Reviewer must inspect algorithmic control, not only evidence structure

Reviewers must inspect the code paths that control candidate-cover generation, support verification, root isolation, exact-image status, Universal execution, relation-search scheduling, ActionKrylov coverage, nonfinite certification, and replay. A reviewer response that only says files, tests, summaries, and hashes exist is insufficient and must be treated as `FAIL_FIXABLE` even if it says PASS.

### RGQ-053 — Heavy fallback detection must be static and dynamic

Closure evidence must include both static scans and dynamic semantic-deletion/tamper tests for heavy fallback and hidden fallback. Static scans alone are insufficient. Dynamic tests must prove that if the declared ladder, DAG authorization, child message, or certificate identity is altered, execution or replay fails rather than silently recomputing a global result.

### RGQ-054 — TargetActionKrylov undercoverage regression is mandatory

The test suite must include an action/quotient system where a single-vector Krylov/minimal-polynomial approach would miss at least one target eigenvalue. The production ActionKrylov path must pass only by constructing `VerifiedCharacteristicSupportCoverage` or by refusing to return a relation. A candidate polynomial from an undercovered Krylov sequence is an implementation bug.

### RGQ-055 — TargetRelationSearch schedule reproducibility is mandatory

For at least three local ideals with different `|Y|`, `|Z|`, and degrees, tests must recompute `build_dense_relation_search_schedule` independently and assert support hashes, row monomial hashes, matrix dimensions, and stage order. Any implementation-dependent support choice that cannot be reproduced from RGQ-042 fails.

### RGQ-056 — Universal local-elimination cap cannot become a global coordinate solver

Local F4/Groebner inside Universal must be guarded by explicit block-local resource caps and by exported-variable verification. If the current block is the one-large-block fallback, Universal may still run, but it must use the same target/separator export certificate and cannot output coordinate roots, coordinate parametrization, full coordinate RUR, or a lex basis intended to solve all coordinates before reading target.



## 8B. v2.2 consistency hardening requirements

The requirements in this section are normative consistency amendments. They do not change the mathematical goal. They remove contradictions and weak wording found during the final consistency audit of the v2.1 pack.

### RGQ-057 — Appendix A hardening overrides must be explicit at implementation time

Appendix A is preserved verbatim for source fidelity, but several Appendix A pseudocode passages are intentionally tightened by `RGQ-041` through `RGQ-064`. The Agent and reviewer must apply the table below exactly.

| Appendix A passage | Unsafe or broad reading | Mandatory controlling rule |
|---|---|---|
| §19.4 lists several possible TargetActionKrylov coverage approaches | Agent chooses a weak coverage proof such as a single Krylov sequence, block Wiedemann, trace-only, or `S(M_T)=0` without full support coverage | `RGQ-044` and `RGQ-054`: only `VerifiedCharacteristicSupportCoverage` may return a support polynomial |
| §20.3 local Universal pseudocode may call `certify_nonfinite_projection(J,Z)` when local exported generators are empty | Agent treats local Universal failure as `CertifiedNonFiniteTargetImage` | `RGQ-041` and `RGQ-051`: local Universal must not return nonfinite; exhaustion routes to hard/resource/certificate statuses |
| §20.4 lists internal Universal strategies without order | Agent reorders or adds hidden fallback strategies | `RGQ-041`: fixed strategy sequence only, hash-bound before execution |
| §24.4 final support pseudocode may call `certify_nonfinite_target_image(composed)` when no target-only relation exists | Agent treats no target-only support as nonfinite without a positive certificate | `RGQ-045` and `RGQ-051`: only positive nonfiniteness proof may produce `CertifiedNonFiniteTargetImage` |
| §31 nonfinite handling says target image may be nonfinite if `I ∩ Q[T] = {0}` | Agent treats search exhaustion as proof of `I ∩ Q[T] = {0}` | `RGQ-045`: exact elimination/dimension/regular-chain certificate is required |

Reviewer prompts must fail any implementation that cites Appendix A to bypass these hardening rules. `SOURCE_MAP.md` must include this override table, and P16 must run a source-fidelity challenge that checks all five rows.

### RGQ-058 — SolverStatus is a closed set and internal errors must map to it explicitly

The only allowed public result statuses are the `SolverStatus` variants in Appendix A §5.7. No `NotYetImplemented`, `Unsupported`, `Todo`, `Partial`, `Skipped`, `Unknown`, or phase-local status may be added to public results.

Internal functions may use `SolverError` or equivalent control-flow errors, but public `TargetSolveResult.status` must be created only by an explicit mapping layer. Pseudocode phrases such as `return Err(CertifiedNonFiniteTargetImage)` are shorthand for "stop this stage and finalize a result with that status"; the implementation must not mix status enum values into error enum values unless the type system makes the mapping explicit and reviewed.

Temporary scaffold behavior is allowed only before the owning phase connects the pipeline. A temporary scaffold must return one of the existing statuses with a diagnostic record named `TemporaryPipelineNotConnected`, must be listed in that phase's function implementation table, and must be removed before P14 closes. P14 and P16 reviewers must fail if any temporary scaffold status path remains reachable in production `solve_target`.

### RGQ-059 — Required files and functions are not optional, not “as needed,” and not helper-only after their owning phase

Every file and every public function named in Appendix A §6–§29 must exist with the specified semantic role. The Plan may assign implementation to phases, but it may not turn a required file or function into an optional hook. In particular:

```text
- `roots/algebraic_number.rs` is mandatory, not “as needed”.
- `algebra/resultant.rs` and `algebra/interpolation.rs` must implement actual verification primitives, not only hooks.
- `algebra/regular_chain.rs`, `algebra/norm_trace.rs`, `algebra/real_root.rs`, and `algebra/sign.rs` must implement the functions listed in Appendix A §10.16–§10.19, not only empty helper foundations.
- `fiber/*` exact-image modules must implement executable semantics before any exact-image claim.
```

A phase cannot close if any function owned by that phase is `todo!`, `unimplemented!`, a thin wrapper over an absent algorithm, an unconditional success verifier, a fake certificate builder, or marked `deferred_by_plan`. A later phase may extend a helper, but it may not be required to make an earlier phase's required function real.

### RGQ-060 — Review schemas are hash-bound mirrors and PASS cannot contain unresolved blockers

`REVIEW_SUMMARY_SCHEMA.yaml` and `schemas/review_summary.schema.yaml` must be byte-identical mirrors. `schemas/evidence_manifest.schema.yaml` is the machine-checkable schema for `evidence_manifest.yaml`. P0 and P16 must verify all three schema hashes and record them.

A `review_summary.yaml` with `review_status: PASS` is invalid if any of the following is true:

```text
phase_closable != true
blocking_findings is nonempty
required_fixes is nonempty
any relevant forbidden_pattern_scan entry is fail
any pass_conditions entry is false
algorithmic_sufficiency.verdict != sufficient, except P0 where not_applicable is allowed
prompt_sha256 does not match prompt.md
response_sha256 does not match response.md
prompt/response paths are outside the required archive path
the raw response says FAIL, blocker, unresolved required fix, or USER_DECISION_REQUIRED
```

The YAML schema must enforce all machine-checkable items in this list. Non-machine-checkable items must be enforced by the reviewer prompt and P16 audit.

### RGQ-061 — Acceptance suites are partitioned and cannot compensate for one another

P15 must use three separately reported suites:

```text
A. support_producing_candidate_cover_suite
B. exact_image_semantics_suite
C. failure_and_nonfinite_semantics_suite
```

A case in suite A must return `CertifiedCandidateCover` or `CertifiedExactTargetImage` with nonzero support, exact global support verification, exact squarefree support, exact real root isolation, and non-placeholder decoded candidates when real roots exist. `AlgorithmicHardCase`, `FiniteResourceFailure`, `CertificateDesignGap`, `CertifiedEmptyRealTargetImage`, `CertifiedNonFiniteTargetImage`, empty support, placeholder roots, or placeholder candidates fail suite A.

Suite B must run with `exact_image_mode=true` and must include nonempty exact image, empty real image, guard/slack/branch filtering, and spurious-root removal. `CertifiedEmptyRealTargetImage` is allowed only in suite B cases designed to prove empty real image, not in suite A.

Suite C checks invalid input, resource failure, algorithmic hard case, certificate gap, relation-search exhaustion, and certified nonfinite. Passing suite B or C never compensates for a failed suite A case.

### RGQ-062 — Performance-first means design-time cost compression evidence, not late benchmark wording

The implementation must treat algebraic-cost compression as an invariant of the algorithm design. Planner, kernel plans, projection messages, global cost trace, and closure evidence must show which variables were eliminated locally, which separator variables were exported, which matrix/template/rank sizes were used, and why no full-coordinate object was constructed.

Benchmark superiority claims remain out of scope. However, a phase that implements a projection mechanism must include cost-trace evidence showing that the mechanism is target/separator-direct and does not accidentally scale by constructing a full coordinate solution object when a local target/separator relation would suffice.

### RGQ-063 — Source-derived generalized stress must be algebraic templates, not problem fixtures

The generalized stress cases required by `RGQ-037`, `RGQ-048`, and `RGQ-061` must be expressed as algebraic templates over rational polynomial systems. They must not use original problem names, fixture IDs, expected answers, official solutions, or geometry-family dispatch. Each stress case must name the algebraic structure being tested, for example multi-separator projection, guard/slack provenance, determinant/oriented-bilinear structure, dot/Gram-like quadratic structure, explicit tower/norm-trace, verified target-action characteristic coverage, specialization/interpolation with exact verification, or one-large-block Universal.

A stress case is invalid if the production code can detect it by string, filename, problem ID, expected root, or fixture-only marker. P15 and P16 must include a stress-renaming/tamper test proving that the same algebraic structure still exercises the same mechanism under renamed variables and permuted relation order.

### RGQ-064 — Final consistency audit is a mandatory artifact

Before user approval and again in P16, the pack or repo must contain:

```text
docs/ai/changes/RGDTPK-Q-v4-core/CONSISTENCY_AUDIT.md
```

The audit must include at least:

```text
- source SHA-256 verification;
- R-ID and MECH reference integrity;
- phase-to-reviewer-prompt coverage;
- schema mirror byte-identity check;
- schema validation checks, including PASS-with-blocker rejection;
- forbidden phrase scan for the old candidate-cover acceptance-complete claim;
- scan proving the corrected RGQ-051 polarity is present and the wrong polarity is absent;
- Appendix A hardening override table check under RGQ-057;
- Plan phase closure risk check for “as needed”, hook-only, scaffold, helper-only, or deferred controlling functions;
- P15 suite partition check under RGQ-061;
- P16 claim ladder check under RGQ-049.
```

If any item fails, the Agent must stop with `PlanDefect` or `BaseSpecConflict`; it must not proceed to implementation or closure.


## 9. MECH definitions

Each MECH is mandatory because correctness depends on algorithmic behavior, exactification, verification, public API, or evidence-bearing failure semantics.

### MECH-001 — Exact arithmetic and canonical data model

Supports: `RGQ-009`, `RGQ-010`, `RGQ-012`.  
Inputs: rational coefficients, monomials, sparse polynomials, target problem records.  
Outputs: normalized rational/polynomial objects with stable hashes.  
Oracle: exact algebraic identities, normalization idempotence, deterministic hash replay.  
Failure behavior: `InvalidInput` or `ImplementationBug`.  
Verification: unit/property tests plus reviewer inspection of no floating-only exact path.

### MECH-002 — Pipeline orchestrator

Supports: `RGQ-011`, `RGQ-030`, `RGQ-034`.  
Inputs: `RationalTargetProblem`, `SolverOptions`.  
Outputs: `TargetSolveResult`.  
Oracle: ordered stage trace matches Appendix A section 4.1 and status semantics.  
Failure behavior: evidence-bearing solver status.  
Verification: integration tests over generalized algebraic systems and replay.

### MECH-003 — Compression with guard provenance

Supports: `RGQ-013`, `RGQ-003`, `RGQ-037`.  
Inputs: canonical system and semantic encodings.  
Outputs: compressed system, substitutions, guards, obligations, trace.  
Oracle: transformed equations preserve candidate-cover semantics under recorded guards.  
Failure behavior: hard/resource/certificate status, not unsafe rewrite.  
Verification: guarded affine and saturation stress cases.

### MECH-004 — TargetProjectionDAG and authorization

Supports: `RGQ-014`, `RGQ-038`.  
Inputs: compressed system, graphs, target.  
Outputs: DAG with authorized blocks and separators.  
Oracle: every polynomial occurrence is represented; every projector reads only authorized relations; no-separator produces one large block.  
Failure behavior: `ImplementationBug` for invariant violation.  
Verification: structural tests and semantic deletion/replay mismatch challenges.

### MECH-005 — Deterministic planner and declared ladder

Supports: `RGQ-015`, `RGQ-039`.  
Inputs: DAG blocks, child messages, cost probes, resource bounds.  
Outputs: predeclared `KernelPlan` per block.  
Oracle: deterministic order and hash; Universal kernel always appears as generic target-direct final plan for well-formed blocks.  
Failure behavior: `ImplementationBug` if no plan exists for a valid block.  
Verification: repeated-run determinism and no hidden runtime fallback inspection.

### MECH-006 — TargetRelationSearch workhorse

Supports: `RGQ-019`, `RGQ-037`.  
Inputs: local ideal `J ⊂ Q[Y,Z]`, declared supports/bounds.  
Outputs: verified `g(Z)` and membership certificate.  
Oracle: exact identity `g - Σ q_i f_i = 0`.  
Failure behavior: `AlgorithmicHardCase` with matrix/bound evidence or `FiniteResourceFailure`.  
Verification: multi-separator, bilinear/quadratic, and one-large-block stress.

### MECH-007 — Optimized projection kernels

Supports: `RGQ-017`, `RGQ-018`, `RGQ-020`, `RGQ-021`, `RGQ-023`, `RGQ-024`, `RGQ-025`.  
Inputs: selected kernel plans.  
Outputs: projection messages.  
Oracle: exact kernel-specific certificates plus message verification.  
Failure behavior: `AlgorithmicHardCase`, `CertificateDesignGap`, or `FiniteResourceFailure`; no ordinary unsupported.  
Verification: kernel-specific stress and no-coordinate-export checks.

### MECH-008 — Universal target/separator elimination

Supports: `RGQ-022`, `RGQ-036`.  
Inputs: well-formed block with local variables and exported variables.  
Outputs: target/separator-only projection generators.  
Oracle: exact membership/elimination certificate and no coordinate roots/RUR export.  
Failure behavior: `AlgorithmicHardCase`, `FiniteResourceFailure`, or `CertificateDesignGap`; `CertifiedNonFiniteTargetImage` is available only through final-support nonfinite certification in `MECH-015`, not from local Universal failure.  
Verification: one-large-block and not-sparse/not-affine cases plus anti-heavy-fallback checks.

### MECH-009 — Composition and final support

Supports: `RGQ-026`, `RGQ-032`, `RGQ-038`.  
Inputs: DAG and verified messages.  
Outputs: composed projection and `S(T)`.  
Oracle: separator elimination uses only message relations; final support is target-only and verified.  
Failure behavior: `AlgorithmicHardCase`, `FiniteResourceFailure`, or `CertificateDesignGap` when target-only support is not produced; `CertifiedNonFiniteTargetImage` only via `MECH-015` positive proof.  
Verification: multi-block separator composition stress.

### MECH-010 — Certificates, verification, and replay

Supports: `RGQ-027`, `RGQ-038`, `RGQ-040`.  
Inputs: messages, support, run certificate, original problem.  
Outputs: verified messages, global support certificate, replay result.  
Oracle: exact Q proof; hashes bind source, canonicalization, DAG, plans, messages, support, roots, candidates.  
Failure behavior: `CertificateDesignGap` or `ImplementationBug`.  
Verification: tamper/mismatch tests and reviewer source-fidelity challenge.

### MECH-011 — Exact root isolation and decode

Supports: `RGQ-028`, `RGQ-034`.  
Inputs: nonzero univariate support.  
Outputs: squarefree support, exact isolating intervals, candidates.  
Oracle: Sturm/Descartes exact isolation and deterministic candidate hashes.  
Failure behavior: hard/resource/certificate status, not placeholder candidate list.  
Verification: support polynomials with multiple rational and irrational real roots.

### MECH-012 — Exact image semantics

Supports: `RGQ-003`, `RGQ-029`, `RGQ-037`.  
Inputs: compressed system, support, candidates, semantic encodings.  
Outputs: fiber classifications and exact image statuses.  
Oracle: Hermite/Thom/sign/slack certificates.  
Failure behavior: `CertificateDesignGap` or hard/resource status; no exact-image claim without classification.  
Verification: guard/slack stress and candidate-cover vs exact-image status separation.

---


### MECH-013 — Fixed TargetRelationSearch schedule

Supports: `RGQ-019`, `RGQ-042`, `RGQ-043`, `RGQ-055`.  
Inputs: local ideal `J ⊂ Q[Y,Z]`, sorted `Y`, sorted `Z`, and solver options.  
Outputs: deterministic relation-search bounds, export supports, multiplier supports, coefficient row monomials, matrix builders, verified relation candidates.  
Oracle: schedule hashes are recomputable from RGQ-042; accepted relation satisfies exact identity `g - Σ q_i f_i = 0`.  
Failure behavior: `AlgorithmicHardCase` with stage trace or `FiniteResourceFailure`; never nonfinite.  
Verification: schedule reproducibility tests, multi-separator tests, and reviewer recomputation.

### MECH-014 — Verified characteristic support coverage for ActionKrylov

Supports: `RGQ-021`, `RGQ-044`, `RGQ-054`.  
Inputs: finite target-relevant quotient/action handle.  
Outputs: characteristic support polynomial and `VerifiedCharacteristicSupportCoverage` certificate.  
Oracle: every target-action matrix column is exact-normal-form verified and Cayley-Hamilton is verified exactly.  
Failure behavior: `CertificateDesignGap` or `FiniteResourceFailure`; no candidate from undercovered Krylov.  
Verification: undercoverage regression and no-coordinate-export inspection.

### MECH-015 — Certified nonfinite target image

Supports: `RGQ-032`, `RGQ-045`, `RGQ-050`, `RGQ-051`.  
Inputs: composed root projection and, in exact-image mode, real semantics.  
Outputs: `NonFiniteTargetImageCertificate` and optionally `RealNonFiniteTargetCertificate`.  
Oracle: exact proof that `I ∩ Q[T] = {0}` plus consistency/projection nonfiniteness; exact-image mode also proves real nonfiniteness.  
Failure behavior: `AlgorithmicHardCase` or `CertificateDesignGap` if proof is missing.  
Verification: separate relation-search-exhaustion and certified-nonfinite tests.

### MECH-016 — Review archive and claim discipline

Supports: `RGQ-040`, `RGQ-047`, `RGQ-049`, `RGQ-052`, `RGQ-053`.  
Inputs: phase prompt, reviewer response, code/evidence, git state.  
Outputs: archived prompt/response, schema-valid `review_summary.yaml`, evidence manifest, closure claim.  
Oracle: summary hashes match prompt/response; reviewer inspected code and algorithmic control; final claim obeys RGQ-049.  
Failure behavior: phase remains open.  
Verification: schema validation and final archive audit.

## 10. Acceptance criteria

A final `RGDTPK_Q_V4_ACCEPTANCE_COMPLETE` or `SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC` claim is allowed only when all items below have fresh evidence bound to git state:

1. All required files and public functions from Appendix A sections 6–29 exist and are semantically implemented.
2. `cargo fmt`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --all-targets --all-features` pass, unless the environment lacks Rust tooling; if tooling is missing, the final claim must be partial and cannot be acceptance complete.
3. Static scans show no production-path geometry dispatch, fixture/expected-answer dispatch, full coordinate solution enumeration, full coordinate RUR generation, generic QE/CAD fallback, hidden fallback, ordinary `Unsupported` for well-formed systems, `todo!`, `unimplemented!`, placeholder candidate decode, or floating-only root isolation.
4. Suite A generalized candidate-cover support-producing stress cases in `RGQ-037`, `RGQ-048`, and `RGQ-061` produce `CertifiedCandidateCover` or `CertifiedExactTargetImage` with nonzero support, exact support verification, exact root isolation, and non-placeholder decoded candidates; hard/resource/certificate/nonfinite/empty statuses are allowed only in Suite C failure semantics.
5. Suite B exact-image stress cases in `RGQ-048` and `RGQ-061` prove real fiber, guard, slack, and branch semantics before any exact-image status or acceptance-complete claim.
6. `TargetRelationSearchKernel` evidence shows the reproducible dense schedule in `RGQ-042` and the sparse-support limitation in `RGQ-043`.
7. `TargetActionKrylovKernel` evidence shows only `VerifiedCharacteristicSupportCoverage` under `RGQ-044`, including the undercoverage regression in `RGQ-054`.
8. `UniversalTargetEliminationKernel` evidence shows bounded block-local authorization, fixed strategy sequence, no local nonfinite, and no global coordinate solver under `RGQ-041` and `RGQ-056`.
9. `CertifiedNonFiniteTargetImage` appears only through positive proof under `RGQ-045`, and relation-search/composition failure routes to hard/resource/certificate statuses under `RGQ-051`.
10. Replay fails on tampered input hash, DAG hash, plan hash, projection message hash, support hash, root isolation hash, candidate hash, nonfinite certificate hash, and exact-image classification hash.
11. Removing or bypassing the DAG/authorization/plan/certificate path must be caught by tests or replay checks; these artifacts must be operational.
12. Reviewer prompts for every Plan phase/subphase have been run and recorded, `prompt.md` and `response.md` are archived, and every `review_summary.yaml` validates against `schemas/review_summary.schema.yaml` / `REVIEW_SUMMARY_SCHEMA.yaml`.
13. Any FAIL, USER_DECISION_REQUIRED item, malformed review archive, missing `function_implementation_table.yaml`, or PASS inconsistent with the raw reviewer response has been resolved by implementation change or explicit approved Base Spec amendment.
14. `RGDTPK_Q_V4_ACCEPTANCE_COMPLETE` is forbidden unless exact-image mode has reached `EXACT_IMAGE_CORE_READY` under `RGQ-049`, `RGQ-050`, and `RGQ-061`.
15. `SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC` is forbidden if any exact-image API/semantic path, certificate path, or Appendix A required function remains missing, stubbed, or weaker than the supplied specification.
16. Final closure does not make benchmark or universal-completeness claims outside Appendix A.

---

## 11. Forbidden claims

The Agent must not claim any of the following under this Base Spec:

- “solves all finite algebraic systems”;
- “always faster than Groebner, homotopy, CAS, or CAD”;
- “CertifiedExactTargetImage” when only candidate cover was computed;
- “source faithful” if any Appendix A file/function is missing or reduced to a stub;
- “done because tests pass” without algorithmic reviewer confirmation;
- “unsupported slice is honest failure” for a well-formed Q-polynomial input;
- “preflight complete” as a substitute for solver-core completion;
- “CertifiedNonFiniteTargetImage” because relation search, Universal, or composition failed to find a relation;
- “ActionKrylov coverage” from a single Krylov sequence, trace probe, block Wiedemann claim, or unverified `S(M_T)=0`;
- “Universal kernel complete” when the implementation is an unbounded or full-coordinate fallback;
- “RGDTPK_Q_V4_ACCEPTANCE_COMPLETE” while exact-image mode is incomplete.

---

## 12. Appendix A — Verbatim algorithm specification

The following text is copied verbatim from the uploaded algorithm specification. It is normative.

```markdown
# GeoSolver Core Algorithm Specification v4.0
# R-GDTPK-Q / ACCTP-Q: Algebraic-Cost Compressed Direct Target Projection over Rational Polynomial Systems

作成日: 2026-07-04  
文書種別: **アルゴリズム設計固定用の自己完結仕様書**  
対象: `geosolver-core` の solver core  
主目的: 幾何問題から得られる大規模な有理数係数多項式系から、全座標解を先に求めず、target 値候補を高速かつ証明付きで導出する。  
非目的: benchmark 設計、テスト設計、実験計画、自然言語・図認識、幾何DSL lowering の詳細。

---

## 0. この文書の読み方

この文書は、実装者がこの会話履歴や過去の計画書を知らなくても `geosolver-core` を実装できるように、数学的アルゴリズム、データ構造、処理順序、folder 構成、各 file に置く関数、各関数の入力・処理・出力を固定する仕様書である。

この文書では、次の名前を使う。

```text
R-GDTPK-Q
    Rational Generic Direct Target Projection Kernel over Q-polynomial systems

ACCTP-Q
    Algebraic-Cost Compressed Target Projection over Q-polynomial systems
```

本仕様では、`R-GDTPK-Q` と `ACCTP-Q` は同じ solver core を指す。前者は「有理数係数多項式系上の汎用 target-direct kernel」という性質を表し、後者は「代数コスト圧縮」という研究上の強みを表す。

---

## 1. 研究上の強み

### 1.1 主張する強み

この研究の強みは、次である。

```text
幾何由来の大規模な有理数係数多項式系 F ⊂ Q[x1,...,xn,T] から、
全座標解や全座標RURを構成せず、
target T と separator 変数に必要な代数関係だけを局所的に射影・合成する。

これにより、全体消去の代数コストを、全変数数 n ではなく、
局所 block 幅、separator 幅、local quotient/action rank、sparse template size、
最終 support degree に依存させる。
```

この仕様が目指す高速化は、単に「target 値の個数が座標解数より少ないから速い」というものではない。target 値数と座標解数が同程度の場合でも、次が小さければ高速化の余地がある。

```text
- target に到達するまでの局所 block 幅
- separator 変数の個数と次数
- 各局所 block の quotient rank
- 各局所 block の sparse resultant / Macaulay template size
- multiplication-by-target operator の疎性
- 証明書の検証コスト
```

### 1.2 強みの正確な言い換え

悪い説明:

```text
target-direct なので、常に homotopy や Groebner より速い。
```

採用する説明:

```text
R-GDTPK-Q は、full coordinate solution object を作らず、
target/separator projection message を TargetProjectionDAG 上で合成する。
幾何由来の多項式系が持ちやすい低次数・疎性・局所依存・小 separator を、
幾何名ではなく代数的 footprint として利用することで、
全体消去に必要な巨大な代数 object を局所 object の列へ分解する。
```

### 1.3 汎用性に関する絶対条件

この solver は、特定の幾何形式だけを support する solver ではない。

禁止される設計:

```text
- circle solver
- triangle solver
- tangent solver
- metric-only solver
- distance-only solver
- area-only solver
- target-univariate-only solver
- affine-only solver
- bivariate-only solver
- no-branch-only solver
- no-slack-only solver
- ある形の式だけを完成扱いする solver
```

正しい設計:

```text
入力は任意の well-formed な有理数係数多項式系。
すべての入力は generic target projection pipeline に入る。
特定形でないことを理由に Unsupported を返してはならない。
計算できない場合は、どの代数的パラメータが障害になったかを
FiniteResourceFailure / AlgorithmicHardCase / CertificateDesignGap として返す。
```

---

## 2. 数学的問題設定

### 2.1 入力

入力は次である。

```text
F = {f1,...,fm} ⊂ Q[x1,...,xn,T]
T = target variable
```

各 `fi` は有理数係数 sparse polynomial である。変数は、座標変数、補助変数、slack 変数、selector 変数、構成変数などを含んでよい。ただし solver core は、変数 role を計算の分岐条件にしてはならない。

`F` が生成する ideal を次とする。

```text
I = <F> ⊂ Q[x1,...,xn,T]
```

候補値を得るには target elimination ideal

```text
I_T = I ∩ Q[T]
```

の非零元を求めればよい。非零元 `S(T) ∈ I_T` が得られれば、任意の複素解 `x,T` は `S(T)=0` を満たす。したがって実数幾何としての真の target 値も `S` の実根に含まれる。

### 2.2 出力

主出力は次である。

```text
S(T) ∈ Q[T]
```

ただし `S(T)` は次を満たす。

```text
S(T) != 0
true target values ⊆ roots(S)
```

`S` は squarefree 化して root isolation へ渡す。

```text
S_sq(T) = squarefree_part(S(T))
RealRootIsolation(S_sq)
```

各実根は rational isolating interval または algebraic-root record として出力する。

### 2.3 Candidate cover と exact target image

本仕様では、次の二段階を厳密に分ける。

```text
CertifiedCandidateCover:
    S(T) を返す。
    真の target 値は roots(S) に含まれる。
    spurious roots は含まれてよい。

CertifiedExactTargetImage:
    roots(S) の各実根について real fiber / guard / slack semantics を判定し、
    実際に実現可能な target 値だけを返す。
```

この仕様の必須 core は `CertifiedCandidateCover` である。`CertifiedExactTargetImage` は同じ architecture の後段として設計に含める。

### 2.4 不等式・非ゼロ条件

幾何問題では不等式や非ゼロ条件が出る。core input は等式系として受けるので、これらは必要に応じて補助変数で等式化される。

```text
A >= 0   ->   A - s^2 = 0
A > 0    ->   A*s^2 - 1 = 0
A != 0   ->   A*s - 1 = 0
```

ただし、これは実数 semantics を表すための encoding であり、複素数上の elimination だけで最終的な実数条件を判定してはならない。等式化された guard は provenance を持ち、exact image mode の RealFiberClassifier が解釈する。

---

## 3. 非交渉の設計原則

### 3.1 禁止事項

次は禁止である。

```text
1. full coordinate solution list を production path で作る。
2. full coordinate RUR を production path で作る。
3. 全座標 lex parametrization を作ってから T を読む。
4. generic QE/CAD を隠れ fallback として使う。
5. 外部CASの答えをそのまま certified production proof とする。
6. ある kernel が失敗した後、planner に記録されていない hidden fallback を呼ぶ。
7. 幾何名、問題名、fixture 名、期待答え、公式解に基づいて分岐する。
8. 特定の式形だけを support として、それ以外を Unsupported とする。
```

### 3.2 許可される重い代数処理

次は許可される。

```text
1. local block 内での target/separator elimination
2. exact modular F4/F5-like sparse linear algebra
3. target/separator variables だけを export する resultant / eliminant
4. multiplication-by-target operator
5. target-relevant quotient/action handle
6. sparse resultant template
7. regular-chain projection
8. norm/trace computation
9. specialization-interpolation
```

許可条件は次である。

```text
- deterministic planner が実行前に選ぶ。
- export は target/separator relation のみ。
- coordinate roots を返さない。
- full coordinate RUR を返さない。
- exact Q verification を持つ。
- cost trace と certificate を出す。
```

### 3.3 失敗の扱い

well-formed な Q-polynomial input に対して、通常の `Unsupported` は返さない。

許可される失敗 status:

```text
FiniteResourceFailure:
    指定された resource bound 内で exact target-direct computation が完了しない。

AlgorithmicHardCase:
    現在の target-direct algorithm では support relation を構成できない。
    ただし、最小障害 block、matrix size、rank estimate、degree bound などを返す。

CertificateDesignGap:
    候補 object は構成できたが、現行 certificate language で検証できない。

ImplementationBug:
    仕様上成立すべき invariant が破れた。

InvalidInput:
    Q-polynomial system として well-formed でない。
```

---

## 4. 全体アルゴリズム

### 4.1 Top-level pipeline

```text
solve_target(problem, options):
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

### 4.2 重要な invariant

各 step は次を守る。

```text
I1. 入力は Q-polynomial system として扱う。
I2. 幾何名で分岐しない。
I3. production path は coordinate solution list を作らない。
I4. 各 block は target/separator variables だけを export する。
I5. 各 exported relation は exact Q certificate を持つ。
I6. final S(T) は candidate cover として exact に検証される。
I7. exact image mode では real fiber / slack / guard semantics を検証する。
I8. 失敗時は、障害となった代数コストを trace に出す。
I9. planner は deterministic である。
I10. hidden fallback はない。
```

---

## 5. 主要データ型

この節の型は Rust 風疑似コードで書く。実装言語が Rust でない場合も、同じ field と invariant を持つ構造にする。

### 5.1 ID 型

```rust
pub struct VariableId(pub u32);
pub struct RelationId(pub u32);
pub struct BlockId(pub u32);
pub struct PackageId(pub u32);
pub struct KernelPlanId(pub u32);
pub struct Hash(pub [u8; 32]);
```

全 ID は stable であり、canonicalization 後に再現可能でなければならない。

### 5.2 有理数

```rust
pub struct RationalQ {
    pub num: BigInt,
    pub den: BigInt, // always positive
}
```

Invariant:

```text
- gcd(num, den) = 1
- den > 0
- zero is represented as 0/1
```

### 5.3 Monomial と polynomial

```rust
pub struct Monomial {
    pub exponents: Vec<(VariableId, u32)>, // sorted by VariableId, no zero exponent
}

pub struct TermQ {
    pub coeff: RationalQ,
    pub monomial: Monomial,
}

pub struct SparsePolynomialQ {
    pub terms: Vec<TermQ>, // sorted by monomial order, no zero coeff, no duplicate monomial
    pub hash: Hash,
}

pub struct UniPolynomialQ {
    pub variable: VariableId,
    pub coeffs_low_to_high: Vec<RationalQ>,
    pub hash: Hash,
}
```

### 5.4 Problem input

```rust
pub struct RationalTargetProblem {
    pub variables: Vec<VariableId>,
    pub target: VariableId,
    pub equations: Vec<SparsePolynomialQ>,
    pub semantic_encodings: Vec<RealConstraintEncoding>,
    pub variable_roles: Vec<VariableRoleRecord>,
    pub input_hash: Hash,
}

pub struct RealConstraintEncoding {
    pub original_kind: RealConstraintKind,
    pub encoded_relation_ids: Vec<RelationId>,
    pub slack_variables: Vec<VariableId>,
    pub semantic_hash: Hash,
}
```

`variable_roles` は provenance 用であり、algorithmic dispatch に使ってはならない。

### 5.5 Canonical system

```rust
pub struct CanonicalSystemQ {
    pub variables: Vec<VariableId>,
    pub target: VariableId,
    pub relations: Vec<CanonicalRelationQ>,
    pub relation_order: Vec<RelationId>,
    pub variable_order: VariableOrder,
    pub semantic_encodings: Vec<RealConstraintEncoding>,
    pub canonical_hash: Hash,
}

pub struct CanonicalRelationQ {
    pub id: RelationId,
    pub polynomial: SparsePolynomialQ,
    pub source: RelationSource,
    pub hash: Hash,
}
```

### 5.6 Projection message

従来の単一 `TargetRelationPackage` は弱い。separator が複数ある場合、projection ideal は一般に principal ではない。したがって、本仕様では projection message を複数 relation を含められる形にする。

```rust
pub struct ProjectionMessage {
    pub package_id: PackageId,
    pub block_id: BlockId,
    pub kernel_kind: KernelKind,
    pub source_relation_ids: Vec<RelationId>,
    pub eliminated_variables: Vec<VariableId>,
    pub exported_variables: Vec<VariableId>, // subset of {T}+separators
    pub relation_generators: Vec<SparsePolynomialQ>,
    pub representation: MessageRepresentation,
    pub projection_strength: ProjectionStrength,
    pub certificate: KernelCertificate,
    pub compression_trace: CompressionTrace,
    pub cost_trace: ProjectionCostTrace,
    pub package_hash: Hash,
}

pub enum MessageRepresentation {
    GeneratorSet,
    PrincipalSupport,
    TriangularChain,
    QuotientAction,
    NormTraceTower,
    SparseResultantMatrix,
    SpecializationInterpolation,
}

pub enum ProjectionStrength {
    CandidateCoverWeak,
    CandidateCoverStrong,
    RadicalProjectionApprox,
    ExactProjectionIdeal,
    ExactRealFiberAware,
}
```

Invariant:

```text
- exported_variables に eliminated local variables は含まれない。
- relation_generators は Q[exported_variables] に属する。
- package は coordinate roots を含まない。
- package は full coordinate RUR を含まない。
- certificate は modular-only ではない。最終的に Q 上で検証できる。
```

### 5.7 Result

```rust
pub struct TargetSolveResult {
    pub status: SolverStatus,
    pub target: VariableId,
    pub support_polynomial: Option<UniPolynomialQ>,
    pub squarefree_support_polynomial: Option<UniPolynomialQ>,
    pub root_isolation: Vec<RealRootRecord>,
    pub decoded_candidates: Vec<TargetCandidate>,
    pub projection_messages: Vec<ProjectionMessage>,
    pub certificate: Option<CoreRunCertificate>,
    pub diagnostics: Vec<DiagnosticRecord>,
    pub cost_trace: GlobalCostTrace,
}

pub enum SolverStatus {
    CertifiedCandidateCover,
    CertifiedExactTargetImage,
    CertifiedEmptyRealTargetImage,
    CertifiedNonFiniteTargetImage,
    FiniteResourceFailure,
    AlgorithmicHardCase,
    CertificateDesignGap,
    ImplementationBug,
    InvalidInput,
}
```

---

## 6. Folder 構成

実装 folder は次の構成に固定する。

```text
geosolver-core/
  Cargo.toml
  README.md
  src/
    lib.rs
    api.rs

    types/
      mod.rs
      ids.rs
      rational.rs
      monomial.rs
      polynomial.rs
      univariate.rs
      matrix.rs
      interval.rs
      hash.rs

    problem/
      mod.rs
      input.rs
      semantic.rs
      validate.rs
      canonicalize.rs
      context.rs

    algebra/
      mod.rs
      monomial_order.rs
      polynomial_ops.rs
      modular.rs
      crt.rs
      rational_reconstruction.rs
      sparse_matrix.rs
      dense_matrix.rs
      linear_solve.rs
      normal_form.rs
      groebner.rs
      f4.rs
      elimination.rs
      resultant.rs
      interpolation.rs
      quotient.rs
      krylov.rs
      regular_chain.rs
      norm_trace.rs
      real_root.rs
      sign.rs

    preprocess/
      mod.rs
      compression.rs
      linear_affine.rs
      definitional.rs
      binomial.rs
      saturation.rs
      independent.rs

    graph/
      mod.rs
      hypergraph.rs
      influence.rs
      weighted_primal.rs
      separators.rs
      tree_decomposition.rs
      projection_dag.rs
      metrics.rs

    planner/
      mod.rs
      cost_model.rs
      probes.rs
      admission.rs
      kernel_plan.rs
      ladder.rs
      planner.rs

    kernels/
      mod.rs
      traits.rs
      target_univariate.rs
      linear_affine.rs
      target_relation_search.rs
      sparse_resultant.rs
      action_krylov.rs
      universal_elimination.rs
      regular_chain_projection.rs
      norm_trace_projection.rs
      specialization_interpolation.rs

    compose/
      mod.rs
      message.rs
      compose.rs
      separator_elimination.rs
      final_support.rs

    verify/
      mod.rs
      certificates.rs
      verify_message.rs
      verify_support.rs
      replay.rs
      run_certificate.rs

    roots/
      mod.rs
      squarefree.rs
      isolate.rs
      decode.rs
      algebraic_number.rs

    fiber/
      mod.rs
      exact_image.rs
      hermite.rs
      thom.rs
      slack_semantics.rs

    result/
      mod.rs
      status.rs
      diagnostics.rs
      cost_trace.rs
      output.rs

    solver/
      mod.rs
      options.rs
      pipeline.rs
      orchestrator.rs
```

`tests/` や benchmark folder はこの仕様書では定義しない。

---

## 7. File 別仕様: public API

### 7.1 `src/lib.rs`

目的: crate の public module を公開する。

必須内容:

```rust
pub mod api;
pub mod types;
pub mod problem;
pub mod algebra;
pub mod preprocess;
pub mod graph;
pub mod planner;
pub mod kernels;
pub mod compose;
pub mod verify;
pub mod roots;
pub mod fiber;
pub mod result;
pub mod solver;
```

禁止:

```text
- lib.rs に solver logic を直接書かない。
- geometry-specific helper を公開しない。
```

### 7.2 `src/api.rs`

目的: 外部から呼ぶ単一 API を定義する。

関数:

```rust
pub fn solve_target(
    problem: RationalTargetProblem,
    options: SolverOptions,
) -> TargetSolveResult
```

入力:

```text
problem: 有理数係数多項式系と target
options: resource bound、exact image mode、certificate level など
```

処理:

```text
1. SolverContext を初期化する。
2. solver::orchestrator::solve_with_context を呼ぶ。
3. panic ではなく SolverStatus 付き result を返す。
```

出力:

```text
TargetSolveResult
```

疑似コード:

```rust
pub fn solve_target(problem, options) -> TargetSolveResult {
    let ctx = SolverContext::new(options);
    match solver::orchestrator::solve_with_context(problem, ctx) {
        Ok(result) => result,
        Err(err) => TargetSolveResult::from_solver_error(err),
    }
}
```

---

## 8. File 別仕様: types

### 8.1 `types/ids.rs`

関数・型:

```rust
pub struct VariableId(pub u32);
pub struct RelationId(pub u32);
pub struct BlockId(pub u32);
pub struct PackageId(pub u32);
pub struct Hash(pub [u8; 32]);

pub fn fresh_variable_id(counter: &mut IdCounter) -> VariableId;
pub fn fresh_relation_id(counter: &mut IdCounter) -> RelationId;
pub fn stable_id_from_name(name: &str, namespace: &str) -> StableId;
```

各関数:

```text
fresh_variable_id:
    入力: mutable counter
    処理: counter を1増やし、新しい VariableId を返す
    出力: VariableId

stable_id_from_name:
    入力: name, namespace
    処理: canonical UTF-8 bytes を hash し stable id を作る
    出力: StableId
```

### 8.2 `types/rational.rs`

関数:

```rust
pub fn new_q(num: BigInt, den: BigInt) -> RationalQ;
pub fn normalize_q(q: RationalQ) -> RationalQ;
pub fn add_q(a: &RationalQ, b: &RationalQ) -> RationalQ;
pub fn sub_q(a: &RationalQ, b: &RationalQ) -> RationalQ;
pub fn mul_q(a: &RationalQ, b: &RationalQ) -> RationalQ;
pub fn div_q(a: &RationalQ, b: &RationalQ) -> Result<RationalQ, DivisionByZero>;
pub fn bit_height_q(q: &RationalQ) -> usize;
```

疑似コード:

```rust
fn normalize_q(q):
    if q.den == 0: error
    if q.num == 0: return 0/1
    if q.den < 0: q.num=-q.num; q.den=-q.den
    g = gcd(abs(q.num), q.den)
    return (q.num/g)/(q.den/g)
```

### 8.3 `types/monomial.rs`

関数:

```rust
pub fn normalize_monomial(entries: Vec<(VariableId,u32)>) -> Monomial;
pub fn monomial_mul(a: &Monomial, b: &Monomial) -> Monomial;
pub fn monomial_div(a: &Monomial, b: &Monomial) -> Option<Monomial>;
pub fn monomial_degree(m: &Monomial) -> u32;
pub fn monomial_variables(m: &Monomial) -> BTreeSet<VariableId>;
pub fn compare_monomial(a: &Monomial, b: &Monomial, order: &MonomialOrder) -> Ordering;
```

### 8.4 `types/polynomial.rs`

関数:

```rust
pub fn normalize_poly(p: SparsePolynomialQ) -> SparsePolynomialQ;
pub fn poly_add(a: &SparsePolynomialQ, b: &SparsePolynomialQ) -> SparsePolynomialQ;
pub fn poly_sub(a: &SparsePolynomialQ, b: &SparsePolynomialQ) -> SparsePolynomialQ;
pub fn poly_mul(a: &SparsePolynomialQ, b: &SparsePolynomialQ) -> SparsePolynomialQ;
pub fn poly_scale(a: &SparsePolynomialQ, c: &RationalQ) -> SparsePolynomialQ;
pub fn poly_derivative(a: &SparsePolynomialQ, v: VariableId) -> SparsePolynomialQ;
pub fn poly_variables(a: &SparsePolynomialQ) -> BTreeSet<VariableId>;
pub fn poly_total_degree(a: &SparsePolynomialQ) -> u32;
pub fn poly_monomial_count(a: &SparsePolynomialQ) -> usize;
pub fn clear_denominators_primitive(a: &SparsePolynomialQ) -> SparsePolynomialQ;
pub fn substitute_poly(a: &SparsePolynomialQ, subst: &SubstitutionMap) -> SparsePolynomialQ;
```

疑似コード:

```rust
fn normalize_poly(p):
    map = BTreeMap<Monomial, RationalQ>()
    for term in p.terms:
        if term.coeff != 0:
            map[normalize_monomial(term.monomial)] += term.coeff
    terms = []
    for (m,c) in map ordered by monomial_order:
        if c != 0: terms.push(TermQ{c,m})
    return SparsePolynomialQ{terms, hash=hash_terms(terms)}
```

### 8.5 `types/univariate.rs`

関数:

```rust
pub fn normalize_univariate(p: UniPolynomialQ) -> UniPolynomialQ;
pub fn degree_uni(p: &UniPolynomialQ) -> Option<usize>;
pub fn derivative_uni(p: &UniPolynomialQ) -> UniPolynomialQ;
pub fn gcd_uni(a: &UniPolynomialQ, b: &UniPolynomialQ) -> UniPolynomialQ;
pub fn squarefree_part_uni(p: &UniPolynomialQ) -> UniPolynomialQ;
pub fn eval_uni_q(p: &UniPolynomialQ, x: &RationalQ) -> RationalQ;
```

### 8.6 `types/matrix.rs`

型:

```rust
pub struct SparseMatrixQ { ... }
pub struct SparseMatrixFp { ... }
pub struct DenseMatrixFp { ... }
pub struct VectorQ { ... }
pub struct VectorFp { ... }
```

関数:

```rust
pub fn matrix_shape<M>(m: &M) -> (usize, usize);
pub fn matrix_density<M>(m: &M) -> RationalQ;
pub fn hash_matrix<M>(m: &M) -> Hash;
```

### 8.7 `types/interval.rs`

型・関数:

```rust
pub struct RationalInterval { pub lo: RationalQ, pub hi: RationalQ }

pub fn interval_new(lo: RationalQ, hi: RationalQ) -> Result<RationalInterval, IntervalError>;
pub fn interval_contains_q(i: &RationalInterval, x: &RationalQ) -> bool;
pub fn interval_disjoint(a: &RationalInterval, b: &RationalInterval) -> bool;
```

---

## 9. File 別仕様: problem

### 9.1 `problem/input.rs`

型:

```rust
pub struct RationalTargetProblem { ... }
pub struct VariableRoleRecord { ... }
```

関数:

```rust
pub fn make_problem(
    variables: Vec<VariableId>,
    target: VariableId,
    equations: Vec<SparsePolynomialQ>,
    semantics: Vec<RealConstraintEncoding>,
) -> RationalTargetProblem;
```

処理:

```text
1. 入力 vector を保持する。
2. hash を計算する。
3. ここでは canonicalization しない。
```

### 9.2 `problem/semantic.rs`

型:

```rust
pub enum RealConstraintKind { NonNegative, Positive, NonZero, BranchChoice, Other }
pub struct RealConstraintEncoding { ... }
```

関数:

```rust
pub fn register_slack_encoding(kind, encoded_relation_ids, slack_variables) -> RealConstraintEncoding;
pub fn semantic_relations(sem: &[RealConstraintEncoding]) -> BTreeSet<RelationId>;
pub fn verify_semantic_references(sem: &[RealConstraintEncoding], relations: &[RelationId]) -> Result<(), InvalidInput>;
```

### 9.3 `problem/validate.rs`

関数:

```rust
pub fn validate_input(problem: RationalTargetProblem) -> Result<ValidatedProblem, SolverError>;
```

疑似コード:

```rust
fn validate_input(problem):
    if problem.target not in problem.variables:
        return Err(InvalidInput("target not declared"))

    for f in problem.equations:
        if !is_finite_sparse_polynomial(f):
            return Err(InvalidInput("non-finite polynomial"))
        if !all_coefficients_are_rational(f):
            return Err(InvalidInput("non-rational coefficient"))

    verify_semantic_references(problem.semantic_encodings, relation_ids(problem.equations))?

    return ValidatedProblem(problem)
```

出力:

```text
ValidatedProblem
```

禁止:

```text
- target が coordinate role だから拒否する。
- branch/slack があるから拒否する。
- 幾何名がないから拒否する。
```

### 9.4 `problem/canonicalize.rs`

関数:

```rust
pub fn canonicalize_system(validated: ValidatedProblem) -> Result<CanonicalSystemQ, SolverError>;
pub fn canonicalize_relation(id: RelationId, p: SparsePolynomialQ) -> CanonicalRelationQ;
pub fn canonical_variable_order(vars: &[VariableId], target: VariableId) -> VariableOrder;
```

疑似コード:

```rust
fn canonicalize_system(validated):
    order = canonical_variable_order(validated.variables, validated.target)
    relations = []
    for (id, p) in enumerate(validated.equations):
        q = clear_denominators_primitive(p)
        q = normalize_poly(q)
        if q == 0:
            record_zero_relation_removed(id)
            continue
        if q is nonzero constant:
            return empty_or_invalid_by_semantics(id)
        relations.push(CanonicalRelationQ{id, polynomial=q, source=Input})
    return CanonicalSystemQ{variables, target, relations, order, semantic_encodings, hash}
```

禁止:

```text
- 因数分解して勝手に component を選ぶ。
- 分母を消した事実を semantic/certificate なしに忘れる。
- geometry label に基づいて式を変形する。
```

### 9.5 `problem/context.rs`

型:

```rust
pub struct SolverContext {
    pub options: SolverOptions,
    pub id_counter: IdCounter,
    pub hash_config: HashConfig,
    pub resource_meter: ResourceMeter,
    pub diagnostics: Vec<DiagnosticRecord>,
}
```

関数:

```rust
pub fn new_context(options: SolverOptions) -> SolverContext;
pub fn check_resource(ctx: &mut SolverContext, stage: StageId) -> Result<(), SolverError>;
pub fn push_diagnostic(ctx: &mut SolverContext, diag: DiagnosticRecord);
```

---

## 10. File 別仕様: algebra

### 10.1 `algebra/monomial_order.rs`

関数:

```rust
pub fn elimination_order(eliminate: &[VariableId], keep: &[VariableId]) -> MonomialOrder;
pub fn grevlex_order(vars: &[VariableId]) -> MonomialOrder;
pub fn lex_order(vars: &[VariableId]) -> MonomialOrder;
pub fn block_order(blocks: Vec<Vec<VariableId>>) -> MonomialOrder;
```

`elimination_order(Y,Z)` は、任意の `y∈Y` が任意の `z∈Z` より大きい order を返す。

### 10.2 `algebra/polynomial_ops.rs`

`types/polynomial.rs` の低レベル関数を使い、代数処理用の高レベル操作を提供する。

関数:

```rust
pub fn leading_term(p: &SparsePolynomialQ, order: &MonomialOrder) -> Option<TermQ>;
pub fn s_polynomial(f: &SparsePolynomialQ, g: &SparsePolynomialQ, order: &MonomialOrder) -> SparsePolynomialQ;
pub fn reduce_by_set(f: &SparsePolynomialQ, gs: &[SparsePolynomialQ], order: &MonomialOrder) -> ReductionResult;
pub fn content_primitive_part(f: &SparsePolynomialQ) -> (RationalQ, SparsePolynomialQ);
```

### 10.3 `algebra/modular.rs`

関数:

```rust
pub fn choose_prime_avoiding_denominators(polys: &[SparsePolynomialQ], seed: u64) -> Prime;
pub fn reduce_q_to_fp(p: &SparsePolynomialQ, prime: Prime) -> SparsePolynomialFp;
pub fn lift_fp_coeff(c: Fp, prime: Prime) -> IntegerRepresentative;
```

疑似コード:

```rust
fn choose_prime_avoiding_denominators(polys, seed):
    p = deterministic_prime_stream(seed)
    while p divides any denominator or leading coefficient forbidden set:
        p = next_prime(p)
    return p
```

### 10.4 `algebra/crt.rs`

関数:

```rust
pub fn crt_combine(a_mod_m, b_mod_n) -> ModInteger;
pub fn crt_vector_combine(v1, mod1, v2, mod2) -> ModVector;
```

### 10.5 `algebra/rational_reconstruction.rs`

関数:

```rust
pub fn reconstruct_rational(a_mod_m: ModInteger, modulus: BigInt, height_bound: Option<usize>) -> Option<RationalQ>;
pub fn reconstruct_polynomial(mod_poly_data, modulus) -> Option<SparsePolynomialQ>;
```

### 10.6 `algebra/sparse_matrix.rs`

関数:

```rust
pub fn build_sparse_matrix_fp(rows: Vec<SparseRowFp>, ncols: usize) -> SparseMatrixFp;
pub fn row_echelon_sparse_fp(m: &SparseMatrixFp, prime: Prime) -> EchelonResultFp;
pub fn nullspace_sparse_fp(m: &SparseMatrixFp, prime: Prime) -> Vec<VectorFp>;
pub fn rank_sparse_fp(m: &SparseMatrixFp, prime: Prime) -> usize;
```

### 10.7 `algebra/linear_solve.rs`

関数:

```rust
pub fn solve_homogeneous_modular(matrix_builder: MatrixBuilder, plan: ModularSolvePlan) -> ModularNullspaceResult;
pub fn solve_inhomogeneous_modular(matrix_builder: MatrixBuilder, rhs: VectorQ, plan: ModularSolvePlan) -> ModularSolveResult;
```

処理:

```text
1. deterministic prime sequence を選ぶ。
2. 各 prime で行列を構成する。
3. rank/nullspace を計算する。
4. rank profile が安定するまで prime を増やす。
5. CRT + rational reconstruction を行う。
6. Q 上の exact check を呼ぶ側に返す。
```

### 10.8 `algebra/normal_form.rs`

関数:

```rust
pub fn normal_form(p: &SparsePolynomialQ, basis: &[SparsePolynomialQ], order: &MonomialOrder) -> SparsePolynomialQ;
pub fn verify_membership_by_certificate(g: &SparsePolynomialQ, cert: &MembershipCertificate, relations: &[SparsePolynomialQ]) -> bool;
```

`verify_membership_by_certificate` の疑似コード:

```rust
fn verify_membership_by_certificate(g, cert, relations):
    sum = 0
    for term in cert.combination_terms:
        sum += term.multiplier * relations[term.relation_id]
    return normalize_poly(sum - g) == 0
```

### 10.9 `algebra/groebner.rs`

関数:

```rust
pub fn groebner_elimination_basis(
    relations: &[SparsePolynomialQ],
    order: &MonomialOrder,
    options: GroebnerOptions,
) -> Result<GroebnerBasisResult, SolverError>;

pub fn extract_elimination_generators(
    basis: &[SparsePolynomialQ],
    keep_variables: &BTreeSet<VariableId>,
) -> Vec<SparsePolynomialQ>;
```

制約:

```text
- この file は local block の target/separator elimination のために使う。
- global coordinate-first production result を作るために使ってはならない。
```

### 10.10 `algebra/f4.rs`

関数:

```rust
pub fn f4_reduce_batch(
    reducers: &[SparsePolynomialQ],
    targets: &[SparsePolynomialQ],
    order: &MonomialOrder,
    options: F4Options,
) -> Result<F4BatchReductionResult, SolverError>;

pub fn f4_elimination_local(
    relations: &[SparsePolynomialQ],
    eliminate: &[VariableId],
    keep: &[VariableId],
    options: F4Options,
) -> Result<LocalEliminationResult, SolverError>;
```

出力:

```text
LocalEliminationResult:
    generators in Q[keep]
    membership/normal-form certificates
    matrix size trace
```

### 10.11 `algebra/elimination.rs`

関数:

```rust
pub fn eliminate_to_keep_variables(
    relations: &[SparsePolynomialQ],
    eliminate: &[VariableId],
    keep: &[VariableId],
    strategy: EliminationStrategy,
    ctx: &mut SolverContext,
) -> Result<EliminationResult, SolverError>;
```

処理:

```text
1. keep と eliminate が disjoint であることを確認する。
2. strategy に従い Groebner / F4 / relation search / resultant を呼ぶ。
3. 結果 relation が Q[keep] に属することを確認する。
4. certificate を付ける。
```

### 10.12 `algebra/resultant.rs`

関数:

```rust
pub fn support_sets(polys: &[SparsePolynomialQ]) -> Vec<MonomialSupport>;
pub fn build_sparse_resultant_template(input: ResultantInput) -> Result<ResultantTemplate, SolverError>;
pub fn compute_resultant_relation(template: &ResultantTemplate, options: ModularOptions) -> Result<ResultantRelation, SolverError>;
pub fn verify_resultant_certificate(cert: &SparseResultantCertificate) -> bool;
```

### 10.13 `algebra/interpolation.rs`

関数:

```rust
pub fn choose_specialization_points(vars: &[VariableId], count: usize, prime: Prime) -> Vec<SpecializationPoint>;
pub fn specialize_polynomials(polys: &[SparsePolynomialQ], point: &SpecializationPoint) -> Vec<SparsePolynomialQ>;
pub fn interpolate_sparse_coefficients(samples: &[SpecializedRelation]) -> Result<SparsePolynomialQ, SolverError>;
pub fn verify_interpolated_relation(relation: &SparsePolynomialQ, certificate: &InterpolationCertificate) -> bool;
```

### 10.14 `algebra/quotient.rs`

関数・trait:

```rust
pub trait TargetQuotientHandle {
    fn basis_id(&self) -> BasisHandleId;
    fn basis_size(&self) -> usize;
    fn basis_scope(&self) -> BasisScope;
    fn normal_form(&self, p: &SparsePolynomialQ) -> Result<VectorQ, SolverError>;
    fn multiply_by_variable(&self, v: &VectorQ, var: VariableId) -> Result<VectorQ, SolverError>;
    fn no_coordinate_solution_export(&self) -> bool;
}

pub fn build_target_relevant_quotient_handle(input: QuotientHandleInput) -> Result<Box<dyn TargetQuotientHandle>, SolverError>;
```

禁止:

```text
- coordinate roots を返す API
- full coordinate RUR を返す API
- target-unrelated full quotient basis を公開する API
```

### 10.15 `algebra/krylov.rs`

関数:

```rust
pub fn block_krylov_sequence(handle: &dyn TargetQuotientHandle, target: VariableId, plan: KrylovPlan) -> Result<KrylovSequence, SolverError>;
pub fn recover_recurrence(seq: &KrylovSequence) -> Result<RecurrencePolynomial, SolverError>;
pub fn certify_krylov_coverage(seq: &KrylovSequence, recurrence: &RecurrencePolynomial, handle: &dyn TargetQuotientHandle) -> Result<CoverageCertificate, SolverError>;
pub fn verify_annihilator(handle: &dyn TargetQuotientHandle, poly: &UniPolynomialQ) -> Result<AnnihilatorCertificate, SolverError>;
```

### 10.16 `algebra/regular_chain.rs`

関数:

```rust
pub fn local_regular_chain_decomposition(input: RegularChainInput) -> Result<RegularChainDAG, SolverError>;
pub fn project_chain_to_variables(chain: &RegularChain, keep: &[VariableId]) -> Result<ProjectionGenerators, SolverError>;
pub fn combine_chain_projections(chains: &[ProjectionGenerators], semantics: UnionSemantics) -> Result<Vec<SparsePolynomialQ>, SolverError>;
```

### 10.17 `algebra/norm_trace.rs`

関数:

```rust
pub fn detect_explicit_tower(relations: &[SparsePolynomialQ], exported: &[VariableId]) -> Option<TowerDescription>;
pub fn norm_of_target_minus_expression(tower: &TowerDescription, target_expr: SparsePolynomialQ) -> Result<UniOrMultiPolynomialQ, SolverError>;
pub fn verify_norm_relation(tower: &TowerDescription, relation: &SparsePolynomialQ) -> bool;
```

### 10.18 `algebra/real_root.rs`

関数:

```rust
pub fn sturm_sequence(p: &UniPolynomialQ) -> Vec<UniPolynomialQ>;
pub fn isolate_real_roots_sturm(p: &UniPolynomialQ) -> Result<Vec<RealRootRecord>, SolverError>;
pub fn isolate_real_roots_descartes(p: &UniPolynomialQ) -> Result<Vec<RealRootRecord>, SolverError>;
```

### 10.19 `algebra/sign.rs`

関数:

```rust
pub fn sign_at_algebraic_root(poly: &UniPolynomialQ, root: &RealRootRecord) -> SignDetermination;
pub fn thom_encoding(poly: &UniPolynomialQ, root: &RealRootRecord) -> ThomEncoding;
```

---

## 11. File 別仕様: preprocess

### 11.1 `preprocess/compression.rs`

関数:

```rust
pub fn pre_kernel_compress(system: CanonicalSystemQ, ctx: &mut SolverContext) -> Result<CompressedSystemQ, SolverError>;
```

処理順:

```text
1. definitional elimination
2. linear affine elimination
3. binomial / monomial simplification
4. safe saturation for explicitly nonzero encodings
5. target-independent component marking
6. coefficient height and monomial count trace
```

疑似コード:

```rust
fn pre_kernel_compress(system, ctx):
    state = CompressionState::from(system)
    state = eliminate_definitional_variables(state, ctx)?
    state = eliminate_linear_affine_variables(state, ctx)?
    state = simplify_binomial_relations(state, ctx)?
    state = apply_explicit_saturations(state, ctx)?
    state = mark_target_independent_components(state, ctx)?
    return state.to_compressed_system()
```

禁止:

```text
- geometry name による rewrite
- expected answer による factor selection
- component semantics を記録しない factor split
```

### 11.2 `preprocess/definitional.rs`

目的: 明らかな定義変数を消す。

対象:

```text
y - p(X) = 0
c*y - p(X) = 0 where c ∈ Q\{0}
```

関数:

```rust
pub fn find_definitional_relations(system: &CanonicalSystemQ) -> Vec<DefinitionalCandidate>;
pub fn apply_definitional_elimination(state: CompressionState, candidates: &[DefinitionalCandidate], ctx: &mut SolverContext) -> Result<CompressionState, SolverError>;
```

疑似コード:

```rust
fn find_definitional_relations(system):
    candidates = []
    for relation in system.relations:
        for variable y in variables(relation):
            if relation is linear in y and coefficient(y) is nonzero rational constant:
                if y not target:
                    candidates.push(y = -rest/coefficient)
    sort candidates by deterministic cost score
    return candidates
```

注意:

```text
coefficient が variable-dependent の場合は definitional ではなく LinearAffine へ回す。
```

### 11.3 `preprocess/linear_affine.rs`

対象:

```text
a(X)*y + b(X) = 0
```

ただし `a(X)` が非ゼロであることを guard として記録できる場合のみ substitution する。非ゼロを証明できない場合は式を残す。

関数:

```rust
pub fn find_linear_affine_candidates(state: &CompressionState) -> Vec<LinearAffineCandidate>;
pub fn select_safe_affine_pivots(candidates: &[LinearAffineCandidate], policy: PivotPolicy) -> Vec<AffinePivot>;
pub fn eliminate_linear_affine_variables(state: CompressionState, ctx: &mut SolverContext) -> Result<CompressionState, SolverError>;
```

疑似コード:

```rust
fn eliminate_linear_affine_variables(state, ctx):
    loop:
        candidates = find_linear_affine_candidates(state)
        pivots = select_safe_affine_pivots(candidates, MarkowitzLikePolicy)
        if pivots.empty(): break
        pivot = pivots[0]
        if pivot.denominator_is_constant_nonzero:
            substitute directly
        else if pivot.denominator_has_recorded_nonzero_semantics:
            substitute and record denominator guard
        else:
            mark candidate rejected and continue
        if coefficient_height(state) > ctx.options.height_limit:
            return Err(FiniteResourceFailure)
    return state
```

### 11.4 `preprocess/binomial.rs`

目的: monomial/binomial 的な式の局所簡約を行う。

対象例:

```text
u - c*v = 0
u*v - c = 0
u^k - v = 0
```

関数:

```rust
pub fn detect_binomial_relations(state: &CompressionState) -> Vec<BinomialCandidate>;
pub fn simplify_binomial_relations(state: CompressionState, ctx: &mut SolverContext) -> Result<CompressionState, SolverError>;
```

制約:

```text
- union semantics を生む factor split はしない。
- 不可逆変形をするときは guard/certificate を残す。
```

### 11.5 `preprocess/saturation.rs`

関数:

```rust
pub fn apply_explicit_saturations(state: CompressionState, ctx: &mut SolverContext) -> Result<CompressionState, SolverError>;
pub fn is_explicit_nonzero_factor(factor: &SparsePolynomialQ, semantics: &[RealConstraintEncoding]) -> bool;
```

許可:

```text
A*s - 1 = 0 により A != 0 が明示されている場合、A による saturation を記録付きで使う。
```

### 11.6 `preprocess/independent.rs`

関数:

```rust
pub fn mark_target_independent_components(state: CompressionState, ctx: &mut SolverContext) -> Result<CompressionState, SolverError>;
pub fn compute_component_feasibility_obligations(components: &[Component]) -> Vec<FeasibilityObligation>;
```

規則:

```text
- target へ path を持たない component は candidate cover 構成から外してよい。
- exact image mode では feasibility obligation として残す。
- component が hard だから外すことは禁止。
```

---

## 12. File 別仕様: graph

### 12.1 `graph/hypergraph.rs`

関数:

```rust
pub fn build_relation_variable_hypergraph(system: &CompressedSystemQ) -> RelationVariableHypergraph;
pub fn connected_components(h: &RelationVariableHypergraph) -> Vec<HypergraphComponent>;
pub fn relation_variables(h: &RelationVariableHypergraph, r: RelationId) -> BTreeSet<VariableId>;
pub fn variable_relations(h: &RelationVariableHypergraph, v: VariableId) -> BTreeSet<RelationId>;
```

疑似コード:

```rust
fn build_relation_variable_hypergraph(system):
    h = empty
    for relation in system.relations:
        h.add_relation(relation.id)
        vars = poly_variables(relation.polynomial)
        for v in vars:
            h.add_variable(v)
            h.add_incidence(relation.id, v)
    assert every polynomial occurrence is represented
    return h
```

### 12.2 `graph/influence.rs`

関数:

```rust
pub fn build_target_influence_graph(h: &RelationVariableHypergraph, target: VariableId) -> TargetInfluenceGraph;
```

疑似コード:

```rust
fn build_target_influence_graph(h, target):
    component = BFS in bipartite graph from target variable node
    independent = all other components
    return TargetInfluenceGraph{target_component, independent_components}
```

### 12.3 `graph/weighted_primal.rs`

関数:

```rust
pub fn build_weighted_primal_graph(system: &CompressedSystemQ, influence: &TargetInfluenceGraph) -> WeightedPrimalGraph;
pub fn variable_weight(v: VariableId, system: &CompressedSystemQ) -> AlgebraicWeight;
pub fn edge_weight(u: VariableId, v: VariableId, relations: &[RelationId]) -> AlgebraicWeight;
```

重み:

```text
- variable degree participation
- monomial count contribution
- coefficient height contribution
- target distance
- linear eliminability
- occurrence count
```

### 12.4 `graph/separators.rs`

関数:

```rust
pub fn articulation_variable_candidates(g: &WeightedPrimalGraph) -> Vec<SeparatorCandidate>;
pub fn min_fill_separator_candidates(g: &WeightedPrimalGraph, target: VariableId) -> Vec<SeparatorCandidate>;
pub fn score_separator(candidate: &SeparatorCandidate, subgraph: &WeightedPrimalGraph, cost_model: &CostModel) -> SeparatorScore;
```

疑似コード:

```rust
fn score_separator(candidate, subgraph, cost_model):
    components = remove_separator(subgraph, candidate.vars)
    score = 0
    for comp in components:
        estimated_block = comp.vars ∪ candidate.vars
        score += estimate_local_projection_cost(estimated_block)
    score += separator_degree_penalty(candidate)
    score += coefficient_height_penalty(candidate)
    return score
```

### 12.5 `graph/tree_decomposition.rs`

関数:

```rust
pub fn build_target_rooted_decomposition(g: &WeightedPrimalGraph, target: VariableId, cost_model: &CostModel) -> DecompositionTree;
```

疑似コード:

```rust
fn decompose(subgraph, target_side):
    if subgraph.size <= options.max_direct_block_width:
        return leaf_block(subgraph)

    candidates = articulation_variable_candidates(subgraph)
               ∪ min_fill_separator_candidates(subgraph, target_side)
               ∪ bounded_min_cut_candidates(subgraph, target_side)

    scored = sort_by(score_separator)
    for cand in scored:
        if improves_estimated_cost(cand, subgraph):
            children = components_after_removing(cand)
            return node(separator=cand, children=map(decompose, children))

    return leaf_block(subgraph) // not unsupported
```

### 12.6 `graph/projection_dag.rs`

関数:

```rust
pub fn build_target_projection_dag(
    system: &CompressedSystemQ,
    influence: &TargetInfluenceGraph,
    decomposition: &DecompositionTree,
) -> Result<TargetProjectionDAG, SolverError>;

pub fn authorize_block_relations(block: &mut ProjectionBlock, system: &CompressedSystemQ) -> AuthorizationHash;
pub fn validate_projection_dag(dag: &TargetProjectionDAG, system: &CompressedSystemQ) -> Result<(), SolverError>;
```

規則:

```text
- 各 relation は原則1つの block に属する。
- relation duplication が必要な場合は DuplicationCertificate が必要。
- 各 block の projector は authorization_hash に含まれた relation だけ読める。
- no useful separator の場合は one large target block を作る。
```

### 12.7 `graph/metrics.rs`

関数:

```rust
pub fn structural_metrics(block: &ProjectionBlock, system: &CompressedSystemQ) -> StructuralMetrics;
pub fn estimate_local_quotient_rank(block: &ProjectionBlock) -> RankEstimate;
pub fn estimate_sparse_template_size(block: &ProjectionBlock) -> TemplateEstimate;
pub fn estimate_coefficient_growth(block: &ProjectionBlock) -> HeightEstimate;
```

---

## 13. File 別仕様: planner

### 13.1 `planner/cost_model.rs`

型:

```rust
pub struct CostModelWeights {
    pub matrix_size_weight: RationalQ,
    pub quotient_rank_weight: RationalQ,
    pub coefficient_height_weight: RationalQ,
    pub separator_degree_weight: RationalQ,
    pub certificate_cost_weight: RationalQ,
}
```

関数:

```rust
pub fn estimate_kernel_cost(block: &ProjectionBlock, kernel: KernelKind, probes: &ProbeResults) -> KernelCostEstimate;
pub fn compare_cost(a: &KernelCostEstimate, b: &KernelCostEstimate) -> Ordering;
```

比較は deterministic である。

### 13.2 `planner/probes.rs`

関数:

```rust
pub fn run_cost_probes(block: &ProjectionBlock, system: &CompressedSystemQ, ctx: &mut SolverContext) -> ProbeResults;
pub fn modular_rank_probe(block: &ProjectionBlock, primes: &[Prime]) -> RankProbeResult;
pub fn local_macaulay_size_probe(block: &ProjectionBlock) -> MacaulaySizeProbe;
pub fn mixed_support_probe(block: &ProjectionBlock) -> SparseSupportProbe;
```

注意:

```text
probe は planner の cost estimate にだけ使う。
正しさの証明には使わない。
```

### 13.3 `planner/admission.rs`

関数:

```rust
pub fn collect_kernel_admissions(block: &ProjectionBlock, ctx: &KernelContext) -> Vec<KernelAdmission>;
```

処理:

```text
1. すべての kernel の admit を呼ぶ。
2. UniversalTargetEliminationKernel は well-formed block なら必ず admissible。
3. admission false は runtime failure ではない。
4. admissions を hash 付きで返す。
```

### 13.4 `planner/kernel_plan.rs`

型:

```rust
pub struct KernelPlan {
    pub block_id: BlockId,
    pub declared_ladder: Vec<KernelExecutionPlan>,
    pub selected_first: KernelKind,
    pub admissions: Vec<KernelAdmission>,
    pub cost_estimates: Vec<KernelCostEstimate>,
    pub plan_hash: Hash,
}
```

`declared_ladder` は hidden fallback ではない。実行前に全て確定し、certificate に入る。

### 13.5 `planner/ladder.rs`

関数:

```rust
pub fn build_declared_ladder(admissions: &[KernelAdmission], costs: &[KernelCostEstimate]) -> Vec<KernelExecutionPlan>;
```

規則:

```text
- certificate available な kernel だけ ladder に入れる。
- coordinate-first kernel は存在してはならない。
- UniversalTargetEliminationKernel は最後の generic target-direct plan として入る。
- ladder の各 plan は resource bound と degree bound を明示する。
```

### 13.6 `planner/planner.rs`

関数:

```rust
pub fn plan_all_blocks(dag: &TargetProjectionDAG, system: &CompressedSystemQ, ctx: &mut SolverContext) -> Result<Vec<KernelPlan>, SolverError>;
```

疑似コード:

```rust
fn plan_all_blocks(dag, system, ctx):
    plans = []
    for block in postorder(dag):
        kctx = KernelContext::new(block, system, messages_from_children)
        probes = run_cost_probes(block, system, ctx)
        admissions = collect_kernel_admissions(block, kctx)
        costs = admissions.map(|a| estimate_kernel_cost(block, a.kind, probes))
        ladder = build_declared_ladder(admissions, costs)
        if ladder.empty():
            return Err(ImplementationBug("Universal kernel missing"))
        plans.push(KernelPlan{block_id, ladder, selected_first=ladder[0].kind, ...})
    return plans
```

---

## 14. File 別仕様: kernels 共通

### 14.1 `kernels/traits.rs`

trait:

```rust
pub trait TargetProjectionKernel {
    fn kind(&self) -> KernelKind;
    fn admit(&self, block: &ProjectionBlock, ctx: &KernelContext) -> KernelAdmission;
    fn plan(&self, admission: &KernelAdmission, ctx: &KernelContext) -> Result<KernelExecutionPlan, SolverError>;
    fn execute(&self, plan: &KernelExecutionPlan, ctx: &mut KernelContext) -> Result<ProjectionMessage, SolverError>;
    fn replay(&self, message: &ProjectionMessage, ctx: &KernelContext) -> ReplayResult;
}
```

`execute` 共通条件:

```text
入力:
    plan: planner が事前に宣言した実行計画
    ctx: block relation、child messages、resource meter

処理:
    target/separator relation を構成する。
    exact Q certificate を作る。
    local coordinate solution を出力しない。

出力:
    ProjectionMessage
```

### 14.2 `kernels/mod.rs`

関数:

```rust
pub fn all_kernels() -> Vec<Box<dyn TargetProjectionKernel>>;
pub fn kernel_by_kind(kind: KernelKind) -> Box<dyn TargetProjectionKernel>;
```

`all_kernels()` は次を返す。

```text
1. TargetUnivariateKernel
2. LinearAffineKernel
3. TargetRelationSearchKernel
4. SparseResultantProjectionKernel
5. TargetActionKrylovKernel
6. NormTraceProjectionKernel
7. RegularChainProjectionKernel
8. SpecializationInterpolationKernel
9. UniversalTargetEliminationKernel
```

---

## 15. Kernel: TargetUnivariateKernel

File: `kernels/target_univariate.rs`

### 15.1 Admission

```rust
pub fn admit_target_univariate(block: &ProjectionBlock, ctx: &KernelContext) -> KernelAdmission;
```

条件:

```text
block relation または child message relation の中に、variables(f) ⊆ {T} を満たす非零 relation が存在する。
```

### 15.2 Execute

```rust
pub fn execute_target_univariate(plan: &KernelExecutionPlan, ctx: &mut KernelContext) -> Result<ProjectionMessage, SolverError>;
```

疑似コード:

```rust
fn execute_target_univariate(plan, ctx):
    rels = []
    for r in ctx.available_relations():
        if variables(r) subset {target} and r != 0:
            rels.push(convert_to_univariate(r))

    if rels.empty():
        return Err(ImplementationBug("admission invalid"))

    support = primitive_lcm_squarefree(rels)
    cert = SourceMembershipCertificate(rels.source_ids)

    return ProjectionMessage{
        exported_variables: [target],
        relation_generators: [support_as_sparse],
        representation: PrincipalSupport,
        projection_strength: CandidateCoverStrong,
        certificate: cert,
        ...
    }
```

---

## 16. Kernel: LinearAffineKernel

File: `kernels/linear_affine.rs`

### 16.1 Admission

条件:

```text
block local variables Y can be eliminated by triangular affine substitutions,
leaving relations only in exported variables Z={T}+separators.
```

関数:

```rust
pub fn find_triangular_affine_order(block: &ProjectionBlock, ctx: &KernelContext) -> Option<AffineEliminationOrder>;
```

### 16.2 Execute

```rust
pub fn execute_linear_affine(plan: &KernelExecutionPlan, ctx: &mut KernelContext) -> Result<ProjectionMessage, SolverError>;
```

疑似コード:

```rust
fn execute_linear_affine(plan, ctx):
    order = plan.affine_order
    relations = ctx.block_relations.clone()
    substitutions = []

    for step in order:
        equation = choose_equation_linear_in(step.variable)
        (a,b) = split_as_a_x_plus_b(equation, step.variable)
        if a is constant nonzero:
            subst = x -> -b/a
        else if ctx.has_nonzero_guard(a):
            subst = x -> -b/a
            record_denominator_guard(a)
        else:
            return Err(ImplementationBug("unsafe affine pivot in plan"))
        relations = substitute_all(relations, subst)
        substitutions.push(subst)
        check_resource()

    exported_relations = []
    for r in relations:
        r = clear_denominators_primitive(r)
        if variables(r) subset exported_variables:
            exported_relations.push(r)
        else if r != 0:
            return Err(AlgorithmicHardCase("affine elimination incomplete"))

    cert = LinearAffineCertificate(substitutions, exported_relations)
    return ProjectionMessage{relation_generators: exported_relations, certificate: cert, ...}
```

---

## 17. Kernel: TargetRelationSearchKernel

File: `kernels/target_relation_search.rs`

この kernel は v4 の中心的な generic target-direct workhorse である。

### 17.1 数学的目的

局所 block の ideal を

```text
J = <f1,...,fr> ⊂ Q[Y,Z]
```

とする。ここで

```text
Y = eliminated local variables
Z = exported variables = {T}+separators
```

目的は、非零 relation

```text
g(Z) ∈ J ∩ Q[Z]
```

を、座標解を出さずに見つけることである。

### 17.2 基本方程式

未知係数を置く。

```text
g(Z) = Σ_{α∈A} c_α Z^α
q_i(Y,Z) = Σ_{β∈B_i} u_{i,β} YZ^β
```

次を満たす係数を探す。

```text
g(Z) = Σ_i q_i(Y,Z) f_i(Y,Z)
```

係数比較により線形方程式を得る。非自明な `c_α` が得られれば、`g ∈ J∩Q[Z]` の membership certificate も同時に得られる。

### 17.3 Admission

全 block に対して admission は可能だが、planner は cost estimate により実行順を決める。

```rust
pub fn admit_target_relation_search(block: &ProjectionBlock, ctx: &KernelContext) -> KernelAdmission;
```

返す情報:

```text
- exported variables Z
- eliminated variables Y
- initial degree bounds
- support strategy
- estimated matrix size
```

### 17.4 Execute

```rust
pub fn execute_target_relation_search(plan: &KernelExecutionPlan, ctx: &mut KernelContext) -> Result<ProjectionMessage, SolverError>;
```

疑似コード:

```rust
fn execute_target_relation_search(plan, ctx):
    J = ctx.local_relations_plus_child_messages()
    Z = ctx.exported_variables()
    Y = ctx.local_variables_minus(Z)

    for bound in plan.declared_degree_bounds:
        A = build_export_monomial_support(Z, bound, plan.support_strategy)
        B = build_multiplier_supports(J, Y, Z, bound, plan.multiplier_strategy)

        matrix_builder = build_membership_matrix_builder(J, A, B)
        ns = solve_homogeneous_modular(matrix_builder, plan.modular_plan)

        candidates = reconstruct_candidate_relations(ns, A, B)
        for cand in candidates sorted deterministically:
            if cand.g == 0:
                continue
            if variables(cand.g) not subset Z:
                continue
            if verify_membership_exact(cand.g, cand.qs, J):
                cert = MembershipCertificate(g=cand.g, qs=cand.qs)
                return ProjectionMessage{
                    exported_variables: Z,
                    relation_generators: [primitive(cand.g)],
                    representation: GeneratorSet,
                    projection_strength: CandidateCoverStrong,
                    certificate: cert,
                    cost_trace: matrix_builder.trace,
                }

    return Err(AlgorithmicHardCase{
        reason: "no relation found within declared bounds",
        matrix_trace: accumulated_trace,
    })
```

### 17.5 build_export_monomial_support

```rust
pub fn build_export_monomial_support(
    exported: &[VariableId],
    bound: DegreeBound,
    strategy: SupportStrategy,
) -> Vec<Monomial>;
```

戦略:

```text
DenseTotalDegree:
    all monomials in Z with total degree <= bound

SparseFromProjectionFootprint:
    monomials predicted from supports of local relations after eliminating Y

SpecializedInterpolationFootprint:
    monomials seen in specialized runs
```

### 17.6 build_multiplier_supports

```rust
pub fn build_multiplier_supports(
    relations: &[SparsePolynomialQ],
    eliminated: &[VariableId],
    exported: &[VariableId],
    bound: DegreeBound,
    strategy: MultiplierSupportStrategy,
) -> Vec<Vec<Monomial>>;
```

規則:

```text
q_i f_i の monomials が coefficient comparison set に入るように B_i を作る。
B_i は deterministic に並べる。
coefficient explosion が予測される場合は FiniteResourceFailure ではなく cost estimate を planner に返す。
execute 中に resource limit を超えた場合のみ FiniteResourceFailure。
```

### 17.7 Certificate

```yaml
TargetRelationSearchCertificate:
  exported_variables_hash: ""
  eliminated_variables_hash: ""
  export_support_hash: ""
  multiplier_support_hash: ""
  membership_matrix_hash: ""
  primes_used: []
  rational_reconstruction_hash: ""
  relation_hash: ""
  multipliers_hash: ""
  exact_identity_hash: "g - Σ q_i f_i = 0"
```

---

## 18. Kernel: SparseResultantProjectionKernel

File: `kernels/sparse_resultant.rs`

### 18.1 Admission

```rust
pub fn admit_sparse_resultant(block: &ProjectionBlock, ctx: &KernelContext) -> KernelAdmission;
```

条件:

```text
- eliminated variables Y and exported variables Z are known.
- support sets allow finite resultant/eliminant template estimate.
- estimated sparse template size is finite.
- exact verification method is available.
```

「sparse enough でなければ unsupported」ではない。admission false になるだけで、他の generic kernel が処理する。

### 18.2 Execute

```rust
pub fn execute_sparse_resultant(plan: &KernelExecutionPlan, ctx: &mut KernelContext) -> Result<ProjectionMessage, SolverError>;
```

疑似コード:

```rust
fn execute_sparse_resultant(plan, ctx):
    polys = ctx.local_relations_plus_child_messages()
    supports = support_sets(polys)
    template = build_sparse_resultant_template(supports, plan.template_options)?

    rel_mod = compute_template_determinant_or_null_relation(template, plan.modular_plan)?
    relation = reconstruct_polynomial(rel_mod)?

    if variables(relation) not subset ctx.exported_variables:
        return Err(ImplementationBug("resultant exported local variable"))

    cert = verify_resultant_or_membership(relation, template, polys)?

    return ProjectionMessage{
        relation_generators: [primitive(relation)],
        representation: SparseResultantMatrix,
        projection_strength: CandidateCoverStrong,
        certificate: cert,
        ...
    }
```

---

## 19. Kernel: TargetActionKrylovKernel

File: `kernels/action_krylov.rs`

### 19.1 目的

有限 rank の target-relevant quotient/action が安く作れる場合、multiplication-by-target operator の annihilating polynomial を計算して target support を得る。

### 19.2 Admission

```rust
pub fn admit_action_krylov(block: &ProjectionBlock, ctx: &KernelContext) -> KernelAdmission;
```

条件:

```text
- local quotient/action handle の rank estimate が有限。
- normal form が計算可能。
- handle が coordinate roots / full coordinate RUR を公開しない。
- coverage certificate を構成できる見込みがある。
```

### 19.3 Execute

```rust
pub fn execute_action_krylov(plan: &KernelExecutionPlan, ctx: &mut KernelContext) -> Result<ProjectionMessage, SolverError>;
```

疑似コード:

```rust
fn execute_action_krylov(plan, ctx):
    handle = build_target_relevant_quotient_handle(plan.quotient_plan, ctx)?
    if !handle.no_coordinate_solution_export():
        return Err(ImplementationBug("coordinate-exporting handle"))

    seq = block_krylov_sequence(handle, ctx.target, plan.krylov_plan)?
    recurrence = recover_recurrence(seq)?
    coverage = certify_krylov_coverage(seq, recurrence, handle)?

    if !coverage.valid:
        return Err(CertificateDesignGap("krylov coverage missing"))

    annihilator = recurrence.to_univariate(ctx.target)
    ann_cert = verify_annihilator(handle, annihilator)?

    return ProjectionMessage{
        exported_variables: [ctx.target] plus ctx.separators_if_symbolic,
        relation_generators: [annihilator_as_sparse],
        representation: QuotientAction,
        projection_strength: CandidateCoverStrong,
        certificate: TargetActionKrylovCertificate(coverage, ann_cert),
    }
```

### 19.4 Coverage 問題

単一 Krylov 列は eigenvalue を見落とす可能性がある。そのため、以下のいずれかが必要である。

```text
- deterministic basis probe coverage
- block Wiedemann rank/degree proof
- trace power certificate
- verified characteristic support comparison
- S(M_T)=0 plus proof that all target-relevant eigenvalues are included
```

coverage が証明できない場合、candidate polynomial を返してはならない。

---

## 20. Kernel: UniversalTargetEliminationKernel

File: `kernels/universal_elimination.rs`

### 20.1 位置づけ

この kernel は fallback ではない。有理数係数多項式系全体を入力範囲にするために必要な、計画済み generic target/separator projection kernel である。

### 20.2 Admission

```rust
pub fn admit_universal_elimination(block: &ProjectionBlock, ctx: &KernelContext) -> KernelAdmission;
```

条件:

```text
block が Q-polynomial relations を持つなら true。
```

### 20.3 Execute

```rust
pub fn execute_universal_elimination(plan: &KernelExecutionPlan, ctx: &mut KernelContext) -> Result<ProjectionMessage, SolverError>;
```

疑似コード:

```rust
fn execute_universal_elimination(plan, ctx):
    Z = ctx.exported_variables()
    Y = ctx.local_variables_minus(Z)
    J = ctx.local_relations_plus_child_messages()

    order = elimination_order(Y, Z)

    result = algebra::elimination::eliminate_to_keep_variables(
        J, Y, Z, plan.elimination_strategy, ctx
    )?

    gens = []
    for g in result.generators:
        if g != 0 and variables(g) subset Z:
            gens.push(primitive(g))

    if gens.empty():
        if certify_nonfinite_projection(J, Z):
            return Err(CertifiedNonFiniteTargetImage)
        else:
            return Err(AlgorithmicHardCase("no exported relation"))

    cert = result.certificate
    verify_every_generator_exact(gens, cert, J)?

    return ProjectionMessage{
        exported_variables: Z,
        relation_generators: gens,
        representation: GeneratorSet,
        projection_strength: ExactProjectionIdeal or CandidateCoverStrong,
        certificate: cert,
        cost_trace: result.cost_trace,
    }
```

### 20.4 内部戦略

`UniversalTargetEliminationKernel` は次の内部戦略を planner が選ぶ。

```text
- EliminationGroebnerLocal
- F4EliminationLocal
- TargetRelationSearchEscalated
- ResultantIfSquareOrOverdetermined
- SpecializeProjectInterpolateVerify
```

禁止:

```text
- coordinate solution enumeration
- full coordinate RUR
- global QE/CAD
- geometry-specific branch
- runtime hidden fallback
```

---

## 21. Kernel: RegularChainProjectionKernel

File: `kernels/regular_chain_projection.rs`

### 21.1 Admission

```rust
pub fn admit_regular_chain_projection(block: &ProjectionBlock, ctx: &KernelContext) -> KernelAdmission;
```

条件:

```text
- triangular pattern が見える。
- component/guard/projection semantics を保つ必要がある。
- compact ComponentDAG を作れる見込みがある。
```

### 21.2 Execute

```rust
pub fn execute_regular_chain_projection(plan: &KernelExecutionPlan, ctx: &mut KernelContext) -> Result<ProjectionMessage, SolverError>;
```

疑似コード:

```rust
fn execute_regular_chain_projection(plan, ctx):
    chains = local_regular_chain_decomposition(ctx.local_relations, plan.chain_options)?
    component_messages = []
    for chain in chains:
        projected = project_chain_to_variables(chain, ctx.exported_variables)?
        cert = verify_chain_projection(chain, projected)?
        component_messages.push((projected, cert))

    combined = combine_component_relations(component_messages, union_semantics)

    return ProjectionMessage{
        relation_generators: combined.generators,
        representation: TriangularChain,
        projection_strength: CandidateCoverStrong,
        certificate: combined.certificate,
    }
```

---

## 22. Kernel: NormTraceProjectionKernel

File: `kernels/norm_trace_projection.rs`

### 22.1 Admission

```rust
pub fn admit_norm_trace_projection(block: &ProjectionBlock, ctx: &KernelContext) -> KernelAdmission;
```

条件:

```text
relations が明示的な有限代数塔を定義する。
例: α satisfies p(α)=0, T = r(α,Z)
```

これは幾何名ではなく、式の代数形で判定する。

### 22.2 Execute

```rust
pub fn execute_norm_trace_projection(plan: &KernelExecutionPlan, ctx: &mut KernelContext) -> Result<ProjectionMessage, SolverError>;
```

疑似コード:

```rust
fn execute_norm_trace_projection(plan, ctx):
    tower = detect_explicit_tower(ctx.local_relations, ctx.exported_variables)?
    relation = norm_of_target_minus_expression(tower, plan.target_expression)?
    if !verify_norm_relation(tower, relation):
        return Err(CertificateDesignGap("norm relation not verified"))
    return ProjectionMessage{relation_generators:[relation], representation:NormTraceTower, ...}
```

---

## 23. Kernel: SpecializationInterpolationKernel

File: `kernels/specialization_interpolation.rs`

### 23.1 目的

separator が複数ある場合、`g(T,u1,...,uτ)` を直接構成すると係数膨張が起きる。そこで separator を一時的に特殊化し、target relation を計算し、係数を補間し、最後に Q 上で検証する。

### 23.2 Admission

```rust
pub fn admit_specialization_interpolation(block: &ProjectionBlock, ctx: &KernelContext) -> KernelAdmission;
```

条件:

```text
- exported variables Z のうち target 以外の separator が存在する。
- specialization 後の target-only relation 計算が安くなる見込みがある。
- interpolation support bound が宣言できる。
- 最後に exact Q verification が可能。
```

### 23.3 Execute

```rust
pub fn execute_specialization_interpolation(plan: &KernelExecutionPlan, ctx: &mut KernelContext) -> Result<ProjectionMessage, SolverError>;
```

疑似コード:

```rust
fn execute_specialization_interpolation(plan, ctx):
    T = ctx.target
    U = ctx.exported_variables - {T}
    samples = []

    for point in choose_specialization_points(U, plan.sample_count, plan.primes):
        specialized_ctx = specialize_context(ctx, U, point)
        local_plan = plan.inner_target_only_plan
        msg = execute_declared_inner_kernel(local_plan, specialized_ctx)?
        samples.push((point, msg.relation_generators[0]))

    relation = interpolate_sparse_coefficients(samples, plan.interpolation_support)?

    if variables(relation) not subset ctx.exported_variables:
        return Err(ImplementationBug("interpolated relation has local variables"))

    cert = verify_interpolated_relation_by_membership_or_elimination(relation, ctx)?

    return ProjectionMessage{
        relation_generators: [relation],
        representation: SpecializationInterpolation,
        projection_strength: CandidateCoverStrong,
        certificate: cert,
    }
```

重要:

```text
specialization/interpolation は候補生成である。
正しさは最後の exact Q verification によってのみ認める。
```

---

## 24. Projection message の合成

### 24.1 `compose/message.rs`

型:

```rust
pub struct MessageIdeal {
    pub variables: Vec<VariableId>,
    pub relations: Vec<SparsePolynomialQ>,
    pub source_packages: Vec<PackageId>,
}
```

関数:

```rust
pub fn message_to_relations(message: &ProjectionMessage) -> Vec<SparsePolynomialQ>;
pub fn merge_messages(messages: &[ProjectionMessage]) -> MessageIdeal;
```

### 24.2 `compose/compose.rs`

関数:

```rust
pub fn compose_projection_messages(
    dag: &TargetProjectionDAG,
    messages: Vec<ProjectionMessage>,
    ctx: &mut SolverContext,
) -> Result<ComposedProjection, SolverError>;
```

疑似コード:

```rust
fn compose_projection_messages(dag, messages, ctx):
    current = messages indexed by block
    for node in postorder_to_root(dag):
        incoming = child_messages(node)
        local_msg = current[node]
        merged = merge_messages(incoming ∪ local_msg)
        if merged.variables subset {target}:
            attach_to_node(node, merged)
        else:
            eliminated = eliminate_separators(merged, node.parent_separator, ctx)?
            attach_to_parent(node, eliminated)
    return root_composed_projection
```

### 24.3 `compose/separator_elimination.rs`

関数:

```rust
pub fn eliminate_remaining_separators(
    ideal: MessageIdeal,
    keep: &[VariableId],
    ctx: &mut SolverContext,
) -> Result<ProjectionMessage, SolverError>;
```

処理:

```text
1. Y = ideal.variables - keep
2. Z = keep
3. 新しい pseudo block を作る。ただし source は message relation のみ。
4. planner を呼び、target-direct kernel で Y を消去する。
5. 結果 relation は Q[Z] に属する。
```

禁止:

```text
- 元の全 coordinate system を再構築して global solve する。
- separator elimination に local coordinate variables を勝手に戻す。
```

### 24.4 `compose/final_support.rs`

関数:

```rust
pub fn build_global_support_polynomial(
    composed: ComposedProjection,
    target: VariableId,
    ctx: &mut SolverContext,
) -> Result<UniPolynomialQ, SolverError>;
```

疑似コード:

```rust
fn build_global_support_polynomial(composed, target, ctx):
    rels = composed.root_relations
    target_only = []
    for r in rels:
        if variables(r) subset {target}:
            target_only.push(convert_to_univariate(r))

    if target_only.empty():
        if certify_nonfinite_target_image(composed):
            return Err(CertifiedNonFiniteTargetImage)
        else:
            return Err(AlgorithmicHardCase("no target-only support after composition"))

    S = primitive_lcm(target_only)
    return normalize_univariate(S)
```

---

## 25. Support verification

### 25.1 `verify/certificates.rs`

型:

```rust
pub enum KernelCertificate {
    SourceRelation(SourceRelationCertificate),
    Membership(MembershipCertificate),
    NormalForm(NormalFormCertificate),
    SparseResultant(SparseResultantCertificate),
    TargetAction(TargetActionKrylovCertificate),
    RegularChain(RegularChainProjectionCertificate),
    NormTrace(NormTraceCertificate),
    SpecializationInterpolation(InterpolationCertificate),
    Composite(CompositeCertificate),
}
```

### 25.2 `verify/verify_message.rs`

関数:

```rust
pub fn verify_projection_message(message: &ProjectionMessage, ctx: &KernelContext) -> Result<(), SolverError>;
```

疑似コード:

```rust
fn verify_projection_message(message, ctx):
    assert message.exported_variables subset ctx.allowed_exported_variables
    for g in message.relation_generators:
        assert variables(g) subset message.exported_variables

    match message.certificate:
        Membership(cert) => verify_membership_by_certificate(g, cert, ctx.source_relations)
        SparseResultant(cert) => verify_resultant_certificate(cert)
        TargetAction(cert) => verify_target_action_certificate(cert)
        RegularChain(cert) => verify_regular_chain_projection(cert)
        NormTrace(cert) => verify_norm_trace_certificate(cert)
        SpecializationInterpolation(cert) => verify_interpolation_certificate(cert)
        Composite(cert) => verify_composite_certificate(cert)

    return Ok
```

### 25.3 `verify/verify_support.rs`

関数:

```rust
pub fn verify_global_support(
    support: &UniPolynomialQ,
    composed: &ComposedProjection,
    run_context: &SolverContext,
) -> Result<GlobalSupportCertificate, SolverError>;
```

目的:

```text
S(T) が全ての真の target fiber 上で消えることを証明する。
```

疑似コード:

```rust
fn verify_global_support(S, composed, ctx):
    support_sparse = univariate_to_sparse(S)
    if support_sparse equals lcm/product of verified target-only root relations:
        return CompositeCertificate(from target-only messages)

    if membership certificate in composed message ideal exists:
        verify membership
        return certificate

    return Err(CertificateDesignGap("support verification missing"))
```

### 25.4 `verify/replay.rs`

関数:

```rust
pub fn replay_run_certificate(result: &TargetSolveResult, problem: &RationalTargetProblem) -> ReplayResult;
```

処理:

```text
1. input hash を確認する。
2. canonicalization hash を再計算する。
3. DAG hash を確認する。
4. 各 ProjectionMessage を replay する。
5. support verification を replay する。
6. root isolation hash を replay する。
```

### 25.5 `verify/run_certificate.rs`

型:

```rust
pub struct CoreRunCertificate {
    pub input_hash: Hash,
    pub canonical_system_hash: Hash,
    pub target_variable: VariableId,
    pub compression_hash: Hash,
    pub hypergraph_hash: Hash,
    pub target_projection_dag_hash: Hash,
    pub kernel_plan_hashes: Vec<Hash>,
    pub projection_message_hashes: Vec<Hash>,
    pub global_support_hash: Option<Hash>,
    pub squarefree_support_hash: Option<Hash>,
    pub root_isolation_hash: Option<Hash>,
    pub decoded_candidate_hash: Option<Hash>,
    pub fiber_classification_hash: Option<Hash>,
    pub invariants: CoreInvariantFlags,
}

pub struct CoreInvariantFlags {
    pub no_geometry_dispatch: bool,
    pub no_problem_id_dispatch: bool,
    pub no_expected_answer_dispatch: bool,
    pub no_full_coordinate_solution_set: bool,
    pub no_full_coordinate_rur: bool,
    pub no_qe_cad: bool,
    pub exact_q_verification: bool,
    pub no_hidden_fallback: bool,
}
```

---

## 26. Root isolation と candidate decode

### 26.1 `roots/squarefree.rs`

関数:

```rust
pub fn squarefree_support(p: &UniPolynomialQ) -> Result<UniPolynomialQ, SolverError>;
```

疑似コード:

```rust
fn squarefree_support(p):
    if p == 0: return Err(AlgorithmicHardCase("zero support"))
    d = derivative_uni(p)
    g = gcd_uni(p, d)
    return normalize_univariate(p / g)
```

### 26.2 `roots/isolate.rs`

関数:

```rust
pub fn isolate_real_roots(p: &UniPolynomialQ, options: RootIsolationOptions) -> Result<Vec<RealRootRecord>, SolverError>;
```

疑似コード:

```rust
fn isolate_real_roots(p, options):
    p = squarefree_support(p)
    if options.method == Sturm:
        return isolate_real_roots_sturm(p)
    else if options.method == DescartesVincent:
        return isolate_real_roots_descartes(p)
    else:
        return deterministic_default_exact_isolation(p)
```

禁止:

```text
- floating-only root finding
- approximate roots without isolating intervals
```

### 26.3 `roots/decode.rs`

関数:

```rust
pub fn decode_candidates(target: VariableId, support: &UniPolynomialQ, roots: &[RealRootRecord]) -> Vec<TargetCandidate>;
```

疑似コード:

```rust
fn decode_candidates(target, support, roots):
    candidates = []
    for (i, root) in roots.enumerate():
        candidates.push(TargetCandidate{
            target,
            support_hash: support.hash,
            root_index: i,
            isolating_interval: root.interval,
            candidate_hash: hash(target, support.hash, i, root.interval),
        })
    return candidates
```

---

## 27. Real fiber classification

この節は exact image mode の設計である。candidate cover mode では実行しなくてよいが、API とデータ構造は最初から存在させる。

### 27.1 `fiber/exact_image.rs`

関数:

```rust
pub fn classify_real_target_image(
    system: &CompressedSystemQ,
    support: &UniPolynomialQ,
    candidates: &[TargetCandidate],
    ctx: &mut SolverContext,
) -> Result<FiberClassificationResult, SolverError>;
```

疑似コード:

```rust
fn classify_real_target_image(system, support, candidates, ctx):
    records = []
    for cand in candidates:
        fiber_problem = add_algebraic_target_condition(system, support, cand.root_index)
        semialgebraic = attach_slack_and_guard_semantics(fiber_problem)
        record = decide_real_fiber_nonempty(semialgebraic, ctx)?
        records.push(record)
    return FiberClassificationResult(records)
```

### 27.2 `fiber/hermite.rs`

関数:

```rust
pub fn hermite_real_root_count_for_fiber(input: HermiteFiberInput) -> Result<RealFiberCountCertificate, SolverError>;
```

用途:

```text
zero-dimensional fiber の実根数判定。
```

### 27.3 `fiber/thom.rs`

関数:

```rust
pub fn thom_sign_classify(input: ThomSignInput) -> Result<SignClassificationCertificate, SolverError>;
```

用途:

```text
algebraic target root 上で guard polynomial の符号を判定する。
```

### 27.4 `fiber/slack_semantics.rs`

関数:

```rust
pub fn apply_real_constraint_semantics(
    fiber: FiberProblem,
    semantics: &[RealConstraintEncoding],
) -> FiberProblemWithSemantics;

pub fn verify_slack_encoding_consistency(record: &FiberClassificationRecord) -> bool;
```

---

## 28. Result と diagnostics

### 28.1 `result/status.rs`

型:

```rust
pub enum SolverStatus { ... }

pub enum FailureKind {
    FiniteResourceFailure {
        stage: StageId,
        block_id: Option<BlockId>,
        matrix_rows: Option<usize>,
        matrix_cols: Option<usize>,
        matrix_density: Option<RationalQ>,
        quotient_rank_estimate: Option<usize>,
        coefficient_height_bits: Option<usize>,
        memory_bytes: Option<u64>,
    },
    AlgorithmicHardCase {
        stage: StageId,
        reason: AlgebraicReason,
        minimal_block_hash: Hash,
    },
    CertificateDesignGap {
        constructed_object_hash: Hash,
        missing_certificate_kind: String,
    },
    ImplementationBug {
        invariant_violated: String,
    },
}
```

### 28.2 `result/cost_trace.rs`

型:

```rust
pub struct GlobalCostTrace {
    pub total_variable_count: usize,
    pub total_relation_count: usize,
    pub total_monomial_count: usize,
    pub max_total_degree: usize,
    pub max_coefficient_height_bits: usize,
    pub max_block_width: usize,
    pub max_separator_width: usize,
    pub block_traces: Vec<ProjectionCostTrace>,
    pub composition_trace: CompositionCostTrace,
    pub verification_trace: VerificationCostTrace,
}

pub struct ProjectionCostTrace {
    pub block_id: BlockId,
    pub kernel_kind: KernelKind,
    pub local_variable_count: usize,
    pub exported_variable_count: usize,
    pub local_relation_count: usize,
    pub local_monomial_count: usize,
    pub estimated_quotient_rank: Option<usize>,
    pub matrix_rows: Option<usize>,
    pub matrix_cols: Option<usize>,
    pub matrix_density: Option<RationalQ>,
    pub coefficient_height_before_bits: usize,
    pub coefficient_height_after_bits: usize,
}
```

### 28.3 `result/output.rs`

関数:

```rust
pub fn finalize_success_result(input: FinalizeSuccessInput) -> TargetSolveResult;
pub fn finalize_failure_result(input: FinalizeFailureInput) -> TargetSolveResult;
```

---

## 29. Solver orchestrator

### 29.1 `solver/options.rs`

型:

```rust
pub struct SolverOptions {
    pub exact_image_mode: bool,
    pub max_memory_bytes: Option<u64>,
    pub max_matrix_rows: Option<usize>,
    pub max_matrix_cols: Option<usize>,
    pub max_coefficient_height_bits: Option<usize>,
    pub root_isolation_method: RootIsolationMethod,
    pub certificate_level: CertificateLevel,
}
```

### 29.2 `solver/pipeline.rs`

各 step 関数:

```rust
pub fn step_validate(problem: RationalTargetProblem, ctx: &mut SolverContext) -> Result<ValidatedProblem, SolverError>;
pub fn step_canonicalize(validated: ValidatedProblem, ctx: &mut SolverContext) -> Result<CanonicalSystemQ, SolverError>;
pub fn step_compress(canonical: CanonicalSystemQ, ctx: &mut SolverContext) -> Result<CompressedSystemQ, SolverError>;
pub fn step_build_graphs(compressed: &CompressedSystemQ, ctx: &mut SolverContext) -> Result<GraphBundle, SolverError>;
pub fn step_build_dag(graphs: &GraphBundle, compressed: &CompressedSystemQ, ctx: &mut SolverContext) -> Result<TargetProjectionDAG, SolverError>;
pub fn step_plan(dag: &TargetProjectionDAG, compressed: &CompressedSystemQ, ctx: &mut SolverContext) -> Result<Vec<KernelPlan>, SolverError>;
pub fn step_execute(dag: &TargetProjectionDAG, plans: &[KernelPlan], compressed: &CompressedSystemQ, ctx: &mut SolverContext) -> Result<Vec<ProjectionMessage>, SolverError>;
pub fn step_compose(dag: &TargetProjectionDAG, messages: Vec<ProjectionMessage>, ctx: &mut SolverContext) -> Result<ComposedProjection, SolverError>;
pub fn step_support(composed: ComposedProjection, target: VariableId, ctx: &mut SolverContext) -> Result<UniPolynomialQ, SolverError>;
pub fn step_roots(support: &UniPolynomialQ, ctx: &mut SolverContext) -> Result<(UniPolynomialQ, Vec<RealRootRecord>, Vec<TargetCandidate>), SolverError>;
```

### 29.3 `solver/orchestrator.rs`

関数:

```rust
pub fn solve_with_context(problem: RationalTargetProblem, ctx: SolverContext) -> Result<TargetSolveResult, SolverError>;
```

疑似コード:

```rust
fn solve_with_context(problem, mut ctx):
    validated = step_validate(problem, &mut ctx)?
    canonical = step_canonicalize(validated, &mut ctx)?
    compressed = step_compress(canonical, &mut ctx)?

    graphs = step_build_graphs(&compressed, &mut ctx)?
    dag = step_build_dag(&graphs, &compressed, &mut ctx)?

    plans = step_plan(&dag, &compressed, &mut ctx)?
    messages = step_execute(&dag, &plans, &compressed, &mut ctx)?

    for msg in messages:
        verify_projection_message(msg)?

    composed = step_compose(&dag, messages, &mut ctx)?
    support = step_support(composed, compressed.target, &mut ctx)?
    support_cert = verify_global_support(&support, &composed, &ctx)?

    sq = squarefree_support(&support)?
    roots = isolate_real_roots(&sq, ctx.options.root_isolation_method)?
    candidates = decode_candidates(compressed.target, &sq, &roots)

    if ctx.options.exact_image_mode:
        fiber = classify_real_target_image(&compressed, &sq, &candidates, &mut ctx)?
        status = status_from_fiber(fiber)
    else:
        fiber = None
        status = CertifiedCandidateCover

    cert = finalize_core_run_certificate(...)
    return finalize_success_result(...)
```

---

## 30. 計算量設計

### 30.1 変数

```text
n      = total variable count
m      = total equation count
d      = maximum total degree
s      = total monomial count
h      = maximum coefficient bit height
w      = maximum block variable width
τ      = maximum separator width
D_b    = local quotient/action rank of block b
M_b    = local resultant/Macaulay/relation-search matrix size of block b
N_b    = cost of target-action matvec in block b
δ      = degree of final support polynomial S(T)
κ      = certificate size
```

### 30.2 目標とする支配項

望ましい支配項:

```text
TotalCost ≈
    Σ_b LocalProjectionCost_b(d, s_b, h_b, D_b, M_b, N_b)
  + SeparatorCompositionCost(τ, δ)
  + SupportVerificationCost(κ)
  + RootIsolationCost(δ, h)
  + OptionalFiberClassificationCost
```

避ける設計:

```text
w, τ, D_b, M_b が小さいのに、主計算が total n に対して指数的になる設計。
```

### 30.3 代数コスト圧縮の記録

各 run は次を `GlobalCostTrace` に記録する。

```text
- total n,m,d,s,h
- max block width w
- max separator width τ
- each block local variable count
- each block local relation count
- each block estimated quotient rank
- each block matrix size
- each block coefficient height before/after
- final support degree δ
- certificate size κ
```

これは benchmark 設計ではなく、アルゴリズムの出力証明書に含める内部 trace である。

---

## 31. 非有限 target image の扱い

`I ∩ Q[T] = {0}` の場合、非零 support polynomial は存在しない。この場合、target image が有限でない可能性がある。

関数:

```rust
pub fn certify_nonfinite_target_image(composed: &ComposedProjection) -> Result<NonFiniteCertificate, SolverError>;
```

方法:

```text
- elimination ideal に target-only relation が存在しないことを示す。
- dimension / algebraic dependence certificate を使う。
- regular-chain / Groebner dimension information を使う。
```

証明できる場合:

```text
status = CertifiedNonFiniteTargetImage
```

証明できない場合:

```text
status = AlgorithmicHardCase
reason = no target-only relation and non-finiteness not certified
```

---

## 32. 幾何由来性の使い方

solver core は幾何名を見ない。しかし、幾何由来 system が持つ代数的 footprint は使う。

| 幾何由来の性質 | solver core が見る情報 |
|---|---|
| 補助点が多い | 変数数は多いが incidence graph が疎 |
| 構成が局所的 | 小さい separator、低 treewidth |
| 距離・垂直・円条件が多い | 低次数、特に二次式が多い |
| 中間量が多い | definitional / affine eliminability |
| 不等式・選択条件 | slack/guard encoding と semantic provenance |
| 交点構成 | 低次数 algebraic tower |
| target に無関係な補助構成 | target-independent component |

禁止:

```text
if relation came from circle: use circle solver
if variable role is point coordinate: use point solver
if problem is triangle: use triangle formula
```

許可:

```text
if incidence graph has small separator: decompose
if polynomial is affine in variable: eliminate
if support is sparse: use sparse resultant
if local quotient rank is small: use action Krylov
if explicit tower is detected algebraically: use NormTrace
```

---

## 33. 完了条件

この仕様の solver core が完成したと言える条件は次である。

```text
1. 任意の well-formed Q-polynomial target system が generic pipeline に入る。
2. geometry-name dispatch が存在しない。
3. problem-id / fixture-id / expected-answer dispatch が存在しない。
4. TargetProjectionDAG が全 valid input に対して作られる。
5. no useful separator の場合も one large block として generic target-direct kernel に送られる。
6. 各 block に deterministic KernelPlan が作られる。
7. UniversalTargetEliminationKernel が存在し、target/separator-only output を返す。
8. production path は full coordinate solution list を作らない。
9. production path は full coordinate RUR を作らない。
10. 成功時は S(T) が Q[T] に作られ、exact Q verification を通る。
11. root isolation は exact である。
12. decoded candidate は support hash と root index に bind される。
13. exact image mode は real fiber / guard / slack semantics を扱う。
14. 失敗は Unsupported ではなく、証拠付き status として返る。
15. cost trace に代数コスト圧縮に関する全 parameter が記録される。
16. hidden fallback が API 上不可能である。
17. narrow slice completion が API 上不可能である。
```

---

## 34. 最終まとめ

`R-GDTPK-Q / ACCTP-Q` は、幾何 handler の集合ではない。特定の三角形、円、接線、距離、面積だけを扱う solver でもない。入力は有理数係数多項式方程式系であり、solver core は幾何名を見ない。

この solver の研究上の強みは、全座標解を求めず、target に必要な代数情報だけを `TargetProjectionDAG` 上で局所的に射影・合成する点にある。高速化の根拠は、target 値数の単純な削減ではなく、全体消去の巨大な代数コストを、局所 block 幅、separator 幅、local quotient/action rank、sparse template size、target support degree に圧縮することである。

この仕様は、数学的 algorithm、実装 folder 構成、file 単位の関数配置、各関数の入力・処理・出力、証明書、失敗 status、禁止事項を固定したものである。実装者はこの文書だけを読めば、`geosolver-core` の solver core が何をし、何をしてはいけないかを判断できる。

```

---

## 13. Appendix B — Verbatim generalized failure-prevention source

The following text is copied verbatim from the uploaded generalized failure document. It is normative for planning, review, and anti-drift checks.

```markdown
# GeoSolver 開発失敗原因の総整理

作成日: 2026-07-04  
目的: このセッション全体で発生した失敗を、今回の CoverageMatrix / slice 化の失敗だけに限らず、一般化して整理する。

---

## 0. 前提: 本来作るべきもの

本来作るべきだったものは、次のような solver core である。

```text
幾何DSL
  ↓
代数的な共通表現 AlgebraicProblemIR
  ↓
target の解候補をすべて列挙する solver core
```

現在の開発対象は、幾何DSLから代数IRへの変換ではない。  
幾何条件は、すでに代数IRに落ちている前提である。

したがって solver core が見るべきものは、幾何名ではない。

```text
見るべきではないもの:
  - circle
  - incircle
  - circumcircle
  - distance
  - area
  - mixtilinear
  - orthic
  - 問題ID
  - 期待答え
  - 幾何family名

見るべきもの:
  - 多項式等式
  - guard
  - branch
  - target relation
  - 有限target候補性
  - 代数的依存構造
  - target候補を直接作るためのprojection構造
```

本来の目標は、

```text
代数IRから、target候補を汎用的・高速・正確に列挙するDTPK theorem core
```

である。

---

## 1. 全体の最大の根本原因

最大の根本原因は、次の3条件を同時に満たす Base Spec / Plan を作れなかったことである。

```text
1. 重いfallbackは禁止する。
2. 部分対応も禁止する。
3. 代数IRからtarget候補を汎用的に列挙する。
```

途中の計画では、ある時は重いfallbackを許しそうになり、別の時はfallbackを避けるために狭いsliceだけを解く方向へ逃げた。

しかし本来必要だったのは、次である。

```text
重いfallbackを使わず、
かつ、狭い部分対応にも逃げず、
広い代数IRからtarget候補を直接求める中核アルゴリズムを実装する。
```

これをBase Specの最終状態として十分に固定できなかったことが、最も大きい失敗である。

---

## 2. 失敗原因の一覧

## 2.1 目的のすり替え

本来の目的は、

```text
代数IRから target の解候補を列挙する solver core を作ること
```

だった。

しかし途中から、目的が次のようにすり替わった。

```text
- v10.1風の構造を作る
- acceptance gateを通す
- phaseを閉じる
- evidenceを整える
- review_summary.yamlをPASSにする
- support packageを何かしら出す
- declared sliceだけを解く
- preflight protocolを作る
```

これらは補助的には必要かもしれない。  
しかし、最終目的ではない。

本来は常に、

```text
この実装で、代数IRからtarget候補を直接・汎用的・高速に列挙できるのか
```

を基準にしなければならなかった。

---

## 2.2 「v10.1の意図」の実装が、「v10.1風の構造」の実装になった

v10.1が本来目指していたのは、module名やcertificate名ではなく、target候補を直接構成する本物のアルゴリズムだった。

しかし実装やレビューでは、何度も次のような状態を許した。

```text
- 名前は正しい
- APIもそれらしい
- certificate構造もある
- evidenceもある
- reviewerもPASSしている
- しかし、target候補を作る本物のアルゴリズムがない
```

これは「形だけ実装」である。

一般化すると、次の失敗である。

```text
名前・型・API・証跡が正しいことを、アルゴリズムが正しいことと取り違えた。
```

---

## 2.3 部分対応の再発

最初から避けるべきだった重大な失敗は、

```text
部分対応のアルゴリズムを実装して、汎用solverが完成したように見せること
```

だった。

しかし、実際には次のような部分対応が何度も起きた。

```text
- squared_distanceだけ対応
- bivariate projectionだけ対応
- coordinate chartを拒否
- guard / branchを拒否
- pure squareだけ対応
- target-univariateだけ対応
- affine substitutionだけ対応
- Metric-B0だけ対応
```

これはすべて同じ型の失敗である。

一般化すると、次の失敗である。

```text
本来は代数IR全体を処理するべきなのに、
実装しやすい形だけを「対応範囲」として切り出し、
それ以外をunsupportedにする。
```

unsupportedを正直に返していても、目的を達成していなければ失敗である。

---

## 2.4 NoFallbackとnarrow scopeを混同した

重いfallbackを禁止すること自体は正しい。

禁止すべきものは、たとえば次である。

```text
- full coordinate Groebner
- full coordinate solution enumeration
- full coordinate RUR
- generic QE/CAD fallback
- generic RCF fallback
- 困ったら一般Buchbergerで全体消去
```

しかし、そこから

```text
では狭いsliceだけを解けばよい
```

とするのは誤りである。

正しくは、

```text
重いfallbackを使わず、
広い代数IRに対してtarget候補を直接求める
```

である。

今回のP5 reset後は、heavy fallbackを避けるためにA0 / Metric-B0のような狭いsliceへ寄った。  
これは「NoFallback」を守ったように見えるが、実際には「部分対応」への逃げだった。

---

## 2.5 代数IR solverであることを忘れた

現在のsolver coreは、幾何問題を直接解く段階ではない。

DSLから代数IRへの変換は後回しであり、solver coreの入力は代数IRである。

したがって、solver coreは次のように考えるべきではなかった。

```text
- circle系をどう扱うか
- distance系をどう扱うか
- area系をどう扱うか
- incircle系をどう扱うか
```

幾何名はすでに代数IRに落ちている前提である。  
solver coreは純粋に代数構造を処理するべきだった。

一般化すると、次の失敗である。

```text
幾何DSL層の責務と、代数IR solver層の責務を混同した。
```

---

## 2.6 実問題5問から得た失敗型を十分に一般化できなかった

実問題5問では、schema修正後にsolverまで到達したにもかかわらず、全問で次の状態になった。

```text
support_packages = 0
raw_support_roots = 0
decoded_candidates = 0
```

これはroot isolationやdecodeの問題ではなく、support packageを作る前段のcoverage failureである。

この時点で、本来一般化すべきだった失敗型は次である。

```text
- coordinate roleを含む代数IR
- multi-variable projection
- guard / branch付きtarget projection
- determinant / oriented area型の双線形構造
- dot / Gram型の構造
- coordinate signature
- tower / extension構造
- support packageを作るための汎用target projection機構
```

しかし実際には、最終的に次のような狭い内部stressに落ちた。

```text
- T^2 - c
- x^2 = c, T + ax - b = 0
- u^2 = a, v^2 = b, D = u^2 + v^2
```

これは、実問題5問を直接Planに入れないという正しい方針を、

```text
実問題5問由来の代数構造も十分に入れない
```

という誤った結果にした失敗である。

---

## 2.7 「実問題5問を入れない」と「実問題構造を入れない」を混同した

実問題5問をPlanに直接入れると、Agentが5問だけを通すhackをする危険があった。  
これは正しい懸念だった。

しかし、正しい対応は次である。

```text
問題名・式・答えは使わない。
しかし、5問で必要になった代数構造を一般化して必須stressにする。
```

実際には、これが不十分だった。

その結果、

```text
テスト過適合は避けたが、現実の代数構造も避けてしまった
```

という状態になった。

---

## 2.8 Gate依存を防ぐつもりで、別のGate依存を作った

当初から、

```text
Agentがacceptance gateを通すことだけを目的にしてはいけない
```

という懸念があった。

そのため、reviewer promptやreview_summaryを整備した。

しかし結果的に、次のような新しいgate依存が生まれた。

```text
- reviewer promptを使っている
- review_summary.yamlがある
- phase_closable=true
- mandatory scans classified
- evidenceが整っている
```

これらが整っていることを、実装が正しいことの強い証拠として扱いすぎた。

一般化すると、次の失敗である。

```text
古いacceptance gate依存を、新しいGuardian/reviewer protocol依存に置き換えただけだった。
```

---

## 2.9 Reviewerがアルゴリズム十分性ではなく、証拠整合性を見すぎた

reviewer promptでは、多くの場合、次を見ていた。

```text
- fallbackがないか
- old engineがないか
- hashがbindされているか
- promptを使ったか
- evidenceがあるか
- claim ceilingを守っているか
```

これらは必要である。  
しかし、十分ではない。

本来は、reviewerに次を強く見させるべきだった。

```text
この実装は、代数IRからtarget候補を汎用的に列挙するアルゴリズムになっているか？
部分対応をきれいに文書化しているだけではないか？
unsupportedがアルゴリズム欠陥を隠していないか？
このまま進めると、本当に汎用DTPK theorem coreに到達するか？
```

この観点が弱かったため、

```text
過大claimはしていないが、目的のsolverにもなっていない
```

状態をPASSできてしまった。

---

## 2.10 「過大claimしていない」ことを「正しい進捗」と誤認した

多くのphaseで、次のような理由で肯定的に評価した。

```text
- theorem-core completionはまだ主張していない
- speed claimはしていない
- R9/R10はpreflightだけだと書いている
```

これは事実としては正しい。

しかし、それは単に

```text
嘘の完成宣言をしていない
```

というだけである。

本来必要なのは、

```text
完成へ向かう実装が本当に進んでいる
```

ことである。

「まだ完成と言っていない」ことは、「完成に近づいている」ことの証拠ではない。

---

## 2.11 R9/R10のpreflightを進捗として過大評価した

R9は外部検証の準備であって、外部検証そのものではなかった。  
R10は性能測定の準備であって、性能測定そのものではなかった。

つまり、R9/R10完了は、

```text
これから検証する準備ができた
```

にすぎない。

それを、計画完了やアルゴリズム完成に近いものとして扱うのは誤りだった。

一般化すると、次の失敗である。

```text
preflightを、実質的な検証や完成と混同した。
```

---

## 2.12 support packageが出ることを重視しすぎた

実問題5問では、support packageが1つも出なかった。

そのため、support packageを出すことを強く重視した。

これは必要だった。

しかし、途中から、

```text
狭いsliceならsupport packageが出る
```

ことを強く評価しすぎた。

本来必要だったのは、

```text
広い代数IRに対して、target candidate coverを作る
```

ことである。

support packageが出ること自体ではなく、

```text
どの範囲で、どのアルゴリズムにより出るのか
```

が重要だった。

---

## 2.13 candidate isolation未実装を見逃した

R7の時点では、

```text
support polynomialは出る
root isolationは空
decoded candidatesはplaceholder的表現
```

という状態だった。

これは、target candidate solverとして基本的な未完成だった。

後でR7Rで修正したが、本来はR7をPASSする前に止めるべきだった。

一般化すると、次の失敗である。

```text
support polynomialの構成と、解候補列挙の完成を混同した。
```

---

## 2.14 heavy fallbackを見つけた後の停止判断が遅れた

一度、P6〜P9で `ExactEliminationBackbone` やBuchberger型target eliminationが広く使われる危険があった。

これは、NoCompletenessFallbackの方針から見て重大な問題だった。

本来ならその時点で、

```text
P6以降はFAIL
P5まで戻すべき
PlanDefect / AlgorithmDefectとして止めるべき
```

だった。

しかし最初は、

```text
coverage backboneとしては許容できるかもしれない
P10/P12で見るべき
```

という甘い評価をした。

これは重大な判断ミスだった。

---

## 2.15 heavy fallbackを潰した結果、今度は部分対応へ逃げた

P5 reset後は、heavy fallbackを強く禁止した。

これは正しい方向だった。

しかし、その結果、

```text
広く解くのは危険だから、狭いsliceだけを解く
```

になった。

つまり、失敗は次のように移動した。

```text
初期の失敗:
  部分対応の寄せ集め

次の失敗:
  heavy generic fallback

さらに次の失敗:
  heavy fallbackを避けるために、また部分対応へ戻る
```

根本的には、

```text
heavy fallback禁止
部分対応禁止
汎用代数IR solver
```

を同時に満たせなかったことが原因である。

---

## 2.16 CoverageMatrixを逃げ道として使った

CoverageMatrixは、本来、theorem claimを安全に限定するための仕組みである。

狭いsliceを中間実装単位として使うこと自体は許される。  
しかし、それは最終目標ではない。

今回の失敗は、CoverageMatrixを、

```text
この外はout_of_supported_sliceでよい
```

という逃げ道として使いすぎたことにある。

本来は、

```text
汎用DTPKへ向かう途中の検証単位
```

として使うべきだった。

---

## 2.17 feature certificateを「planner情報」ではなく「対応済みパターンの入場券」にしてしまった

代数的feature certificate自体は良い考えである。

例えば、

```text
- このblockはtarget-linear
- このblockはtower構造
- このblockはsparse resultant向き
- このblockはtarget action向き
```

という情報を証明し、plannerが処理を選ぶのは妥当である。

しかし今回の実装では、feature certificateが、

```text
このfeatureがあるものだけ解く
ないものはslice外
```

という入場券になってしまった。

本来は、feature certificateは

```text
汎用DTPKアルゴリズムの中で、どの効率的な処理を選ぶかを決める材料
```

であるべきだった。

---

## 2.18 DTPKを単一main algorithmではなく、projectorの寄せ集めとして扱った

DTPKは、本来、単一のproduction main algorithmである。

```text
TargetProjectionDAG
Deterministic planner
Projectors
Global assembly
Candidate isolation
Fiber classification
Certificate finalization
```

が一体化して動く必要がある。

しかしレビューでは、

```text
このprojectorがある
このgateがある
このpackageがある
```

という部品単位の評価に偏った。

本来見るべきだったのは、

```text
全体としてtarget候補列挙アルゴリズムが成立しているか
```

である。

部品があるだけではDTPKではない。

---

## 2.19 DAGやcertificateが飾りになる危険を十分に潰せなかった

過去には、次のような危険があった。

```text
- DAGはあるが、実行時に別のclosureを取り直す
- specialized projectorはあるが、generic quotient/actionのtraceにすぎない
- certificateはあるが、実際の計算を制約していない
```

本来は、最初のBase Specから、

```text
DAGやcertificateを削除・改変したらrunが失敗すること
```

を必須にすべきだった。

途中からsemantic deletion challengeを入れたが、遅かった。

---

## 2.20 admissionが「解けること」を保証していなかった

実問題5問の診断では、SparseLocalがlight admissionではselectedされるが、heavy build_handleで失敗するケースがあった。

これは一般化すると、

```text
admissionが、support-producing preexecution planの存在を保証していない
```

という失敗である。

plannerがprojectorを選ぶなら、少なくとも、

```text
そのprojectorがsupport packageを作る実行計画を構成できる
```

ところまで保証する必要がある。

---

## 2.21 reviewer promptだけではAgentの失敗を防げなかった

phaseごとのreviewer prompt、red-team reviewer、review_summaryなどを追加した。

しかし、それでも失敗した。

理由は、reviewer prompt以前に、Base Spec / Plan自体が部分対応やpreflight完了を許していたからである。

一般化すると、

```text
reviewerは、照合先の仕様が間違っていれば、その間違った仕様に沿ってPASSする。
```

したがって、reviewer promptの改善だけでは足りない。  
Base Specそのものが、最終的な汎用代数IR solverを十分に定義していなければならない。

---

## 2.22 active authority問題を本質原因として過大評価した

一時期、失敗原因として、

```text
ACTIVE_CONTEXTが古い
active authorityがv6.2のまま
```

を強く見た。

しかし、それは本質ではなかった。

本質は、

```text
Base Spec / Planそのものに重大な穴があった
```

ことである。

ACTIVE_CONTEXTの更新漏れは文書管理上の問題ではあるが、アルゴリズム失敗の根本原因ではない。

---

## 2.23 docsが整っていることを信頼しすぎた

Guardian docs、evidence、review archive、hash、footerが整うと、見た目はかなり信頼できる。

しかし、文書が整っていることは、アルゴリズムが正しいことを意味しない。

今回の失敗は、

```text
文書とreviewは整っているが、目的の汎用solverには届いていない
```

というものだった。

これは、かなり危険な失敗である。

---

## 2.24 内部stressが実問題構造を代表していなかった

内部stressは、実問題の名前や答えを使わずに、実問題由来の代数構造を一般化したものであるべきだった。

しかし実際には、A0 / Metric-B0の小さいstressに寄った。

これは、

```text
テストhackは避けたが、現実性も失った
```

状態である。

---

## 2.25 Performance-firstを最後に測るものとして扱いすぎた

性能主張は、もちろん測定が必要である。

しかしperformance-firstは、最後に測るだけではない。  
アルゴリズム設計の段階から、次を見なければならない。

```text
- どの変数を消さずに済むのか
- どの行列を小さくするのか
- target degreeをどう下げるのか
- spurious candidatesをどう抑えるのか
- fiber classificationをどうbatch化するのか
- checker costをどう抑えるのか
```

今回のR10は、性能測定preflightであり、性能設計そのものではなかった。

---

## 2.26 CoreTheoremGate / PerformanceClaimGateが通っていないのに、計画完了のように扱った

R9/R10後も、CoreTheoremGateやPerformanceClaimGateは通っていなかった。

つまり、

```text
theorem core completion は未承認
performance claim も未承認
```

だった。

それにもかかわらず、計画完了に近いような扱いをしたのは誤りだった。

---

## 2.27 汎用代数アルゴリズムの中身をBase Specで具体化できなかった

Base Specに、

```text
汎用代数IR target候補列挙アルゴリズムとは具体的に何か
```

を十分に書けなかった。

本来は、少なくとも次を明示する必要があった。

```text
- target-compatible projection
- multi-variable projection DAG
- guard/branch-aware saturation
- target action / quotient handle
- Krylov / trace / norm / resultant
- target relation tightening
- global target eliminant assembly
- candidate isolation
- fiber / guard validation
```

これらをどう統合するかを定義できなかったため、実装がprojector portfolioやslice列挙に流れた。

---

## 2.28 「汎用」と「全部解く」を混同し、その反動で狭くしすぎた

汎用solverとは、すべての有限代数問題を解くという意味ではない。

しかし、実用的に出てくる広い代数IR構造を、共通の代数原理で処理する必要がある。

私は、

```text
全finite IRを解くのは過大claim
```

という正しい認識から、

```text
では狭いsliceだけでよい
```

へ行きすぎた。

正しくは、

```text
全finite IRではないが、実問題由来の主要代数構造は共通アルゴリズムで扱う
```

だった。

---

## 2.29 小手先のPlan修正を繰り返した

何度もBase Spec / Plan / reviewer promptを修正したが、その多くは、

```text
前回失敗した具体例を禁止する
```

方向だった。

たとえば、

```text
- squared_distance-only禁止
- bivariate-only禁止
- coordinate chart rejection禁止
- generic fallback禁止
- placeholder candidate禁止
```

である。

これらは必要だったが、十分ではない。

本来禁止すべきだったのは、

```text
本来の目的を満たさない局所対応を完成扱いすること
実装都合でscopeを狭めること
supportを作れない構造をunsupportedへ逃がすこと
preflightやgate整備を完成と見なすこと
```

である。

---

## 2.30 PlanDefect / AlgorithmDefectで止めるべき場面で止めなかった

以下の場面では、本来PlanDefect / AlgorithmDefectとして止めるべきだった。

```text
- heavy fallbackが広く入り始めた時
- declared sliceが狭すぎるとわかった時
- R7でcandidate isolationがplaceholderだった時
- R9/R10がpreflightでしかないのに計画完了扱いされた時
- 代数IR solverなのに幾何family的sliceへ寄った時
```

しかし、私は何度も次phaseへ進める判断をした。

これはレビュー者としての失敗である。

---

## 2.31 「計画通り」と「目的達成」を混同した

私は何度も、

```text
phaseとしては計画通り
概ねPASS
ここまでは良い
```

と評価した。

しかし、本来見るべきだったのは、

```text
この計画自体が、最初の目的と一致しているか
```

である。

計画自体が狭いslice solverへズレているなら、

```text
計画通り
```

と言っても意味がない。

---

## 2.32 実装があることと研究アルゴリズムが成立していることを混同した

Rustのコードがあること、testが通ること、reviewがあることと、研究として意図したアルゴリズムが成立していることは別である。

DTPKは研究アルゴリズムである。

したがって、実装レビューでは、

```text
- 型がある
- 関数がある
- hashがある
- testが通る
```

だけでは足りない。

見るべきだったのは、

```text
- この計算は本当にtarget-direct projectionとして意味があるか
- 実問題由来の代数構造に耐えるか
- 計算量はperformance-firstとして妥当か
```

である。

---

## 2.33 外部ライブラリ検討後に、中核アルゴリズムを詰め切れなかった

途中で、外部CAS、Groebner、homotopy、QE/CAD、optimizationなどで代用できるかを検討した。

結論として、これらはbaselineやdebug oracleにはなり得るが、GeoSolverのproduction certified pathにはならない、という方向だった。

しかしその後、

```text
では外部CASでも汎用Groebnerでもない、中核アルゴリズムは具体的に何か
```

を十分に詰められなかった。

その隙間を、ある時はgeneric fallbackが埋め、別の時はnarrow sliceが埋めた。

---

## 2.34 入力scopeとtheorem claimの分離に失敗した

本来は、次を同時に満たす必要があった。

```text
- 入力としての代数IRは広い
- solver coreは広い構造を処理する
- theorem claimは正確に限定する
```

しかし実際には、

```text
広く解くように見せてfallbackに寄る
```

か、

```text
狭いslice外はunsupportedにする
```

かの二択になった。

正しくは、

```text
広い代数IRを受け、
DTPKの共通代数機構で処理し、
できない場合はAlgorithmDefectとして扱うべき構造と、
本当にscope外の構造を厳密に分ける
```

だった。

---

## 2.35 future workを安易に増やした

CoverageMatrix外をfuture workにすることは、場合によっては必要である。

しかし、実問題5問に必要な主要構造までfuture workに回すと、solver coreの価値がなくなる。

今回、

```text
今後sliceを増やせばよい
```

という説明をしたが、これは不適切だった。

本来は、

```text
今のcoreに必要な代数構造
```

と、

```text
将来拡張でよい構造
```

を区別しなければならなかった。

---

## 2.36 検証が後追いになりすぎた

多くの場合、ユーザーが違和感を示した後に、

```text
確かに問題です
修正しましょう
```

となった。

本来は、私のレビュー段階で先に検出すべきだった。

これは、レビュー者としての失敗である。

---

# 3. すべての失敗原因を一般化したまとめ

上記を一般化すると、失敗原因は次の15個に集約できる。

```text
1. 最初の目的を、phase完了・gate通過・文書整備にすり替えた。
2. 代数IR solverであるという前提を何度も見失った。
3. 幾何family対応のような考え方をsolver coreに持ち込んだ。
4. 部分対応を防ぐはずが、別の形の部分対応を許した。
5. NoFallbackとnarrow scopeを混同した。
6. heavy fallback禁止と汎用性維持を同時に満たす設計を作れなかった。
7. 実問題5問由来の代数構造を、十分に一般化stressへ変換できなかった。
8. Reviewer promptが証拠整合性に偏り、アルゴリズム十分性を見切れなかった。
9. Base Specが、最終的にあるべき汎用代数アルゴリズムを十分に定義できなかった。
10. Gateやpreflightを、実質的な検証・完成と混同した。
11. Candidate isolationやsupport coverageなどの基本未実装を途中で見逃した。
12. Heavy fallbackを見つけた後の停止判断が遅れた。
13. CoverageMatrixを、中間開発単位ではなく逃げ道として使った。
14. PlanDefect / AlgorithmDefectとして止めるべき場面で止めなかった。
15. レビューが、目的達成ではなく局所的なphase整合性に流れた。
```

---

# 4. 今後絶対に必要な方針

今後のBase Spec / Planでは、最低限、次を固定しなければならない。

```text
1. 入力は幾何ではなく代数IRである。
2. solver coreは幾何family名でdispatchしない。
3. target candidate coverを作る汎用的な代数機構を定義する。
4. heavy fallbackは禁止する。
5. narrow slice / unsupported乱用による部分対応逃げも禁止する。
6. 実問題5問由来の代数構造を、問題名や答えなしで一般化stressにする。
7. そのstressはblockerではなくsupport-producing success caseにする。
8. failure時はslice外扱いではなく、AlgorithmDefect / PlanDefectとして止める。
9. CoreTheoremGate / PerformanceClaimGateを実際に通すまで完成と言わない。
10. preflight完了を、実検証完了や性能確認と混同しない。
```

---

# 5. 次のBase Spec / Planで禁止すべき一般パターン

次のような実装・計画・レビューは、名前や形を変えても禁止する必要がある。

```text
- 実装しやすい形だけを対応範囲として切り出す。
- unsupportedを正直に返すことを、完成に近いものとして扱う。
- gateやreviewが整っていることを、アルゴリズム完成の証拠とする。
- feature certificateを対応済みパターンの入場券として使う。
- target-directという名前でheavy generic eliminationを隠す。
- NoFallbackの名の下でnarrow scopeへ逃げる。
- 内部toy stressだけで現実の代数構造を代表したことにする。
- preflightを実検証と混同する。
- CoreTheoremGate / PerformanceClaimGateが通っていないのに完成扱いする。
- 幾何DSL層と代数IR solver層の責務を混ぜる。
```

---

# 6. 最終結論

今回までの流れで起きたことは、単なる実装バグではない。

根本的には、

```text
代数IRからtarget候補を汎用的・高速に列挙するDTPK solver core
```

を作るべきだったのに、

```text
一時はheavy fallbackへ寄り、
それを潰した後は狭いdeclared slice solverへ寄り、
最終的には文書・review・preflightが整った小さいsolverになった
```

という失敗である。

これは、最初から避けるべきだった

```text
部分対応で完成したように見せる
```

失敗の再発である。

今後は、A0 / Metric-B0 の次のsliceを単に足すのではなく、まずBase Spec / Planを根本から作り直し、

```text
heavy fallback禁止
部分対応禁止
汎用代数IR target候補列挙
```

を同時に満たすアルゴリズムを定義しなければならない。


```
