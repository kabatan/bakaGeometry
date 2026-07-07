# CW-ARC-DTP-Q Guardian Plan

Spec ID: `CW-ARC-DTP-Q-CANDIDATE-COVER`
Type: Plan Contract
Status: Draft for user approval
Base Spec: `CW_ARC_DTP_Q_GUARDIAN_BASE_SPEC.md`
Target repo state: empty Rust repository

---

## Context Packet

Spec ID: `CW-ARC-DTP-Q-CANDIDATE-COVER`
Type: Plan Contract
Status: Draft
Parent: Base Spec `CW-ARC-DTP-Q Guardian Base Spec`
Scope: Build the CW-ARC-DTP-Q candidate-cover solver exactly as specified.
Blocking Questions: none.
Plan Boundary: This Plan does not add requirements. If this Plan conflicts with the Base Spec, the Base Spec wins and the Agent must stop.
Allowed final claim if fully completed and reviewed: `ACCEPTANCE_COMPLETE` for certified candidate-cover core, with exact-image fail-closed boundary unless exact image is separately completed.
Forbidden claims before final closure: `PRODUCTION_SAFE`, `SOURCE_FAITHFUL`, `VERIFIED`, or `ACCEPTANCE_COMPLETE`.
Context Packet Authority: non-authoritative digest.

---

## 0. Guardian Execution Rules for the Agent

### PLAN-GUARD-001 — No implementation before approval
Do not edit the repository until the user explicitly approves this Base Spec and Plan for implementation.

### PLAN-GUARD-002 — No plan reinterpretation
Do not reinterpret mathematical requirements. If a step seems too hard, create QuestionDebt and stop the dependent phase. Do not narrow the scope, rename a simpler algorithm, or substitute a fallback.

### PLAN-GUARD-003 — Phase closure is semantic, not ceremonial
A phase closes only when production control-flow, data-flow, exact verifier behavior, tests, and reviewer inspection all match the Base Spec. Test pass alone, reviewer pass alone, or evidence file completion alone is insufficient.

### PLAN-GUARD-004 — Code must not contain plan artifacts as names
Do not name code modules, functions, tests, structs, features, or branches after plan phases, versions, or review states. Good names describe domain semantics: `residual`, `proof`, `guard`, `candidate_resultant`. Bad names: `phase7`, `v2_solver`, `new_fixed_proof`, `legacy_path`, `guardian_hack`.


---

## 0.5 Required MECH Index

Each MECH closes only when its implementation path, exact oracle, tests, evidence, and reviewer inspection all exist. The Agent must not mark a phase complete by creating types or tests alone.

