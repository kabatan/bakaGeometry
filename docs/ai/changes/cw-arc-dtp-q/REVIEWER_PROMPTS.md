# 03. Reviewer Prompts: CW-ARC-DTP-Q Full Implementation v3

## 0. Universal reviewer prompt

Use this at the start of every review.

```text
あなたの仕事は PASS を出すことではない。
Agent の実装が Base Spec を満たしていない最短の証拠を探すことである。
Agent が作った evidence、audit、closure table、test 名、manifest、過去 reviewer の PASS は信用しない。それらは source を探す pointer としてだけ使う。

必ず production code を読む。`#[cfg(test)]` 内だけの実装、doc だけの主張、trace だけの主張では PASS してはならない。

次を必ず出力する:

Verdict: PASS / FAIL
Files inspected:
Functions inspected:
Production call chain checked:
Data-flow checked:
Certificate replay checked:
Forbidden patterns searched:
Route-forcing tests checked:
Tamper tests checked:
Adversarial input families considered:
Remaining uncertainty:
Reason:

FAIL すべき条件:
- function/type/test の名前だけ仕様に似ている。
- certificate struct はあるが verifier が exact identity / replay を再検証していない。
- route が別 route / complete fallback / Groebner に隠れて委譲している。
- route が fixture-shaped / two-polynomial-only / single-equation-only / first-prime-only である。
- fail-closed stub を final completion として扱っている。
- bounded search を complete algorithm と呼んでいる。
- top-level success だけで route correctness を主張している。
- guard や replay が production path で落ちている。

PASS は、source-level control-flow と data-flow を説明でき、forbidden patterns を探し、route-forcing と tamper evidence を production code と照合できた場合だけ出す。
```

## P7-P13 mandatory route-closure addendum

Apply this addendum to every P7-P13 review.

```text
FAIL if P7-P12 route closure is justified by saying P4-P6 primitives exist. Shared primitives are only prerequisites; every route needs its own production control-flow/data-flow inspection.

For each P7-P12 candidate route, require all four evidence classes:
- route-forcing with only that origin enabled;
- complete fallback disabled;
- exact-proof-gate evidence showing the candidate is not adopted without fixed proof and verifier replay;
- meaningful tamper test for the route certificate, replay data, or candidate evidence.

Treat `FairProofSchedule::unbounded()` and final top-level unbounded ideal execution as separate claims. PASS for the iterator alone must not imply PASS for solver-level unbounded orchestration.

In every phase, inspect factor scheduling and candidate adoption for false completion. FAIL if `FactorizationResult::ResourceFailure` or `FactorizationResult::Partial` can be handled as `Complete`, directly or through fallback cloning.

`origin_evidence` is ranking evidence only. FAIL if origin count, merged origins, or repeated route observations are used as an adoption condition, certificate authority, or proof shortcut.
```

## P0 reviewer prompt — Authority lock and gap inventory

```text
Review P0.

Check that Base Spec v3, Plan v3, and Reviewer Prompts v3 are copied into docs.
Inspect `current_gap_inventory.md`.
Do not accept a gap inventory that only says "not applicable" or "safe fail-closed".

You must search source for these patterns and equivalents:
- guard_certificates: Vec::new()
- semantic_guards: Vec::new()
- classify_real_fibers / Incomplete
- complete_target_elimination_fallback
- max_window_degree.unwrap_or
- NoTargetEliminant monomial ideal special case
- reconstruct_from_modular_support / first prime
- factor_schedule returning clone only
- localized_schur_repair returning SupportInformation only
- ExactTargetImage unhandled
- `TODO`, `unimplemented`, `not available`, `Unsupported`, normal-path `ImplementationBug`

FAIL if any known gap is omitted from inventory.
PASS only if every gap is assigned to a later phase that replaces the controlling path, not patches around it.
```

## P1 reviewer prompt — Algebra primitives

```text
Review P1.

Inspect exact polynomial, univariate, finite field, matrix, CRT, and rational reconstruction code.

FAIL if:
- univariate factorization is only squarefree_part wrapped in a Vec.
- rational reconstruction uses only one prime where multi-prime reconstruction is required.
- Q solver inconsistent result lacks a verifiable left-null obstruction.
- Fp solver cannot expose nonzero solution support.
- arithmetic depends on HashMap iteration order for canonical output.

