# 01. Base Spec: CW-ARC-DTP-Q Full Implementation v3

## 0. Authority and scope

This Base Spec defines the final required state of the repository implementing CW-ARC-DTP-Q in `geosolver-core`.

The implementation MUST implement the full algorithmic contract described here. It MUST NOT implement a simplified candidate-cover-only subset and call it complete. It MUST NOT scope out guard handling, certified compression, complete target elimination, or exact target image without an explicit user-approved Base Spec amendment.

The original mathematical authority is the CW-ARC-DTP-Q revised specification v2. This Base Spec narrows implementation ambiguity. If this Base Spec and the source specification appear to conflict, the Agent MUST stop and create QuestionDebt. It MUST NOT choose the easier interpretation.

## 1. Global completion definition

The repository is complete only if all of the following are true.

```text
G1. `solve_target(problem, options)` accepts every well-formed rational target polynomial system.
G2. No production path dispatches on geometry names, fixture names, problem IDs, expected answers, or variable names except by exact structural equality of variables.
G3. Candidate generation and candidate adoption are separated.
G4. Every `CertifiedCandidateCover` has a `SolverCertificate::TargetCover` that `verify_certificate` verifies from the original problem without solver trace.
G5. Every guard used in `GuardedRadicalMembership`, guarded empty proof, or saturation has a valid `GuardCertificate`.
G6. `CertifiedSystemQ` is produced by replayable, verifier-checkable compression steps. Input semantic nonzero guards are not dropped.
G7. Residual / Krylov / resultant / slice / tower outputs are never adopted without fixed exact proof or route-specific exact certificate allowed by this spec.
G8. Unbounded proof search is fair over multiplier support degree, support power, and guard power.
G9. `CompleteTargetEliminationFallback` computes the exact target elimination of `(I : D^∞) ∩ Q[T]` or returns resource failure. It is not a bounded relation search.
G10. `CertifiedNoNonzeroTargetEliminant` is returned only with a verifier-checkable exact elimination-zero certificate.
G11. `CertifiedEmptyAdmissibleSet` is returned only with an exact empty certificate and is never encoded as support `1`.
G12. `CertifiedExactTargetImage` is returned only if all real roots of `squarefree_support` are classified by exact real fiber certificates.
G13. `ExactImageMode::RequireExactImage` has production examples where it returns `CertifiedExactTargetImage` and examples where it fail-closes when resource limits prevent classification.
G14. Every route has a route-forcing test where all other routes and complete fallback are disabled.
G15. Every route has tamper tests showing certificate replay fails after meaningful mutation.
G16. No final production function contains placeholder semantics such as always-incomplete classifier, first-prime-only reconstruction for multi-prime candidate reconstruction, bounded search named complete, or evidence-only certificate validation.
```

## 2. Mandatory repository structure

The repository MUST contain the following files. Additional internal helper files are allowed only if the public contracts below remain unchanged.

```text
Cargo.toml
src/lib.rs
src/arith.rs
src/variable.rs
src/monomial.rs
src/polynomial.rs
src/univariate.rs
src/finite_field.rs
src/matrix_q.rs
src/matrix_fp.rs
src/crt.rs
src/rational_reconstruction.rs
src/problem.rs
src/compression.rs
src/guards.rs
src/certificates.rs
src/verifier.rs
src/window.rs
src/residual.rs
src/candidates/mod.rs
src/candidates/direct.rs
src/candidates/residual_cyclic.rs
src/candidates/krylov.rs
src/candidates/sparse_resultant.rs
src/candidates/slice.rs
src/candidates/norm_trace_tower.rs
src/candidates/localized_schur.rs
src/proof/mod.rs
src/proof/fixed.rs
src/proof/schedule.rs
src/proof/learning.rs
src/proof/multiple_repair.rs
src/elimination/mod.rs
src/elimination/groebner.rs
src/elimination/saturation.rs
src/elimination/target_elimination.rs
src/real/mod.rs
src/real/algebraic.rs
src/real/sturm.rs
src/real/cad.rs
src/real/fiber.rs
src/solver.rs
src/trace.rs
tests/route_forcing_*.rs
tests/certificate_tamper_*.rs
tests/exact_image_*.rs
tests/complete_fallback_*.rs
tests/guard_and_compression_*.rs
docs/ai/changes/cw-arc-dtp-q/BASE_SPEC.md
docs/ai/changes/cw-arc-dtp-q/PLAN.md
docs/ai/changes/cw-arc-dtp-q/REVIEWER_PROMPTS.md
docs/ai/changes/cw-arc-dtp-q/evidence/non_simplification_manifest.md
docs/ai/changes/cw-arc-dtp-q/evidence/route_forcing_matrix.md
docs/ai/changes/cw-arc-dtp-q/evidence/data_flow_proofs.md
```

### 2.1 Naming rules

The code MUST follow these rules.