| MECH-ID | Supports | Semantics | Inputs | Outputs | Oracle | Failure behavior | Required evidence |
|---|---|---|---|---|---|---|---|
| MECH-ALG-Q | BS-DATA-001..003 | Exact sparse polynomial arithmetic over Q | variables, monomials, rational terms | canonical `PolynomialQ`, `UniPolynomialQ` | exact equality after normalization | return error/panic for invalid variable order; never approximate | algebra tests and reviewer data-flow |
| MECH-UNI-Q | BS-DATA-003, BS-NORM-001 | exact univariate gcd/lcm/squarefree/primitive normalization | `UniPolynomialQ` | normalized univariate polynomials | Euclidean identities over Q | no proof adoption from squarefree transform | gcd/lcm/squarefree tests |
| MECH-LINALG-Q | BS-PROOF-003 | exact rational solve and obstruction | `M`, `b` over Q | solution or left-null obstruction | `M u=b` or `λ^T M=0`, `λ^T b!=0` | `ProofFailure::Inconsistent` or resource failure | linear tests + obstruction verification |
| MECH-LINALG-FP | BS-RES-001..002 | exact finite-field linear algebra for candidate search | matrices over Fp | residual basis, null relations | field arithmetic modulo prime | trace-only candidate failure | modular tests |
| MECH-GUARD | BS-CERT-002 | prove guard nonzero when used in D | problem guards, certificates | verified guard factors/product | exact guard certificate replay | `GuardNotCertified` | guard tamper tests |
| MECH-VERIFY | BS-VERIFY-001..002 | independent top-level certificate verifier | problem, `SolverCertificate` | `VerificationResult` | recomputed Q identities | fail closed | tamper matrix |
| MECH-TARGET-CERT | BS-CERT-003 | target support certificate semantics | target cert, problem | verified cover statement | exact `H - Σ q_iF_i == 0` | verification failure | target certificate tests |
| MECH-WINDOW | BS-WIN-001 | row-closed certificate/proof windows | system, supports, degree | `CertificateWindow`, matrices | row closure equality | resource failure only | window tests |
| MECH-RESIDUAL | BS-RES-001..002 | residual-cyclic modular candidate generation | window matrices, primes | `TargetCandidate` | `rho(v)=0 iff v∈col(M)` | trace-only route failure | route-forcing residual test |
| MECH-CANDIDATE-PRIMARY | BS-CAND-001..002 | direct/residual candidate routes | certified system, window | candidates only | no certificate output | empty candidate trace | route-forcing tests |
| MECH-FIXED-PROOF | BS-PROOF-001..004 | exact fixed-target proof | candidate, mode, proof window | `TargetCertificate` | Q identity check | `ProofFailure` | fixed proof tests |
| MECH-LEARNING | BS-LEARN-001..002 | proof-window expansion and fair schedule | witness/obstruction traces | expanded proof windows | predecessor formula and fairness enumeration | resource failure | learning tests |
| MECH-MULTIPLE-REPAIR | BS-REPAIR-001 | low-degree multiple repair | candidate S, proof window | certified P=A*S | fixed proof on P | proof/resource failure | repair tests |
| MECH-SCHUR-REPAIR | BS-REPAIR-002 | localized Schur repair | obstruction scope | certificate or support info | exact replay or no success | trace/support only | scope tests |
| MECH-TOWER | BS-CAND-003 | norm/trace tower candidate route | monic tower structure | candidate | structural tower + fixed proof later | precondition trace | route-forcing tower test |
| MECH-KRYLOV | BS-CAND-004 | target cyclic recurrence candidate route | quotient/residual handle | candidate | recurrence trace only + fixed proof later | precondition trace | route-forcing Krylov test |
| MECH-RESULTANT | BS-CAND-005 | multi-polynomial hidden-variable resultant candidate route | support template, primes | candidate | exact proof later; no modular adoption | resource/precondition trace | 3-polynomial route-forcing test |
| MECH-SLICE | BS-CAND-006 | finite-field slice candidate route | system, affine slices | candidates | exact proof later; no slice adoption | trace-only failure | route-forcing slice test |
| MECH-COMPLETE-FALLBACK | BS-FALLBACK-001 | final exact target elimination fallback | certified system, guards | support/empty/no-eliminant/resource | exact elimination certificate | resource failure | fallback tests |
| MECH-DAG | BS-DAG-001 | target dependency cone and window priority | algebraic incidence | DAG | footprint contains no semantic names | conservative broad windows | DAG tests/static scan |
| MECH-SOLVER | BS-SOLVER-001..002 | top-level solver orchestration | problem, options | `TargetSolveResult` | verifier-acceptable certificate for success | fail closed | integration tests |
| MECH-ROOTS | BS-ROOT-001 | exact real root isolation | squarefree support | rational isolating intervals | exact Sturm/root count | resource failure | root tests |
| MECH-EXACT-IMAGE-SAFETY | BS-IMAGE-001..002 | exact-image success only after complete classification | cover, roots, classifier | exact image or candidate cover/failure | all roots classified exactly | fail closed | exact-image safety tests |

Allowed claim after any individual phase: `LOCAL_VERIFICATION_ONLY` for that phase. `ACCEPTANCE_COMPLETE` is allowed only after P15 closure.


---

## Phase P0 — Guardian scaffold and source anchoring

R-IDs: BS-SRC-001 through BS-SRC-004, BS-TEST-001
Reviewer prompt: RP-P0

