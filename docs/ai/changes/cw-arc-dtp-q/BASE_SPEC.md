# CW-ARC-DTP-Q Guardian Base Spec

Spec ID: `CW-ARC-DTP-Q-CANDIDATE-COVER`
Type: Change Base Spec
Status: Draft for user approval
Scope owner: `geosolver-core` target-value solver
Target repo state: empty Rust repository

---

## Context Packet

Spec ID: `CW-ARC-DTP-Q-CANDIDATE-COVER`
Type: Change Base Spec
Status: Draft
Parent: none assumed; if the repo already has an active Repo or Area Base Spec, this Change Base Spec must be amended to list the required parent R-IDs before implementation.
Scope: Implement the CW-ARC-DTP-Q target-candidate-cover core over rational polynomial systems.
Applies To: `geosolver-core` library crate and its tests.
Required Parent R-IDs: none in an empty repo.
Blocking Questions: none. This spec intentionally resolves ambiguous implementation choices by fixing file paths, public types, control flow, and verification behavior.
Non-blocking Debt: general exact real fiber classification is not required for closing `CertifiedCandidateCover`, but the exact-image API and fail-closed status behavior are required.
Known Exceptions: none.
Read-First R-IDs: BS-CORE-001, BS-CERT-001, BS-PROOF-001, BS-SOLVER-001, BS-FORBID-001.
Context Packet Authority: non-authoritative digest. The body below is authoritative.

---

## 0. Source Authority and Fidelity Rules

### BS-SRC-001 — Primary source
The implementation MUST follow the uploaded CW-ARC-DTP-Q revised specification v2 exactly for the mathematical solver contract. The solver is target-certificate-first: candidate generation by finite-field, specialization, Krylov, resultant, norm/trace, and slice probes is never enough for success; adoption requires an exact certificate over the original rational system or a replay-equivalent certified system.

### BS-SRC-002 — Failure-analysis sources
The uploaded failure-analysis documents are normative anti-pattern sources. Their specific past examples are not to be copied literally as narrow checks; they MUST be generalized into requirements that prevent name-only implementation, certificate shelling, narrow route capture, hidden delegation, evidence overfitting, and gate-only completion.

### BS-SRC-003 — Guardian workflow constraints
Implementation MUST use Guardian Lane. A Base Spec is the correctness authority, a Plan maps tasks to R-IDs/MECHs/verification, and implementation MUST NOT start until this Base Spec and Plan are explicitly approved by the user.

### BS-SRC-004 — Design and naming references
Rust code MUST follow the Rust API Guidelines naming conventions: modules/functions/methods/local variables in `snake_case`, types/traits/enum variants in `UpperCamelCase`, constants/statics in `SCREAMING_SNAKE_CASE`, and conversion methods using `as_`, `to_`, and `into_` according to cost/ownership. Module boundaries MUST follow information-hiding principles: hide volatile representation choices such as residual matrix backend, row-basis representation, monomial ordering internals, and linear algebra backend behind stable contracts.

---

## 1. Exact Scope

### BS-SCOPE-001 — In scope
The repo MUST implement a Rust library crate whose production public API is:

```rust
pub fn solve_target(problem: TargetProblemQ, options: SolverOptions) -> TargetSolveResult;

pub fn verify_certificate(problem: TargetProblemQ, cert: SolverCertificate) -> VerificationResult;
```

The implemented solver MUST accept well-formed rational polynomial target systems:

```text
F = {F_1, ..., F_m} ⊂ Q[X_1, ..., X_n, T]
```

and search for a nonzero target-only support polynomial:

```text
S(T) ∈ Q[T]
```

with an exact certificate proving one of:

```text
S(T) ∈ I,
S(T)^a ∈ I,
D^e S(T)^a ∈ I,
```

where `I = <F_1, ..., F_m>`, `a >= 1`, `e >= 0`, and every guard factor in `D` has an independently verifiable `GuardCertificate`.

### BS-SCOPE-002 — Standard success target
The standard success status for this change is `SolverStatus::CertifiedCandidateCover`. It returns exact target candidate values as real roots of `support`, possibly with extra roots.

### BS-SCOPE-003 — Exact image boundary
The exact-image types, root isolation, and `MaybeClassifyExactTargetImage` control flow MUST exist. `CertifiedExactTargetImage` MUST only be returned when every real root of the squarefree support has an exact `Nonempty` or `Empty` fiber certificate. If the general real-fiber classifier is not complete, the implementation MUST fail closed according to `ExactImageMode` and MUST NOT silently drop unclassified roots.

### BS-SCOPE-004 — Empty and no-eliminant statuses
Exact empty admissible set and no-nonzero-target-eliminant statuses MUST be separate from target cover. The solver MUST NOT fake empty admissible set by returning `S(T)=1` as a cover.

### BS-SCOPE-005 — Out of scope
The following are out of scope for this change and MUST NOT appear as production behavior:

```text
- Geometry DSL lowering.
- Geometry-family dispatch.
- Coordinate-solution enumeration.
- Full coordinate RUR construction.
- Full coordinate lex parametrization followed by reading T.
- Generic numerical solving as proof.
- Generic CAD/QE as a hidden production fallback.
- Problem-name, fixture-name, expected-answer, or official-solution dispatch.
```

---

## 2. Required Repository Layout

The empty repo MUST be created with this layout. Names MUST NOT contain plan version names, phase names, temporary labels, author jokes, benchmark fixture names, or terms such as `v2_impl`, `new_algo`, `hack`, `legacy`, `temp`, `fallback_solver`, or `toy`.

```text
Cargo.toml
README.md
src/
  lib.rs
  arith.rs
  variable.rs
  monomial.rs
  polynomial.rs
  univariate.rs
  finite_field.rs
  linear_q.rs
  linear_fp.rs
  problem.rs
  compression.rs
  guards.rs
  certificates.rs
  verifier.rs
  window.rs
  residual.rs
  candidates.rs
  candidate_direct.rs
  candidate_residual.rs
  candidate_tower.rs
  candidate_krylov.rs
  candidate_resultant.rs
  candidate_slice.rs
  normalize.rs
  dependency_dag.rs
  proof.rs
  proof_learning.rs
  repair_multiple.rs
  repair_schur.rs
  fallback_elimination.rs
  roots.rs
  exact_image.rs
  solver.rs
  trace.rs
  options.rs
  error.rs
  test_support.rs      # cfg(test) only

tests/
  exact_algebra_tests.rs
  verifier_tests.rs
  residual_window_tests.rs
  fixed_proof_tests.rs
  guard_certificate_tests.rs
  candidate_route_forcing_tests.rs
  solver_status_tests.rs
  root_isolation_tests.rs
  anti_simplification_static_tests.rs

docs/ai/
  SPEC_REGISTRY.md
  ACTIVE_CONTEXT.md
  changes/cw-arc-dtp-q/
    BASE_SPEC.md
    PLAN.md
    CLOSURE.md
    source_map.md
    evidence/
    reviews/
```