Required evidence from source:
- Call chain from candidate normalization to CRT/rational reconstruction.
- Tests where coefficient height exceeds a single prime.
- Tests where left-null obstruction verifies λ^T M = 0 and λ^T b != 0.
```

## P2 reviewer prompt — Problem, compression, guards

```text
Review P2.

Inspect `TargetProblemQ`, `CertifiedSystemQ`, `certified_system_from_problem`, guard construction, and replay verification.

FAIL if:
- production `CertifiedSystemQ` construction always returns empty `guard_certificates`.
- semantic NonZero guards are not converted into `InputSemanticNonzero` certificates.
- compression replay is empty after canonicalization or zero-removal.
- proof/guard verification creates a dummy problem with `semantic_guards: Vec::new()`.
- guard provenance is ignored when verifying InputSemanticNonzero.

You must verify a source call chain:
`solve_target` -> validation/canonicalization -> certified compression -> guard certificates -> guarded proof verifier.

Tamper checks required:
- Change guard polynomial: verifier rejects.
- Change guard kind from NonZero: verifier rejects.
- Remove replay step after normalization: verifier rejects or compression verification fails.
```

## P3 reviewer prompt — Certificate verifier

```text
Review P3.

Inspect `verify_certificate` and every helper it calls.

FAIL if:
- ExactIdentityKind is trusted without recomputing polynomial identities.
- GuardedRadicalMembership does not recompute guard_product from verified guards.
- `ExactTargetImage` is rejected as "not handled" in final code.
- `NoNonzeroTargetEliminant` verifier is monomial-only in final code.
- EmptyAdmissibleSet can be represented as support polynomial 1.
- Any solver success certificate cannot be verified from original TargetProblemQ alone.

You must describe exact equations checked for:
- IdealMembership
- RadicalMembership
- GuardedRadicalMembership
- EmptyAdmissibleSet
- ExactTargetImage
- NoNonzeroTargetEliminant
```

## P4 reviewer prompt — Window and residual oracle

```text
Review P4.

Inspect row-closed window construction, membership matrix construction, target power matrix, modular reduction, and residual oracle.

FAIL if:
- caller-provided row_monomials can affect matrix rows without recomputation.
- residual reduce(v)=0 iff v ∈ col(M) is not tested.
- denominator-admissible prime filtering is missing.
- residual oracle stores decorative trace not used by ResidualCyclic.

You must check tests for forged windows and residual oracle quotient behavior over multiple primes.
```

## P5 reviewer prompt — Fixed proof and fair schedule

```text
Review P5.

Inspect `prove_fixed_target`, proof schedule, proof learning, and obstruction expansion.

FAIL if:
- successful proof does not recompute H - Σ q_i F_i exactly.
- GuardedRadical builds D without verifying guard certificates.
- unbounded mode can loop forever over windows and never reaches fair proof tuple enumeration or complete fallback behavior.
- schedule is fair over powers but not support degree.
- obstruction expansion does not add Pred_F(row) supports.

You must produce:
- proof target formula for each CertificateMode from code.
- fairness argument for all finite tuples (d_B,a,e) in unbounded mode.
- line-level evidence that exact identity check gates certificate construction.
```

## P6 reviewer prompt — Candidate normalization and factor schedule

```text
Review P6.

Inspect candidate normalization, modular merge, rational reconstruction, ranking, and factor schedule.

FAIL if:
- first modular prime is lifted directly to Q in the multi-prime route.
- single-prime heuristic is ranked as certified reconstruction.
- factor_schedule returns only the original candidate in final implementation.
- squarefree support replaces proof candidate without separate certificate.
- ranking result can become adoption condition.

Check tests:
- high coefficient CRT reconstruction.
- reducible polynomial factor trial.
- squarefree part is root isolation only.
```

## P7 reviewer prompt — DirectTargetEquation

```text
Review P7.

Also apply the P7-P13 mandatory route-closure addendum.

FAIL if:
- route uses variable name string rather than Variable equality.
- direct candidate is returned as certified without fixed proof.
- route-forcing test still permits other routes or complete fallback.

Check source call chain from DirectTargetEquation oracle to fixed proof gate.
```

## P8 reviewer prompt — ResidualCyclic

```text
Review P8.

Also apply the P7-P13 mandatory route-closure addendum.

Inspect residual-cyclic route in production code.

Required data-flow:
M_W and N_d -> modular reduction -> residual oracle -> relation nullspace -> modular multiplier solve -> active support -> CRT/reconstruction -> candidate -> fixed proof.

