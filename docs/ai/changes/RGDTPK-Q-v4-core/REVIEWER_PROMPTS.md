# Reviewer Prompts for R-GDTPK-Q / ACCTP-Q Guardian implementation — v2.2 consistency-hardened

## General instructions for every reviewer

You are a read-only Guardian reviewer. Your job is to prevent drift from `BASE_SPEC.md`. Do not pass a phase because files, tests, evidence folders, or review summaries exist. Pass only if the implementation behavior satisfies the listed R-IDs and MECHs.

Before reviewing any phase:

1. Read the listed Base Spec R-IDs, especially `RGQ-041` through `RGQ-064` when relevant.
2. Read the phase or subphase in `PLAN.md`.
3. Read `REVIEW_ARCHIVE_SCHEMA.md`.
4. Inspect actual code paths, not only evidence summaries.
5. Inspect command outputs and confirm they are fresh after the last relevant code change.
6. Inspect `function_implementation_table.yaml` for the phase and verify it against code.
7. Look for generalized failure patterns: heavy fallback, narrow slice, hidden fallback, decorative DAG/certificate, unsupported hiding algorithmic gaps, preflight as completion, placeholder roots/candidates, geometry dispatch, exact-image overclaim, review-summary-only closure, non-spec statuses, hook-only required functions, and PASS summaries with blockers.

Return exactly one status:

```text
PASS
FAIL_FIXABLE
FAIL_BLOCKING
USER_DECISION_REQUIRED
```

A PASS must include reviewed R-IDs, MECHs, files inspected, commands inspected, evidence inspected, algorithmic sufficiency judgment, semantic/tamper challenges considered, and residual risks. A PASS is invalid if controlling code is a stub, wrapper over a forbidden path, hard-coded stress matcher, fake certificate, unconditional verifier, or placeholder implementation.

A FAIL must include exact R-ID/MECH, code/evidence location, why this is algorithmic rather than merely missing paperwork, and the minimal fix required.

You must explicitly say whether `review_summary.yaml` is schema-valid, whether `REVIEW_SUMMARY_SCHEMA.yaml` and `schemas/review_summary.schema.yaml` are byte-identical when relevant, and whether the summary is consistent with `prompt.md` and `response.md`. Do not accept a phase if the summary says PASS while the response contains unresolved blockers, required fixes, or a non-PASS status.

---

## P0 — Source anchoring and review schema

Review target: Plan P0.

Check:

- Source hashes match the two supplied documents recorded in `BASE_SPEC.md`.
- `BASE_SPEC.md` includes v2 hardening R-IDs `RGQ-041` through `RGQ-064`.
- `PLAN.md` includes split P3a–P3f and P8a–P8d, corrected P15/P16 claim rules, the RGQ-061 three-suite partition, and the RGQ-064 consistency audit.
- `REVIEW_ARCHIVE_SCHEMA.md` exists and contains prompt/response/evidence-manifest archive requirements.
- `SOURCE_MAP.md` and `ISSUE_FIX_MAP.md` map the eight holes to R-IDs and phases.
- No algorithmic R-ID is claimed as closed.

Fail if setup is treated as solver progress, if the review/evidence schemas are optional, or if `REVIEW_SUMMARY_SCHEMA.yaml` differs from `schemas/review_summary.schema.yaml`.

---

## P1 — Types and exact algebra base

Review target: Plan P1.

Check exact rational, monomial, polynomial, univariate, matrix, interval, and hash behavior. Verify deterministic normalization and hash stability. Verify that the public status enum remains closed under RGQ-058 and no `NotYetImplemented`/`Unsupported` status was added. Fail if any exact path uses floating arithmetic, dummy hashes, unreduced rationals, placeholder fields, non-spec statuses, or compile-only tests.

---

## P2 — Problem layer and statuses

Review target: Plan P2.

Check validation/canonicalization/status semantics. Fail if coordinate/slack/branch roles affect acceptance, if `Unsupported` is a normal status, if geometry labels are inspected, or if semantic provenance is discarded.

---

## P3a — Monomial orders, polynomial ops, reductions, membership

Review target: Plan P3a.

Check exact reductions and membership verification by reconstructing `g - Σ q_i f_i`. Fail if membership proof checks only hashes, returns unconditional success, ignores coefficients, or lacks negative tests.

---

## P3b — Modular arithmetic, CRT, reconstruction, matrices

Review target: Plan P3b.

Check deterministic prime selection, modular rank/nullspace, CRT, rational reconstruction, and exact-check handoff. Fail if modular evidence is accepted as proof without Q verification or if prime/order choices are nondeterministic.

---

## P3c — Local Groebner/F4 elimination APIs

Review target: Plan P3c.

