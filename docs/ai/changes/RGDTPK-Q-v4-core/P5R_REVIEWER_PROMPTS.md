# P5R Reviewer Prompts

The reviewer is a boundary reviewer for a research algorithm implementation. The reviewer must not pass a phase just because the code compiles, tests pass, documents exist, schemas validate, or the Agent avoided overclaim wording.

The reviewer must inspect source code and compare it against `P5R_BASE_SPEC_AMENDMENT.md`, the v2.2 Base Spec, the v2.2 Plan, and the primitive scope ledger.

Reviewer output must include:

```yaml
review_status: PASS | FAIL
phase_closable: true | false
blocking_findings: []
required_fixes: []
algorithmic_sufficiency:
  verdict: sufficient | insufficient
  rationale: "..."
```

A PASS with nonempty `blocking_findings` or `required_fixes` is invalid.

---

## P5R-a reviewer prompt — evidence rebinding and claim consistency

Review P5R-a. You must inspect `CLOSURE.md`, `ACTIVE_CONTEXT.md`, P5/P5R evidence, review summaries, command outputs, and Git commit binding.

Fail if any active closure or evidence still relies on `unborn-master-no-commit`, if `CLOSURE.md` still says only documentation was prepared, or if any document implies candidate-cover, exact-image, planner, kernel, public pipeline, performance, or acceptance readiness.

Required checks:

```text
1. Current Git commit SHA is recorded.
2. fmt/full tests/P5 graph tests/static scans were rerun after the commit or file hashes were rebound after the commit.
3. Claim ceiling is exactly PARTIAL_MECHANISM_READY:MECH-004.
4. CLOSURE.md, ACTIVE_CONTEXT.md, evidence manifests, and review summaries agree.
5. P6_READINESS.md exists and is not falsely complete.
```

Do not pass if the Agent only changed wording without fresh evidence.

---

## P5R-b reviewer prompt — no fake F4

Review P5R-b. Inspect `algebra/f4.rs`, `algebra/elimination.rs`, strategy enums, tests, and the primitive ledger.

Fail if production code can still claim or select F4 while the implementation is only a direct wrapper around Groebner/Buchberger or per-target `reduce_by_set`.

Required checks:

```text
1. Route A or Route B is explicitly chosen.
2. If Route A: actual batch matrix/F4-style reduction exists and is tested.
3. If Route B: production planner/kernel cannot select fake F4 as LocalF4.
4. Documentation and ledger do not overclaim F4 readiness.
5. Exported generators still require exact Q membership verification.
```

Run or inspect tests that fail if F4 is merely a wrapper but claims production F4.

---

## P5R-c reviewer prompt — guarded rational affine semantics

Review P5R-c. Inspect `preprocess/linear_affine.rs`, compression state/substitution structures, guard records, saturation witness logic, tests, and cost/diagnostic trace.

Fail if guarded nonconstant affine substitution is still restricted to polynomial quotient cases without an exact guarded-rational alternative.

Required checks:

```text
1. Safe nonconstant denominator with explicit nonzero witness can be used even when numerator/denominator is not polynomial.
2. Unsafe nonconstant denominator without witness is not substituted and not treated as InvalidInput/Unsupported.
3. Denominator guard and provenance are preserved.
4. Transformed relations are exact after denominator clearing or rational substitution.
5. Target variable cannot be eliminated by preprocessing.
6. Tests include the safe system with (x+1)*s-1, (x+1)*y-(T+x), y-2.
7. Tests include the unsafe no-witness variant.
```

Do not pass if the implementation simply documents the limitation.

---

## P5R-d reviewer prompt — quotient/action provenance

Review P5R-d. Inspect `algebra/quotient.rs`, `algebra/krylov.rs`, any new provenance files, production kernel interfaces, and tests.

Fail if `TargetActionKrylov` production support can be generated from an externally injected action matrix without independent normal-form certificates from authorized relations.

Required checks:

```text
1. Production and debug/test explicit handles are separated.
2. Production handles are bound to authorized block relations or equivalent authorization hash.
3. Every action column has an independent normal-form certificate.
4. Column verification is not circular through the same injected columns.
5. Production TargetActionKrylov rejects debug explicit handles.
6. Tampered authorization hash or action column certificate fails.
7. VerifiedCharacteristicSupportCoverage remains the only accepted coverage kind.
```

Do not pass if the code only hashes the handle or checks self-consistency of action columns.

---

## P5R-e reviewer prompt — primitive scope ledger and anti-overclaim

Review P5R-e. Inspect `PRIMITIVE_SCOPE_LEDGER.md`, `PLAN.md`, `SOURCE_MAP.md`, and any references in reviewer prompts.

Fail if narrow primitives can still be read as generic kernel completion.

Required checks:

```text
1. Ledger covers resultant, interpolation, regular-chain, norm-trace, F4, linear-affine, quotient, and Krylov/action primitives.
2. Each entry states capability, limitations, allowed production use, exact verification, allowed failure, forbidden claim, and later expansion phase.
3. P6/P8/P9 plan sections reference the ledger and cannot overclaim primitive completeness.
4. The ledger is not used to make narrow-slice completion acceptable.
5. Later reviewers are instructed to consult the ledger.
```

Do not pass if the ledger is vague, apologetic, or merely descriptive without binding consequences.

---

## P5R-f reviewer prompt — P6 readiness audit

Review P5R-f. Inspect all P5R evidence, all P5R review summaries, `P6_READINESS.md`, `CLOSURE.md`, `ACTIVE_CONTEXT.md`, static scans, and relevant code changes.

Fail unless P6 can start without inheriting the known unsafe paths.

Required checks:

```text
1. P5R-a through P5R-e have PASS review archives.
2. P5R-specific tests and full crate tests pass on current commit.
3. Static scan hits are classified, not ignored.
4. P6_READINESS.md explicitly answers all seven readiness questions.
5. Fake F4 production claim is impossible.
6. Guarded rational affine semantics are implemented or exactly equivalent.
7. Production TargetActionKrylov cannot use self-certifying injected handles.
8. Primitive scope ledger blocks overclaim.
9. Claim ceiling remains PARTIAL_MECHANISM_READY:MECH-004.
10. Public orchestration is not represented as solved before P14/P15/P16.
```

The reviewer must include a final explicit sentence:

```text
P6 may begin: yes/no.
```

If the answer is no, `phase_closable` must be false.