`src/lib.rs` MUST expose only the public API and core public data types. Matrix backends, monomial enumeration details, and route internals MUST remain private unless a type is explicitly listed in this Base Spec as public.

---

## 3. Required Public Data Model

### BS-DATA-001 — Variables
File: `src/variable.rs`

```rust
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Variable {
    pub symbol: String,
}
```

Rules:

```text
- `Variable.symbol` is only an identifier for equality, display, and stable ordering.
- Production algorithms MUST NOT dispatch on substrings such as "circle", "distance", "area", "x", "y", "mixtilinear", fixture names, or problem names.
- The only semantic distinction allowed is equality with `TargetProblemQ.target`.
```

### BS-DATA-002 — Exact sparse multivariate polynomial
File: `src/polynomial.rs`

```rust
pub type Rational = num_rational::BigRational;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Monomial {
    pub exponents: Vec<u32>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PolynomialQ {
    pub variables: Vec<Variable>,
    pub terms: std::collections::BTreeMap<Monomial, Rational>,
}
```

Required functions and behavior:

```rust
impl PolynomialQ {
    pub fn zero(variables: Vec<Variable>) -> Self;
    pub fn one(variables: Vec<Variable>) -> Self;
    pub fn from_term(variables: Vec<Variable>, coefficient: Rational, monomial: Monomial) -> Self;
    pub fn normalize(&mut self);
    pub fn is_zero(&self) -> bool;
    pub fn support(&self) -> Vec<Monomial>;
    pub fn degree(&self) -> u32;
    pub fn add(&self, rhs: &Self) -> Self;
    pub fn sub(&self, rhs: &Self) -> Self;
    pub fn mul(&self, rhs: &Self) -> Self;
    pub fn pow(&self, exponent: usize) -> Self;
    pub fn scale(&self, factor: &Rational) -> Self;
    pub fn substitute_variable(&self, variable: &Variable, replacement: &PolynomialQ) -> Self;
    pub fn depends_only_on(&self, allowed: &[Variable]) -> bool;
    pub fn to_univariate_in(&self, target: &Variable) -> Option<UniPolynomialQ>;
}
```

Pseudocode for `normalize`:

```text
remove every term with zero coefficient
assert every monomial has exactly variables.len() exponents
keep BTreeMap order as canonical sparse order
```

Forbidden simplifications:

```text
- Floating point coefficients.
- HashMap iteration order as canonical output.
- String parsing as the primary polynomial representation.
- Implicit variable creation during arithmetic.
- Silently aligning polynomials with different variable orders.
```

### BS-DATA-003 — Exact univariate target polynomial
File: `src/univariate.rs`

```rust
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct UniPolynomialQ {
    pub variable: Variable,
    pub coefficients: Vec<Rational>, // coefficient of T^k at index k
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct UniPolynomialFp {
    pub variable: Variable,
    pub modulus: u64,
    pub coefficients: Vec<u64>,
}
```

Required functions:

```rust
impl UniPolynomialQ {
    pub fn zero(variable: Variable) -> Self;
    pub fn one(variable: Variable) -> Self;
    pub fn degree(&self) -> Option<usize>;
    pub fn is_zero(&self) -> bool;
    pub fn normalize(&mut self);
    pub fn primitive_integer_normalized(&self) -> Self;
    pub fn squarefree_part(&self) -> Self;
    pub fn gcd(&self, rhs: &Self) -> Self;
    pub fn lcm(&self, rhs: &Self) -> Self;
    pub fn factor_squarefree_over_q(&self) -> Vec<Self>;
    pub fn pow(&self, exponent: usize) -> Self;
    pub fn to_multivariate(&self, variables: &[Variable]) -> PolynomialQ;
}
```

Rules:

```text
- `primitive_integer_normalized` MUST clear denominators, divide integer content, and make the leading coefficient positive.
- Squarefree form is for root isolation and exact-image stage only. It MUST NOT replace the proof target unless the squarefree part has its own exact certificate.
- `gcd` is used for same-ideal verified cover refinement.
- `lcm` is used only for component-union cover composition.
```

### BS-DATA-004 — Input problem
File: `src/problem.rs`

```rust
#[derive(Clone, Debug)]
pub struct TargetProblemQ {
    pub equations: Vec<PolynomialQ>,
    pub variables: Vec<Variable>,
    pub target: Variable,
    pub semantic_guards: Vec<GuardRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuardRecord {
    pub polynomial: PolynomialQ,
    pub kind: GuardKind,
    pub provenance: GuardProvenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuardKind {
    NonZero,
    Positive,
    Negative,
    NonNegative,
    NonPositive,
    OtherSemanticCondition,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuardProvenance {
    pub description: String,
}
```

Invariants:

```text
- `target` MUST be in `variables`.
- Every equation and guard polynomial MUST use exactly `variables` as its variable order.
- `semantic_guards` MUST NOT be used as candidate-adoption evidence unless converted into a valid `GuardCertificate`.
- Non-`NonZero` guards MUST NOT be used as factors in D unless a separate nonzero `GuardCertificate` is built and verified.
```

### BS-DATA-005 — Certified system and compression replay
File: `src/compression.rs`

```rust
#[derive(Clone, Debug)]
pub struct CertifiedSystemQ {
    pub equations: Vec<PolynomialQ>,
    pub variables: Vec<Variable>,
    pub target: Variable,
    pub guard_certificates: Vec<GuardCertificate>,
    pub replay: CompressionReplayCertificate,
}

#[derive(Clone, Debug)]
pub struct CompressionReplayCertificate {
    pub steps: Vec<CompressionStepCertificate>,
}

#[derive(Clone, Debug)]
pub enum CompressionStepCertificate {
    DefinitionSubstitution { variable: Variable, expression: PolynomialQ, identity: ExactIdentity },
    AffineElimination { pivot: PolynomialQ, pivot_guard: GuardCertificate, identity: ExactIdentity },
    ExplicitGuardSaturation { guard: GuardCertificate, identity: ExactIdentity },
    PrimitiveNormalization { before: PolynomialQ, after: PolynomialQ, multiplier: Rational },
    ZeroEquationRemoval { removed: PolynomialQ },
}
```