```text
- Rust modules, functions, and methods: snake_case.
- Rust types, traits, enum variants: UpperCamelCase.
- Constants: SCREAMING_SNAKE_CASE.
- Conversion methods:
  - `as_*` returns a borrowed or cheap view.
  - `to_*` clones or computes a converted value.
  - `into_*` consumes self.
- Do not use names containing plan versions, phase labels, temporary labels, or implementation excuses.
- Forbidden names in production API: `v2`, `v3`, `phase`, `hack`, `toy`, `simple`, `fake`, `stub`, `baka`, `debug_only`, `experimental_complete`.
- A bounded search MUST be named `try_*`, `bounded_*`, or `search_*`. It MUST NOT be named `complete_*`.
```

## 3. Data model

### 3.1 `TargetProblemQ`

File: `src/problem.rs`

```rust
pub struct TargetProblemQ {
    pub equations: Vec<PolynomialQ>,
    pub variables: Vec<Variable>,
    pub target: Variable,
    pub semantic_guards: Vec<GuardRecord>,
}

pub struct GuardRecord {
    pub polynomial: PolynomialQ,
    pub kind: GuardKind,
    pub provenance: GuardProvenance,
}

pub enum GuardKind {
    NonZero,
    Positive,
    Negative,
    NonNegative,
    NonPositive,
    OtherSemanticCondition,
}
```

Required invariants:

```text
P1. `target` must appear exactly once in `variables`.
P2. Every equation and guard polynomial must use exactly `variables` in the same order.
P3. `semantic_guards` are not candidates and are not proof by themselves.
P4. `GuardKind::NonZero` may become `GuardCertificate::InputSemanticNonzero` only through certified compression/guard construction.
P5. Positive/negative/nonnegative/nonpositive guards may be used for real fiber classification; they may be used as nonzero guard factors only after an explicit certificate deriving nonzero.
```

### 3.2 `CertifiedSystemQ`

File: `src/compression.rs`

```rust
pub struct CertifiedSystemQ {
    pub equations: Vec<PolynomialQ>,
    pub variables: Vec<Variable>,
    pub target: Variable,
    pub guard_certificates: Vec<GuardCertificate>,
    pub replay: CompressionReplayCertificate,
}
```

`certified_system_from_problem(problem)` MUST NOT merely clone the equations and set empty guard/replay in all cases. It MUST perform the following deterministic process:

```text
C1. Validate problem; reject invalid inputs with `InvalidInput`.
C2. Canonicalize polynomial terms and primitive-normalize equations.
C3. Remove zero equations with `ZeroEquationRemoval` replay steps.
C4. For each `GuardKind::NonZero` semantic guard, create an `InputSemanticNonzero` guard certificate and put it in `guard_certificates`.
C5. Optionally perform definition substitution, affine elimination, explicit guard saturation, and primitive normalization, but only with replay steps.
C6. Return `CertifiedSystemQ` whose replay can be verified from the original `TargetProblemQ`.
```

Allowed compression steps:

```rust
pub enum CompressionStepCertificate {
    IdentityInput,
    DefinitionSubstitution { variable: Variable, expression: PolynomialQ, identity: ExactIdentity },
    AffineElimination { eliminated: Variable, pivot: PolynomialQ, pivot_guard: GuardCertificate, identity: ExactIdentity },
    ExplicitGuardSaturation { guard: GuardCertificate, identity: ExactIdentity },
    PrimitiveNormalization { before: PolynomialQ, after: PolynomialQ, multiplier: Rational },
    ZeroEquationRemoval { removed: PolynomialQ },
}
```

Required verifier behavior:

```text
- `verify_compression_replay(problem, certified)` must replay each step from the original problem.
- It must verify that every equation in `certified.equations` is explained by replay.
- It must verify that every guard certificate in `certified.guard_certificates` is valid for the original problem.
- Empty replay is allowed only if no equation was changed and no semantic guard exists. If semantic nonzero guards exist, replay must still contain guard construction evidence or guard certificates must be directly verifiable against the original problem.
```

Forbidden simplifications:

```text
FAIL if production `CertifiedSystemQ` creation always uses `guard_certificates: Vec::new()`.
FAIL if production `CertifiedSystemQ` creation always uses `CompressionReplayCertificate { steps: Vec::new() }` while canonicalization or guard transfer happened.
FAIL if guarded proof code constructs `TargetProblemQ { semantic_guards: Vec::new() }` for guard verification.
FAIL if semantic guards are ignored by exact image classification.
```

## 4. Certificates and verifier

Files:

```text
src/guards.rs
src/certificates.rs
src/verifier.rs
```

### 4.1 Guard certificates

```rust
pub enum GuardCertificate {
    InputSemanticNonzero { guard: PolynomialQ, record: GuardRecord },
    AlgebraicNonvanishing { guard: PolynomialQ, certificate: NullstellensatzCertificate },
    RealAdmissibleNonvanishing { guard: PolynomialQ, certificate: RealInfeasibilityCertificate },
    DerivedProduct { product: PolynomialQ, factors: Vec<GuardCertificate>, identity: ExactIdentity },
}
```

Verifier MUST check:

```text
G-CERT-1. InputSemanticNonzero: original problem contains an exactly equal `GuardRecord` with `GuardKind::NonZero`.
G-CERT-2. AlgebraicNonvanishing: verify `1 = Σ q_i F_i + h * guard` exactly over Q.
G-CERT-3. RealAdmissibleNonvanishing: verify real infeasibility of `F=0 ∧ semantic_guards ∧ guard=0` using the certificate payload.
G-CERT-4. DerivedProduct: recursively verify all factors and exact polynomial identity `product = Π factor.guard`.
```

