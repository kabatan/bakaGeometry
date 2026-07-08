# 02. Plan: CW-ARC-DTP-Q Full Implementation v3

## 0. Plan contract

この Plan は `01_BASE_SPEC_CW_ARC_DTP_Q_FULL_V3.md` を完全に実装するための作業計画である。Plan のどの phase も Base Spec を弱めてはならない。

Agent は次を守る。

```text
P0. Base Spec とこの Plan に曖昧さを見つけたら QuestionDebt として停止する。
P1. 実装困難を理由に scope を狭めてはならない。
P2. 既存実装が Base Spec より弱い場合、弱い path に evidence を足すのではなく controlling path を置き換える。
P3. Phase completion は docs / tests / reviewer PASS だけではない。production code の semantic closure が必要である。
P4. Reviewer prompt は phase ごとに必ず適用する。
P5. Each phase must update `non_simplification_manifest.md`, but the manifest is not evidence by itself.
P6. If a final disqualifier from Base Spec §15 remains, final phase cannot close.
```

## 1. Repository initialization and authority lock

### P0 — Authority lock and current implementation quarantine

Goal: establish that Base Spec v3 is the only implementation authority.

Actions:

```text
1. Copy Base Spec, Plan, Reviewer Prompts into `docs/ai/changes/cw-arc-dtp-q/`.
2. Create `docs/ai/changes/cw-arc-dtp-q/evidence/current_gap_inventory.md`.
3. Search current repo for final disqualifier patterns:
   - empty guard construction
   - always-incomplete exact image
   - bounded complete fallback
   - first-prime reconstruction
   - factor_schedule clone-only
   - Schur support-only
   - ExactTargetImage verifier unhandled
   - NoTargetEliminant monomial-only verifier
4. Mark each finding as `replace`, not `patch`, unless already fully conformant.
```

Acceptance:

```text
- Gap inventory exists and lists every disqualifier found.
- No phase is allowed to close by saying current implementation is acceptable unless reviewer confirms Base Spec data-flow.
```

Reviewer: use P0 prompt.

## 2. Algebra and verification foundation

### P1 — Exact algebra primitives closure

Goal: exact polynomial, matrix, finite field, CRT, rational reconstruction foundation.

Actions:

```text
1. Implement/verify sparse multivariate Q polynomial operations.
2. Implement/verify univariate Q operations: primitive normalization, gcd, lcm, squarefree, exact factorization for conformance families.
3. Implement finite field prime arithmetic with denominator admissibility.
4. Implement Q linear solver with left-null obstruction.
5. Implement Fp linear solver with solution, nullspace, rank, and active nonzero support extraction.
6. Implement CRT and rational reconstruction with explicit failure.
7. Add algebra property tests.
```

Acceptance:

```text
- Univariate `factor_squarefree_over_q` is not clone/squarefree-only.
- CRT reconstruction test includes coefficient height larger than one prime.
- Q solver inconsistent case emits valid left-null obstruction.
- Fp solver can recover active multiplier support from a solution vector.
```

Reviewer: use P1 prompt.

### P2 — Problem validation, certified compression, and guard construction

Goal: production `CertifiedSystemQ` no longer drops guards or replay.

Actions:

```text
1. Implement `validate_target_problem`.
2. Implement canonicalization with primitive normalization and zero removal replay.
3. Implement `certified_system_from_problem`:
   - transfer NonZero semantic guards to `InputSemanticNonzero` certificates;
   - include replay/canonicalization evidence;
   - preserve original variable order and target.
4. Implement `verify_compression_replay`.
5. Add tests where semantic NonZero guard reaches guarded proof mode.
6. Add tamper tests for guard record mismatch and replay mutation.
```

Acceptance:

```text
- A problem with NonZero guard yields nonempty `system.guard_certificates`.
- Changing guard polynomial or provenance makes verification fail.
- Production proof path never reconstructs a dummy problem with empty semantic guards for guard verification.
```

Reviewer: use P2 prompt.

### P3 — Certificate verifier closure

Goal: `verify_certificate` handles every SolverCertificate variant and trusts only exact replay.

Actions:

```text
1. Implement guard certificate verification.
2. Implement target certificate verification for ideal, radical, guarded radical, composite gcd/lcm.
3. Implement empty certificate verification.
4. Implement exact image certificate verification shell and then full verification after P16.
5. Implement no-target-eliminant certificate verification shell that initially returns CertificateDesignGap, then fully close after P15. This temporary gap must be tracked and cannot survive final phase.
6. Add tamper tests:
   - multiplier tamper
   - support tamper
   - guard_product tamper
   - guard certificate tamper
   - composite rule tamper
```

