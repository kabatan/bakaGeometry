# P5R Base Spec Amendment — Pre-Planner Remediation Before P6

Spec ID: `RGDTPK-Q-v4-core-P5R`  
Parent: `docs/ai/changes/RGDTPK-Q-v4-core/BASE_SPEC.md` v2.2  
Status: Mandatory hardening amendment  
Insertion point: after P5, before P6  
Scope: current `geosolver-core` implementation after P5, especially `algebra/f4.rs`, `preprocess/linear_affine.rs`, `algebra/quotient.rs`, `algebra/krylov.rs`, Guardian closure/evidence, and primitive-scope tracking.

This amendment exists because P5 graph/DAG work can be mostly correct while the surrounding P3/P4 primitives still allow later phases to drift into one of two forbidden outcomes:

```text
1. narrow partial solver disguised as generic target-direct solver;
2. heavy fallback solver disguised as UniversalTargetEliminationKernel.
```

P5R must be applied before any P6 planner, admission, declared ladder, or kernel-plan work.

---

## P5R-RGQ-065 — P5R is a mandatory phase barrier

P6 must not begin until P5R is closed. The Agent must insert P5R into `PLAN.md` between P5 and P6 and must mark P6 prerequisites as including `P5R-a` through `P5R-f`.

P5R closure is not optional, not advisory, and not satisfied by reviewer PASS alone. It requires code changes, documentation changes, fresh command evidence, archived reviewer prompt/response, schema-valid `review_summary.yaml`, and a current Git commit binding.

P5R does not close any candidate-cover, exact-image, kernel-execution, or acceptance-complete claim.

---

## P5R-RGQ-066 — Commit-bound evidence and claim consistency

All current P0–P5 evidence that is used to justify continuing must be rebound to the current Git commit after the GitHub push.

The following strings must not appear in any active closing evidence or review summary after P5R closes:

```text
unborn-master-no-commit
No implementation closure exists yet
Allowed current claim: documentation pack prepared for user approval
```

`CLOSURE.md`, `ACTIVE_CONTEXT.md`, P5 evidence manifest, and P5/P5R review summaries must agree on the current claim ceiling:

```text
PARTIAL_MECHANISM_READY:MECH-004
```

They must also explicitly state that the following are not complete:

```text
planner admission
kernel planning
projection-message execution
candidate-cover construction
exact-image classification
run certificate replay
public orchestration
performance claim
RGDTPK_Q_V4_ACCEPTANCE_COMPLETE
```

A reviewer must fail P5R if any document implies that P5 graph/DAG completion is candidate-cover readiness, generic kernel readiness, exact-image readiness, or final acceptance.

---

## P5R-RGQ-067 — No fake F4 or F4-by-name

The implementation must not call a sequential Buchberger/Groebner wrapper `F4`, `F4-like`, or `sparse linear algebra F4` in production evidence or planner/kernel claims.

One of the following must be true before P5R closes:

### Option A — implement real local F4

`algebra/f4.rs` implements a real local F4-style batch reduction:

```text
- constructs symbolic monomial rows for selected S-polynomial batches;
- builds sparse Macaulay/F4 matrices over exact modular fields;
- performs row reduction as the batch reduction mechanism;
- reconstructs exact Q reducers/results or produces candidate rows that exact Q verification checks;
- exports only Q[keep] generators with membership/normal-form certificates;
- records matrix row/column/nonzero traces;
- fails as FiniteResourceFailure or CertificateDesignGap when resource or proof bounds are insufficient.
```

### Option B — demote current file and prevent production F4 claim

If real local F4 is not implemented in P5R, the current `algebra/f4.rs` must be explicitly demoted:

```text
- rename public strategy names so they do not claim F4 semantics, or
- add an explicit `F4ImplementationLevel::NotProductionF4` / equivalent guard,
- prevent P6/P8d planner from selecting it as `LocalF4` or using it for performance claims,
- update UniversalTargetEliminationKernel plan language so current implementation cannot satisfy `LocalF4OrGroebnerEliminationToKeepZ` by pretending to be F4.
```

A reviewer must fail P5R if `f4_elimination_local` is still just a call to `groebner_elimination_basis` while production code or evidence claims F4 readiness.

---

## P5R-RGQ-068 — Guarded affine substitution must support rational affine semantics, not only polynomial quotient cases

The linear-affine preprocessing semantics from Appendix A section 11.3 are mandatory:

```text
a(X) * y + b(X) = 0
```

If `a(X)` is nonzero by explicit guard/witness semantics, the solver may use the substitution:

```text
y = -b(X) / a(X)
```

The implementation must not silently narrow this to cases where `b(X)` is polynomially divisible by `a(X)`.

P5R must add one of the following production-safe implementations:

### Option A — rational substitution representation

Add a rational substitution layer, for example:

```rust
pub struct RationalExpressionQ {
    pub numerator: SparsePolynomialQ,
    pub denominator: SparsePolynomialQ,
    pub denominator_guard: GuardRecord,
    pub expression_hash: Hash,
}

pub enum CompressionExpression {
    Polynomial(SparsePolynomialQ),
    Rational(RationalExpressionQ),
}
```

Then update compression to apply rational substitutions by clearing denominators while recording guards, denominator powers, and exact provenance.

### Option B — denominator-cleared transformation without storing rational expressions

If rational expressions are not stored, then for each guarded affine pivot the implementation must transform every affected relation by exact denominator clearing and record a certificate proving that the transformed relation is implied on the guarded locus.

For either option, all of the following are mandatory:

```text
- unsafe nonconstant denominator without explicit nonzero witness is not substituted;
- safe nonconstant denominator with explicit nonzero witness is not dropped merely because numerator/denominator is not a polynomial quotient;
- denominator guard remains in `guards` and exact-image feasibility/semantics obligations;
- relation transformation has exact certificate/provenance;
- target variable is not eliminated by this preprocessing;
- no geometry names, variable roles, fixtures, or expected answers are used.
```

A reviewer must fail P5R if `select_safe_affine_pivots` still requires `polynomial_expression.is_some()` for guarded nonconstant denominators without an alternative exact guarded-rational path.

---

## P5R-RGQ-069 — TargetActionKrylov quotient/action provenance must not be self-certifying

`TargetActionKrylov` may return a support polynomial only when the target action matrix is proven to come from the authorized block relations and verified quotient/normal-form basis.

A self-consistent externally supplied action matrix is not a quotient/action certificate.

Production code must not let `TargetActionKrylovKernel` accept a `TargetQuotientHandle` whose action columns were merely injected through an input struct without proof that each column is the normal form of:

```text
T * basis_element
```

with respect to the authorized block relations.

P5R must add production provenance structures equivalent to:

```rust
pub struct ProvenancedTargetQuotientHandle {
    pub basis_id: BasisHandleId,
    pub authorized_relation_hash: Hash,
    pub basis_polynomials: Vec<SparsePolynomialQ>,
    pub normal_form_basis_certificate: NormalFormBasisCertificate,
    pub action_columns: BTreeMap<VariableId, Vec<ActionColumnCertificate>>,
    pub quotient_handle_hash: Hash,
}

pub struct ActionColumnCertificate {
    pub variable: VariableId,
    pub basis_index: usize,
    pub input_polynomial_hash: Hash,     // hash(T * basis_element) or hash(var * basis_element)
    pub normal_form_vector: VectorQ,
    pub normal_form_certificate: NormalFormCertificate,
    pub source_relation_authorization_hash: Hash,
    pub column_hash: Hash,
}
```

The exact field names may differ, but the semantics must not differ.

The production builder must satisfy:

```text
- input relations come from the block authorization hash;
- normal-form basis is built or certified from those relations;
- every action column is recomputed or exact-verified against source relations;
- no coordinate roots or full coordinate RUR are exported;
- column verification is not circular through the same injected action columns;
- TargetActionKrylovKernel cannot use debug/test-only explicit handles.
```

Existing explicit/injected handles may remain only as test fixtures or low-level algebra test helpers if they are named and gated so production kernels cannot use them.

A reviewer must fail P5R if `target_action_matrix` verifies columns by comparing them to `handle.normal_form(...)` where that `normal_form` itself uses the same injected columns and no independent relation-based normal-form certificate.

---

## P5R-RGQ-070 — Primitive scope ledger and anti-overclaim barrier

P5R must create:

```text
docs/ai/changes/RGDTPK-Q-v4-core/PRIMITIVE_SCOPE_LEDGER.md
```

The ledger must classify every P3/P4 primitive that is not yet a full generic kernel. At minimum it must include:

```text
algebra/resultant.rs
algebra/interpolation.rs
algebra/regular_chain.rs
algebra/norm_trace.rs
algebra/f4.rs
preprocess/linear_affine.rs
algebra/quotient.rs
algebra/krylov.rs
```

For each primitive, the ledger must state:

```text
- current implemented capability;
- exact limitations;
- whether production kernels may call it;
- what exact verification is required after calling it;
- what status must be returned on exhaustion or unsupported algebraic shape;
- what claim is forbidden until later phases expand it.
```

The ledger must not be used to make narrow-slice completion acceptable. It must be used to prevent narrow primitives from being overclaimed as generic solver mechanisms.

A reviewer must fail P5R if P6/P8/P9 could reasonably treat binary resultant, one-variable interpolation, single-chain regular-chain, single-variable tower norm/trace, polynomial-only affine substitution, fake F4, or externally injected quotient handles as completed generic kernels.

---

## P5R-RGQ-071 — P6 readiness requires support-producing plan feasibility, not just graph readiness

P5R closure must include a `P6_READINESS.md` file stating the exact state of prerequisites for P6.

It must explicitly answer:

```text
1. Are P5 graph/DAG artifacts operational and commit-bound?
2. Are fake-F4 claims impossible?
3. Are guarded affine semantics no longer narrowed to polynomial quotient only?
4. Are TargetActionKrylov production handles provenance-bound to authorized relations?
5. Are narrow primitives prevented from being overclaimed as generic kernels?
6. Is public orchestration still not connected, and is that claim ceiling explicit?
7. Which later phases remain responsible for TargetRelationSearch schedule, Universal fixed ladder, composition, final support, roots, exact image, and replay?
```

P6 may begin only if all answers are explicit and no answer relies on future reviewer interpretation.

---

## P5R-RGQ-072 — P5R reviewer must judge algorithmic drift, not just evidence formatting

The P5R reviewer must use `P5R_REVIEWER_PROMPTS.md`. The reviewer must inspect code, not only evidence files.

The reviewer must fail if any of the following is true:

```text
- P5R only changes documentation while code still allows the unsafe path;
- F4 remains a Groebner wrapper but production can claim F4;
- guarded affine still ignores non-polynomial rational substitutions;
- quotient/action verification remains self-certifying;
- primitive limitations are documented but P6 can still overclaim them;
- evidence is not current-commit bound;
- CLOSURE.md or ACTIVE_CONTEXT.md conflicts with P5/P5R claim ceiling;
- public API is presented as candidate-cover capable before P14/P15/P16;
- any failure is routed to CertifiedNonFiniteTargetImage without a positive nonfinite certificate;
- any well-formed input can be rejected as Unsupported or equivalent non-spec status.
```