### 4.2 Target certificates

```rust
pub enum TargetCertificate {
    IdealMembership { support: UniPolynomialQ, multipliers: Vec<PolynomialQ>, identity: ExactIdentity },
    RadicalMembership { support: UniPolynomialQ, power: usize, multipliers: Vec<PolynomialQ>, identity: ExactIdentity },
    GuardedRadicalMembership { support: UniPolynomialQ, support_power: usize, guard_power: usize, guard_product: PolynomialQ, guard_certificates: Vec<GuardCertificate>, multipliers: Vec<PolynomialQ>, identity: ExactIdentity },
    CompositeCover { support: UniPolynomialQ, children: Vec<TargetCertificate>, rule: CompositeRule, component_union_source: Option<ComponentUnionSource> },
}
```

Verifier MUST check:

```text
T-CERT-1. support is nonzero and has variable equal to problem.target.
T-CERT-2. IdealMembership: `support(T) - Σ q_i F_i == 0` exactly.
T-CERT-3. RadicalMembership: power > 0 and `support(T)^power - Σ q_i F_i == 0` exactly.
T-CERT-4. GuardedRadicalMembership: support_power > 0; every guard certificate valid; `guard_product == Π guards` exactly; `guard_product^guard_power * support(T)^support_power - Σ q_i F_i == 0` exactly.
T-CERT-5. SameIdealGcd composite: each child verified against same problem; support equals primitive gcd of child supports.
T-CERT-6. ComponentUnionLcm composite: component union source is present and replay-verifiable; support equals primitive lcm of child supports.
```

Forbidden simplifications:

```text
FAIL if verifier trusts `ExactIdentityKind` without recomputing the polynomial identity.
FAIL if verifier trusts `guard_product` without recomputing product from valid guard certificates.
FAIL if verifier accepts ExactTargetImage by default or rejects it as "not handled" in final implementation.
FAIL if verifier accepts NoTargetEliminant only for a narrow monomial ideal special case.
```

### 4.3 Solver-level certificates

```rust
pub enum SolverCertificate {
    TargetCover(TargetCertificate),
    ExactTargetImage(ExactTargetImageCertificate),
    EmptyAdmissibleSet(EmptyAdmissibleSetCertificate),
    NoNonzeroTargetEliminant(NoTargetEliminantCertificate),
}
```

`verify_certificate(problem, cert)` MUST handle every variant in final implementation.

## 5. Algebra primitives

Files:

```text
src/arith.rs
src/variable.rs
src/monomial.rs
src/polynomial.rs
src/univariate.rs
src/finite_field.rs
src/matrix_q.rs
src/matrix_fp.rs
src/crt.rs
src/rational_reconstruction.rs
```

Required functionality:

```text
A1. Sparse multivariate polynomial over Q with exact add/sub/mul/pow/evaluate/substitute/support.
A2. Sparse univariate polynomial over Q with primitive normalization, derivative, squarefree part, gcd, lcm, factor_squarefree_over_q.
A3. Finite field arithmetic for prime fields with denominator admissibility checks.
A4. Dense or sparse exact Q linear solver returning either solution or left-null obstruction.
A5. Fp linear solver returning nullspace, solution, rank, and active nonzero columns.
A6. CRT for matching modular univariate polynomials across primes.
A7. Rational reconstruction from CRT modulus with explicit success/failure bound.
A8. Deterministic term ordering; no hash-order-dependent algorithm output.
```

`factor_squarefree_over_q` MUST NOT return only `[squarefree_part(self)]` in final implementation. It MUST factor squarefree univariate rational polynomials at least by rational-root extraction and squarefree irreducible factor splitting using exact algorithms sufficient for all conformance families. If full factorization is resource-limited, it must return a resource failure and not claim factor trial completed.

## 6. Certificate windows and residual oracle

Files:

```text
src/window.rs
src/residual.rs
```

### 6.1 Certificate window

```rust
pub struct CertificateWindow {
    pub target_degree: usize,
    pub multiplier_supports: Vec<Vec<Monomial>>,
    pub row_monomials: Vec<Monomial>,
}
```

`make_row_closed_certificate_window(system, target_degree, multiplier_supports)` MUST construct:

```text
C = Supp{1,T,...,T^d} ∪ ⋃_i Supp(B_i F_i)
```

It MUST recompute `row_monomials` and ignore forged row sets passed by callers.

### 6.2 Residual oracle

```rust
pub trait ResidualOracleFp {
    fn modulus(&self) -> u64;
    fn reduce(&self, vector: &[u64]) -> Vec<u64>;
    fn is_in_column_space(&self, vector: &[u64]) -> bool;
}
```

Required contract:

```text
R-ORACLE-1. reduce(v) == 0 iff v is in the column space of M_p,W.
R-ORACLE-2. reduce(reduce(v)) == reduce(v).
R-ORACLE-3. The implementation may use echelon bases, sparse row bases, or Schur handles, but the external behavior is exactly the quotient residual map.
```