Check that Groebner/F4/elimination are local block target/separator operations only. Output must be restricted to keep variables and exact certificates. Fail if any API computes coordinate solution lists, full coordinate RUR, global coordinate lex parametrization, or solve-all-coordinates then reads target.

---

## P3d — Resultant and interpolation primitives

Review target: Plan P3d.

Check sparse resultant and interpolation primitives. They may generate candidate relations, but final acceptance requires exact Q membership/elimination verification. Fail if bad samples/templates can pass or if sparse support replaces the mandatory dense relation-search schedule.

---

## P3e — Quotient/action and characteristic coverage primitives

Review target: Plan P3e.

Check `TargetQuotientHandle` exposes no coordinate roots/RUR. Check target action matrix materialization, per-column normal-form verification, exact characteristic polynomial, and exact Cayley-Hamilton verification. Fail if single-vector Krylov, block Wiedemann, trace powers, or `S(M_T)=0` without characteristic support coverage can produce a relation.

---

## P3f — Regular-chain, norm/trace, real-root/sign helpers

Review target: Plan P3f.

Check algebraic tower detection, regular-chain projection semantics, exact root helpers, and exact sign/Thom helpers. Fail if detection uses geometry names, floating-only root/sign code, or drops component/guard semantics.

---

## P4 — Preprocess

Review target: Plan P4.

Check ordered compression steps, guard-provenance preservation, explicit saturation only from explicit nonzero encodings, and exact-image feasibility obligations for target-independent components. Fail on unsafe rewrite or component selection without certificate.

---

## P5 — Graphs and DAG

Review target: Plan P5.

Check every polynomial occurrence in hypergraph, algebraic-only weights, no-separator one-large-block path, authorization hashes, duplication certificates, and mismatch tests. Fail if DAG is decorative or execution can read arbitrary relations.

---

## P6 — Planner and declared ladder

Review target: Plan P6.

Check all kernels are admitted/probed, Universal admission is true for well-formed blocks, selected plans contain concrete support-producing information, TargetRelationSearch plans use RGQ-042, Universal plans use RGQ-041, and declared ladder is hash-bound before execution. Fail if admission is just a feature label or runtime chooses unplanned kernels.

P6 prerequisite: inspect `PRIMITIVE_SCOPE_LEDGER.md` and `P6_READINESS.md`. Fail if P6 begins before P5R-a through P5R-f have PASS review archives, or if P6 treats binary resultants, one-variable interpolation, single-chain regular-chain, single-variable tower norm/trace, non-production F4, polynomial-only affine substitution, or debug explicit quotient/action handles as completed generic kernels.

---

## P7 — Kernel infrastructure, TargetUnivariate, LinearAffine

Review target: Plan P7.

Check the kernel registry lists all nine kernels. TargetUnivariate must compute primitive LCM squarefree support from verified target-only relations. LinearAffine must use only safe pivots or recorded nonzero guards and must export only allowed variables. Fail on unsafe pivots, admission/execute mismatch, or missing replay path.

---

## P8a — TargetRelationSearch deterministic schedule

Review target: Plan P8a.

Check RGQ-042 exactly:

- `z_seed`, `e_cap_default`, `A_e`, `B_i,e`, and `C_e` are computed by formula.
- Support hashes are reproducible from `J,Y,Z,options`.
- The membership matrix represents `g(Z) - Σ q_i f_i = 0`.
- Modular nullspace only generates candidates.
- Exact Q identity is required before return.
- Exhaustion returns hard/resource status, never nonfinite.

Fail if there is sparse-only search, hand-picked degree/support bounds, stress-specific supports, or relation return without exact identity.

Also inspect `PRIMITIVE_SCOPE_LEDGER.md`. Fail if TargetRelationSearch is used as a narrow feature gate or if P8a claims generic support-producing readiness without the declared dense schedule and exact membership route.

---

## P8b — SparseResultant and SpecializationInterpolation kernels

Review target: Plan P8b.

Check actual exact verification functions for resultant and interpolated relations, not hook-only placeholders. Fail if “not sparse” becomes unsupported, if interpolation proof is only sample agreement, if candidate generation is treated as proof, or if Appendix A §10.12–§10.13 required functions are deferred.

Also inspect `PRIMITIVE_SCOPE_LEDGER.md`. Fail if the current binary resultant or one-variable interpolation helpers are overclaimed as generic P8b kernels.

---

## P8c — ActionKrylov with VerifiedCharacteristicSupportCoverage

Review target: Plan P8c.

Check RGQ-044 and RGQ-054. The only accepted coverage kind is `VerifiedCharacteristicSupportCoverage`; the support polynomial is the exact characteristic polynomial of the verified target-action matrix. The undercoverage regression must be present. Fail any candidate from weaker Krylov coverage.