### Work

1. Create `docs/ai/SPEC_REGISTRY.md`, `docs/ai/ACTIVE_CONTEXT.md`, and `docs/ai/changes/cw-arc-dtp-q/`.
2. Copy the approved Base Spec into `docs/ai/changes/cw-arc-dtp-q/BASE_SPEC.md`.
3. Copy this Plan into `docs/ai/changes/cw-arc-dtp-q/PLAN.md`.
4. Create `source_map.md` with these source classes:
   - CW-ARC-DTP-Q revised spec v2: `EXACT` for algorithm semantics.
   - Guardian failure analysis: `EXACT` for anti-simplification rules.
   - GeoSolver failure causes: `EXACT` for avoiding heavy-fallback/narrow-slice failure.
   - Rust API Guidelines and Parnas/information-hiding references: `REFERENCE_ONLY` for naming/module design.
5. Create empty `evidence/` and `reviews/` directories.

### Acceptance

- `ACTIVE_CONTEXT.md` lists current spec, current phase, open R-IDs, required reviewers, forbidden claims, and next action in fewer than 80 lines.
- `source_map.md` does not make any summary authoritative over the Base Spec.
- No source file implementation exists yet except documentation.

### Stop if

- The repo already has a conflicting Base Spec and no approved parent R-ID mapping exists.
- The Agent attempts to implement code before user approval.

---

## Phase P1 — Rust crate skeleton and naming contract

R-IDs: BS-SCOPE-001, BS-SCOPE-005, BS-API-001, BS-API-002
Reviewer prompt: RP-P1

### Work

1. Create `Cargo.toml` for package `geosolver-core` and library crate `geosolver_core`.
2. Add dependencies:
   - `num-bigint`
   - `num-rational`
   - `num-integer`
   - `num-traits`
   - no floating-point numeric dependency in production proof/verifier paths.
3. Create all files required by the Base Spec layout.
4. In `src/lib.rs`, expose only the minimal public API names listed in BS-API-001.
5. Add compile-time feature policy:
   - no placeholder feature names such as `use_x`, `with_new_solver`, `phase_x`.
   - no route forcing feature in production.
6. Add a first static test file `tests/anti_simplification_static_tests.rs` that scans production `src/*.rs` for globally banned naming patterns.

### Acceptance

- `cargo test` runs and passes for empty/skeleton implementation where appropriate.
- Public exports are limited to Base Spec API.
- Static test rejects bad names in production code.
- No solver logic returns success yet.

### Stop if

- The Agent introduces public API not listed in BS-API-001 without amending the Base Spec.
- The Agent uses phase/version words in code names.

---

## Phase P2 — Exact algebra core

R-IDs: BS-DATA-001, BS-DATA-002, BS-DATA-003
MECHs: MECH-ALG-Q, MECH-UNI-Q
Reviewer prompt: RP-P2

### Work

1. Implement `Variable` in `variable.rs` exactly as specified.
2. Implement `Monomial` operations in `monomial.rs`:
   - multiplication
   - divisibility
   - quotient if divisible
   - total degree
   - canonical ordering
3. Implement `PolynomialQ` in `polynomial.rs`:
   - exact sparse representation with `BTreeMap`
   - addition, subtraction, multiplication, power, scaling, support, degree
   - `depends_only_on`
   - `to_univariate_in`
   - exact variable-order checking
4. Implement `UniPolynomialQ` and `UniPolynomialFp` in `univariate.rs`:
   - normalization
   - primitive integer normalization
   - Euclidean gcd
   - lcm
   - squarefree part using derivative and gcd
   - conversion to multivariate
5. Add exact algebra tests:
   - zero normalization
   - variable-order mismatch panics/errors
   - primitive normalization
   - gcd/lcm correctness
   - squarefree does not mutate proof target

### Acceptance

- Every polynomial identity test is exact over `BigRational`.
- No production proof/verifier/root-record path uses `f32` or `f64`.
- `PolynomialQ` cannot silently combine different variable orders.
- `UniPolynomialQ::squarefree_part` exists but is not used by any proof function.