Rules:

```text
- `CertifiedSystemQ` MUST be obtained only by replayable certificate-preserving rewrites.
- Forbidden rewrites: arbitrary factor selection, branch selection by expected answer, geometry-name formulas, and external CAS simplification without replay.
- If a planned compression is not replayable, skip it and keep the original equations.
```

---

## 4. Certificate Language

### BS-CERT-001 — Exact identity is not trusted by itself
File: `src/certificates.rs`

```rust
#[derive(Clone, Debug)]
pub struct ExactIdentity {
    pub kind: ExactIdentityKind,
}

#[derive(Clone, Debug)]
pub enum ExactIdentityKind {
    IdealMembership,
    RadicalMembership,
    GuardedRadicalMembership,
    GuardProduct,
    AlgebraicInfeasibility,
    GuardedAlgebraicInfeasibility,
    CompressionReplay,
}
```

`ExactIdentity` is a typed label for the identity being checked. The verifier MUST recompute the polynomial identity from the certificate payload and the input problem. It MUST NOT trust a stored zero residual, hash, string, trace, or prior solver claim.

### BS-CERT-002 — Guard certificates
File: `src/guards.rs`

```rust
#[derive(Clone, Debug)]
pub enum GuardCertificate {
    InputSemanticNonzero {
        guard: PolynomialQ,
        record: GuardRecord,
    },
    AlgebraicNonvanishing {
        guard: PolynomialQ,
        certificate: NullstellensatzCertificate,
    },
    RealAdmissibleNonvanishing {
        guard: PolynomialQ,
        certificate: RealInfeasibilityCertificate,
    },
    DerivedProduct {
        product: PolynomialQ,
        factors: Vec<GuardCertificate>,
        identity: ExactIdentity,
    },
}

#[derive(Clone, Debug)]
pub struct NullstellensatzCertificate {
    pub multipliers: Vec<PolynomialQ>,
    pub guard_multiplier: PolynomialQ,
    pub identity: ExactIdentity, // 1 = Σ q_i F_i + q_g guard
}

#[derive(Clone, Debug)]
pub enum RealInfeasibilityCertificate {
    VerifiedByExactAlgebraicCertificate(NullstellensatzCertificate),
    VerifiedByExternalReplay { replay: String },
}
```

Verifier behavior:

```text
InputSemanticNonzero:
  pass only if the original input contains an identical GuardRecord with GuardKind::NonZero.

AlgebraicNonvanishing:
  recompute 1 - (Σ q_i F_i + q_g guard) over Q and pass only if zero.

RealAdmissibleNonvanishing:
  pass only for payloads with implemented exact replay. Unknown strings or unimplemented external payloads fail.

DerivedProduct:
  recursively verify factors, recompute product of factor guards, and compare exactly with `product`.
```

### BS-CERT-003 — Target certificates
File: `src/certificates.rs`

```rust
#[derive(Clone, Debug)]
pub enum TargetCertificate {
    IdealMembership {
        support: UniPolynomialQ,
        multipliers: Vec<PolynomialQ>,
        identity: ExactIdentity,
    },
    RadicalMembership {
        support: UniPolynomialQ,
        power: usize,
        multipliers: Vec<PolynomialQ>,
        identity: ExactIdentity,
    },
    GuardedRadicalMembership {
        support: UniPolynomialQ,
        support_power: usize,
        guard_power: usize,
        guard_product: PolynomialQ,
        guard_certificates: Vec<GuardCertificate>,
        multipliers: Vec<PolynomialQ>,
        identity: ExactIdentity,
    },
    CompositeCover {
        support: UniPolynomialQ,
        children: Vec<TargetCertificate>,
        rule: CompositeRule,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompositeRule {
    SameIdealGcd,
    ComponentUnionLcm,
}
```

Verifier behavior:

```text
IdealMembership:
  H = support.to_multivariate(problem.variables)
  verify H - Σ multipliers[i] * F_i == 0.

RadicalMembership:
  require power >= 1
  H = support^power as multivariate
  verify H - Σ multipliers[i] * F_i == 0.

GuardedRadicalMembership:
  require support_power >= 1
  require guard_power >= 0
  verify every guard certificate
  verify guard_product equals the product of verified guard factors
  H = guard_product^guard_power * support^support_power
  verify H - Σ multipliers[i] * F_i == 0.

CompositeCover / SameIdealGcd:
  verify all child certificates against the same original problem
  recompute gcd(child supports) and compare with support after primitive normalization.

CompositeCover / ComponentUnionLcm:
  allowed only for an explicit component-union certificate source
  recompute lcm(child supports) and compare with support after primitive normalization.
```

### BS-CERT-004 — Solver-level certificates
File: `src/certificates.rs`

```rust
#[derive(Clone, Debug)]
pub enum SolverCertificate {
    TargetCover(TargetCertificate),
    ExactTargetImage(ExactTargetImageCertificate),
    EmptyAdmissibleSet(EmptyAdmissibleSetCertificate),
    NoNonzeroTargetEliminant(NoTargetEliminantCertificate),
}

#[derive(Clone, Debug)]
pub enum EmptyAdmissibleSetCertificate {
    AlgebraicInfeasibility {
        multipliers: Vec<PolynomialQ>,
        identity: ExactIdentity, // 1 = Σ q_i F_i
    },
    GuardedAlgebraicInfeasibility {
        guard_product: PolynomialQ,
        guard_power: usize,
        guard_certificates: Vec<GuardCertificate>,
        multipliers: Vec<PolynomialQ>,
        identity: ExactIdentity, // D^e = Σ q_i F_i
    },
    RealInfeasibility {
        certificate: RealInfeasibilityCertificate,
    },
}

#[derive(Clone, Debug)]
pub struct NoTargetEliminantCertificate {
    pub saturated_ideal_description: SaturatedIdealCertificate,
    pub elimination_certificate: EliminationZeroCertificate,
    pub guard_certificates: Vec<GuardCertificate>,
}
```

Rules:

```text
- A `TargetCertificate` alone is not enough for top-level result verification; top-level verification MUST verify `SolverCertificate`.
- Empty admissible set MUST return `SolverStatus::CertifiedEmptyAdmissibleSet`, never `CertifiedCandidateCover` with support 1.
- No nonzero target eliminant is an algebraic statement only and MUST NOT imply real non-finiteness.
```