Reviewer must inspect tests proving the iff property by randomized small matrices over several primes.

## 7. Candidate data model and normalization

File: `src/candidates/mod.rs`

```rust
pub trait CandidateOracle {
    fn generate(&self, system: &CertifiedSystemQ, window: &CertificateWindow) -> Vec<TargetCandidate>;
}

pub struct TargetCandidate {
    pub support_mod_primes: Vec<UniPolynomialFp>,
    pub reconstructed: Option<UniPolynomialQ>,
    pub origin: CandidateOrigin,
    pub traces: Vec<CandidateTrace>,
}
```

Candidate normalization MUST:

```text
N1. Normalize each modular candidate to primitive monic representative over Fp.
N2. Merge candidates from different primes only if degrees and normalized CRT residues match.
N3. Reconstruct Q coefficients only through CRT + rational reconstruction when more than one prime is available.
N4. If only one prime is available, mark reconstruction as heuristic candidate and require exact proof before any output; do not score it as multi-prime reconstructed.
N5. Normalize Q candidates by clearing denominators, dividing content, and positive leading coefficient.
N6. Keep squarefree support only for root isolation; never replace proof target with squarefree part unless the squarefree factor itself is separately certified.
N7. Factor schedule must attempt actual univariate factors and original candidate; it must not be `vec![candidate.clone()]` in final implementation.
```

Ranking is proof attempt order only. It is never adoption.

## 8. Candidate routes

Every route below must be production-reachable, route-forceable, no-fallback testable, and tamper-testable.

### 8.1 DirectTargetEquation

File: `src/candidates/direct.rs`

Required control-flow:

```text
1. Iterate over certified system equations.
2. Select equations whose support contains only powers of target T.
3. Convert to `UniPolynomialQ`.
4. Primitive-normalize and return as candidate.
5. Record equation index trace.
```

Forbidden:

```text
- Reading variable names such as "T" instead of comparing `Variable` objects.
- Returning direct candidate as certified without fixed proof.
```

### 8.2 ResidualCyclic

File: `src/candidates/residual_cyclic.rs`

Required control-flow:

```text
1. Build row-closed window W.
2. Build exact Q membership matrix M_W and target power matrix N_d.
3. Select admissible primes not dividing any denominator in equations, guards, replay data, or target power vectors.
4. For each prime p:
   a. Reduce M_W and N_d modulo p.
   b. Build residual oracle rho_p,W from M_p,W.
   c. Compute residuals r_k = rho(vec(T^k)).
   d. Solve nullspace relation Σ c_k r_k = 0.
   e. For each relation c, solve M_p,W u = N_p,d c to recover at least one modular multiplier vector u.
   f. Record active multiplier supports Act_i,p = {b in B_i | u_{i,b} != 0}.
5. Merge modular supports across primes by degree and normalized residues.
6. Apply CRT + rational reconstruction to get optional Q candidate.
7. Return candidate with modular witnesses and reconstructed Q candidate only when reconstruction succeeds.
```

Required data-flow:

```text
- `support_mod_primes` must be the normalized relation polynomial from residual nullspace.
- `reconstructed` must be derived from CRT/rational reconstruction over the collected primes, not from the first prime alone.
- `active_multiplier_supports` must be derived from nonzero entries of an actual modular solution u, not copied from the full window.
```

Forbidden simplifications:

```text
FAIL if reconstruction maps the first prime into [-p/2,p/2] and calls that Q reconstruction when multiple primes are available.
FAIL if active support trace equals the entire window by default.
FAIL if residual relation is adopted without fixed proof.
FAIL if denominator-admissibility of primes is skipped.
```

Minimum conformance families:

```text
RC-F1. Two-equation finite target family with nontrivial multiplier support.
RC-F2. Same family with coefficient height exceeding any single prime, forcing CRT reconstruction.
RC-F3. Family where a wrong first-prime lift exists but exact proof rejects it.
RC-F4. Guarded family where a denominator/guard denominator excludes a prime.
```

### 8.3 TargetCyclicKrylov

File: `src/candidates/krylov.rs`

Required control-flow:

```text
1. Build target-relevant quotient handle from the same window/residual contract as M_W.
2. Compute residual classes of 1,T,T^2,... up to target_degree.
3. Find a minimal recurrence relation among those classes using exact Q or modular Krylov linear algebra.
4. Return the recurrence polynomial as candidate.
5. Record quotient rank estimate, recurrence length, and basis columns used.
```

Required data-flow:

```text
- Recurrence must be computed from residual classes / quotient handle.
- If exact Q relation search is used, it must be explicitly named and treated as a candidate route, not hidden as Krylov if no quotient recurrence is built.
```

Forbidden:

```text
FAIL if route simply calls complete fallback or Groebner elimination.
FAIL if route only appends target powers to a Q matrix and does not expose a quotient/residual recurrence handle.
FAIL if recurrence candidate is adopted without fixed proof.
```

Minimum conformance families:

```text
KR-F1. Finite quotient where target recurrence degree is smaller than full quotient rank.
KR-F2. Positive-dimensional system with finite target image and finite target recurrence.
KR-F3. Case where an early recurrence is spurious and exact proof rejects it.
```

