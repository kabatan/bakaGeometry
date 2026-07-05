# P5R Plan — Mandatory Remediation Before P6

This plan must be inserted after P5 and before P6 in `docs/ai/changes/RGDTPK-Q-v4-core/PLAN.md`.

P5R is a closeable phase group only when every subphase P5R-a through P5R-f is closed. The Agent must not ask a reviewer to pass “P5R” as one bulk phase until all subphase review archives exist and pass.

---

## P5R-a — Rebind evidence to current Git commit and repair claim documents

**Supports:** P5R-RGQ-065, P5R-RGQ-066, P5R-RGQ-071.  
**Files to modify:**

```text
docs/ai/ACTIVE_CONTEXT.md
docs/ai/changes/RGDTPK-Q-v4-core/CLOSURE.md
docs/ai/changes/RGDTPK-Q-v4-core/evidence/P5/evidence_manifest.yaml, if present
docs/ai/changes/RGDTPK-Q-v4-core/evidence/P5/notes.md
docs/ai/changes/RGDTPK-Q-v4-core/P6_READINESS.md
```

**Required implementation tasks:**

1. Run `git rev-parse --verify HEAD` and record the commit SHA.
2. Rerun at least:

```text
cargo fmt --manifest-path geosolver-core/Cargo.toml -- --check
cargo test --manifest-path geosolver-core/Cargo.toml -- --nocapture
```

3. Rerun P5 graph-focused tests and P5 static scans from the existing P5 evidence.
4. Update P5 evidence or add a P5R evidence overlay that binds the same P5 claim to the current commit.
5. Update `CLOSURE.md` so it does not say documentation-only progress. It must say current maximum claim is exactly:

```text
PARTIAL_MECHANISM_READY:MECH-004
```

6. Add explicit negative claims:

```text
Not candidate-cover ready.
Not exact-image ready.
Not planner ready.
Not kernel execution ready.
Not public pipeline ready.
Not acceptance complete.
```

7. Create `P6_READINESS.md` with a checklist. Initially it may mark later P5R items incomplete; P5R-f must finalize it.

**Forbidden shortcuts:**

```text
- Do not keep `unborn-master-no-commit` in active closure evidence.
- Do not claim P5 graph/DAG is solver-core completion.
- Do not delete old evidence instead of adding a current commit overlay.
```

**Closure evidence:**

```text
evidence/P5R-a/commands.txt
evidence/P5R-a/command_outputs.txt
evidence/P5R-a/static_scans.txt
evidence/P5R-a/claim_consistency_matrix.yaml
reviews/P5R-a/<timestamp>/prompt.md
reviews/P5R-a/<timestamp>/response.md
reviews/P5R-a/<timestamp>/review_summary.yaml
reviews/P5R-a/<timestamp>/evidence_manifest.yaml
```

**Reviewer prompt:** `P5R_REVIEWER_PROMPTS.md#P5R-a`.

---

## P5R-b — Eliminate fake F4 claim path

**Supports:** P5R-RGQ-067.  
**Files to modify:**

```text
geosolver-core/src/algebra/f4.rs
geosolver-core/src/algebra/elimination.rs
geosolver-core/src/algebra/mod.rs
geosolver-core/src/result/diagnostics.rs or cost trace files if needed
docs/ai/changes/RGDTPK-Q-v4-core/PRIMITIVE_SCOPE_LEDGER.md
```

**Required implementation tasks:**

Choose exactly one route and document the chosen route in `PRIMITIVE_SCOPE_LEDGER.md`.

### Route A: real local F4

1. Implement symbolic row collection for selected S-polynomial batches.
2. Build sparse Macaulay/F4 matrices over deterministic modular primes.
3. Reduce rows by exact modular row reduction.
4. Reconstruct candidate reducers or generator rows over Q.
5. Verify every exported generator by exact Q membership certificate.
6. Record matrix row/column/nonzero traces.
7. Add tests proving batch matrix reduction is actually used, not just sequential `reduce_by_set`.

### Route B: demote current F4 wrapper

1. Rename public strategy names or add production guard so current `f4_elimination_local` cannot satisfy `LocalF4` production semantics.
2. If a compatibility function remains, name it honestly, for example `groebner_backed_batch_reduction_for_tests` or `local_groebner_batch_reduction`.
3. Ensure `EliminationStrategy::LocalF4` is not selectable in production unless Route A is implemented.
4. Add a static or unit test proving planner/kernel code cannot select fake F4 as a production F4 strategy.

