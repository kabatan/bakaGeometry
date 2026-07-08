# 04. Route Checklists and Test Matrix

## 0. 使い方

この文書は Agent の evidence ではなく、reviewer が production code を直接検査するための checklist である。表を埋めるだけでは PASS できない。各項目は source-level call chain、data-flow、certificate replay、route-forcing、tamper test で確認する。

## 1. Route-forcing matrix

| Route | Production entry | Other routes disabled? | Complete fallback disabled? | Route-only success test | Route-only spurious/reject test | Tamper test | Reviewer source call-chain checked | PASS |
|---|---|---:|---:|---|---|---|---|---:|
| DirectTargetEquation | `candidates/direct.rs` |  |  |  |  |  |  |  |
| ResidualCyclic | `candidates/residual_cyclic.rs` |  |  |  |  |  |  |  |
| TargetCyclicKrylov | `candidates/krylov.rs` |  |  |  |  |  |  |  |
| HiddenVariableSparseResultant | `candidates/sparse_resultant.rs` |  |  |  |  |  |  |  |
| SliceSpecialization | `candidates/slice.rs` |  |  |  |  |  |  |  |
| NormTraceTower | `candidates/norm_trace_tower.rs` |  |  |  |  |  |  |  |
| LocalizedSchur | `candidates/localized_schur.rs` |  |  |  |  |  |  |  |
| CompleteTargetElimination | `elimination/target_elimination.rs` | n/a | n/a |  |  |  |  |  |
| ExactTargetImage | `real/fiber.rs` | n/a | n/a |  |  |  |  |  |

## 2. Non-simplification checklist

### 2.1 Certified compression and guards

FAIL if any item is true:

```text
- CertifiedSystemQ creation always clones equations and clears guards/replay.
- Input semantic NonZero guards are not converted to GuardCertificate.
- GuardedRadical proof path verifies guards against a dummy problem with empty semantic guards.
- GuardProduct is accepted without recomputing product.
- Compression replay cannot be checked from original TargetProblemQ.
```

Required tests:

```text
- NonZero guard transfer test.
- Guard tamper reject test.
- Replay tamper reject test.
- Guarded radical certificate using input semantic NonZero guard.
```

### 2.2 Fixed proof

FAIL if any item is true:

```text
- Linear solve success is enough without exact identity check.
- Radical proof allows support_power = 0.
- Guarded proof uses guard_product without certificates.
- Inconsistent proof is treated as proof that S is false, rather than window failure.
```

Required tests:

```text
- Ideal membership exact proof.
- Radical proof where S itself is not in I but S^a is.
- Guarded radical proof requiring D.
- Obstruction predecessor expansion.
```

### 2.3 ResidualCyclic

FAIL if any item is true:

```text
- Active supports equal full window by default.
- Reconstruction is first-prime lift.
- Residual relation does not come from rho(T^k).
- Modular candidate bypasses exact Q proof.
```

Required tests:

```text
- High coefficient CRT reconstruction.
- Multiple primes required.
- Spurious modular relation rejected.
- Active support smaller than full window.
```

### 2.4 TargetCyclicKrylov

FAIL if any item is true:

```text
- No quotient/residual handle.
- Hidden Groebner/complete fallback call.
- Top-level success through other route used as evidence.
```

Required tests:

```text
- Finite target recurrence lower than full quotient rank.
- Positive-dimensional finite target image.
- Spurious recurrence rejection.
```

### 2.5 HiddenVariableSparseResultant

FAIL if any item is true:

```text
- General route requires polynomial_count == 2.
- Only Sylvester resultant exists.
- Candidate not derived from sparse template determinant/minor/null relation.
- Route calls complete fallback/Groebner and labels result resultant.
```

Required tests:

```text
- Two-polynomial resultant.
- Three-polynomial overdetermined resultant.
- Sparse support advantage family.
- Spurious factor rejected by fixed proof.
```

### 2.6 SliceSpecialization

FAIL if any item is true:

```text
- Single-equation substitution route.
- Does not build full sliced system.
- Slice agreement/gcd used as proof.
- Original unsliced fixed proof absent.
```

Required tests:

```text
- Single equation slice misleading, global sliced system useful.
- Spurious slice candidate rejected.
- Multiple slice families.
```

### 2.7 NormTraceTower

FAIL if any item is true:

```text
- Only monic coefficient 1 and coefficient ±1 target expression supported while claiming general tower.
- Nonmonic leading coefficient inverted without guard.
- Multiplication matrix not exact.
- Characteristic polynomial route bypasses proof gate.
```

Required tests:

```text
- Monic tower.
- Guarded nonmonic tower.
- Repeated factor tower.
```

### 2.8 LocalizedSchur

FAIL if any item is true:

```text
- Always returns SupportInformation.
- Never builds exact certificate for target-only local relation.
- Full Schur outside complete fallback.
- No replay into original system.
```

Required tests:

```text
- Support-only local repair.
- Certified target-only local repair.
- Full-scope obstruction refused.
```

### 2.9 CompleteTargetElimination

FAIL if any item is true:

```text
- Bounded window search named complete.
- Saturation D^∞ not implemented via U*D-1 or equivalent.
- No-target-eliminant verifier is monomial-only.
- Empty set returned as support 1.
- Full coordinate solution enumeration or RUR used.
```

Required tests:

```text
- Unguarded target eliminant.
- Guarded saturation target eliminant.
- Algebraic empty.
- No target eliminant non-monomial family.
- Resource-limited failure.
```

### 2.10 ExactTargetImage

FAIL if any item is true:

```text
- Classifier always Incomplete.
- ExactTargetImage verifier unhandled.
- Nonempty certificate uses floats.
- Empty certificate is unverified text.
- Unclassified roots dropped.
- Guards ignored.
```

Required tests:

```text
- All roots nonempty.
- One spurious root empty.
- Guard removes a root.
- RequireExactImage resource-limited fail-closed.
```

## 3. Static search checklist

Reviewer must search production source for these strings and equivalents:

```text
guard_certificates: Vec::new()
semantic_guards: Vec::new()
classify_real_fibers
Incomplete
not handled
not available
Unsupported
unimplemented!
todo!
TODO
ImplementationBug
max_window_degree.unwrap_or
polynomial_count == 2
polynomials.len() != 2
factor_schedule
vec![candidate.clone()]
slice_count: 2
CompleteFallbackResult::ResourceFailure
NoTargetEliminant
monomial_non_target_ideal
```

A hit is not automatically FAIL if it is test-only or impossible in production, but reviewer must explain why.

## 4. Required adversarial families

No family may use geometry names, problem IDs, or expected answer dispatch.

```text
A1. High coefficient modular reconstruction:
    target support has coefficients larger than any single small prime.

A2. Guarded saturation:
    target relation exists in I:D^∞ but not plainly in I.

A3. Positive-dimensional finite target image:
    coordinate set infinite, target values finite.

A4. No target eliminant:
    elimination ideal in Q[T] is zero, not limited to monomial ideal.

A5. Sparse resultant 3+ polynomial:
    three equations needed to eliminate non-target variables.

A6. Slice misleading single equation:
    individual equation slice gives wrong candidate; full sliced system gives useful candidate.

A7. Schur local certificate:
    obstruction scope proper subset; local target relation is certifiable.

A8. Exact image with spurious root:
    candidate cover has extra real root with empty fiber.

A9. Nonmonic tower with guard:
    tower leading coefficient needs nonzero guard proof.

A10. Resource failure:
    strict limits stop exact elimination / exact image without unsound success.
```

## 5. Final PASS form

A final reviewer PASS must include this statement with concrete file/function evidence:

```text
I inspected production code, not only Agent evidence.
I verified that every route has a route-forced no-fallback path.
I verified that every success certificate is replayed by `verify_certificate` from original `TargetProblemQ`.
I searched final disqualifiers and found none in production paths.
I verified exact image is implemented, not an always-incomplete stub.
I verified complete target elimination is exact saturated elimination, not bounded relation search.
I verified guard records are not dropped.
I verified modular reconstruction is not first-prime-only.
I verified no top-level success is used as route correctness evidence.
```

If reviewer cannot honestly write any sentence above, verdict must be FAIL.