### 8.4 HiddenVariableSparseResultant

File: `src/candidates/sparse_resultant.rs`

This route MUST implement a general multi-polynomial sparse eliminant template. It MUST NOT be a two-polynomial Sylvester resultant route disguised as sparse resultant.

Required control-flow:

```text
1. Select a target-relevant block from CertifiedSystemQ using algebraic incidence and Newton supports; do not use names or fixtures.
2. Keep target T as hidden variable. Choose eliminated variables X_B.
3. Build a sparse Macaulay / resultant-style template over Fp[T] or over Q[T]:
   - row monomials are monomials in X_B chosen from support-set expansion;
   - columns are multiplier monomials times input equations;
   - coefficients are univariate polynomials in T.
4. Support m >= 2 equations. For overdetermined systems, use maximal minors or null-relation templates as specified below.
5. Compute a determinantal or null-relation eliminant polynomial in Fp[T].
6. Normalize, merge across primes, reconstruct Q candidate using CRT/rational reconstruction.
7. Return only candidate traces; adoption still requires fixed exact proof.
```

For rectangular templates:

```text
- If columns == rows, candidate is det(A(T)).
- If columns > rows, compute nonzero maximal row-rank minors using deterministic pivot subsets; candidate is gcd/lcm-normalized common vanishing polynomial from selected maximal minors.
- If rows > columns, compute left null conditions and determinant of selected square subtemplates after support expansion.
- In all cases, template rank data and selected rows/columns must be recorded.
```

Required data-flow:

```text
- Candidate polynomial must be derived from the constructed sparse template.
- The template must include contributions from all selected equations unless a recorded algebraic reason excludes an equation.
- Multi-polynomial input must not be reduced to pairwise resultants without exact lcm/gcd aggregation and proof gate.
```

Forbidden simplifications:

```text
FAIL if production route has a hard condition `polynomial_count == 2` or equivalent and no separate general path.
FAIL if route computes only univariate Sylvester resultant while claiming sparse resultant.
FAIL if route calls complete target elimination or Groebner and labels the output resultant.
FAIL if determinant/null relation output is not the source of the candidate.
FAIL if 3+ polynomial conformance route is absent.
```

Minimum conformance families:

```text
SR-F1. Two-polynomial sparse resultant case.
SR-F2. Three-polynomial overdetermined case where all equations are needed.
SR-F3. Sparse support case where dense Macaulay degree is larger but sparse template is small.
SR-F4. Case where selected minor gives spurious factor and exact proof filters it.
```

### 8.5 SliceSpecialization

File: `src/candidates/slice.rs`

Required control-flow:

```text
1. Select non-target variables to slice using algebraic incidence; never slice target.
2. Generate deterministic generic affine slices over admissible primes.
3. Build sliced system containing all original equations plus slice equations.
4. Compute target candidate for the sliced system using residual-cyclic or sparse eliminant on the sliced system.
5. Record slice equations, assignments, prime, and the route used inside the slice.
6. Combine slice observations only as candidate ranking/aggregation; never as certificate.
7. Return candidates that still require fixed proof in the original unsliced system.
```

Forbidden simplifications:

```text
FAIL if route substitutes non-target variables into each equation independently and treats each single-equation target polynomial as slice candidate.
FAIL if route does not build a sliced system with all original equations.
FAIL if route uses slice gcd as adoption evidence.
FAIL if deterministic assignments are hard-coded to one or two slices without resource-driven schedule.
```

Minimum conformance families:

```text
SL-F1. System where any single equation slice gives wrong target polynomial but global sliced system gives useful candidate.
SL-F2. System where slice candidate is spurious and fixed proof rejects it.
SL-F3. System needing two different affine slice families to recover stable candidate.
```

### 8.6 NormTraceTower

File: `src/candidates/norm_trace_tower.rs`

Required control-flow:

```text
1. Detect triangular algebraic tower using equations of the form a_j(Y_<j) * Y_j^d + lower = 0.
2. If leading coefficient a_j is not 1, require a valid guard certificate proving a_j nonzero on admissible fibers before using it.
3. Build quotient basis with exponents below each tower degree.
4. Reduce multiplication by target expression in the tower quotient.
5. Construct multiplication matrix for the target element.
6. Candidate is characteristic polynomial or minimal polynomial derived from exact matrix computation.
7. Return candidate only; adoption requires fixed proof unless the route also constructs a full exact membership certificate.
```

Forbidden simplifications:

```text
FAIL if route accepts only monic coefficient 1 towers and claims general NormTraceTower without documenting and testing guarded nonmonic path.
FAIL if target expression must have coefficient ±1 only.
FAIL if reduction by tower is not exact.
FAIL if characteristic polynomial is computed by ad hoc determinant code without tests over rational coefficients and nontrivial tower degrees.
```

Minimum conformance families:

```text
NT-F1. Monic two-level tower.
NT-F2. Nonmonic leading coefficient with InputSemanticNonzero guard.
NT-F3. Tower where characteristic polynomial has repeated factors and factor schedule must handle them correctly.
```

### 8.7 LocalizedSchur

File: `src/candidates/localized_schur.rs`

Required control-flow:

```text
1. Collect left-null obstructions from failed fixed proofs.
2. Determine minimal obstruction scope Ω by row monomial / equation support incidence.
3. Determine boundary variables Z_Ω = {T} ∪ separators(Ω).
4. Build local membership equation M_Ω u + N_Ω f = 0 over exact Q.
5. Solve for frontier relation space V_Ω.
6. If a target-only relation in T is derived, construct `TargetCertificate` by replaying to original system and return `LocalizedSchur` candidate with certificate.
7. If no target-only relation is derived, return support information that expands proof windows.
```

Forbidden simplifications:

```text
FAIL if localized Schur never attempts to construct a target-only exact certificate.
FAIL if localized Schur always returns `SupportInformation` even for conformance family with target-only local relation.
FAIL if full-system Schur is used outside complete fallback.
FAIL if local relation is accepted without replay into original system.
```

Minimum conformance families:

```text
LS-F1. Obstruction localized to a proper subset and only support expansion is needed.
LS-F2. Obstruction localized to a proper subset and exact target-only local relation is produced.
LS-F3. Scope would be full system; route must refuse and defer to complete fallback.
```

## 9. Fixed exact proof and proof search

Files:

```text
src/proof/fixed.rs
src/proof/schedule.rs
src/proof/learning.rs
src/proof/multiple_repair.rs
```

### 9.1 Fixed proof

`prove_fixed_target(input)` MUST:

```text
1. Validate candidate is nonzero and in target variable.
2. Validate proof window supports match system equations.
3. For CertificateMode:
   - Ideal: H = S(T)
   - Radical {a}: require a >= 1; H = S(T)^a
   - GuardedRadical {a,e}: require a >= 1; build D from verified system.guard_certificates; H = D^e S(T)^a
4. Build row set C_H = Supp(H) ∪ ⋃ Supp(B_i F_i).
5. Build exact Q linear system M_H u = vec(H).
6. Solve over Q.
7. If inconsistent, return left-null obstruction with row monomials and coefficients.
8. If consistent, restore multipliers q_i.
9. Recompute H - Σ q_i F_i as sparse polynomial over Q.
10. If nonzero, return ImplementationBug / IdentityCheckFailed; never return certificate.
11. Return the exact TargetCertificate.
```

### 9.2 Fair search

Unbounded search MUST be fair over:

```text
- proof window support degree d_B,
- support power a,
- guard power e.
```

Required schedule:

```text
for weight = 0..∞:
    for d_B = 0..weight:
        for a = 1..=weight+1:
            for e = 0..=weight:
                if d_B + a + e <= weight + 1:
                    attempt(d_B, a, e)
```

The exact schedule may differ, but reviewer must be able to prove:

```text
For every finite tuple (d_B, a, e), unbounded mode eventually attempts it.
```

Forbidden simplifications:

```text
FAIL if unbounded mode enters an infinite window iterator and never reaches complete fallback or fair proof tuple scheduling.
FAIL if `max_window_degree = None` causes nontermination on conformance failures without trace/resource behavior defined.
FAIL if proof mode schedule is fair over powers but not over support degree.
```

### 9.3 Proof learning

Required:

```text
- Initial proof window must include modular active support from actual modular multipliers.
- Obstruction expansion must add predecessor supports Pred_F(r).
- Exhaustive degree expansion must eventually include all monomials up to every finite degree in unbounded mode.
```

### 9.4 Low-degree multiple repair

Required:

```text
1. For candidate S, search A(T) up to configured degree.
2. Construct P(T)=A(T)S(T).
3. Attempt fixed proof on P, P^a, D^eP^a using the fair schedule within resource bounds.
4. If certified, output support P, not S.
```

Forbidden:

```text
FAIL if repair returns original uncertified S.
FAIL if repair accepts a multiple without exact certificate.
```

## 10. Complete target elimination fallback

Files:

```text
src/elimination/groebner.rs
src/elimination/saturation.rs
src/elimination/target_elimination.rs
```

This is the only place where complete exact target elimination is allowed. It is not coordinate solving and must not enumerate coordinate solutions.

Required algebra:

```text
Given I = <F_1,...,F_m> and certified guard product D:
(I : D^∞) ∩ Q[T]
```

Required implementation:

```text
1. Build certified guard product D from valid guard certificates. If no guards, D = 1.
2. If D != 1, introduce saturation variable U and ideal J = <F_1,...,F_m, U*D - 1> in Q[X,T,U].
3. Use exact Groebner basis or equivalent exact elimination for an elimination order with eliminated variables X,U greater than T.
4. Extract nonzero univariate polynomials in Q[T] from the elimination basis.
5. If a nonzero S(T) is found, derive a `GuardedRadicalMembership` or `IdealMembership` certificate by replaying Groebner reductions and clearing U denominators to produce D^e S(T)^a = Σ q_i F_i.
6. If 1 is in the saturated ideal, return `EmptyAdmissibleSetCertificate`.
7. If no nonzero T-only polynomial exists, return `NoTargetEliminantCertificate` containing a verifier-checkable elimination-zero certificate.
8. If resource limits stop the exact elimination, return `ResourceFailure` with actual matrix/pair counts.
```

Groebner certificate requirements:

```text
- Each basis polynomial must carry a representation as a combination of original generators and saturation equation.
- For a support S, verifier must replay/verify D^e S^a = Σ q_i F_i over Q.
- For no-target-eliminant, verifier must either recompute the deterministic elimination basis from certificate data or verify a Buchberger/reduction certificate proving the elimination ideal basis has no nonzero Q[T] generator.
```

Forbidden simplifications:

```text
FAIL if complete fallback is bounded by `max_window_degree` and then returns `ResourceFailure` or `NoVerifiedTargetCertificate` without exact elimination attempt when resource is unbounded.
FAIL if `CertifiedNoNonzeroTargetEliminant` is accepted only for monomial ideals.
FAIL if complete fallback returns target support from a relation search without saturation replay.
FAIL if complete fallback enumerates full coordinate solutions or full coordinate RUR.
```

Minimum conformance families:

```text
CTE-F1. Unguarded finite target eliminant.
CTE-F2. Guarded saturation where eliminant appears only after saturating D.
CTE-F3. Algebraically empty ideal returning CertifiedEmptyAdmissibleSet.
CTE-F4. No nonzero target eliminant positive-dimensional algebraic family with exact zero-elimination certificate.
CTE-F5. Resource-limited run returns FiniteResourceFailure with no unsound success.
```

## 11. Exact real root and exact target image

Files:

```text
src/real/algebraic.rs
src/real/sturm.rs
src/real/cad.rs
src/real/fiber.rs
```

### 11.1 Root isolation

`isolate_real_roots_squarefree` MUST return exact algebraic roots:

```rust
pub struct AlgebraicRealRoot {
    pub polynomial: UniPolynomialQ,
    pub isolating_interval: RationalInterval,
    pub index: usize,
}
```

Required:

```text
- polynomial must be squarefree.
- interval must contain exactly one real root.
- no floating approximation as proof.
```

### 11.2 Real fiber classification

For each root α of squarefree support, classify:

```text
∃X ∈ R^n such that F(X, α)=0 and semantic_guards are satisfied.
```

Required final implementation:

```text
1. Represent α as AlgebraicNumber over Q with minimal polynomial and isolating interval.
2. Build polynomial system over Q(α) for the fiber F(X,α)=0.
3. Include semantic guards with correct real sign semantics:
   - NonZero: g(X,α) != 0
   - Positive: g(X,α) > 0
   - Negative: g(X,α) < 0
   - NonNegative: g(X,α) >= 0
   - NonPositive: g(X,α) <= 0
4. Use exact recursive projection-lifting CAD or equivalent exact real algebraic feasibility algorithm over Q(α).
5. If nonempty, return `RealFiberNonemptyCertificate` containing an exact algebraic sample point and sign/equality proof.
6. If empty, return `RealFiberEmptyCertificate` containing a CAD/projection infeasibility certificate or Positivstellensatz/RCF certificate that verifier can replay.
7. If resource limits prevent classification, return incomplete trace, not `CertifiedExactTargetImage`.
```

Verifier requirements:

```text
- Verify nonempty sample point by exact algebraic evaluation of all equalities and guards.
- Verify empty certificate by replaying CAD cell decomposition or the exact infeasibility certificate.
- Verify every root of squarefree_support is classified exactly once.
```

Forbidden simplifications:

```text
FAIL if `classify_real_fibers` always returns Incomplete.
FAIL if exact image verifier rejects ExactTargetImage as "not handled" in final implementation.
FAIL if nonempty certificate uses floats or approximate numeric coordinates.
FAIL if empty certificate is a string or external CAS output without replay.
FAIL if unclassified roots are silently dropped.
```

Minimum conformance families:

```text
EI-F1. Cover S(T)=T^2-2 with both roots real and both fibers nonempty.
EI-F2. Cover with one spurious real root; exact image rejects it via empty fiber certificate.
EI-F3. Guard removes one root through NonZero or Positive guard.
EI-F4. RequireExactImage with small resource limit fails closed.
```

## 12. Solver control-flow

File: `src/solver.rs`

Required top-level pseudocode:

```text
solve_target(problem, options):
    validated = validate(problem)
    canonical = canonicalize(validated)
    system = certified_compress(canonical)
    verify internal compression in debug/check mode

    early_empty = try_cheap_empty_admissible_set_certificate(system)
    if early_empty.valid:
        return CertifiedEmptyAdmissibleSet

    dag = build_target_dependency_dag(system)
    window_schedule = fair_certificate_window_schedule(system, dag, options)

    verified = []
    obstructions = []

    for schedule_step in window_schedule:
        W = schedule_step.window
        candidates = []
        candidates += DirectTargetEquation.generate(system, W)
        candidates += NormTraceTower.generate(system, W)
        candidates += ResidualCyclic.generate(system, W)
        candidates += TargetCyclicKrylov.generate(system, W)
        candidates += HiddenVariableSparseResultant.generate(system, W)
        candidates += SliceSpecialization.generate(system, W)

        for candidate in normalize_rank_and_merge(candidates):
            for factor_candidate in factor_schedule(candidate):
                proof_result = fair_fixed_proof_search(system, factor_candidate, W, obstructions, options)
                if proof_result.valid:
                    verified.push(proof_result.certificate)
                    cover = refine_and_finalize_same_ideal_gcd(verified)
                    return maybe_classify_exact_target_image(problem, cover, options)

                repair_result = low_degree_multiple_repair(system, factor_candidate, proof_result.last_window, options)
                if repair_result.valid:
                    verified.push(repair_result.certificate)
                    cover = refine_and_finalize_same_ideal_gcd(verified)
                    return maybe_classify_exact_target_image(problem, cover, options)

        schur_result = localized_schur_repair(system, obstructions, options)
        if schur_result.valid:
            verified.push(schur_result.certificate)
            cover = refine_and_finalize_same_ideal_gcd(verified)
            return maybe_classify_exact_target_image(problem, cover, options)
        if schur_result.support_information:
            feed support information into future proof windows

        if schedule_step.resource_exhausted:
            break

    final = complete_target_elimination_fallback(system, options)
    match final:
        CertifiedSupport(cert) => maybe_classify_exact_target_image(problem, cover_from(cert), options)
        CertifiedEmpty(cert) => CertifiedEmptyAdmissibleSet
        CertifiedNoTargetEliminant(cert) => CertifiedNoNonzeroTargetEliminant
        ResourceFailure(trace) => FiniteResourceFailure or NoVerifiedTargetCertificate according to options
```

Required status consistency:

```text
- CertifiedCandidateCover: cover Some, certificate Some(TargetCover), exact_image None.
- CertifiedExactTargetImage: exact_image Some, certificate Some(ExactTargetImage), cover may be None or derivable from image certificate but no ambiguity.
- CertifiedEmptyAdmissibleSet: cover None, exact_image None, certificate Some(EmptyAdmissibleSet).
- CertifiedNoNonzeroTargetEliminant: cover None, exact_image None, certificate Some(NoNonzeroTargetEliminant).
- NoVerifiedTargetCertificate: no success certificate.
- FiniteResourceFailure: no success certificate and trace includes actual resource blocker.
```

Forbidden:

```text
FAIL if complete fallback is reached only after an infinite unbounded iterator.
FAIL if `assert!` panic is used as normal solver behavior outside tests.
FAIL if route control exists only under `#[cfg(test)]` and no production route observability exists.
FAIL if `ImplementationBug` is used for unimplemented algorithm variants.
```

## 13. Route-forcing and no-fallback harness

The repo MUST include a test-only but source-visible route-forcing harness. It must allow reviewers to run each route while disabling all other candidate routes and complete fallback.

Required API under `#[cfg(test)]`:

```rust
pub(crate) struct RouteForcing {
    pub enabled_origins: BTreeSet<CandidateOrigin>,
    pub allow_complete_fallback: bool,
    pub allow_other_heavy_routes: bool,
}

pub(crate) fn solve_target_with_route_forcing(problem: TargetProblemQ, options: SolverOptions, forcing: RouteForcing) -> TargetSolveResult;
```

Every route MUST have tests:

```text
- route-only candidate generation
- route-only solve when fixed proof is possible
- route-only rejection when candidate is spurious
- no complete fallback trace
- no other route trace
- tamper certificate rejection
```

## 14. Documentation artifacts required

Agent must maintain, but reviewer must not blindly trust:

```text
docs/ai/changes/cw-arc-dtp-q/evidence/non_simplification_manifest.md
docs/ai/changes/cw-arc-dtp-q/evidence/route_forcing_matrix.md
docs/ai/changes/cw-arc-dtp-q/evidence/data_flow_proofs.md
```

`non_simplification_manifest.md` MUST contain one section per route:

```text
- Production call chain
- Required data-flow objects
- Forbidden simplifications searched
- Route-forcing tests
- Tamper tests
- Why this is not name-only
- Why this is not certificate-shell-only
- Why this is not fallback-only
```

Reviewer must verify each claim from source.

## 15. Final disqualifiers

The implementation is automatically non-conformant if any final production path contains one of the following patterns without explicit user-approved Base Spec amendment:

```text
D1. `guard_certificates: Vec::new()` as the only CertifiedSystemQ construction path.
D2. `semantic_guards: Vec::new()` in proof/guard verification path where original problem guards are needed.
D3. `classify_real_fibers` or equivalent always returning Incomplete.
D4. `verify_certificate` rejecting ExactTargetImage as unhandled.
D5. `complete_target_elimination_fallback` bounded only by window degree and not doing exact elimination.
D6. `NoTargetEliminantCertificate` verifier limited to monomial ideals.
D7. modular reconstruction from only the first prime in a multi-prime route.
D8. `factor_schedule` returning only the original candidate in final implementation.
D9. localized Schur never attempting certificate construction.
D10. sparse resultant route rejecting all polynomial_count != 2 or equivalent narrow route.
D11. slice route using single-equation substitution as global slice system.
D12. route tests proving only top-level success, not route-forced no-fallback success.
D13. production `Unsupported`, `not available`, `TODO`, `unimplemented!`, or normal `ImplementationBug` for spec-required variants.
D14. Hidden call to full coordinate RUR, full coordinate solution enumeration, fixture-specific branch, or problem-name branch.
```