### Stop if

- Arithmetic uses floats for equality, proof, root records, verifier, or adoption.
- Polynomial representation is string-based or non-canonical.

---

## Phase P3 — Exact finite-field and linear algebra backends

R-IDs: BS-WIN-001, BS-RES-001, BS-PROOF-003
MECHs: MECH-LINALG-Q, MECH-LINALG-FP
Reviewer prompt: RP-P3

### Work

1. Implement `finite_field.rs`:
   - modular normalization
   - add/sub/mul/pow/inv for prime modulus
   - reject non-prime modulus in constructors used by production.
2. Implement `linear_q.rs`:
   - exact Gaussian elimination over Q
   - solve `M u = b`
   - compute one exact left-null obstruction for inconsistent systems.
3. Implement `linear_fp.rs`:
   - row/column reduction over finite fields
   - nullspace over Fp
   - relation solve for residual target powers.
4. Tests:
   - rational solve with unique, multiple, and inconsistent systems
   - left-null obstruction verifies `λ^T M = 0` and `λ^T b != 0`
   - finite-field null relation tests

### Acceptance

- `linear_q` returns exact rational vectors.
- Inconsistency returns an actual obstruction, not just `None`.
- Finite-field code is not used as proof over Q.

### Stop if

- Modular success is promoted to exact success.
- Inconsistent proof-window solve lacks obstruction support.

---

## Phase P4 — Problem model, compression replay, guard certificates, and verifier foundation

R-IDs: BS-DATA-004, BS-DATA-005, BS-CERT-001, BS-CERT-002, BS-CERT-004, BS-VERIFY-001, BS-VERIFY-002
MECHs: MECH-GUARD, MECH-VERIFY
Reviewer prompt: RP-P4

### Work

1. Implement `TargetProblemQ`, `GuardRecord`, `GuardKind`, `GuardProvenance`.
2. Implement `CertifiedSystemQ` and `CompressionReplayCertificate`.
3. Implement `GuardCertificate` and `NullstellensatzCertificate`.
4. Implement `SolverCertificate` and empty/no-eliminant certificate types.
5. Implement verifier functions:
   - `verify_guard_certificate`
   - `verify_empty_certificate`
   - `verify_no_target_eliminant_certificate` as conservative fail until fallback certificate exists
   - `verify_certificate` top-level dispatch
6. Implement validation/canonicalization:
   - target in variables
   - all polynomial variable orders match
   - semantic guards preserved
   - no semantic guard converted into D without certificate
7. Tests:
   - InputSemanticNonzero verifies only with identical input guard record
   - derived product exact product check
   - Nullstellensatz tamper failure
   - empty `1 = Σ q_i F_i` certificate verifies and tamper fails

### Acceptance

- Verifier recomputes all identities from input and certificate payloads.
- No hash/string/trace is accepted as certificate.
- Unknown `RealInfeasibilityCertificate` payload fails closed.

### Stop if

- Guard product alone is trusted.
- `GuardKind::Positive` or other semantic guard is used as nonzero without a separate certificate.

---

## Phase P5 — Target certificate verifier and exact identity proof acceptance

R-IDs: BS-CERT-003, BS-CERT-004, BS-VERIFY-001, BS-VERIFY-002
MECHs: MECH-TARGET-CERT
Reviewer prompt: RP-P5

### Work

1. Implement `TargetCertificate` and `CompositeRule`.
2. Implement verifier for:
   - IdealMembership
   - RadicalMembership
   - GuardedRadicalMembership
   - CompositeCover/SameIdealGcd
   - CompositeCover/ComponentUnionLcm with explicit component-union source marker.
3. Add tests:
   - `F = {T^2 - 2}` proves `T^2 - 2` by ideal membership.
   - `F = {T^2}` proves `T` by radical membership only if certificate for `T^2` exists.
   - guarded radical refuses missing guard certificate.
   - same-ideal verified covers refine by gcd, not product.
   - component union uses lcm only.
   - tampering support coefficient fails.