**Mandatory tests:**

```text
- a test that would fail if `f4_elimination_local` is merely a direct wrapper around `groebner_elimination_basis` while claiming production F4;
- a test that any current non-F4 implementation is labelled non-production or non-F4;
- a test that exact Q membership still verifies exported generators.
```

**Forbidden shortcuts:**

```text
- Do not keep F4 naming for a Groebner wrapper.
- Do not hide this as a documentation limitation only.
- Do not let UniversalTargetEliminationKernel later select fake F4.
```

**Reviewer prompt:** `P5R_REVIEWER_PROMPTS.md#P5R-b`.

---

## P5R-c — Implement guarded rational affine semantics or explicit equivalent transformation

**Supports:** P5R-RGQ-068.  
**Files to modify:**

```text
geosolver-core/src/preprocess/linear_affine.rs
geosolver-core/src/preprocess/compression.rs
geosolver-core/src/preprocess/mod.rs
geosolver-core/src/preprocess/saturation.rs
geosolver-core/src/problem/semantic.rs
geosolver-core/src/result/diagnostics.rs
```

Add helper files only if they clarify the implementation, for example:

```text
geosolver-core/src/preprocess/rational_substitution.rs
```

**Required implementation tasks:**

1. Extend affine pivot representation so a safe nonconstant denominator with explicit nonzero witness can be represented even when `numerator / denominator` is not a polynomial.
2. Record denominator guard and source witness relation.
3. Apply the transformation to remaining relations by exact denominator clearing or by rational expression substitution with subsequent clearing.
4. Preserve exact provenance so later certificate/fiber code knows that the transformation is valid only under the guard.
5. Keep unsafe nonconstant denominator candidates as rejected candidates without returning Unsupported.
6. Update compression trace with counts for guarded rational affine pivots.
7. Ensure target variable is never eliminated by this preprocessing.

**Mandatory algebraic stress tests:**

Use only algebraic variable IDs and polynomial equations; no geometry names.

### safe guarded rational affine non-polynomial case

Construct:

```text
variables: T, x, y, s
relations:
  (x + 1) * s - 1 = 0          // explicit nonzero witness for x+1
  (x + 1) * y - (T + x) = 0    // y = (T+x)/(x+1), not generally polynomial
  y - 2 = 0                    // after guarded clearing, produces T - x - 2 = 0 or equivalent
```

The test must prove:

```text
- guarded affine pivot is used;
- denominator guard is recorded;
- transformed relation is exact and contains no y;
- no unsupported status is used;
- transformation certificate/provenance is present.
```

### unsafe nonconstant denominator rejection without solver-scope rejection

Construct the same system without `(x + 1) * s - 1 = 0`. The test must prove:

```text
- no unsafe substitution is performed;
- relation remains in the compressed system;
- the phase does not return Unsupported or InvalidInput for this reason.
```

**Forbidden shortcuts:**

```text
- Do not require polynomial quotient for guarded nonconstant denominators.
- Do not drop guard semantics.
- Do not factor-split components without semantics.
- Do not treat absence of guard as invalid input.
```

**Reviewer prompt:** `P5R_REVIEWER_PROMPTS.md#P5R-c`.

---

## P5R-d — Make TargetActionKrylov production handles provenance-bound

**Supports:** P5R-RGQ-069.  
**Files to modify:**

```text
geosolver-core/src/algebra/quotient.rs
geosolver-core/src/algebra/krylov.rs
geosolver-core/src/algebra/normal_form.rs
geosolver-core/src/algebra/elimination.rs
geosolver-core/src/verify/certificates.rs, if existing enough to patch
```

Add helper files only if needed, for example:

```text
geosolver-core/src/algebra/quotient_provenance.rs
geosolver-core/src/algebra/normal_form_basis.rs
```

**Required implementation tasks:**

1. Split quotient handles into:

```text
ProductionProvenancedTargetQuotientHandle
DebugExplicitTargetQuotientHandle or test-only helper
```

2. Production handles must be constructed from authorized block relations or from an object carrying the authorization hash.
3. For each basis element and variable action column, produce or verify an independent normal-form certificate.
4. `normal_form(T * basis_element)` used for certificate checking must not depend on the same injected action column being verified.
5. `TargetActionKrylovKernel` must accept only production-provenanced handles.
6. Low-level Krylov algebra tests may still use explicit handles, but production kernel tests must prove explicit handles are rejected.

**Mandatory tests:**