---

## 5. Result and Options API

File: `src/options.rs`, `src/solver.rs`, `src/trace.rs`

```rust
#[derive(Clone, Debug)]
pub struct SolverOptions {
    pub resource_limits: ResourceLimits,
    pub exact_image_mode: ExactImageMode,
}

#[derive(Clone, Debug)]
pub struct ResourceLimits {
    pub max_window_degree: Option<usize>,
    pub max_proof_weight: Option<usize>,
    pub max_matrix_rows: Option<usize>,
    pub max_matrix_cols: Option<usize>,
    pub max_candidate_count: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExactImageMode {
    CoverOnly,
    TryExactImage,
    RequireExactImage,
}

#[derive(Clone, Debug)]
pub struct TargetSolveResult {
    pub status: SolverStatus,
    pub cover: Option<CertifiedCandidateCover>,
    pub exact_image: Option<CertifiedExactTargetImage>,
    pub certificate: Option<SolverCertificate>,
    pub trace: SolverTrace,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SolverStatus {
    CertifiedCandidateCover,
    CertifiedExactTargetImage,
    CertifiedEmptyAdmissibleSet,
    CertifiedNoNonzeroTargetEliminant,
    NoVerifiedTargetCertificate,
    FiniteResourceFailure,
    CertificateDesignGap,
    InvalidInput,
    ImplementationBug,
}
```

Rules:

```text
- Production `SolverOptions` MUST NOT contain geometry names, fixture names, expected values, route-forcing options, or hidden fallback switches.
- Route forcing is allowed only under `#[cfg(test)]` in `test_support.rs`.
- `Unsupported` MUST NOT be a solver status. Candidate routes may return an empty candidate list with a trace explaining why the route was not applicable.
```

---

## 6. Window and Residual Oracle

### BS-WIN-001 — Certificate windows
File: `src/window.rs`

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CertificateWindow {
    pub target_degree: usize,
    pub multiplier_supports: Vec<Vec<Monomial>>, // one B_i per equation
    pub row_monomials: Vec<Monomial>,            // C
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProofWindow {
    pub multiplier_supports: Vec<Vec<Monomial>>,
}
```

Required functions:

```rust
pub fn make_row_closed_certificate_window(
    system: &CertifiedSystemQ,
    target_degree: usize,
    multiplier_supports: Vec<Vec<Monomial>>,
) -> CertificateWindow;

pub fn build_membership_matrix_q(
    system: &CertifiedSystemQ,
    window: &CertificateWindow,
) -> MembershipMatrixQ;

pub fn build_target_power_matrix_q(
    system: &CertifiedSystemQ,
    window: &CertificateWindow,
) -> TargetPowerMatrixQ;
```

Pseudocode for row closure:

```text
C := support(1), support(T), ..., support(T^d)
for each equation F_i:
  for each b in B_i:
    C := C ∪ support(b * F_i)
sort C by the canonical monomial order
return W = (d, B_i, C)
```

Rules:

```text
- Row closure MUST be recomputed from equations and supports, not trusted from caller input.
- Failure to prove in a window MUST NOT be interpreted as proof that S ∉ I.
```

### BS-RES-001 — Residual oracle contract
File: `src/residual.rs`

```rust
pub trait ResidualOracleFp {
    fn modulus(&self) -> u64;
    fn reduce(&self, vector: &[u64]) -> Vec<u64>;
    fn is_in_column_space(&self, vector: &[u64]) -> bool {
        self.reduce(vector).iter().all(|x| *x == 0)
    }
}

pub struct DenseEchelonResidualOracleFp { /* private fields */ }
```

Construction pseudocode:

```text
input: M_{p,W}
compute a row-echelon basis for the column space using exact F_p arithmetic
store only private data needed to compute v mod col(M)
for reduce(v): eliminate v by the same column-space basis and return canonical residual
```

Required theorem-level behavior:

```text
rho(v) == 0 iff v is in col(M_{p,W}).
```

Forbidden simplifications:

```text
- Returning a hash, rank, or boolean trace instead of residual vectors.
- Treating stable rank over several primes as proof over Q.
- Exposing matrix backend details through the public solver API.
```

### BS-RES-002 — Residual-cyclic candidate search
File: `src/candidate_residual.rs`

```rust
pub fn residual_cyclic_candidates(
    system: &CertifiedSystemQ,
    window: &CertificateWindow,
    primes: &[u64],
) -> Vec<TargetCandidate>;
```

Pseudocode:

```text
for each admissible prime p:
  reduce M_W and N_d modulo p
  build ResidualOracleFp for M_{p,W}
  for k = 0..d:
    r_k := rho(vec_C(T^k))
  solve Σ c_k r_k = 0 over F_p for nonzero c
  for each relation c:
    S_p(T) := Σ c_k T^k
    record TargetCandidate with origin ResidualCyclic and modular witness trace
attempt CRT/rational reconstruction only as reconstruction trace
return candidates; do not produce TargetCertificate here
```

Acceptance-specific behavior:

```text
- Candidate route may be wrong or incomplete.
- Any candidate from this route MUST remain a candidate until `ProveFixedTarget` returns an exact certificate.
```

---

## 7. Candidate Oracle Layer

### BS-CAND-001 — Candidate model
File: `src/candidates.rs`

```rust
pub trait CandidateOracle {
    fn generate(&self, system: &CertifiedSystemQ, window: &CertificateWindow) -> Vec<TargetCandidate>;
}

#[derive(Clone, Debug)]
pub struct TargetCandidate {
    pub support_mod_primes: Vec<UniPolynomialFp>,
    pub reconstructed: Option<UniPolynomialQ>,
    pub origin: CandidateOrigin,
    pub traces: Vec<CandidateTrace>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CandidateOrigin {
    DirectTargetEquation,
    NormTraceTower,
    ResidualCyclic,
    TargetCyclicKrylov,
    HiddenVariableSparseResultant,
    SliceSpecialization,
    LocalizedSchur,
    CompleteTargetElimination,
}
```

Rules:

```text
- Candidate oracles MUST NOT return `TargetCertificate`, except that `LocalizedSchur` and `CompleteTargetElimination` may carry an exact certificate in a separate result type that is verified before solver success.
- Candidate traces are diagnostic only and MUST NOT be accepted by verifier.
- Every route MUST be reachable from top-level planning unless the route's algebraic preconditions fail and a trace records the failed precondition.
```

