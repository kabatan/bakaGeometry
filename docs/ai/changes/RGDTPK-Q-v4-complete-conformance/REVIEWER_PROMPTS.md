# Reviewer Prompts: R-GDTPK-Q / ACCTP-Q v4 有限 Candidate-Cover 準拠修正

## Global reviewer instructions

You are reviewing a Guardian Lane implementation of `RGDTPK-Q-v4-finite-candidate-cover`.
The review scope is finite candidate-cover completion only. Exact target-image equality and real-fiber/slack/guard final classification are out of scope except for the required exact-image request scope guard.

Source authority:
- Original source: `docs/ai/sources/geosolver_core_r_gdtpk_q_algorithm_spec_v4.md`, blob sha `ef108f0dc95880d2e3030c96872b9073be995274`.
- Base Spec: `docs/ai/changes/RGDTPK-Q-v4-complete-conformance/BASE_SPEC.md`.
- Plan: `docs/ai/changes/RGDTPK-Q-v4-complete-conformance/PLAN.md`.

Return exactly one of:
```text
PASS
FAIL
NEEDS_MORE_EVIDENCE
```

Do not PASS from tests alone. You must inspect implementation code and compare it with Base Spec R-IDs and source sections. Tests, CI, and audit scripts are supporting evidence only.

A reviewer PASS means:
- no visible source drift within the finite candidate-cover layer,
- no hidden simplification,
- no test-only implementation,
- no forbidden fallback,
- required certificates are real and replayable,
- failure statuses carry source-required evidence.

Return FAIL immediately if any of these are found:
- `todo!`, `unimplemented!`, placeholder production function, or fake certificate in a required path.
- In-scope source-required algorithm implemented only under `#[cfg(test)]`.
- Descartes root isolation silently delegates to Sturm.
- F4 implementation is only a wrapper around Groebner.
- UniversalTargetElimination internal strategy list differs from source section 20.4 without approved Base Spec amendment.
- A production path returns a candidate relation without exact Q verification.
- A route fallback is not declared in the plan/ladder/certificate.
- Geometry-name, fixture-id, expected-answer, or variable-role dispatch.
- Coordinate solution list, full coordinate RUR, global lex parametrization, QE/CAD fallback.
- Nonfinite certificate based only on tiny rational witness search while claiming finite candidate-cover completeness.
- Regular-chain implementation lacks component/guard/projection semantics but claims candidate-cover completion for its route.
- Exact-image mode silently accepts/rejects or returns exact-image success instead of an explicit out-of-scope guard.

For every review, include:
1. Scope reviewed.
2. Files inspected.
3. R-IDs checked.
4. Source sections checked.
5. Evidence accepted.
6. Evidence missing.
7. Blocking findings.
8. Decision.

---

## RP-P0 — Source lock and audit harness reviewer

Review phase: P0  
Relevant R-IDs: BS-R000, BS-R010, BS-R150  
Files to inspect:
- `docs/ai/changes/RGDTPK-Q-v4-complete-conformance/**`
- `geosolver-core/scripts/audit_v4_conformance.py`
- `docs/ai/sources/geosolver_core_r_gdtpk_q_algorithm_spec_v4.md`

Checks:
1. Confirm the Base Spec and Plan are copied into the repo.
2. Confirm source path and blob sha are recorded.
3. Confirm audit script scans production files and is not written to suppress known gaps.
4. Confirm current gap inventory honestly lists all known noncompliance.
5. Confirm implementation has not started without explicit user approval if this is only planning.

PASS only if the harness makes failures visible. P0 may PASS even when audit reports implementation failures, but only if the audit is strict and honest.

---

## RP-P1 — Type/public API reviewer

Review phase: P1  
Relevant R-IDs: BS-R020, BS-R030, BS-R031, BS-R032, BS-R130

Inspect:
- public API,
- types,
- normalization,
- result/status/cost trace.

Checks:
1. `lib.rs` is only module exports.
2. `api::solve_target` returns `TargetSolveResult` and does not panic for normal solver errors.
3. Rational normalization is exact and total for valid inputs.
4. Polynomial normalization removes zero coefficients and duplicate monomials.
5. Hashes are deterministic and structure-bound.
6. Matrix density/shape/hash are exact.
7. Intervals are rational and validated.

FAIL if any source-required type/function is missing, placeholder, or float-based.

---

## RP-P2 — Problem/semantic/canonicalization reviewer

Review phase: P2  
Relevant R-IDs: BS-R001, BS-R040, BS-R041, BS-R042

Checks:
1. Variable roles never affect validation or dispatch.
2. Slack/branch semantic references are validated and preserved.
3. Canonicalization clears denominators primitively and preserves relation provenance.
4. Constant contradictions and invalid semantics return source statuses.
5. Resource context produces evidence-bearing `FiniteResourceFailure`.

