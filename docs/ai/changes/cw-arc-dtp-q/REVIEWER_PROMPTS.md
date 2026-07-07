# CW-ARC-DTP-Q Guardian Reviewer Prompts

Spec ID: `CW-ARC-DTP-Q-CANDIDATE-COVER`
Type: Reviewer Prompt Pack
Status: Draft for user approval
Base Spec: `CW_ARC_DTP_Q_GUARDIAN_BASE_SPEC.md`
Plan: `CW_ARC_DTP_Q_GUARDIAN_PLAN.md`

---

## Common reviewer instructions for every phase

You are a read-only adversarial reviewer. Your job is not to help the implementer pass. Your job is to find the shortest concrete evidence that the implementation does not satisfy the approved Base Spec and Plan.

Do not rely on:

```text
- Agent summaries.
- Source-to-code maps.
- Closure tables.
- Audit scripts.
- Test names.
- Prior reviewer PASS.
- Comments claiming an algorithm is implemented.
```

Use them only as pointers. Inspect production code directly.

For every PASS or FAIL, your review must include:

```text
1. Files inspected.
2. Functions/types inspected.
3. Public/planner call chain checked.
4. Data-flow checked.
5. Forbidden patterns searched.
6. Route-forcing evidence checked when relevant.
7. Certificate tamper/replay evidence checked when relevant.
8. Remaining uncertainty.
9. Verdict: PASS or FAIL.
```

Global FAIL conditions:

```text
- Any success path lacks exact certificate verification.
- Candidate traces, modular rank, random seed, score, or planner estimate are accepted as proof.
- Verifier trusts hashes, strings, source maps, or solver trace instead of recomputing exact identities.
- A general route is only a toy/slice/two-polynomial/fixture path.
- A route is implemented by hidden delegation to another route or fallback.
- Production code branches on geometry names, fixture names, expected answers, or official solutions.
- Production path builds full coordinate solution lists, full coordinate RUR, or full coordinate lex parametrization to read T.
- Complete fallback is called before admitted candidate/proof routes are exhausted, except in tests that explicitly target fallback.
- Any phase is closed because tests pass but production control-flow/data-flow does not match the Base Spec.
```

---

## RP-P0 — Guardian scaffold and source anchoring

Review target: docs only.

PASS only if:

```text
- BASE_SPEC.md and PLAN.md are present under docs/ai/changes/cw-arc-dtp-q/.
- source_map.md classifies primary algorithm spec as EXACT.
- failure-analysis docs are treated as generalized anti-pattern authority.
- ACTIVE_CONTEXT.md is short and operational, not a replacement for the Base Spec.
- No implementation code has been written before user approval.
```

FAIL if:

```text
- source_map.md weakens the uploaded CW-ARC-DTP-Q spec.
- summaries are made authoritative over the Base Spec.
- implementation starts before approval.
```

---

## RP-P1 — Crate skeleton and naming

Inspect:

```text
Cargo.toml
src/lib.rs
all src file names
tests/anti_simplification_static_tests.rs
```

PASS only if:

```text
- Package/crate naming follows geosolver-core/geosolver_core.
- Modules are domain-named and snake_case.
- Public exports are limited to Base Spec public API.
- No phase/version/temporary names appear in production code.
- Static anti-simplification test exists and scans production files.
```

FAIL if:

```text
- Production API exposes route forcing.
- Files or code names include phase numbers, v2/new/legacy/temp/hack/toy/fixture/expected.
- Extra public API appears without a Base Spec amendment.
```

---

## RP-P2 — Exact algebra core

Inspect:

```text
src/variable.rs
src/monomial.rs
src/polynomial.rs
src/univariate.rs
tests/exact_algebra_tests.rs
```

Data-flow to verify:

```text
Variable order -> Monomial exponent vector -> PolynomialQ BTreeMap terms -> exact operations -> UniPolynomialQ conversion.
```

PASS only if:

```text
- Polynomial arithmetic is exact over BigRational.
- BTreeMap or equivalent canonical ordering is used.
- Different variable orders cannot be silently merged.
- squarefree_part is separate from proof target.
- gcd/lcm are exact univariate operations.
```

FAIL if:

```text
- f32/f64 are used in proof-relevant algebra.
- polynomial equality depends on string formatting.
- HashMap iteration order is canonical output.
- variable names are inspected for geometry semantics.
```

---

## RP-P3 — Linear algebra

Inspect:

```text
src/finite_field.rs
src/linear_q.rs
src/linear_fp.rs
related tests
```

Data-flow to verify:

```text
Exact matrix -> Gaussian elimination -> solution or exact left-null obstruction.
Finite-field matrix -> modular relation -> trace/candidate only.
```

PASS only if:

```text
- Q linear solve returns exact rational solution or exact obstruction.
- Obstruction verifies λ^T M = 0 and λ^T b != 0.
- Fp solve is not used as proof over Q.
```