### Acceptance

- `verify_certificate(problem, SolverCertificate::TargetCover(cert))` works without solver trace.
- Same-ideal product support is rejected if gcd is required by certificate rule.
- Squarefree support alone is not accepted without certificate.

### Stop if

- Verifier depends on solver trace, planner output, or candidate origin.

---

## Phase P6 — Certificate windows and residual oracle

R-IDs: BS-WIN-001, BS-RES-001, BS-RES-002
MECHs: MECH-WINDOW, MECH-RESIDUAL
Reviewer prompt: RP-P6

### Work

1. Implement `CertificateWindow` and `ProofWindow`.
2. Implement row-closed window construction exactly from support formula.
3. Implement membership matrix and target power matrix construction over Q.
4. Implement modular reduction of matrices for admissible primes.
5. Implement `DenseEchelonResidualOracleFp` behind `ResidualOracleFp` trait.
6. Implement residual-cyclic relation solve.
7. Tests:
   - row closure includes all `support(bF_i)` and target powers
   - residual zero iff vector is in column space on known matrices
   - residual-cyclic finds a modular candidate for `F = {X^2 - 2, T - X}` in a suitable window

### Acceptance

- Residual oracle exposes only trait contract, not matrix backend.
- Candidate output from residual route is not a certificate.
- Static reviewer can trace `M_W -> residual oracle -> r_k -> modular relation -> TargetCandidate`.

### Stop if

- Residual route returns success status or certificate.
- Residual oracle only stores rank/hash.

---

## Phase P7 — Candidate normalization, ranking, direct and residual candidate routes

R-IDs: BS-CAND-001, BS-CAND-002, BS-RES-002, BS-NORM-001, BS-TEST-001
MECHs: MECH-CANDIDATE-PRIMARY
Reviewer prompt: RP-P7

### Work

1. Implement `TargetCandidate`, `CandidateOrigin`, `CandidateTrace`, `CandidateOracle`.
2. Implement direct target equation route.
3. Integrate residual-cyclic route as primary candidate route.
4. Implement normalization, ranking, and factor schedule.
5. Add `#[cfg(test)]` route-forcing harness.
6. Tests:
   - direct target equation route forcing with all other routes disabled
   - residual route forcing with fallback disabled
   - zero candidate discarded
   - squarefree candidate not substituted as proof target
   - ranking affects proof order only

### Acceptance

- `TargetCandidate` contains no certificate.
- Direct and residual routes are top-level reachable.
- Route-forcing tests show top-level candidate path can reach fixed proof later without fallback.

### Stop if

- Candidate origin or score is used as adoption condition.
- Route forcing appears in production API.

---

## Phase P8 — Fixed proof, proof learning, and fairness

R-IDs: BS-PROOF-001 through BS-PROOF-004, BS-LEARN-001, BS-LEARN-002
MECHs: MECH-FIXED-PROOF, MECH-LEARNING
Reviewer prompt: RP-P8

### Work

1. Implement `FixedProofInput`, `CertificateMode`, `ProofFailure`.
2. Implement fair certificate mode schedule.
3. Implement `prove_fixed_target` for ideal, radical, and guarded radical modes.
4. Implement exact left-null obstruction emission.
5. Implement proof-window expansion by obstruction predecessors.
6. Implement initial proof-window learning from modular witness active support.
7. Tests:
   - `F = {X^2 - 2, T - X}` proves `T^2 - 2` by exact multipliers
   - radical proof requires `support_power >= 1`
   - guarded radical refuses unverified guards
   - inconsistent proof window expands by predecessor support
   - unbounded schedule enumerates tuples by weight in a fairness unit test

### Acceptance

- Adoption happens only after Q identity check.
- Modular proof construction, if present, is followed by rational multiplier reconstruction and Q verification.
- Inconsistency is scoped to current window and mode only.

### Stop if

- Proof function accepts modular trace.
- Proof failure `Inconsistent` is treated as global non-membership.

---

## Phase P9 — Low-degree multiple repair and localized Schur repair