```text
- malicious injected action columns are rejected by production builder;
- explicit debug handle cannot be passed to production TargetActionKrylovKernel;
- action column certificate fails when source relation authorization hash is tampered;
- verified characteristic support coverage still rejects single-vector undercoverage;
- no coordinate roots or full coordinate RUR APIs are introduced.
```

**Forbidden shortcuts:**

```text
- Do not prove action columns by circular comparison with a normal_form method that uses those columns.
- Do not rely only on handle hash or basis hash.
- Do not allow production code to build a quotient/action handle from arbitrary external action columns.
```

**Reviewer prompt:** `P5R_REVIEWER_PROMPTS.md#P5R-d`.

---

## P5R-e — Primitive scope ledger and anti-overclaim wiring

**Supports:** P5R-RGQ-070.  
**Files to create/modify:**

```text
docs/ai/changes/RGDTPK-Q-v4-core/PRIMITIVE_SCOPE_LEDGER.md
docs/ai/changes/RGDTPK-Q-v4-core/SOURCE_MAP.md
docs/ai/changes/RGDTPK-Q-v4-core/PLAN.md
```

**Required implementation tasks:**

1. Create `PRIMITIVE_SCOPE_LEDGER.md` with entries for all required primitives.
2. Mark current capability and limitations exactly. Do not soften.
3. Patch P6, P8, and P9 plan sections so that:

```text
- binary resultant primitive cannot close generic SparseResultantProjectionKernel;
- one-variable interpolation primitive cannot close generic SpecializationInterpolationKernel;
- single-chain regular-chain primitive cannot close generic RegularChainProjectionKernel;
- single-variable tower primitive cannot close generic NormTraceProjectionKernel;
- fake F4 cannot close Universal local F4 route;
- polynomial-only affine pivot cannot be treated as full guarded affine semantics;
- explicit injected quotient/action handle cannot close production TargetActionKrylovKernel.
```

4. Add reviewer instructions that later phase reviewers must consult this ledger.

**Required ledger entry template:**

```markdown
## <file or primitive>

Current implemented capability:

Exact limitations:

Production use allowed before expansion:

Required exact verification after use:

Allowed failure on exhaustion:

Forbidden claim:

Later phase that must expand or replace this primitive:
```

**Forbidden shortcuts:**

```text
- Do not use the ledger as permission to keep a narrow solver.
- Do not mark a primitive as generic because it has tests.
- Do not omit limitations to make P6 easier.
```

**Reviewer prompt:** `P5R_REVIEWER_PROMPTS.md#P5R-e`.

---

## P5R-f — P6 readiness audit

**Supports:** P5R-RGQ-065 through P5R-RGQ-072.  
**Files to modify:**

```text
docs/ai/changes/RGDTPK-Q-v4-core/P6_READINESS.md
docs/ai/changes/RGDTPK-Q-v4-core/CLOSURE.md
docs/ai/ACTIVE_CONTEXT.md
docs/ai/changes/RGDTPK-Q-v4-core/PLAN.md
docs/ai/changes/RGDTPK-Q-v4-core/REVIEWER_PROMPTS.md
```

**Required implementation tasks:**

1. Run full crate tests and all P5R-specific tests.
2. Run static scans for:

```text
TODO|todo|stub|placeholder|dummy|fake|unimplemented!|panic!("TODO")
Unsupported|NotYetImplemented|Skipped
coordinate solution|coordinate roots|full coordinate RUR|QE|CAD|homotopy
CertifiedNonFiniteTargetImage
F4
unborn-master-no-commit
```

3. For hits that are expected in docs or tests, classify them in `static_scans.txt` with exact file/line reason.
4. Fill `P6_READINESS.md` with explicit yes/no answers required by P5R-RGQ-071. All P5R readiness items must be yes. Later phases may remain no, but must be clearly assigned to P6+.
5. Update `CLOSURE.md` to state P5R closure and claim ceiling only.
6. Archive reviewer prompt/response/summary/manifest.

**P6 may start only if:**

```text
- all P5R subphase reviews PASS;
- no active evidence is commit-unbound;
- CLOSURE and ACTIVE_CONTEXT agree with P5R;
- fake F4 path is impossible;
- guarded rational affine path is implemented or explicitly equivalent;
- production TargetActionKrylov cannot use injected self-certifying handles;
- primitive scope ledger blocks overclaim;
- P6_READINESS.md says P6 is allowed and explains remaining non-P6 work.
```

**Reviewer prompt:** `P5R_REVIEWER_PROMPTS.md#P5R-f`.