Also inspect `PRIMITIVE_SCOPE_LEDGER.md`. Fail if production TargetActionKrylov can accept debug explicit handles or externally injected self-certifying action columns.

---

## P8d — UniversalTargetEliminationKernel

Review target: Plan P8d.

Check RGQ-041 and RGQ-056. Universal must be bounded, local, preplanned, authorization-bound, and target/separator-export only. It must use exactly the fixed strategy sequence, must not return local nonfinite, and must not use global coordinate-first elimination. Fail on unbounded Groebner/F4, hidden fallback, full coordinate RUR/roots, or relation-search failure routed to nonfinite.

Also inspect `PRIMITIVE_SCOPE_LEDGER.md`. Fail if Universal local elimination is closed by non-production F4 naming, narrow helper primitives alone, or debug quotient/action handles.

---

## P9 — RegularChain and NormTrace kernels

Review target: Plan P9.

Check component/guard/projection semantics and algebraic tower norm verification. Fail on geometry-name detection, unverified norm relation, or dropped component semantics.

Also inspect `PRIMITIVE_SCOPE_LEDGER.md`. Fail if single-chain regular-chain helpers or single-variable tower norm/trace helpers are overclaimed as generic P9 kernel completion.

---

## P10 — Composition, final support, nonfinite certification

Review target: Plan P10.

Check separator elimination uses only message relations, final support is target-only and verified, and nonfinite status requires RGQ-045 certificate. Check RGQ-057 Appendix overrides and RGQ-058 status/error mapping. Fail if no target relation, relation-search exhaustion, or local Universal failure can become `CertifiedNonFiniteTargetImage` without the required proof, or if status enum values are mixed into internal errors without an explicit reviewed mapping.

---

## P11 — Verification and replay

Review target: Plan P11.

Check every certificate variant performs real exact verification, global support proof is exact, replay binds all hashes and invariant flags, and tamper/deletion tests fail. Fail on unconditional `Ok(())`, hash-only algebraic proof, or invariant flags set true without enforcement.

---

## P12 — Roots and decode

Review target: Plan P12.

Check exact squarefree support, exact real root isolation, mandatory `roots/algebraic_number.rs`, and decoded candidates bound to support hash/root index/interval. Fail on floating-only roots, approximate roots without rational isolating intervals, placeholder candidates, optional/deferred algebraic-number implementation, or success with empty roots when real roots exist.

---

## P13 — Exact image mode

Review target: Plan P13.

Check real fiber, guard, slack, and branch semantics actually execute before exact-image statuses. Fail if complex candidate-cover evidence is used as exact real image proof, if semantic encodings are ignored, or if exact-image nonfinite lacks real nonfinite certificate.

---

## P14 — Orchestrator and cost trace

Review target: Plan P14.

Check full pipeline order, no panic for expected solver errors, truthful statuses, all result fields, cost trace fields, and invariant flags. Fail if any stage is skipped, if cost trace is dummy zeros where data exists, or if candidate-cover/exact-image statuses are confused.

---

## P15 — Acceptance stress and anti-drift checks

Review target: Plan P15.

Check the three suites from RGQ-061 separately: support-producing candidate-cover, exact-image semantics, and failure/nonfinite semantics. Every suite A case must return candidate-cover or nonempty exact-image success with nonzero support, exact support verification, exact root isolation, and non-placeholder candidates. Hard-case/resource/certificate-gap/empty-image/nonfinite is a failure for suite A. Check anti-fallback, anti-decoration, variable-renaming, and relation-permutation dynamic tests. Fail if hard cases are counted as acceptance, if stress strings are hard-coded, if exact-empty is counted as support-producing acceptance, or if only toy examples represent the generalized structures.

---

## P16 — Final closure and claim ladder

Review target: Plan P16.

Check all R-IDs and MECHs required by the claimed label, especially RGQ-057 through RGQ-064. Verify fresh full commands, static scans, review archive schema, schema mirror hash, PASS-with-blocker rejection, prompt/response hashes, support-producing table, exact-image semantics table, failure/nonfinite table, `CONSISTENCY_AUDIT.md`, replay/tamper evidence, and exact final claim.

Allowed labels only:

```text
SCAFFOLD_READY
PARTIAL_MECHANISM_READY:<MECH-ID>
CANDIDATE_COVER_CORE_READY
EXACT_IMAGE_CORE_READY
RGDTPK_Q_V4_ACCEPTANCE_COMPLETE
SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC
```

Fail if the closure uses the old phrase `the old candidate-cover-only acceptance-complete phrase`, claims acceptance complete without exact image, claims benchmark superiority, claims universal finite-system completeness, or claims geometry DSL support.