### BS-CAND-002 — Direct target equation
File: `src/candidate_direct.rs`

```rust
pub fn direct_target_equation_candidates(system: &CertifiedSystemQ) -> Vec<TargetCandidate>;
```

Pseudocode:

```text
for each equation F_i:
  if F_i is nonzero and depends only on target:
    convert F_i to UniPolynomialQ
    normalize as candidate
    return candidate with origin DirectTargetEquation
```

Forbidden:

```text
- Treating the direct equation as proof without fixed proof verification.
- Reading variable names other than equality with target.
```

### BS-CAND-003 — NormTraceTower
File: `src/candidate_tower.rs`

The route MUST implement a real structural detector for explicit algebraic towers, not a fixture detector.

Required tower shape:

```text
variables can be ordered y_1, ..., y_r so that each level has a monic equation
p_j(y_j; previous variables, T) = 0
and target expression is represented by a polynomial relation in the tower.
```

Pseudocode:

```text
detect monic triangular tower by variable incidence and main-variable degree
if no tower exists: return empty candidate trace `precondition_not_met`
construct multiplication matrix for the target expression in the finite free module implied by the monic tower
compute characteristic/norm polynomial det(Z - M_target) exactly or modulo primes with reconstruction
map Z to T if the norm is target-only
return candidate with origin NormTraceTower
```

Acceptance:

```text
- A route-forcing test MUST include at least a two-level tower not named by fixtures.
- Adoption still requires fixed proof.
```

### BS-CAND-004 — TargetCyclicKrylov
File: `src/candidate_krylov.rs`

Required behavior:

```text
- Use a target-relevant quotient handle derived from a finite row basis or residual oracle.
- Generate recurrence from the sequence 1, T, T^2, ... in that handle.
- Return only candidates unless exact proof succeeds later.
```

Forbidden:

```text
- Calling full coordinate RUR.
- Calling coordinate solution enumeration.
- Returning recurrence as proof.
```

### BS-CAND-005 — HiddenVariableSparseResultant
File: `src/candidate_resultant.rs`

This route MUST be general enough to handle multi-polynomial templates. It MUST NOT be a two-polynomial-only Sylvester wrapper.

Required control flow:

```text
1. Collect Newton supports for all equations in the selected target dependency cone.
2. Select eliminated variables X and hidden target T by algebraic incidence, not by names.
3. Build a monomial template from support-set/Macaulay-style degree expansion.
4. Build the modular template matrix over admissible primes.
5. Compute determinant/null-relation candidates modulo primes.
6. Reconstruct a target-only candidate over Q when possible.
7. Return TargetCandidate only; exact proof is separate.
```

Required static FAIL patterns:

```text
- Any production branch that rejects all `equations.len() != 2` for the entire resultant route.
- Any production branch that supports only one eliminated variable without a general path or documented resource failure evidence.
- Any production route that calls Groebner fallback and labels the result as sparse resultant.
- Any acceptance based solely on modular determinant stability.
```

Route-forcing acceptance:

```text
- Must include at least one 3-polynomial hidden-variable eliminant family.
- Other candidate routes and complete fallback must be disabled in the test harness.
- The route may return a candidate that is later certified by fixed proof; top-level success through another route does not close this R-ID.
```

### BS-CAND-006 — SliceSpecialization
File: `src/candidate_slice.rs`

Required behavior:

```text
1. Choose deterministic generic affine slices over admissible finite fields for variables other than target.
2. Solve only for target candidates in the sliced residual/eliminant system.
3. Record slice equations and modular trace.
4. Return candidates only.
```

Forbidden:

```text
- Using agreement across slices as proof.
- GCD across slices as success without exact certificate.
- Silently dropping branches because a slice looks inconsistent.
```

---

## 8. Candidate Normalization and Ranking

File: `src/normalize.rs`

### BS-NORM-001 — Normalization

```rust
pub fn normalize_candidate(candidate: TargetCandidate) -> Option<TargetCandidate>;
pub fn rank_candidates(candidates: Vec<TargetCandidate>) -> Vec<TargetCandidate>;
pub fn factor_schedule(candidate: &TargetCandidate) -> Vec<TargetCandidate>;
pub fn refine_verified_same_ideal(certificates: Vec<TargetCertificate>) -> TargetCertificate;
```

Normalization pseudocode:

```text
if reconstructed is None and reconstruction cannot be done: keep modular-only candidate for trace but do not send to fixed proof
if reconstructed polynomial is zero: discard
clear denominators
remove integer content
make leading coefficient positive
preserve original support for proof; compute squarefree only for roots
```

Ranking order:

```text
1. exact origin candidates
2. lower degree
3. reproduced over more primes
4. reproduced by more origins
5. lower coefficient height
6. smaller active support trace
```

Rules:

```text
- Ranking order is proof-attempt order only.
- Ranking MUST NOT be an acceptance condition.
- Same-ideal verified covers MUST refine by gcd, not product.
- Component-union covers MUST combine by lcm only when component union semantics has a certificate.
```

---

## 9. Fixed-S Exact Proof

File: `src/proof.rs`

### BS-PROOF-001 — Input and modes

```rust
#[derive(Clone, Debug)]
pub struct FixedProofInput {
    pub system: CertifiedSystemQ,
    pub candidate: UniPolynomialQ,
    pub proof_window: ProofWindow,
    pub certificate_mode: CertificateMode,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CertificateMode {
    Ideal,
    Radical { support_power: usize },
    GuardedRadical { support_power: usize, guard_power: usize },
}
```

### BS-PROOF-002 — Fair certificate mode schedule

```rust
pub fn fair_certificate_mode_schedule(limits: &ResourceLimits) -> Vec<CertificateMode>;
```

Pseudocode:

```text
emit Ideal first
for weight = 1..limit_or_unbounded:
  for a = 1..=weight:
    emit Radical { support_power: a } if a <= weight
  for a = 1..=weight:
    for e = 0..=weight:
      if a + e <= weight:
        emit GuardedRadical { support_power: a, guard_power: e }
```

The schedule MUST be fair over `(a, e, d_B)` when combined with proof-window degree expansion. In unbounded ideal execution, every finite tuple must eventually be attempted.

### BS-PROOF-003 — Prove fixed target

```rust
pub fn prove_fixed_target(input: FixedProofInput) -> Result<TargetCertificate, ProofFailure>;
```

Pseudocode:

```text
S := primitive normalized candidate
mode := input.certificate_mode
if mode == Ideal:
  a := 1; e := 0; D := 1
if mode == Radical:
  require support_power >= 1
  a := support_power; e := 0; D := 1
if mode == GuardedRadical:
  require support_power >= 1
  verify all system.guard_certificates
  construct D as exact product of verified guard factors
  a := support_power; e := guard_power
H := D^e * S(T)^a as PolynomialQ over system.variables
C_H := support(H) ∪ ⋃ support(B_i * F_i)
M_H := columns vec_C_H(b * F_i)
b := vec_C_H(H)
solve M_H u = b over Q
if inconsistent:
  compute exact left-null obstruction λ with λ^T M_H = 0 and λ^T b != 0
  return ProofFailure::Inconsistent { obstruction }
restore multipliers q_i from u and proof_window supports
if H - Σ q_i F_i != 0 over Q:
  return ProofFailure::ImplementationBug-equivalent
return matching TargetCertificate with explicit multipliers
```

Forbidden:

```text
- Accepting modular reconstruction without Q identity check.
- Treating linear-solve success as enough when identity check fails.
- Treating inconsistency in one proof window as proof that S is false.
- Using semantic guards in D unless their GuardCertificate verifies.
```

### BS-PROOF-004 — Modular proof construction
The implementation MAY use modular linear algebra, CRT, and rational reconstruction to find multipliers, but only if the reconstructed rational multipliers are explicit and the Q identity check passes. Modular rank stability and successful reconstruction are trace only.

---

## 10. Proof Window Learning

File: `src/proof_learning.rs`

### BS-LEARN-001 — Witness and obstruction traces

```rust
#[derive(Clone, Debug)]
pub struct ModularWitnessTrace {
    pub prime: u64,
    pub active_multiplier_supports: Vec<Vec<Monomial>>,
}

#[derive(Clone, Debug)]
pub struct LeftNullObstruction {
    pub row_monomials: Vec<Monomial>,
    pub coefficients: Vec<Rational>,
}
```

### BS-LEARN-002 — Expansion by predecessors

```rust
pub fn expand_by_obstruction_predecessors(
    system: &CertifiedSystemQ,
    proof_window: &ProofWindow,
    obstruction: &LeftNullObstruction,
) -> ProofWindow;
```

Pseudocode:

```text
for each row monomial r with nonzero λ_r:
  for each equation F_i:
    for each monomial ν in support(F_i):
      if r is divisible by ν:
        b := r / ν
        add b to B_i
return expanded proof window
```

Rules:

```text
- Active support and obstruction support guide proof search only.
- Unbounded search MUST include exhaustive monomial support enumeration by degree.
- Resource-limited failure MUST return `FiniteResourceFailure` or `NoVerifiedTargetCertificate`, never unsound success.
```

---

## 11. Repairs and Complete Fallback

### BS-REPAIR-001 — Low-degree multiple repair
File: `src/repair_multiple.rs`

```rust
pub fn low_degree_multiple_repair(
    system: &CertifiedSystemQ,
    candidate: &UniPolynomialQ,
    proof_window: &ProofWindow,
    limits: &ResourceLimits,
) -> Result<TargetCertificate, ProofFailure>;
```

Pseudocode:

```text
for deg_A from 0..limit:
  construct unknown A(T) of degree deg_A
  form P(T) = A(T) * candidate
  solve combined linear system for A coefficients and multipliers
  for each nonzero reconstructed P:
    run prove_fixed_target on P
    if valid: return certificate for P, not for original candidate
return failure/resource exceeded
```

Rule: The repaired support is `P(T)`. The original `S(T)` MUST NOT be returned unless it has its own certificate.

### BS-REPAIR-002 — Localized Schur repair
File: `src/repair_schur.rs`

Required behavior:

```text
- Use obstruction incidence to identify a minimal scope Ω.
- Boundary variables are `{target} ∪ separators(Ω)`.
- Build local membership condition `M_Ω u + N_Ω f = 0`.
- Output only either exact-certificate target candidate or support information for proof-window expansion.
```

Forbidden:

```text
- Full-system Schur repair except inside final exact target elimination fallback.
- Treating an uncertified Schur relation as solver success.
- Calling complete fallback and labelling it localized Schur.
```

### BS-FALLBACK-001 — Complete target elimination fallback
File: `src/fallback_elimination.rs`

```rust
pub enum CompleteFallbackResult {
    CertifiedSupport(TargetCertificate),
    CertifiedEmpty(EmptyAdmissibleSetCertificate),
    CertifiedNoTargetEliminant(NoTargetEliminantCertificate),
    ResourceFailure(CostTrace),
}

pub fn complete_target_elimination_fallback(system: &CertifiedSystemQ, limits: &ResourceLimits) -> CompleteFallbackResult;
```

Required behavior:

```text
- Compute only `(I : D^∞) ∩ Q[T]` or exact certificates about it.
- Do not enumerate coordinate solutions.
- Do not construct full coordinate RUR.
- If a nonzero target eliminant is returned, return it with an exact membership/guarded membership certificate.
- If empty admissible set is certified, return `CertifiedEmptyAdmissibleSet` path.
- If no target eliminant is certified, return `CertifiedNoNonzeroTargetEliminant` path, not real nonfinite.
```

Allowed heavy operation:

```text
Exact Groebner/elimination or ideal quotient computation is allowed only here and only for the target elimination statement. It MUST be unreachable in route-forcing tests unless the test explicitly targets fallback.
```

---

## 12. Exact Roots and Exact Image

File: `src/roots.rs`, `src/exact_image.rs`