FAIL if:

```text
- inconsistent systems return only boolean failure without obstruction.
- modular rank stability is treated as exact proof.
- modular computations write TargetCertificate directly.
```

---

## RP-P4 — Problem, compression, guards, verifier foundation

Inspect:

```text
src/problem.rs
src/compression.rs
src/guards.rs
src/certificates.rs
src/verifier.rs
related tests
```

PASS only if:

```text
- Input semantic guards remain separate from algebraic proof.
- InputSemanticNonzero checks identical input GuardRecord with NonZero kind.
- AlgebraicNonvanishing recomputes 1 = Σ q_i F_i + q_g guard.
- DerivedProduct recursively verifies factors and exact product.
- Unknown real certificate payload fails closed.
```

FAIL if:

```text
- guard_product is trusted without factor certificates.
- Positive/Negative guards are used as nonzero without proof.
- Compression replay can contain unreplayable external CAS simplification.
```

---

## RP-P5 — Target certificate verifier

Inspect:

```text
src/certificates.rs
src/verifier.rs
tests/verifier_tests.rs
```

PASS only if:

```text
- IdealMembership verifies H - Σ q_i F_i == 0 over Q.
- RadicalMembership verifies support^a identity with a >= 1.
- GuardedRadicalMembership verifies guards, guard product, and Q identity.
- Composite SameIdealGcd recomputes gcd of verified child supports.
- ComponentUnionLcm requires explicit component-union semantics.
- Tamper tests fail.
```

FAIL if:

```text
- TargetCertificate verification uses solver trace.
- certificate identity field stores a claimed zero residual that is trusted.
- squarefree support is accepted as proof target without its own certificate.
```

---

## RP-P6 — Windows and residual oracle

Inspect:

```text
src/window.rs
src/residual.rs
src/candidate_residual.rs
tests/residual_window_tests.rs
```

Data-flow to verify:

```text
system equations + multiplier supports -> row-closed C -> M_W and N_d -> Fp reduction -> residual oracle -> r_k -> modular relation -> TargetCandidate.
```

PASS only if:

```text
- Row closure is recomputed exactly from supports.
- ResidualOracleFp satisfies reduce(v)=0 iff v is in column space for tested matrices.
- residual-cyclic route returns TargetCandidate, not TargetCertificate.
- Matrix backend is hidden behind residual trait.
```

FAIL if:

```text
- residual oracle is only a rank/hash check.
- residual route can set SolverStatus directly.
- C is caller-trusted without recomputation.
```

---

## RP-P7 — Primary candidate routes

Inspect:

```text
src/candidates.rs
src/candidate_direct.rs
src/candidate_residual.rs
src/normalize.rs
src/test_support.rs
tests/candidate_route_forcing_tests.rs
```

PASS only if:

```text
- CandidateOracle returns only candidates and traces.
- Direct target equation route checks target-only equations structurally.
- Residual route is reachable from solver planning.
- Normalization clears denominators/content and keeps proof target distinct from squarefree root support.
- Route forcing exists only under cfg(test).
```

FAIL if:

```text
- candidate origin, score, or repeated-prime count is accepted as proof.
- public SolverOptions exposes route forcing.
- top-level pass through fallback is used to close direct/residual route.
```

---

## RP-P8 — Fixed proof and learning

Inspect:

```text
src/proof.rs
src/proof_learning.rs
tests/fixed_proof_tests.rs
```

Data-flow to verify:

```text
candidate S + mode + proof window -> H = D^e S^a -> C_H -> M_H u = vec(H) -> q_i -> exact identity check -> TargetCertificate.
```

PASS only if:

```text
- Q identity check is the only adoption gate.
- Guarded mode constructs D only from verified guard certificates.
- left-null obstruction is emitted for inconsistent windows.
- expansion by obstruction predecessors follows Pred_F(r).
- fair schedule covers support power, guard power, and support degree.
```

FAIL if:

```text
- modular reconstruction is accepted without Q identity.
- proof window inconsistency is treated as S not in I.
- semantic guards are used in D without certificates.
```

---

## RP-P9 — Repairs

Inspect:

```text
src/repair_multiple.rs
src/repair_schur.rs
related tests
```

PASS only if:

```text
- low-degree repair returns certified P=A*S, not uncertified S.
- Schur repair scope is derived from obstruction incidence.
- Schur output without exact certificate is only support information/trace.
- Full-system Schur appears only in final fallback if at all.
```

FAIL if:

```text
- repair delegates to complete fallback while claiming repair route.
- local Schur uses all equations without obstruction-local reason.
- uncertified Schur relation becomes solver success.
```

---

## RP-P10 — Non-primary candidate routes

Inspect:

```text
src/candidate_tower.rs
src/candidate_krylov.rs
src/candidate_resultant.rs
src/candidate_slice.rs
tests/candidate_route_forcing_tests.rs
```

Required call-chain evidence:

```text
solver/window planner -> route function -> route-specific algebraic core -> TargetCandidate -> fixed proof later.
```

PASS only if:

```text
- NormTraceTower detects monic triangular tower structurally.
- TargetCyclicKrylov recurrence comes from target-relevant quotient/residual handle.
- HiddenVariableSparseResultant has a multi-polynomial template path.
- SliceSpecialization never uses slice agreement as proof.
- Each route has route-forcing test with other routes and fallback disabled.
```

FAIL if:

```text
- Resultant route rejects all equations.len() != 2 inputs.
- Resultant route is only univariate Sylvester but claims sparse/multi-polynomial resultant.
- Any route calls Groebner/fallback and labels output as its own candidate.
- Tests prove only top-level success through another route.
- Route uses fixture-shaped detection.
```

---

## RP-P11 — Complete fallback

Inspect:

```text
src/fallback_elimination.rs
src/solver.rs
related tests
```

PASS only if:

```text
- Fallback computes only target elimination statement `(I:D^∞) ∩ Q[T]` or exact certificates about it.
- It returns only CertifiedSupport, CertifiedEmpty, CertifiedNoTargetEliminant, or ResourceFailure.
- It is called only after candidate/proof/repair routes are exhausted.
- It does not enumerate coordinate solutions or produce full coordinate RUR.
```

FAIL if:

```text
- Fallback is reachable from candidate routes.
- NoTargetEliminant is treated as real nonfinite.
- Empty is returned as cover with S=1.
```

---

## RP-P12 — Top-level solver integration

Inspect:

```text
src/dependency_dag.rs
src/solver.rs
src/options.rs
src/trace.rs
related tests
```

PASS only if:

```text
- Target Dependency DAG uses only algebraic footprint.
- Window planner produces row-closed windows and exhaustive degree schedule under unbounded settings.
- solve_target control-flow matches Base Spec pseudocode.
- Success statuses always carry matching SolverCertificate.
- Candidate cover finalization uses gcd for same ideal.
```

FAIL if:

```text
- geometry strings influence planner/solver.
- fallback order is changed to pass tests cheaply.
- status and certificate disagree.
- success can occur without verifier-acceptable certificate.
```

---

## RP-P13 — Roots and exact-image safety

Inspect:

```text
src/roots.rs
src/exact_image.rs
src/solver.rs
tests/root_isolation_tests.rs
tests/solver_status_tests.rs
```

PASS only if:

```text
- Root isolation uses exact rational intervals.
- Each AlgebraicRealRoot interval contains exactly one real root by exact check.
- CertifiedCandidateCover includes support, squarefree support, exact roots, and TargetCertificate.
- CertifiedExactTargetImage requires all roots classified.
- TryExactImage and RequireExactImage fail closed when classifier incomplete.
```

FAIL if:

```text
- root records are floating approximations only.
- unclassified roots are dropped.
- partial classification is returned as CertifiedExactTargetImage.
```

---

## RP-P14 — Full anti-simplification audit

Inspect all production code and tests.

Mandatory searches:

```text
Unsupported
expected
fixture
circle
distance
area
incircle
mixtilinear
orthic
RUR
coordinate_solution
solve_all
lex_param
len() != 2
polynomials.len() != 2
f64
f32
TODO
panic!("unsupported")
```

PASS only if:

```text
- Hits are absent or strictly confined to tests/docs with explicit anti-pattern checks.
- Each route has non-simplification manifest and reviewer verified it in code.
- Route-forcing matrix proves routes independently.
- Tamper matrix proves certificates are real.
- No route closes from top-level success via another route.
```

FAIL if:

```text
- Any known-bad pattern appears in production control-flow.
- Manifests are accepted without code inspection.
- An algorithm is present by name/type only.
```

---

## RP-FINAL / RP-CLOSURE — Final closure review

Inspect:

```text
docs/ai/changes/cw-arc-dtp-q/CLOSURE.md
all evidence files
all review files
all production source files changed
all tests
```

PASS only if closure claim exactly matches evidence.

Required final statement shape:

```text
This implementation completes the CW-ARC-DTP-Q certified candidate-cover core as specified by R-IDs ... . It returns CertifiedCandidateCover only with exact TargetCertificate verification. It includes exact root isolation and exact-image fail-closed control flow. It does not claim general CertifiedExactTargetImage completion unless all real-fiber classifier requirements were implemented and reviewed.
```

FAIL if closure claims:

```text
- general exact target image without complete real-fiber classification.
- production-safe if reviewers did not inspect production code.
- verified route correctness based on top-level tests only.
- source-faithful while source_map is missing/incomplete.
```