R-IDs: BS-REPAIR-001, BS-REPAIR-002
MECHs: MECH-MULTIPLE-REPAIR, MECH-SCHUR-REPAIR
Reviewer prompt: RP-P9

### Work

1. Implement low-degree multiple repair with unknown `A(T)`.
2. Ensure repaired support `P=A*S` is the returned support when certified.
3. Implement obstruction-scope detection for localized Schur repair.
4. Implement local membership equation builder `M_Ω u + N_Ω f = 0`.
5. Schur output may be either exact certificate or proof-window support information.
6. Tests:
   - low-degree multiple returns repaired `P`, not original `S`
   - Schur repair does not run on full system when obstruction scope is local
   - uncertified Schur relation is trace/support info only

### Acceptance

- No full-system Schur outside complete fallback.
- No Schur relation is adopted without exact replay certificate.

### Stop if

- Repair hides fallback delegation.
- Repaired candidate returns old uncertified support.

---

## Phase P10 — Non-primary candidate routes

R-IDs: BS-CAND-003, BS-CAND-004, BS-CAND-005, BS-CAND-006
MECHs: MECH-TOWER, MECH-KRYLOV, MECH-RESULTANT, MECH-SLICE
Reviewer prompt: RP-P10

### Work

1. Implement NormTraceTower structural detection and candidate generation.
2. Implement TargetCyclicKrylov candidate recurrence from target-relevant quotient/residual handle.
3. Implement HiddenVariableSparseResultant candidate route with multi-polynomial template path.
4. Implement SliceSpecialization candidate route with deterministic finite-field affine slices.
5. Add route-forcing tests for each route with all other routes and fallback disabled.
6. Add tamper tests showing generated candidates require fixed proof before success.

### Acceptance

- Each route has a production call chain from solver planning to route body.
- Each route's main computation, not another route, produces its candidate.
- HiddenVariableSparseResultant is not two-polynomial-only.
- Slice agreement is never used as proof.
- Top-level success through another route does not close this phase.

### Stop if

- A route is name-only, trace-only, fixture-shaped, or fallback-only.
- Resultant route rejects all `equations.len() != 2` inputs as hard case.

---

## Phase P11 — Complete target elimination fallback

R-IDs: BS-FALLBACK-001, BS-CERT-004
MECHs: MECH-COMPLETE-FALLBACK
Reviewer prompt: RP-P11

### Work

1. Implement `complete_target_elimination_fallback`.
2. Implement exact target-only elimination for `(I : D^∞) ∩ Q[T]` using exact polynomial elimination without coordinate solution enumeration.
3. Implement certificates for:
   - nonzero target support
   - empty admissible set
   - no nonzero target eliminant
   - resource failure
4. Ensure fallback is called only after candidate/proof/repair routes are exhausted.
5. Tests:
   - fallback can prove a simple target eliminant
   - fallback empty returns `CertifiedEmptyAdmissibleSet`
   - no-target-eliminant does not imply real nonfinite
   - route-forcing tests fail if fallback is reached when disabled

### Acceptance

- Fallback returns only the four allowed result variants.
- No coordinate solution list, full coordinate RUR, or lex parametrization output exists.
- Fallback certificates verify through top-level verifier.

### Stop if

- Fallback becomes a hidden general solver used by candidate routes.

---

## Phase P12 — Dependency DAG, window planner, and top-level solver integration

R-IDs: BS-DAG-001, BS-SOLVER-001, BS-SOLVER-002, BS-SCOPE-002, BS-SCOPE-004
MECHs: MECH-DAG, MECH-SOLVER
Reviewer prompt: RP-P12

### Work

1. Implement target dependency DAG from algebraic footprint only.
2. Implement certificate-window planner with exhaustive degree enumeration under unbounded settings.
3. Implement `solve_target` exactly following Base Spec pseudocode.
4. Implement `refine_and_finalize` using same-ideal gcd.
5. Implement status construction:
   - `CertifiedCandidateCover`
   - `CertifiedExactTargetImage`
   - `CertifiedEmptyAdmissibleSet`
   - `CertifiedNoNonzeroTargetEliminant`
   - failure statuses.