### BS-ROOT-001 — Exact real root records

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RationalInterval {
    pub lower: Rational,
    pub upper: Rational,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlgebraicRealRoot {
    pub polynomial: UniPolynomialQ,
    pub isolating_interval: RationalInterval,
    pub index: usize,
}

pub fn isolate_real_roots_squarefree(poly: &UniPolynomialQ) -> Vec<AlgebraicRealRoot>;
```

Required algorithm:

```text
- Compute squarefree support exactly.
- Use Sturm sequence or an equivalent exact rational isolation algorithm.
- Return rational intervals, each containing exactly one real root.
- Floating approximations may be stored only as trace, not as root records.
```

### BS-IMAGE-001 — Candidate cover output

```rust
#[derive(Clone, Debug)]
pub struct CertifiedCandidateCover {
    pub support: UniPolynomialQ,
    pub squarefree_support: UniPolynomialQ,
    pub real_roots: Vec<AlgebraicRealRoot>,
    pub certificate: TargetCertificate,
}
```

### BS-IMAGE-002 — Exact target image output

```rust
#[derive(Clone, Debug)]
pub struct ExactTargetImageCertificate {
    pub cover: TargetCertificate,
    pub squarefree_support: UniPolynomialQ,
    pub root_classifications: Vec<RealRootFiberCertificate>,
}

#[derive(Clone, Debug)]
pub struct CertifiedExactTargetImage {
    pub support: UniPolynomialQ,
    pub squarefree_support: UniPolynomialQ,
    pub values: Vec<AlgebraicRealRoot>,
    pub rejected_roots: Vec<AlgebraicRealRoot>,
    pub certificate: ExactTargetImageCertificate,
}

#[derive(Clone, Debug)]
pub enum RealRootFiberCertificate {
    Nonempty { root: AlgebraicRealRoot, certificate: RealFiberNonemptyCertificate },
    Empty { root: AlgebraicRealRoot, certificate: RealFiberEmptyCertificate },
}
```

Rules:

```text
- Partial root classification is trace only.
- `CertifiedExactTargetImage` requires classifications for every real root of squarefree support.
- `TryExactImage` returns `CertifiedCandidateCover` if classification is incomplete.
- `RequireExactImage` returns `NoVerifiedTargetCertificate` or `FiniteResourceFailure` if classification is incomplete.
```

---

## 13. Dependency DAG and Window Planning

File: `src/dependency_dag.rs`

### BS-DAG-001 — Algebraic footprint only

```rust
pub fn build_target_dependency_dag(system: &CertifiedSystemQ) -> TargetDependencyDag;
pub fn plan_certificate_windows(system: &CertifiedSystemQ, dag: &TargetDependencyDag, limits: &ResourceLimits) -> Vec<CertificateWindow>;
```

Required footprint:

```text
- relation-variable incidence
- graph distance from target
- separator size
- monomial support
- degree
- affine eliminability
- explicit tower detectability
- quotient-rank estimate
```

Forbidden:

```text
- Geometry names.
- Variable-name semantics beyond equality with target.
- Expected answers.
- Fixture names.
```

Window planner requirements:

```text
- Every planned window is finite and row-closed.
- Target dependency cone and small separator support are prioritized.
- Unbounded ideal execution enumerates all multiplier monomial supports by degree eventually.
- Planner estimates never affect proof soundness.
```

---

## 14. Top-Level Solver

File: `src/solver.rs`

### BS-SOLVER-001 — Required top-level pseudocode

```text
solve_target(problem, options):
  P0 := validate(problem)
  if invalid: return InvalidInput
  P1 := canonicalize(P0)
  P  := certified_compress(P1)

  early_empty := try_cheap_empty_admissible_set_certificate(P)
  if early_empty.valid:
    return CertifiedEmptyAdmissibleSet(early_empty.certificate)

  dag := build_target_dependency_dag(P)
  windows := plan_certificate_windows(P, dag, options.resource_limits)

  verified := []
  collected_obstructions := []

  for W in windows:
    candidates := []
    candidates += direct_target_equation_candidates(P)
    candidates += norm_trace_tower_candidates(P, W)
    candidates += residual_cyclic_candidates(P, W, primes)
    candidates += target_cyclic_krylov_candidates(P, W)
    candidates += hidden_variable_sparse_resultant_candidates(P, W)
    candidates += slice_specialization_candidates(P, W)

    for S in rank_candidates(normalize(candidates)):
      for Fctr in factor_schedule(S):
        PW := learn_initial_proof_window(W, Fctr.traces)
        for mode in fair_certificate_mode_schedule(options.resource_limits):
          result := prove_fixed_target(P, Fctr, PW, mode)
          if result.valid:
            verified += result.certificate
            cover := refine_and_finalize(verified, SameIdealGcd)
            return maybe_classify_exact_target_image(problem, cover, options)
          while result is Inconsistent and can_expand(PW, limits):
            collected_obstructions += result.obstruction
            PW := expand_by_obstruction_predecessors(P, PW, result.obstruction)
            result := prove_fixed_target(P, Fctr, PW, mode)
            if result.valid:
              verified += result.certificate
              cover := refine_and_finalize(verified, SameIdealGcd)
              return maybe_classify_exact_target_image(problem, cover, options)

        repaired := low_degree_multiple_repair(P, Fctr, PW, limits)
        if repaired.valid:
          verified += repaired.certificate
          cover := refine_and_finalize(verified, SameIdealGcd)
          return maybe_classify_exact_target_image(problem, cover, options)

    schur := localized_schur_repair(P, W, collected_obstructions)
    if schur.valid:
      verified += schur.certificate
      cover := refine_and_finalize(verified, SameIdealGcd)
      return maybe_classify_exact_target_image(problem, cover, options)

  final := complete_target_elimination_fallback(P, limits)
  match final:
    CertifiedSupport(cert): return maybe_classify_exact_target_image(problem, cert, options)
    CertifiedEmpty(cert): return CertifiedEmptyAdmissibleSet(cert)
    CertifiedNoTargetEliminant(cert): return CertifiedNoNonzeroTargetEliminant(cert)
    ResourceFailure(trace): return NoVerifiedTargetCertificate or FiniteResourceFailure
```

### BS-SOLVER-002 — MaybeClassifyExactTargetImage

```text
if options.exact_image_mode == CoverOnly:
  return CertifiedCandidateCover(cover)
image := classify_real_fibers(problem, cover)
if image.valid_and_complete:
  return CertifiedExactTargetImage(image.result)
if options.exact_image_mode == RequireExactImage:
  return NoVerifiedTargetCertificate(image.failure_trace)
return CertifiedCandidateCover(cover.with_partial_image_trace(image.partial_trace))
```

Rules:

```text
- The solver MUST fail closed.
- No candidate route can bypass `verify_certificate` semantics.
- No hidden fallback may be called outside the stated top-level location.
- Final support MUST be primitive-normalized and squarefree support MUST be computed separately for roots.
```

---

## 15. Verification Contract

File: `src/verifier.rs`

### BS-VERIFY-001 — Top-level verifier

```rust
pub fn verify_certificate(problem: TargetProblemQ, cert: SolverCertificate) -> VerificationResult;
```

Verifier trusts only:

```text
- input polynomials
- input semantic guard records actually present in `TargetProblemQ`
- certificate multipliers
- replay certificate payloads that are exact identities
- guard certificate exact identities
- exact real certificate payloads with implemented replay
- rational arithmetic
```

Verifier MUST NOT trust:

```text
- modular rank trace
- random seed
- candidate score
- planner estimate
- floating point result
- external CAS output without replay
- `guard_product` without factor certificates
- partial real fiber classification
- source-to-code maps
- reviewer PASS
- audit summaries
```

### BS-VERIFY-002 — Tamper resistance
Every certificate test MUST include at least one tamper case: change a multiplier, remove a guard certificate, alter a support coefficient, alter a guard product, or delete a root classification. The verifier MUST fail.

---

## 16. Mandatory Tests and Test Harness

### BS-TEST-001 — Route forcing harness
File: `src/test_support.rs` under `#[cfg(test)]` only.

```rust
#[cfg(test)]
pub struct TestRouteForcing {
    pub enabled_origins: std::collections::BTreeSet<CandidateOrigin>,
    pub allow_complete_fallback: bool,
}
```

Rules:

```text
- The harness MUST NOT be exposed in public production API.
- Route-specific tests MUST disable all other routes and fallback unless the test explicitly targets fallback.
- Top-level solver success through another route cannot close a route R-ID.
```

### BS-TEST-002 — Required test families
Tests MUST be algebraic families, not named geometry problems. At minimum:

```text
1. Direct target equation:
   F = {T^2 - 2, X^2 - 3}; support T^2 - 2.

2. Residual-cyclic proof:
   F = {X^2 - 2, T - X}; candidate/proof support T^2 - 2.

3. Guarded radical:
   F = {D*(T^2 - 2)} with semantic guard D != 0; support T^2 - 2 is accepted only by guarded radical certificate, and guard-certificate tamper fails.

4. Empty admissible set:
   F = {1}; returns CertifiedEmptyAdmissibleSet, not cover.

5. Same-ideal gcd refinement:
   two verified covers S1, S2 with nontrivial gcd; final support is gcd, not product.

6. Component-union lcm:
   explicit component-union certificate; final support is lcm, not product.

7. Resultant route forcing:
   at least one 3-polynomial hidden-variable eliminant family; resultant route only; fallback disabled.

8. Slice route forcing:
   a positive-dimensional system with finite target image after exact proof; slice candidates are not accepted without proof.

9. Root isolation:
   squarefree support T^2 - 2 returns two rational isolating intervals.

10. Exact image fail-closed:
    RequireExactImage with incomplete classifier returns NoVerifiedTargetCertificate or FiniteResourceFailure, never CertifiedExactTargetImage.
```

### BS-TEST-003 — Static anti-simplification tests
Static tests MUST scan production files for forbidden patterns and fail on detection unless the occurrence is in a test or documentation file and is explicitly expected:

```text
- `Unsupported` as a solver status.
- `equations.len() != 2` or equivalent branch rejecting an entire production route without a general path.
- function or module names containing `toy`, `temp`, `hack`, `legacy`, `v2`, `new_impl`, `fixture`, `expected`.
- public production route forcing options.
- floating point arithmetic in proof, verifier, root records, or certificate adoption path.
- calls to complete fallback from candidate route implementations.
- geometry-name strings in production dispatch.
```

---

## 17. Forbidden Simplifications and Failure Taxonomy

### BS-FORBID-001 — Global forbidden production behavior
Production path MUST NOT:

```text
1. Build a full coordinate solution list.
2. Build a full coordinate RUR.
3. Build full coordinate lex parametrization and read T from it.
4. Branch on geometry names, problem names, fixture names, expected answers, or official solutions.
5. Adopt finite-field, numerical, specialization, Krylov, or resultant results without exact proof.
6. Return nonfinite or no-eliminant without exact complete certificate.
7. Call hidden fallback not stated in this Base Spec.
8. Return `Unsupported` as ordinary solver failure.
9. Treat test/audit/reviewer PASS as implementation evidence by itself.
10. Treat names, types, trace fields, or certificate structs as proof of algorithm implementation.
```

### BS-FORBID-002 — Generalized Guardian failure prevention
The implementation MUST be rejected if any of the following patterns are found:

```text
Name Substitution:
  Algorithm name exists but control/data-flow is a weaker or different algorithm.

Certificate Shelling:
  Certificate structs exist but verifier does not recompute exact identities from input.

Trace Fiction:
  Modular/cost traces exist but do not feed or describe the actual computation.

Narrow Route Capture:
  A general route is limited to one variable, two polynomials, low degree, one fixture shape, or one toy family.

Hidden Delegation:
  A route delegates to another route or complete fallback while claiming its own route success.

Scope Laundering:
  Agent turns difficult in-scope algebra into out-of-scope or unsupported without user approval.

Evidence Overfitting:
  Audit files, closure tables, or test names pass while production code lacks the algorithm.
```

---

## 18. Minimal Public Surface and Redundancy Rules

### BS-API-001 — Minimal public API
The crate root MUST publicly expose only:

```text
- TargetProblemQ
- GuardRecord, GuardKind, GuardProvenance
- Variable
- PolynomialQ, UniPolynomialQ
- SolverOptions, ResourceLimits, ExactImageMode
- TargetSolveResult, SolverStatus
- CertifiedCandidateCover, CertifiedExactTargetImage, AlgebraicRealRoot, RationalInterval
- SolverCertificate and certificate enums
- solve_target
- verify_certificate
```

All route internals, matrix representations, residual basis details, and test forcing controls MUST remain private or `#[cfg(test)]`.

### BS-API-002 — Redundancy control
Implementation MUST NOT add redundant return values such as both `status` and an unchecked boolean success flag, both certificate and unverified support in a route result, or duplicate trace copies of polynomial objects already in certificates. Trace should contain diagnostics, not extra authority.

---

## 19. Acceptance Summary

A final implementation can claim `ACCEPTANCE_COMPLETE` for this Change Base Spec only if all of the following are true:

```text
- Every active R-ID in this Base Spec is implemented or has an approved exception.
- `cargo test` passes.
- Route-forcing tests pass with non-target routes and complete fallback disabled.
- Certificate tamper tests fail as expected.
- Static anti-simplification tests pass.
- Reviewer prompts in the companion reviewer file have been run at each phase and final review.
- Reviewers inspected production code directly, not only evidence files.
- No high-impact QuestionDebt remains open.
- Final closure states only the supported claim: certified candidate-cover core with exact-image fail-closed boundary, unless the exact image classifier is actually completed and reviewed.
```