FAIL if:
- active_multiplier_supports is copied from the full window.
- relation coefficients are not derived from residual nullspace.
- modular multiplier solve M u = N c is absent.
- reconstruction is first-prime-only.
- prime admissibility ignores denominators/guards/replay.
- exact proof gate can be bypassed.

Check route-forcing tests RC-F1..RC-F4 and spurious candidate rejection.
```

## P9 reviewer prompt — TargetCyclicKrylov

```text
Review P9.

Also apply the P7-P13 mandatory route-closure addendum.

Inspect quotient/residual handle and recurrence computation.

FAIL if:
- route is only a Q linear dependence search but called Krylov with no quotient handle.
- route calls Groebner or complete fallback.
- recurrence is accepted without fixed proof.
- tests only show same top-level problem solved by another route.

You must explain how residual classes of 1,T,T^2,... produce recurrence coefficients in source.
```

## P10 reviewer prompt — HiddenVariableSparseResultant

```text
Review P10.

Also apply the P7-P13 mandatory route-closure addendum.

This is adversarial. Look for narrow route capture.

FAIL if any production path:
- rejects all polynomial_count != 2 cases without a separate general path;
- computes only Sylvester resultant and calls it sparse resultant;
- delegates candidate generation to complete fallback or Groebner;
- does not build a sparse Macaulay/resultant template;
- cannot handle a 3+ polynomial conformance family;
- produces candidate from something other than determinant/minor/null-relation of the template;
- treats resultant candidate as proof.

Required reviewer output:
- File/function where template rows are constructed.
- File/function where columns are constructed.
- File/function where determinant/minor/null relation is computed.
- File/function where CRT/reconstruction occurs.
- Route-forced 3+ polynomial test name.
- Tamper test name.
```

## P11 reviewer prompt — SliceSpecialization

```text
Review P11.

Also apply the P7-P13 mandatory route-closure addendum.

FAIL if:
- route substitutes values into each equation independently and returns single-equation target polynomials.
- sliced system does not contain all original equations plus slice equations.
- slice gcd or repeated slice agreement is used as adoption evidence.
- slice assignment schedule is hard-coded to one/two slices without resource schedule.
- fixed proof in original unsliced system is absent.

You must trace:
original system -> slice equations -> sliced system candidate subroute -> original-system fixed proof.
```

## P12 reviewer prompt — NormTraceTower

```text
Review P12.

Also apply the P7-P13 mandatory route-closure addendum.

FAIL if:
- only monic coefficient 1 tower is implemented while claiming guarded/nonmonic tower support.
- nonmonic leading coefficient is inverted without GuardCertificate.
- target expression must be coefficient ±1 only.
- quotient basis and reduction by tower are not exact.
- characteristic polynomial code lacks rational/nontrivial tower tests.

Check NT-F1..NT-F3 route-forced tests.
```

## P13 reviewer prompt — LocalizedSchur

```text
Review P13.

Also apply the P7-P13 mandatory route-closure addendum. For P13, route-forcing and no-fallback evidence apply to LocalizedSchur only when it returns a replayable exact target certificate; support information alone is not solver success.

FAIL if:
- localized_schur_repair always returns SupportInformation.
- no conformance family exists where Schur returns exact certificate.
- local relation is accepted without replay to original system.
- full-system Schur is used outside complete fallback.
- obstruction scope is not derived from row/equation incidence.

You must explain:
obstruction -> scope Ω -> boundary variables -> local membership equation -> target-only relation or support info.
```

## P14 reviewer prompt — Exact elimination substrate

```text
Review P14.

FAIL if:
- Groebner basis polynomials lack representation as combinations of original generators.
- exact elimination engine enumerates coordinate solutions or builds full coordinate RUR.
- resource traces are not derived from actual pair/matrix counts.
- basis tamper tests do not fail.

You must verify representation replay for at least one nontrivial basis polynomial.
```

## P15 reviewer prompt — CompleteTargetEliminationFallback

```text
Review P15.

This phase must close all complete fallback gaps.

FAIL if:
- complete fallback is bounded by max_window_degree rather than exact target elimination.
- saturation via U*D - 1 or equivalent exact ideal quotient is absent when guards exist.
- support certificate does not translate saturation back to D^e S^a = Σ q_i F_i.
- CertifiedNoNonzeroTargetEliminant is monomial-only or heuristic.
- no-target-eliminant verifier does not replay exact elimination-zero certificate.
- empty ideal returns support 1 instead of CertifiedEmptyAdmissibleSet.