6. Tests:
   - no geometry-name dispatch
   - empty set status is not cover
   - finite-field candidate without proof returns failure
   - fallback called only after routes exhausted
   - same-ideal verified covers gcd-refine

### Acceptance

- Solver path matches pseudocode.
- Status, cover, exact_image, certificate fields are mutually consistent.
- Verifier can verify returned certificate without trace.

### Stop if

- Top-level success can occur without `TargetCertificate` or other exact `SolverCertificate`.

---

## Phase P13 — Exact root isolation and exact-image fail-closed behavior

R-IDs: BS-ROOT-001, BS-IMAGE-001, BS-IMAGE-002, BS-SCOPE-003
MECHs: MECH-ROOTS, MECH-EXACT-IMAGE-SAFETY
Reviewer prompt: RP-P13

### Work

1. Implement exact Sturm or equivalent rational root isolation for squarefree univariate polynomials.
2. Implement `CertifiedCandidateCover` creation with support, squarefree_support, exact root records, and certificate.
3. Implement exact-image classifier interface.
4. Implement `MaybeClassifyExactTargetImage` fail-closed control flow.
5. If a general real-fiber classifier is not implemented, implement conservative incomplete result.
6. Tests:
   - `T^2 - 2` isolates two exact rational intervals
   - no float-only root records
   - partial classification is trace only
   - `RequireExactImage` fails closed when classifier incomplete
   - `TryExactImage` returns candidate cover when classifier incomplete

### Acceptance

- `CertifiedCandidateCover` includes exact real root records.
- `CertifiedExactTargetImage` cannot be built with missing root classifications.
- Root intervals are rational and exactly isolating.

### Stop if

- Numerical sampling or floating approximations are accepted as certificates.
- Unclassified roots are dropped.

---

## Phase P14 — Full verification matrix and anti-simplification audit

R-IDs: all active R-IDs, especially BS-FORBID-001, BS-FORBID-002, BS-TEST-001 through BS-TEST-003
Reviewer prompt: RP-P14

### Work

1. Run all tests.
2. Add or update route-forcing test matrix in `docs/ai/changes/cw-arc-dtp-q/evidence/route_forcing_matrix.md`.
3. Add non-simplification manifests for each route:
   - DirectTargetEquation
   - ResidualCyclic
   - NormTraceTower
   - TargetCyclicKrylov
   - HiddenVariableSparseResultant
   - SliceSpecialization
   - LocalizedSchur
   - CompleteTargetEliminationFallback
4. Each manifest must state production call chain, controlling data-flow, forbidden patterns searched, exact replay oracle, route-forcing tests, and tamper tests.
5. Run static anti-simplification scans.
6. Run final reviewer prompts, including RP-FINAL.

### Acceptance

- All route-forcing tests pass with other routes and fallback disabled.
- All tamper tests fail as expected.
- Static scans find no forbidden production patterns.
- Reviewers inspect production code directly and cite functions/files.
- No open high-impact QuestionDebt.

### Stop if

- Any route closes only because top-level solver passed through a different route.
- Any reviewer relies only on evidence documents instead of production code.

---

## Phase P15 — Closure

R-IDs: all active R-IDs
Reviewer prompt: RP-CLOSURE

### Work

1. Create `docs/ai/changes/cw-arc-dtp-q/CLOSURE.md`.
2. State exact claim supported by evidence.
3. Include:
   - git commit hash or working tree state
   - commands run and results
   - route-forcing matrix
   - tamper matrix
   - reviewer summaries
   - residual limitations
   - exact-image claim boundary
4. Do not claim `CertifiedExactTargetImage` general completion unless all real-fiber classification R-IDs were actually implemented and reviewed.

### Acceptance

- Closure claim is no stronger than evidence.
- Closure distinguishes candidate-cover completion from exact-image completion.
- Closure notes any resource-bound limitations.

### Stop if

- Closure says production-safe or fully verified without fresh evidence and reviewer support.