FAIL if semantic provenance is dropped or geometry labels influence behavior.

---

## RP-P3 — Modular algebra and linear solve reviewer

Review phase: P3  
Relevant R-IDs: BS-R050, BS-R051, BS-R052, BS-R053  
MECHs: MECH-01, MECH-02

Checks:
1. Prime selection deterministic and avoids denominators.
2. CRT and rational reconstruction are exact and reject impossible reconstructions.
3. Sparse/dense row reduction has deterministic pivots.
4. Nullspace candidates are not treated as proof.
5. Membership certificates recompute exact Q identities.

FAIL if modular equality is accepted as final proof.

---

## RP-P4 — Groebner/F4/elimination reviewer

Review phase: P4  
Relevant R-IDs: BS-R054, BS-R055, BS-R096  
MECH: MECH-02

Checks:
1. `algebra/f4.rs` is production, not `#[cfg(test)]` only.
2. `f4_reduce_batch` constructs and reduces actual batch matrices.
3. `f4_elimination_local` returns Q[keep] generators with exact certificates.
4. `eliminate_to_keep_variables` dispatches only the declared strategy.
5. No hidden Groebner fallback inside F4 unless the plan explicitly declares Groebner.

FAIL if F4 is just a renamed Groebner call or if certificate verification is missing.

---

## RP-P5 — Preprocessing reviewer

Review phase: P5  
Relevant R-IDs: BS-R060, BS-R061

Checks:
1. Compression order exactly matches source.
2. Definitional elimination excludes target and records substitutions.
3. Affine elimination uses only safe pivots.
4. Binomial simplification is reversible or guarded.
5. Saturation only from explicit nonzero encoding.
6. Target-independent component removal cannot exclude any finite target value from the support cover.

FAIL if any rewrite loses semantic/certificate provenance.

---

## RP-P6 — Graph/DAG reviewer

Review phase: P6  
Relevant R-IDs: BS-R070, BS-R071

Checks:
1. Hypergraph contains every relation-variable incidence.
2. Influence graph BFS starts from target.
3. Weighted graph uses all source cost signals.
4. Decomposition tries source-specified separator families.
5. No useful separator produces one large block, not failure.
6. DAG validates relation coverage, authorization, duplication certificates, and root.

FAIL if any relation can be omitted, duplicated without certificate, or read by an unauthorized kernel.

---

## RP-P7 — Planner/ladder reviewer

Review phase: P7  
Relevant R-IDs: BS-R080, BS-R081, BS-R082  
MECH: MECH-03

Checks:
1. All nine kernels are registered in source order.
2. Admissions call every kernel.
3. Universal is admitted for every relation block.
4. Cost estimates are deterministic and not proof.
5. Declared ladder contains only certificate-capable coordinate-free kernels.
6. Every route has enforceable bounds and failure behavior.
7. Execution cannot call a route absent from declared ladder.

FAIL if hidden fallback remains possible.

---

## RP-P8 — TargetUnivariate/LinearAffine reviewer

Review phase: P8  
Relevant R-IDs: BS-R091, BS-R092

Checks:
1. TargetUnivariate scans local and child message relations.
2. Support uses primitive LCM/squarefree-compatible construction.
3. Source membership certificate binds source IDs/hashes.
4. LinearAffine pivots are safe and guard-bound.
5. Incomplete affine elimination returns correct failure.

FAIL if unsafe denominator division is possible.

---

## RP-P9 — TargetRelationSearch reviewer

Review phase: P9  
Relevant R-IDs: BS-R093  
MECH: MECH-01

Checks:
1. Kernel implements `g(Z)=Σq_i f_i` coefficient comparison.
2. Dense, sparse, and specialized support strategies exist or source-amended reason is present.
3. Modular solve returns candidates only.
4. Every success is exact Q membership verified.
5. Certificate contains all source fields.
6. AlgorithmicHardCase includes accumulated matrix trace.

FAIL if candidate relation can return without exact identity.

---

## RP-P10 — SparseResultant reviewer

Review phase: P10  
Relevant R-IDs: BS-R094

Checks:
1. Template represents source support-set resultant/eliminant semantics.
2. Two-polynomial special cases are not claimed as the full implementation unless source amended.
3. Resource caps return `FiniteResourceFailure`.
4. Resultant relation variables are subset of exported variables.
5. Certificate recomputes exact relation or proves exact membership.

FAIL if a resultant is accepted only by modular trace or small determinant hash.

---

## RP-P11 — ActionKrylov reviewer

Review phase: P11  
Relevant R-IDs: BS-R095

Checks:
1. Quotient handle does not expose coordinate roots, coordinate solution list, full coordinate RUR, or target-unrelated full quotient basis.
2. Krylov sequence uses deterministic coverage probes.
3. Coverage certificate cannot miss target-relevant eigenvalues.
4. `verify_annihilator` is exact.
5. No candidate polynomial returns without coverage.