You must inspect CTE-F1..CTE-F5 tests and provide code-level data-flow for each returned variant.
```

## P16 reviewer prompt — Exact image

```text
Review P16.

This phase must close exact image. Fail-closed stub is not enough.

FAIL if:
- classify_real_fibers always returns Incomplete.
- ExactTargetImage verifier is unhandled.
- nonempty certificate uses floats, approximate roots, or sample strings.
- empty certificate is a text description without replay.
- not all roots of squarefree_support are classified exactly once.
- semantic guard signs are ignored.

You must inspect:
- algebraic real representation;
- exact sign evaluation;
- CAD/RCF or equivalent feasibility engine;
- Nonempty certificate verifier;
- Empty certificate verifier;
- EI-F1..EI-F4 tests.

PASS only if RequireExactImage returns CertifiedExactTargetImage for at least one nontrivial conformance family.
```

## P17 reviewer prompt — Top-level solver integration

```text
Review P17.

Inspect `solve_target`.

FAIL if:
- hidden fallback exists that is not in Base Spec.
- complete fallback is unreachable in bounded/unbounded controlled failure cases.
- any success status lacks verifier-accepted certificate.
- status fields are inconsistent with Base Spec.
- solver options include geometry names, fixture selectors, or algorithm hacks beyond resource/exact image controls.
- normal failures panic with assert instead of returning status.

You must trace input -> compression -> routes -> proof -> cover/exact image/fallback -> certificate.
```

## P18 reviewer prompt — Route-forcing matrix

```text
Review P18.

FAIL if:
- route-forcing harness exists only as docs.
- route-forcing tests still allow other routes.
- route-forcing success depends on complete fallback.
- any route lacks route-only success and route-only rejection/spurious test.
- trace assertions are too weak, e.g. only `candidate:` appears without origin isolation.

Do not accept top-level success as route correctness.
```

## P19 reviewer prompt — Non-simplification manifest

```text
Review P19.

Read the manifest only as a checklist, then verify every item in production source.

FAIL if manifest says "not fallback" but source calls fallback.
FAIL if manifest says "not certificate shell" but verifier does not recompute exact identity.
FAIL if manifest says "general" but source has hard-coded structural restrictions.
FAIL if manifest lacks data-flow proof for any route.

You must independently search forbidden patterns and include results in review output.
```

## P20 reviewer prompt — Adversarial regression suite

```text
Review P20.

FAIL if tests use geometry family names, problem IDs, or expected-answer branches.
FAIL if algebraic stress families are toy-only T^2-c cases.
FAIL if guard/saturation, positive-dimensional finite target, bilinear determinant-like, Gram-like, sparse incidence, and tower structures are absent.
FAIL if resource failure tests are indistinguishable from unsupported scope.

Ensure tests exercise algebraic IR structures, not geometry DSL lowering.
```

## P21 reviewer prompt — Final disqualifier scan

```text
Review P21.

Run static search and source inspection for final disqualifiers:
- TODO / unimplemented / not available / Unsupported in production path
- always Incomplete exact image
- exact image verifier unhandled
- monomial-only no-target eliminant
- first-prime reconstruction
- bounded complete fallback
- dropped guards
- dummy empty semantic guards
- factor_schedule clone-only
- Schur support-only
- polynomial_count == 2 hard general route gate
- single-equation slice route
- hidden full coordinate RUR / full coordinate solution enumeration

FAIL if any remains in final production path.
```

## P22 reviewer prompt — Final review

```text
Perform final adversarial review.

You must not rely on previous PASS results. Re-sample source across all critical paths:
- compression and guards;
- verifier;
- residual candidate;
- sparse resultant;
- slice;
- tower;
- Krylov;
- Schur;
- fixed proof;
- complete fallback;
- exact image;
- solve_target integration.

For each, answer:
1. What is the production entry?
2. What data controls the computation?
3. What certificate is produced?
4. How does verifier replay it?
5. What would make it fail if tampered?
6. Which forbidden simplification did you search for?

Final PASS requires no unresolved QuestionDebt, no CertificateDesignGap in required variants, no final disqualifier, and all route-forcing/tamper/adversarial tests present.
```