Acceptance:

```text
- Verifier recomputes polynomial identities over Q.
- Verifier never accepts based only on `ExactIdentityKind`.
- Any unimplemented variant is marked in gap inventory and has a later phase that must close it.
```

Reviewer: use P3 prompt.

## 3. Window and proof core

### P4 — Certificate windows and residual oracle

Goal: row-closed windows and residual map with exact quotient contract.

Actions:

```text
1. Implement row-closed window construction.
2. Implement membership matrix M_W and target power matrix N_d over Q.
3. Implement modular reduction with admissible prime filtering.
4. Implement residual oracle with property tests: reduce(v)=0 iff v in col(M).
5. Implement forged-row tests showing row_monomials are recomputed.
```

Acceptance:

```text
- Forged window rows cannot affect membership matrix.
- Residual oracle property tested over multiple primes and random small matrices.
```

Reviewer: use P4 prompt.

### P5 — Fixed exact proof and fair schedule

Goal: exact proof gate and unbounded fairness.

Actions:

```text
1. Implement `prove_fixed_target` exactly per Base Spec.
2. Implement `FairProofSchedule` over (d_B, support_power, guard_power).
3. Implement proof window learning from active supports and obstructions.
4. Implement predecessor expansion.
5. Integrate fair proof search with resource limits.
6. Add conformance tests for ideal, radical, guarded radical, obstruction expansion, and fairness enumeration.
```

Acceptance:

```text
- Identity check recomputes H - Σ q_i F_i.
- Guarded radical proof uses actual `system.guard_certificates`.
- For every finite tuple in small range, fair schedule reaches it.
- `max_window_degree = None` does not create an infinite loop that prevents complete fallback from being reachable in controlled tests.
```

Reviewer: use P5 prompt.

### P6 — Candidate normalization, factor schedule, and ranking

Goal: candidate preprocessing cannot hide unsound simplification.

Actions:

```text
1. Implement modular candidate merge across primes.
2. Implement CRT + rational reconstruction.
3. Implement single-prime heuristic marking without pretending it is reconstructed multi-prime evidence.
4. Implement actual factor schedule over Q.
5. Implement ranking using exact origin, degree, prime count, origin count, coefficient height, active support size.
6. Add tests for high coefficient reconstruction and factor trial.
```

Acceptance:

```text
- First-prime-only Q reconstruction does not exist in multi-prime path.
- Factor schedule includes nontrivial factors for reducible candidates.
- Squarefree part is not used as proof target unless separately certified.
```

Reviewer: use P6 prompt.

## 4. Candidate route closure

### P7 — DirectTargetEquation route closure

Actions:

```text
1. Implement exact target-only extraction from certified equations.
2. Route-forcing tests with other routes and complete fallback disabled.
3. Tamper test on resulting certificate after fixed proof.
```

Acceptance:

```text
- Direct route produces candidate but does not adopt without fixed proof.
- Route trace identifies equation index.
```

Reviewer: use P7 prompt.

### P8 — ResidualCyclic route closure

Actions:

```text
1. Implement residual-cyclic control-flow exactly from Base Spec §8.2.
2. Recover modular multiplier solution for active supports.
3. Merge candidates across primes and reconstruct Q candidate via CRT.
4. Add RC-F1 through RC-F4 tests.
5. Add spurious candidate rejection test.
```

Acceptance:

```text
- Active supports are nonzero multiplier supports, not whole window.
- High coefficient candidate requires multiple primes.
- Wrong modular lift is rejected by exact proof.
```

Reviewer: use P8 prompt.

### P9 — TargetCyclicKrylov route closure

Actions:

```text
1. Implement quotient/residual handle for target powers.
2. Implement minimal recurrence detection.
3. Add KR-F1 through KR-F3 tests.
4. Ensure no Groebner/complete fallback hidden call.
```

Acceptance:

```text
- Recurrence derives from residual classes.
- Route-only tests pass with complete fallback disabled.
```

Reviewer: use P9 prompt.

### P10 — HiddenVariableSparseResultant route closure

Actions:

```text
1. Implement sparse template builder for m >= 2 polynomial blocks.
2. Implement square and rectangular template handling.
3. Implement determinant/minor/null-relation computation over Fp[T].
4. Implement CRT/rational reconstruction.
5. Add SR-F1 through SR-F4 tests.
6. Search and remove any polynomial_count == 2 hard route gate unless it is only a subcase of a general path.
```

Acceptance:

```text
- 3+ polynomial route-forced test succeeds or resource-fails with template evidence under strict resource limit; unbounded test succeeds for conformance family.
- Candidate derives from template determinant/minor/null relation.
- No hidden complete fallback or Groebner call.
```

Reviewer: use P10 prompt.

### P11 — SliceSpecialization route closure

Actions:

```text
1. Implement generic affine slice generation over admissible primes.
2. Build full sliced systems with all original equations plus slice equations.
3. Use residual-cyclic or sparse resultant inside the sliced system.
4. Record slice equations and internal route.
5. Add SL-F1 through SL-F3 tests.
```

Acceptance:

```text
- Single-equation substitution is absent from production route.
- Slice candidate cannot be adopted without original-system fixed proof.
- Spurious slice candidate rejection test passes.
```

Reviewer: use P11 prompt.

### P12 — NormTraceTower route closure

Actions:

```text
1. Implement monic and guarded-nonmonic tower detection.
2. Build exact tower quotient basis and reduction.
3. Build target multiplication matrix.
4. Compute characteristic/minimal polynomial exactly.
5. Add NT-F1 through NT-F3 tests.
```

Acceptance:

```text
- Nonmonic tower uses guard certificate, not assumption.
- Route works beyond coefficient ±1 target equation.
- Fixed proof gate still required.
```

Reviewer: use P12 prompt.

### P13 — LocalizedSchur route closure

Actions:

```text
1. Implement obstruction scope detection.
2. Implement boundary variable and frontier support construction.
3. Solve local membership equation.
4. If target-only relation appears, construct original-system certificate.
5. Otherwise return support info.
6. Add LS-F1 through LS-F3 tests.
```

Acceptance:

```text
- There exists a route-forced test where localized Schur returns an exact certificate.
- There exists a test where it returns support info only.
- Full-system Schur is not used outside complete fallback.
```

Reviewer: use P13 prompt.

## 5. Complete fallback and exact image

### P14 — Exact Groebner / elimination certificate substrate

Goal: exact elimination engine with replay, not full coordinate solving.

Actions:

```text
1. Implement deterministic monomial order support.
2. Implement Buchberger/F4-style exact Groebner basis with representation tracking.
3. Every basis polynomial stores combination of original generators.
4. Add verification for basis representation and reductions.
5. Resource limits must expose actual pair/matrix counts.
```

Acceptance:

```text
- Basis polynomial tamper fails verification.
- Engine does not enumerate coordinate solutions.
```

Reviewer: use P14 prompt.

### P15 — CompleteTargetEliminationFallback closure

Actions:

```text
1. Implement saturation via U*D - 1.
2. Run exact target elimination with eliminated variables > target.
3. Extract nonzero Q[T] support and convert representation to TargetCertificate.
4. Implement empty certificate extraction.
5. Implement no-target-eliminant certificate and verifier.
6. Add CTE-F1 through CTE-F5 tests.
```

Acceptance:

```text
- Complete fallback is not bounded relation search.
- Guarded saturation conformance test passes.
- No-target-eliminant is not monomial-only.
- Verifier handles NoTargetEliminant certificate.
```

Reviewer: use P15 prompt.

### P16 — Exact real fiber and CertifiedExactTargetImage closure

Actions:

```text
1. Implement algebraic real number representation.
2. Implement exact sign evaluation for Q(α) coefficients.
3. Implement recursive projection-lifting CAD or equivalent exact RCF feasibility over Q(α).
4. Implement Nonempty sample-point certificate.
5. Implement Empty CAD/RCF certificate.
6. Implement `classify_real_fibers` and exact image verifier.
7. Add EI-F1 through EI-F4 tests.
```

Acceptance:

```text
- `classify_real_fibers` is not always incomplete.
- `verify_certificate` handles ExactTargetImage.
- RequireExactImage returns CertifiedExactTargetImage on small conformance families.
- Numeric floats are not certificate payloads.
```

Reviewer: use P16 prompt.

## 6. Integration and final closure

### P17 — Top-level solver integration

Actions:

```text
1. Wire all routes into `solve_target` in Base Spec order.
2. Ensure options only control resources and exact image mode, not fixture-specific algorithm branches.
3. Ensure status consistency.
4. Ensure no hidden fallback.
5. Add integration tests for each status.
```

Acceptance:

```text
- CertifiedCandidateCover, CertifiedExactTargetImage, CertifiedEmptyAdmissibleSet, CertifiedNoNonzeroTargetEliminant, NoVerifiedTargetCertificate, FiniteResourceFailure, InvalidInput are all reachable through non-fixture tests.
- All success statuses have verifier-accepted certificates.
```

Reviewer: use P17 prompt.

### P18 — Route-forcing matrix and no-fallback closure

Actions:

```text
1. Complete route-forcing harness.
2. Add one route-only success and one route-only rejection test for each route.
3. Add trace assertions proving other routes and fallback disabled.
4. Fill `route_forcing_matrix.md`.
```

Acceptance:

```text
- Top-level success is not used as route correctness.
- Each route has independent route-forced evidence.
```

Reviewer: use P18 prompt.

### P19 — Non-simplification manifest and data-flow proof closure

Actions:

```text
1. For each route, write production call chain.
2. For each route, write required data-flow object list.
3. Search forbidden patterns and record results.
4. Reviewer must verify source, not just manifest.
```

Acceptance:

```text
- Manifest covers all routes and final fallback/exact image.
- Reviewer confirms no name-only, shell-only, fallback-only implementation remains.
```

Reviewer: use P19 prompt.

### P20 — Adversarial regression suite

Actions:

```text
1. Add tests for generalized algebraic structures from previous failures:
   - coordinate-role-free algebraic IR;
   - guard/saturation;
   - determinant/oriented-area-like bilinear structures;
   - dot/Gram-like quadratic structures;
   - tower/extension structures;
   - positive-dimensional but finite target image;
   - sparse incidence graph;
   - dense hard resource failure.
2. No test may use geometry family names or expected answer branches.
```

Acceptance:

```text
- Tests are algebraic, not geometry-name driven.
- Solver produces candidate covers or exact failures according to certificates.
```

Reviewer: use P20 prompt.

### P21 — Final disqualifier scan

Actions:

```text
Search production code for:
- TODO / unimplemented / not available / Unsupported as normal path
- always Incomplete exact image
- exact image verifier unhandled
- monomial-only no-target eliminant
- first-prime reconstruction
- complete fallback bounded by window degree only
- guard dropped
- semantic_guards empty construction in proof path
- factor_schedule clone-only
- localized Schur support-only
- polynomial_count == 2 hard general route gate
- single-equation slice substitution route
- hidden full coordinate RUR or solution enumeration
```

Acceptance:

```text
- No final disqualifier remains.
- Any remaining occurrence is test-only or error-only and reviewer confirms it cannot occur in production success/failure path.
```

Reviewer: use P21 prompt.

### P22 — Final theorem and claim ceiling review

Actions:

```text
1. Run full test suite.
2. Run route-forcing suite.
3. Run tamper suite.
4. Run doc consistency checks.
5. Reviewer performs final adversarial source review.
6. Final claim must match Base Spec exactly.
```

Acceptance:

```text
- No unresolved QuestionDebt.
- No CertificateDesignGap in final required variants.
- Reviewer can explain entire solve_target data-flow from input to certificate verification.
```

Reviewer: use P22 final prompt.

## 7. Phase dependency graph

```text
P0
 └─ P1
    ├─ P2 ─ P3
    ├─ P4 ─ P5 ─ P6
    │       ├─ P7
    │       ├─ P8
    │       ├─ P9
    │       ├─ P10
    │       ├─ P11
    │       ├─ P12
    │       └─ P13
    ├─ P14 ─ P15
    └─ P16
          ↓
       P17 ─ P18 ─ P19 ─ P20 ─ P21 ─ P22
```

## 8. Mandatory stop conditions

Agent must stop and request Base Spec amendment if any of the following becomes necessary:

```text
- Exact image cannot be implemented as specified.
- Complete target elimination cannot produce verifier-checkable no-target-eliminant certificates.
- A route cannot be implemented except by delegating to complete fallback.
- Guard verification requires a semantic assumption not present in `TargetProblemQ`.
- Resource behavior cannot distinguish finite resource failure from no certificate found.
```

Agent must not silently replace the required algorithm with a simplified one.