FAIL if single Krylov recurrence is used without coverage proof.

---

## RP-P12 — Universal reviewer

Review phase: P12  
Relevant R-IDs: BS-R096  
MECHs: MECH-02, MECH-03

Checks:
1. Universal admission true for Q-polynomial relation blocks.
2. Internal strategies exactly match source section 20.4:
   - EliminationGroebnerLocal
   - F4EliminationLocal
   - TargetRelationSearchEscalated
   - ResultantIfSquareOrOverdetermined
   - SpecializeProjectInterpolateVerify
3. NormTrace, RegularChain, ActionKrylov are not Universal internal stages.
4. Strategy is declared before execution and certificate-bound.
5. Every exported generator is in Q[Z] and exactly verified.
6. Empty generator path either certifies nonfinite or returns AlgorithmicHardCase.

FAIL on any hidden strategy, undeclared fallback, or unverified generator.

---

## RP-P13 — RegularChain/NormTrace/Specialization reviewer

Review phase: P13  
Relevant R-IDs: BS-R097, BS-R098, BS-R099

Checks:
1. RegularChain has component/guard/projection semantics and certificates.
2. NormTrace detects algebraic tower by form, not geometry label.
3. Norm relation is exactly verified.
4. Specialization sample points are deterministic and certificate-bound.
5. Interpolated relation is accepted only after exact Q verification.

FAIL if any of these kernels are fixture-shaped wrappers.

---

## RP-P14 — Composition/support/nonfinite/replay reviewer

Review phase: P14  
Relevant R-IDs: BS-R110, BS-R111, BS-R112, BS-R113  
MECH: MECH-04

Checks:
1. Composition follows TargetProjectionDAG.
2. Separator elimination uses message-only pseudo blocks.
3. Original coordinate system is not globally reconstructed.
4. Fixed heuristic thresholds are not used as proof of no eliminant.
5. Support uses primitive LCM where applicable.
6. `verify_global_support` proves target-only or composed-ideal membership.
7. Nonfinite certificate uses elimination/dimension/algebraic-dependence evidence.
8. Replay recomputes input/canonical/DAG/message/support/root hashes.

FAIL if nonfinite proof is only small rational witness search or if replay is only hash comparison.

---

## RP-P15 — Roots/candidates reviewer

Review phase: P15  
Relevant R-IDs: BS-R120, BS-R121  
MECH: MECH-05

Checks:
1. Squarefree rejects zero support.
2. Sturm is exact.
3. Descartes/Vincent is distinct exact implementation, not alias.
4. Isolation intervals are rational and isolate one root each.
5. Candidate hash binds target, support hash, root index, interval.
6. Candidate hashes and intervals are replayable; Sign/Thom routines are not required unless used by candidate-cover code.

FAIL if any root isolation uses float-only approximation.

---

## RP-P16 — Exact-image scope-guard reviewer

Review phase: P16  
Relevant R-IDs: BS-R003, BS-R040, BS-R122  
MECH: MECH-06

Checks:
1. Every exposed exact-image option/status/finalizer path is audited.
2. Exact-image request returns only an explicit `ExactImageOutOfScope` diagnostic or allowed failure status.
3. No `CertifiedExactTargetImage` success is reachable in this scoped repair.
4. Candidate-cover mode does not call exact-image classification or filter candidates by slack/guard semantics.
5. Semantic provenance hashes remain available for certificates when present.
6. The scope guard is replay/certificate-bound.

FAIL if exact-image mode filters candidates, accepts all candidates, rejects candidates, or returns exact-image success.

---

## RP-P17 — Orchestrator/result/cost reviewer

Review phase: P17  
Relevant R-IDs: BS-R130, BS-R131, BS-R140  
MECH: MECH-07

Checks:
1. Pipeline order exactly matches the scoped candidate-cover source path.
2. Message verification occurs before composition.
3. Support verification occurs before roots.
4. Failure results preserve stage/cost evidence.
5. Cost trace contains every source parameter.
6. Final certificate binds all in-scope source fields.

FAIL if any pipeline stage is skipped or reordered for convenience.

---

## RP-P18 — Final finite candidate-cover conformance reviewer

Review phase: P18  
Relevant R-IDs: all  
MECHs: all

Checks:
1. Source-to-code matrix covers every in-scope source section and every in-scope Base Spec R-ID, with exact-image-only sections marked `OUT_OF_SCOPE`.
2. Audit script passes strict mode.
3. Cargo fmt/clippy/test evidence is fresh.
4. All prior reviewer findings are resolved.
5. No blocking QuestionDebt remains.
6. All 16 finite candidate-cover completion conditions are individually PASS.
7. Final claim is no stronger than evidence.

Return PASS only if the implementation is truly source-faithful to the v4 finite candidate-cover layer, not merely test-passing.
