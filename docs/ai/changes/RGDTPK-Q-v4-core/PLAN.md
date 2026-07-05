# Plan: Implement R-GDTPK-Q / ACCTP-Q solver core under Guardian Lane — v2.2 consistency-hardened

## Context Packet

Spec ID: `RGDTPK-Q-v4-core`  
Type: Plan Contract  
Status: DRAFT FOR USER APPROVAL — v2.2 consistency-hardened  
Parent: `BASE_SPEC.md` in this directory.  
Scope: implementation plan for the `geosolver-core/` crate from an empty repo.  
Applies To: all files required by Base Spec `RGQ-007` through `RGQ-030`, v2/v2.2 hardening R-IDs `RGQ-041` through `RGQ-064`, all MECHs, evidence, review archives, and final closure.  
Required Parent R-IDs: all Base Spec R-IDs `RGQ-000` through `RGQ-064`.  
Blocking Questions: none.  
Non-blocking Debt: none accepted for final `RGDTPK_Q_V4_ACCEPTANCE_COMPLETE`.  
Known Exceptions: none.  
Read-First R-IDs: `RGQ-004`, `RGQ-011`, `RGQ-015`, `RGQ-019`, `RGQ-021`, `RGQ-022`, `RGQ-032`, `RGQ-034`, `RGQ-036`, `RGQ-041` through `RGQ-064`.  
Read full file only when: admitting the Plan, beginning a phase or subphase, closing a phase or subphase, handling verification failure, or making any claim.  
Context Packet Authority: non-authoritative digest. The body below is authoritative unless it conflicts with `BASE_SPEC.md`; in a conflict, `BASE_SPEC.md` wins.

---

## 0. Execution rules for the Agent

1. Start implementation only after the user explicitly approves both `BASE_SPEC.md` and this `PLAN.md`.
2. Do not weaken the Base Spec during implementation. If a conflict or impossibility appears, stop and record `PlanDefect` or `BaseSpecConflict`.
3. Do not close a phase with scaffolding, stubs, wrappers, compile-only code, or evidence-only progress.
4. Do not count reviewer PASS as completion. Completion requires implemented behavior, fresh verification evidence, archived prompt/response, schema-valid `review_summary.yaml`, and reviewer algorithmic sufficiency.
5. Every reviewer invocation must follow `REVIEW_ARCHIVE_SCHEMA.md` and validate `review_summary.yaml` against `REVIEW_SUMMARY_SCHEMA.yaml`. Missing `prompt.md`, `response.md`, `review_summary.yaml`, or `evidence_manifest.yaml` keeps the phase open.
6. P3 and P8 are umbrellas only. They cannot be closed as a single bulk phase. Each subphase P3a–P3f and P8a–P8d requires its own implementation evidence and review archive.
7. Every algorithmic phase after P2 must write `evidence/<phase>/function_implementation_table.yaml` as required by `RGQ-046`.
8. Do not use geometry names, problem IDs, fixture IDs, expected answers, or official solutions as dispatch inputs.
9. Do not return ordinary `Unsupported` for a well-formed Q-polynomial target problem.
10. Do not introduce `todo!`, `unimplemented!`, placeholder roots/candidates/fibers, fake certificates, unconditional verification success, dummy hashes, or stress-string dispatch.
11. Hidden fallback is forbidden. Every ladder-like step must be declared before execution and hash-bound in the certificate.
12. Exact-image claims are forbidden until P13 and P16 prove real fiber, guard, slack, and branch semantics. Candidate-cover completion is not acceptance completion.
13. Public statuses are closed under `RGQ-058`; do not add `NotYetImplemented`, `Unsupported`, `Skipped`, or phase-local statuses.
14. Required functions cannot be optional or hook-only after their owning phase under `RGQ-059`.
15. P15 must use the three-suite partition in `RGQ-061`, and P16 must include the consistency audit required by `RGQ-064`.

Each phase must produce:

```text
docs/ai/changes/RGDTPK-Q-v4-core/evidence/<phase>/
  commands.txt
  command_outputs.txt
  function_implementation_table.yaml
  static_scans.txt
  notes.md

docs/ai/changes/RGDTPK-Q-v4-core/reviews/<phase>/<YYYYMMDD-HHMMSSZ>/
  prompt.md
  response.md
  review_summary.yaml
  evidence_manifest.yaml
```

---

## 1. Repository and documentation setup

### P0 — Guardian artifact setup, source anchoring, and review schema

**Supports R-IDs:** `RGQ-000`, `RGQ-035`, `RGQ-040`, `RGQ-047`, `RGQ-052`, `RGQ-060`, `RGQ-064`.  
**MECHs:** starts `MECH-016`; closes no algorithmic MECH.

**Implementation tasks.**

1. Create the Guardian documentation layout:

```text
docs/ai/
  SPEC_REGISTRY.md
  ACTIVE_CONTEXT.md
  sources/
    geosolver_core_r_gdtpk_q_algorithm_spec_v4.md
    geosolver_failure_causes_generalized_2026_07_04.md
  changes/RGDTPK-Q-v4-core/
    BASE_SPEC.md
    PLAN.md
    SOURCE_MAP.md
    REVIEWER_PROMPTS.md
    REVIEW_ARCHIVE_SCHEMA.md
    REVIEW_SUMMARY_SCHEMA.yaml
    ISSUE_FIX_MAP.md
    CLOSURE.md
    SESSION_HANDOFF.md
    evidence/
    reviews/
```

2. Copy both source documents exactly and record SHA-256 hashes.
3. Copy `BASE_SPEC.md`, `PLAN.md`, `SOURCE_MAP.md`, `REVIEWER_PROMPTS.md`, `REVIEW_ARCHIVE_SCHEMA.md`, and `REVIEW_SUMMARY_SCHEMA.yaml` exactly.
4. Create `ISSUE_FIX_MAP.md` mapping the eight user-identified holes to `RGQ-041` through `RGQ-064` and Plan phases.
5. Create `ACTIVE_CONTEXT.md` as a navigation aid only. It must not add or weaken requirements.
6. Create or copy the machine-readable schema files `schemas/review_summary.schema.yaml` and `schemas/evidence_manifest.schema.yaml`; `REVIEW_SUMMARY_SCHEMA.yaml` and `schemas/review_summary.schema.yaml` must be byte-identical mirrors.
7. Create `CONSISTENCY_AUDIT.md` with the checks required by `RGQ-064`.
8. Provide validation commands that reject malformed summaries/manifests, PASS summaries with false pass conditions, PASS summaries with nonempty `blocking_findings` or `required_fixes`, and non-identical review schema mirrors.

**Forbidden shortcuts.**

- No algorithmic R-ID closure.
- No claim that documentation setup implements solver behavior.
- No reviewer summary without archived prompt and response.

**Closure evidence.**

- `sha256sum` of sources and Guardian artifacts.
- `find docs/ai -maxdepth 5 -type f | sort`.
- Schema validation of intentionally minimal valid and invalid review summaries, including PASS-with-blocker rejection.
- Byte-identity hash check for `REVIEW_SUMMARY_SCHEMA.yaml` and `schemas/review_summary.schema.yaml`.
- `CONSISTENCY_AUDIT.md` output for source hashes, R-ID references, phase prompts, schema mirrors, and RGQ-051 polarity.
- Review archive for P0.

**Reviewer prompt.** Use `REVIEWER_PROMPTS.md#P0`.

---

## 2. Rust crate scaffold and exact base types

### P1 — Crate scaffold, public API shell, IDs, hashes, rationals, monomials, polynomials, univariates, matrices, intervals

**Supports R-IDs:** `RGQ-007`, `RGQ-008`, `RGQ-009`, `RGQ-010`.  
**MECHs:** starts `MECH-001`.

**Implementation tasks.**

1. Create `geosolver-core/Cargo.toml` and `README.md`.
2. Add only exact-arithmetic, hashing, deterministic serialization, and testing dependencies. External CAS output may not be a production proof engine.
3. Create every source file listed in Base Spec section 6.
4. Implement `src/lib.rs` and `src/api.rs` with the required public API shape. Before P14 connects the full orchestrator, any temporary scaffold return must use an existing `SolverStatus` plus diagnostic `TemporaryPipelineNotConnected`; no new public status such as `NotYetImplemented` may be added, and P14 must remove the temporary path.
5. Implement `types/ids.rs`, `hash.rs`, `rational.rs`, `monomial.rs`, `polynomial.rs`, `univariate.rs`, `matrix.rs`, and `interval.rs`.
6. Enforce all invariants: normalized rational numbers, sorted monomials, normalized sparse polynomials, deterministic hashes, exact univariate gcd/squarefree helper foundations, exact intervals.
7. Add tests for normalization, exact identities, deterministic hashes, substitution, univariate gcd/squarefree, and interval invariants.

**Closure evidence.**

- `cargo fmt --check`.
- Targeted `types` tests.
- Static scan for `todo!|unimplemented!|placeholder|dummy|fake|stub|panic!("TODO`.
- `function_implementation_table.yaml` for all public functions implemented in P1.

**Reviewer prompt.** Use `REVIEWER_PROMPTS.md#P1`.

---

## 3. Problem input, semantics, validation, canonicalization, context, statuses

### P2 — Problem layer and solver status semantics

**Supports R-IDs:** `RGQ-001`, `RGQ-002`, `RGQ-003`, `RGQ-006`, `RGQ-012`, `RGQ-030`, `RGQ-058`.  
**MECHs:** continues `MECH-001`; starts `MECH-002` and `MECH-012` data/API skeleton.

**Implementation tasks.**

1. Implement `problem/input.rs`, `semantic.rs`, `validate.rs`, `canonicalize.rs`, `context.rs`.
2. Implement `result/status.rs`, `diagnostics.rs`, `cost_trace.rs`, `output.rs` with all specified statuses and evidence fields.
3. Define `SolverError` and conversions into `TargetSolveResult` without panics.
4. Validation must reject only invalid Q-polynomial target systems or invalid semantic references.
5. Coordinate/slack/branch/selector roles must never control solver acceptance or dispatch.
6. Canonicalization must clear denominators, normalize, remove zero relations with diagnostics, handle nonzero constants exactly, and preserve semantic provenance.
7. Add tests for role-insensitive acceptance, branch/slack acceptance, invalid target, invalid semantic reference, denominator normalization, and nonzero constant contradiction.

**Closure evidence.**

- Targeted problem/status tests.
- Static scan for `Unsupported` and geometry/fixture/expected-answer dispatch words in production code.
- Function implementation table.
- Review archive.

**Reviewer prompt.** Use `REVIEWER_PROMPTS.md#P2`.

---

## 4. Algebra primitives — split phases only

P3 is not a closeable phase. It is a group label for P3a through P3f. The Agent must not ask a reviewer to pass “P3” as one phase.

### P3a — Monomial orders, polynomial ops, reductions, exact membership verification

**Supports R-IDs:** `RGQ-010`, `RGQ-027`.  
**MECHs:** continues `MECH-001`; starts `MECH-010` identity verification foundation.

**Implementation tasks.**

1. Implement `algebra/monomial_order.rs`.
2. Implement `algebra/polynomial_ops.rs` leading terms, S-polynomials, reductions, primitive part.
3. Implement `algebra/normal_form.rs` including `verify_membership_by_certificate` by exact identity reconstruction.
4. Add tests where incorrect multipliers fail and correct multipliers pass.

**Fail conditions.** Any unconditional `Ok(())`, any reduction that ignores coefficients, or any membership proof that checks only hashes.

**Reviewer prompt.** Use `REVIEWER_PROMPTS.md#P3a`.

### P3b — Modular arithmetic, CRT, rational reconstruction, sparse/dense matrices, modular linear solving

**Supports R-IDs:** `RGQ-010`, `RGQ-019`, `RGQ-020`, `RGQ-025`.  
**MECHs:** continues `MECH-001`, starts `MECH-006` linear backbone.

**Implementation tasks.**

1. Implement `algebra/modular.rs`, `crt.rs`, `rational_reconstruction.rs`.
2. Implement `algebra/sparse_matrix.rs`, `dense_matrix.rs`, `linear_solve.rs`.
3. Modular solving must expose rank/nullspace/solve traces but must not certify a relation until caller exact-checks over Q.
4. Add deterministic prime-sequence tests, CRT round-trip tests, rational reconstruction tests, matrix rank/nullspace tests, and failure tests for unstable reconstruction.

**Fail conditions.** Floating arithmetic in exact path, nondeterministic primes, or accepting modular relation as proof.

**Reviewer prompt.** Use `REVIEWER_PROMPTS.md#P3b`.

### P3c — Local Groebner/F4 elimination APIs with exported-variable restriction

**Supports R-IDs:** `RGQ-005`, `RGQ-010`, `RGQ-022`, `RGQ-056`.  
**MECHs:** starts local part of `MECH-008`.

**Implementation tasks.**

1. Implement `algebra/groebner.rs` and `algebra/f4.rs` for local block elimination only.
2. Implement `algebra/elimination.rs` dispatcher with disjoint keep/eliminate checks.
3. Every exported generator must be in `Q[keep]` and carry membership or elimination certificates.
4. APIs must not return coordinate roots, full coordinate RUR, or coordinate solution lists.
5. Add tests where non-keep variables in output cause `ImplementationBug`.

**Fail conditions.** Any global coordinate-first solve, unbounded global lex basis, or “compute all coordinates then read target.”

**Reviewer prompt.** Use `REVIEWER_PROMPTS.md#P3c`.

### P3d — Resultants and specialization/interpolation primitives with exact verification

**Supports R-IDs:** `RGQ-020`, `RGQ-025`, `RGQ-043`.  
**MECHs:** continues `MECH-007`.

**Implementation tasks.**

1. Implement `algebra/resultant.rs` support sets, sparse template construction, modular computation, and exact certificate verification functions required by Appendix A §10.12.
2. Implement `algebra/interpolation.rs` specialization point choice, specialization, sparse coefficient interpolation, and exact verification functions required by Appendix A §10.13.
3. Candidate outputs from these primitives must remain untrusted until exact Q membership/elimination verification.
4. Add tests proving bad interpolation samples fail final verification.

**Reviewer prompt.** Use `REVIEWER_PROMPTS.md#P3d`.

### P3e — Quotient/action handles and verified characteristic support coverage primitives

**Supports R-IDs:** `RGQ-021`, `RGQ-044`, `RGQ-054`.  
**MECHs:** implements primitive layer for `MECH-014`.

**Implementation tasks.**

1. Implement `algebra/quotient.rs` `TargetQuotientHandle` without coordinate roots/RUR APIs.
2. Implement `algebra/krylov.rs` functions, but accepted coverage must be `VerifiedCharacteristicSupportCoverage` only.
3. Implement exact construction of target action matrix columns from the quotient handle.
4. Implement exact characteristic polynomial computation and exact Cayley-Hamilton verification.
5. Add undercoverage regression where a single-vector Krylov sequence would miss an eigenvalue.

**Fail conditions.** Returning a relation from single-vector Krylov, block Wiedemann, trace powers, or `S(M_T)=0` without characteristic support coverage.

**Reviewer prompt.** Use `REVIEWER_PROMPTS.md#P3e`.

### P3f — Regular-chain, norm/trace, exact real-root, and sign primitives

**Supports R-IDs:** `RGQ-023`, `RGQ-024`, `RGQ-028`, `RGQ-029`.  
**MECHs:** starts `MECH-007`, `MECH-011`, `MECH-012` with executable primitives, not empty helper foundations.

**Implementation tasks.**

1. Implement `algebra/regular_chain.rs` functions listed in Appendix A §10.16 with component semantics.
2. Implement `algebra/norm_trace.rs` functions listed in Appendix A §10.17 by algebraic form only.
3. Implement `algebra/real_root.rs` exact Sturm and/or Descartes functions listed in Appendix A §10.18.
4. Implement `algebra/sign.rs` exact sign and Thom functions listed in Appendix A §10.19.
5. Add tests for tower norm relation verification and exact sign/root helpers.

**Fail conditions.** Geometry-name tower detection, floating-only root/sign path, or component semantics dropped.

**Reviewer prompt.** Use `REVIEWER_PROMPTS.md#P3f`.

---

## 5. Preprocessing with provenance

### P4 — Compression, definitional/affine/binomial/saturation/independent components

**Supports R-IDs:** `RGQ-013`, `RGQ-003`, `RGQ-037`.  
**MECHs:** closes `MECH-003` only after guard-provenance tests pass.

**Implementation tasks.**

1. Implement `preprocess/compression.rs`, `definitional.rs`, `linear_affine.rs`, `binomial.rs`, `saturation.rs`, `independent.rs` in the exact order specified.
2. Nonconstant affine denominators may be used only with recorded nonzero guard semantics.
3. Explicit saturation may use only explicit nonzero encodings.
4. Target-independent components may be removed from candidate-cover construction but must remain exact-image feasibility obligations.
5. Add guarded affine, unsafe affine rejection, saturation, binomial, and independent-component tests.

**Reviewer prompt.** Use `REVIEWER_PROMPTS.md#P4`.

---

## 6. Graphs, decomposition, TargetProjectionDAG, metrics

### P5 — Algebraic graph construction and operational DAG authorization

**Supports R-IDs:** `RGQ-014`, `RGQ-033`, `RGQ-038`.  
**MECHs:** closes `MECH-004` only after deletion/tamper tests pass.

**Implementation tasks.**

1. Implement relation-variable hypergraph, target influence graph, weighted primal graph, separator candidates, target-rooted decomposition, projection DAG, and metrics.
2. Every polynomial variable occurrence must appear in the hypergraph.
3. If no useful separator exists, create one large target block.
4. Authorization hashes must constrain the relations each projector can read.
5. Add tests for DAG authorization mismatch, relation duplication without certificate, no-separator one-block path, and algebraic-only weighting.

**Reviewer prompt.** Use `REVIEWER_PROMPTS.md#P5`.

---

## 7. Planner, admissions, cost model, declared ladders

### P6 — Deterministic planner with support-producing plans

**Supports R-IDs:** `RGQ-015`, `RGQ-039`, `RGQ-041`, `RGQ-042`, `RGQ-047`, `RGQ-062`.  
**MECHs:** closes `MECH-005` and starts `MECH-013`, `MECH-016`.

**Implementation tasks.**

1. Implement `planner/cost_model.rs`, `probes.rs`, `admission.rs`, `kernel_plan.rs`, `ladder.rs`, `planner.rs`.
2. `collect_kernel_admissions` must call all kernels.
3. Universal admission must be true for every well-formed Q-polynomial block.
4. Every selected plan must include exported variables, eliminated variables, concrete bounds/support/template/rank plans, resource bounds, certificate route, and failure behavior.
5. `build_declared_ladder` must hash-bind the entire ladder before execution.
6. TargetRelationSearch plans must use the RGQ-042 schedule.
7. Universal plans must use the RGQ-041 fixed strategy sequence.
8. Cost probes may affect ordering but are never proof.
9. Add determinism tests: repeated planning yields identical plan hashes.
10. Add hidden-fallback tests: runtime cannot choose a kernel absent from the declared ladder.

**Reviewer prompt.** Use `REVIEWER_PROMPTS.md#P6`.

---

## 8. Kernel infrastructure and first exact kernels

### P7 — Kernel trait, registry, TargetUnivariate, LinearAffine

**Supports R-IDs:** `RGQ-016`, `RGQ-017`, `RGQ-018`, `RGQ-038`.  
**MECHs:** starts `MECH-007`.

**Implementation tasks.**

1. Implement `kernels/traits.rs` and `kernels/mod.rs` returning all nine kernels.
2. Implement `TargetUnivariateKernel` with primitive LCM squarefree support from verified target-only relations.
3. Implement `LinearAffineKernel` with safe pivots, guard recording, denominator clearing, and exported-variable restriction.
4. Add admission/execute consistency tests and unsafe pivot tests.

**Reviewer prompt.** Use `REVIEWER_PROMPTS.md#P7`.

---

## 9. Projection kernels — split phases only

P8 is not a closeable phase. It is a group label for P8a through P8d. The Agent must not ask a reviewer to pass “P8” as one phase.

### P8a — TargetRelationSearch deterministic dense schedule and exact membership execution

**Supports R-IDs:** `RGQ-019`, `RGQ-042`, `RGQ-043`, `RGQ-055`.  
**MECHs:** closes `MECH-006` and `MECH-013` only after schedule reproducibility and exact membership tests pass.

**Implementation tasks.**

1. Implement all required functions from `RGQ-042` in `kernels/target_relation_search.rs`.
2. Execute the dense schedule exactly from `z_seed` through `e_cap`; sparse accelerated attempts may exist but cannot replace the dense schedule.
3. Build membership matrices for `g(Z) - Σ q_i f_i = 0` with row set `C_e`.
4. Use modular nullspace only as candidate generation; accept a candidate only after exact Q identity verification.
5. On exhaustion, return `AlgorithmicHardCase` with full stage trace or `FiniteResourceFailure`; never nonfinite.
6. Add tests for three schedule reproducibility cases with different `|Y|`, `|Z|`, and degrees.
7. Add support-producing tests for multi-separator, bilinear/quadratic, and one-large-block algebraic cases.

**Reviewer prompt.** Use `REVIEWER_PROMPTS.md#P8a`.

### P8b — SparseResultant and SpecializationInterpolation kernels with exact verification

**Supports R-IDs:** `RGQ-020`, `RGQ-025`, `RGQ-043`.  
**MECHs:** continues `MECH-007`.

**Implementation tasks.**

1. Implement `kernels/sparse_resultant.rs` admission, planning, execution, and replay.
2. Implement `kernels/specialization_interpolation.rs` admission, planning, execution, and replay.
3. Every resultant/interpolated relation must be verified exactly over Q before becoming a `ProjectionMessage`.
4. “Not sparse enough” may decline this kernel but must not become solver unsupported.
5. Add bad-template and bad-interpolation tests that fail final verification.

**Reviewer prompt.** Use `REVIEWER_PROMPTS.md#P8b`.

### P8c — TargetActionKrylov kernel with VerifiedCharacteristicSupportCoverage only

**Supports R-IDs:** `RGQ-021`, `RGQ-044`, `RGQ-054`.  
**MECHs:** closes `MECH-014`.

**Implementation tasks.**

1. Implement `kernels/action_krylov.rs` using the quotient/action primitives from P3e.
2. Materialize and verify the target action matrix column by column.
3. Compute characteristic polynomial exactly and verify Cayley-Hamilton exactly.
4. Return only the characteristic support polynomial with the certificate schema in RGQ-044.
5. Add the undercoverage regression required by RGQ-054.
6. Add no-coordinate-root/RUR export tests.

**Reviewer prompt.** Use `REVIEWER_PROMPTS.md#P8c`.

### P8d — UniversalTargetEliminationKernel as bounded local target/separator projection

**Supports R-IDs:** `RGQ-022`, `RGQ-036`, `RGQ-041`, `RGQ-051`, `RGQ-056`.  
**MECHs:** closes `MECH-008` only after one-large-block and anti-heavy-fallback tests pass.

**Implementation tasks.**

1. Implement all required functions from RGQ-041 in `kernels/universal_elimination.rs`.
2. Use exactly the fixed strategy sequence from RGQ-041: TargetRelationSearchEscalated, SparseResultantIfSquareOrOverdetermined, SpecializeProjectInterpolateVerify, then LocalF4OrGroebnerEliminationToKeepZ.
3. Local F4/Groebner may run only inside authorized block with exported variables `Z` as keep variables and explicit resource caps.
4. Universal must not return `CertifiedNonFiniteTargetImage` from a local block; only P10 final-support nonfinite certification may return that status through RGQ-045.
5. Exhaustion must return `AlgorithmicHardCase`, `FiniteResourceFailure`, or `CertificateDesignGap` with stage trace.
6. Add dynamic tests proving altered authorization hash, altered child message, or altered plan hash causes execution/replay failure.
7. Add static and dynamic tests proving no coordinate-first heavy fallback exists.

**Reviewer prompt.** Use `REVIEWER_PROMPTS.md#P8d`.

---

## 10. Remaining optimized kernels

### P9 — RegularChainProjection and NormTraceProjection kernels

**Supports R-IDs:** `RGQ-023`, `RGQ-024`, `RGQ-033`.  
**MECHs:** closes remaining `MECH-007`.

**Implementation tasks.**

1. Implement `kernels/regular_chain_projection.rs` preserving component/guard/projection semantics.
2. Implement `kernels/norm_trace_projection.rs` by detecting algebraic tower forms, computing norm relation, and verifying it exactly.
3. Detection must use algebraic structure only, not geometry names.
4. Add tests for triangular component projection and explicit tower support.

**Reviewer prompt.** Use `REVIEWER_PROMPTS.md#P9`.

---

## 11. Message composition, separator elimination, final support, nonfinite certification

### P10 — Composition and final support with strong nonfinite separation

**Supports R-IDs:** `RGQ-026`, `RGQ-032`, `RGQ-038`, `RGQ-045`, `RGQ-051`, `RGQ-057`, `RGQ-058`.  
**MECHs:** closes `MECH-009`; starts/closes `MECH-015` if nonfinite proof tests pass.

**Implementation tasks.**

1. Implement `compose/message.rs`, `compose.rs`, `separator_elimination.rs`, `final_support.rs`.
2. Separator elimination may use only message relations and target-direct kernels; it must not rebuild the original full coordinate system.
3. `build_global_support_polynomial` must build target-only support from verified root relations.
4. Implement `certify_nonfinite_target_image`, `certify_zero_target_elimination_ideal`, `verify_nonfinite_certificate`, and exact-image real nonfinite function from RGQ-045. These functions must map internal errors to public statuses through the explicit `RGQ-058` result/status layer.
5. Relation-search exhaustion, no local relation, or no target-only support must not imply nonfinite unless RGQ-045 certificate exists.
6. Add tests for multi-block composition, tampered message relation, relation-search exhaustion returning hard-case, and separate certified nonfinite system.

**Reviewer prompt.** Use `REVIEWER_PROMPTS.md#P10`.

---

## 12. Certificates, verification, replay

### P11 — Kernel certificates, support verification, run certificate, replay

**Supports R-IDs:** `RGQ-027`, `RGQ-038`, `RGQ-040`, `RGQ-047`, `RGQ-052`, `RGQ-053`.  
**MECHs:** closes `MECH-010` and advances `MECH-016`.

**Implementation tasks.**

1. Implement every `KernelCertificate` variant in `verify/certificates.rs`.
2. Implement `verify_projection_message` with exported-variable checks and exact variant-specific verification.
3. Implement `verify_global_support` with exact support proof.
4. Implement `replay_run_certificate` and `CoreRunCertificate` with all hashes and invariant flags.
5. Replay must fail on tampered input, canonical system, DAG, kernel plan, message, support, squarefree support, root isolation, and decoded candidates.
6. Add semantic deletion tests required by RGQ-053.
7. Add tests proving unconditional verifier success cannot pass tampered certificates.

**Reviewer prompt.** Use `REVIEWER_PROMPTS.md#P11`.

---

## 13. Exact roots and candidate decode

### P12 — Squarefree support, exact real root isolation, decoded candidates

**Supports R-IDs:** `RGQ-028`, `RGQ-034`, `RGQ-037`, `RGQ-048`.  
**MECHs:** closes `MECH-011`.

**Implementation tasks.**

1. Implement `roots/squarefree.rs` exactly as `p / gcd(p,p')`.
2. Implement `roots/isolate.rs` with exact Sturm and/or Descartes-Vincent isolation.
3. Implement `roots/decode.rs` binding candidate to target, support hash, root index, isolating interval, and candidate hash.
4. Implement `roots/algebraic_number.rs` with concrete algebraic-root records, interval/hash binding, comparison/refinement helpers needed by root isolation, candidate decode, and exact-image sign classification. It is mandatory under `RGQ-059`, not optional.
5. Add tests for no real root, one rational root, multiple rational roots, irrational roots, repeated roots, and high-coefficient polynomials.
6. Add integration tests proving support-producing cases do not return placeholder roots/candidates.

**Reviewer prompt.** Use `REVIEWER_PROMPTS.md#P12`.

---

## 14. Real fiber classification and exact-image mode

### P13 — Exact image API, fiber semantics, and exact-image status gate

**Supports R-IDs:** `RGQ-003`, `RGQ-029`, `RGQ-045`, `RGQ-049`, `RGQ-050`.  
**MECHs:** closes `MECH-012` only when exact-image tests pass.

**Implementation tasks.**

1. Implement `fiber/exact_image.rs`, `hermite.rs`, `thom.rs`, and `slack_semantics.rs`.
2. `classify_real_target_image` must add algebraic target conditions, attach slack/guard/branch semantics, classify each candidate, and produce certificates.
3. Implement Hermite real-root count certificates where applicable.
4. Implement Thom/sign classification for algebraic target roots and guard polynomials.
5. Implement slack semantic consistency verification.
6. Integrate exact-image mode so exact-image statuses are impossible unless classification ran.
7. Exact-image-mode nonfinite requires real nonfinite certificate; otherwise hard/certificate status.
8. Add tests where candidate-cover support has spurious roots removed by semantics, exact image is empty, guard/slack affects classification, and candidate-cover mode does not claim exact image.

**Reviewer prompt.** Use `REVIEWER_PROMPTS.md#P13`.

---

## 15. Orchestrator, public API integration, diagnostics, cost trace

### P14 — Full pipeline integration

**Supports R-IDs:** `RGQ-008`, `RGQ-011`, `RGQ-030`, `RGQ-031`, `RGQ-034`, `RGQ-049`.  
**MECHs:** closes `MECH-002` only after full stage trace tests pass.

**Implementation tasks.**

1. Implement `solver/options.rs`, `pipeline.rs`, and `orchestrator.rs`.
2. `api::solve_target` must execute the full pipeline and never panic for expected solver errors.
3. `TargetSolveResult` must include status, support, squarefree support, roots, candidates, projection messages, certificate, diagnostics, and global cost trace as applicable.
4. Cost trace must record all parameters from Base Spec section 30.
5. Certificate invariant flags must be truthful.
6. Add end-to-end tests for candidate-cover success, exact-image success, exact-image empty, certified nonfinite, hard-case by bounds, and invalid input.

**Reviewer prompt.** Use `REVIEWER_PROMPTS.md#P14`.

---

## 16. Generalized stress, anti-slice, anti-fallback, anti-decoration verification

### P15 — Acceptance stress suites; hard cases cannot satisfy support-producing acceptance

**Supports R-IDs:** `RGQ-034`, `RGQ-035`, `RGQ-036`, `RGQ-037`, `RGQ-038`, `RGQ-039`, `RGQ-040`, `RGQ-048`, `RGQ-049`, `RGQ-050`, `RGQ-053`, `RGQ-055`, `RGQ-061`, `RGQ-062`, `RGQ-063`.  
**MECHs:** cross-MECH final validation; closes no new mechanism by itself.

**Implementation tasks.**

1. Create `support_producing_candidate_cover_suite` with no geometry problem names, no expected-answer dispatch, and no fixture IDs used by production code.
2. Every suite A case must return `CertifiedCandidateCover` or nonempty `CertifiedExactTargetImage`, nonzero support, exact support verification, exact squarefree support, exact real root isolation, and non-placeholder decoded candidates when real roots exist.
3. Suite A must include all support-producing algebraic categories listed in `RGQ-048` and `RGQ-063`, including renamed-variable/permuted-relation variants.
4. Create `exact_image_semantics_suite` for exact-image nonempty image, exact empty real image, guard/slack/branch filtering, and spurious-root removal. These cases must run with `exact_image_mode=true` and cannot compensate for suite A failures.
5. Create `failure_and_nonfinite_semantics_suite` for invalid input, resource failure, algorithmic hard case, certificate gap, relation-search exhaustion, and certified nonfinite. These do not count toward support-producing acceptance.
6. Add anti-fallback checks: no full coordinate solution list/RUR, no QE/CAD/RCF fallback, no hidden fallback outside declared ladder, no ordinary `Unsupported`, no geometry/problem/fixture/expected-answer dispatch.
7. Add anti-decoration checks: DAG hash mismatch fails, authorization mismatch fails, plan hash mismatch fails, certificate identity tampering fails, removing a child message prevents support composition or changes support verification.
8. Add relation-search schedule reproducibility tests required by `RGQ-055`.
9. Any `AlgorithmicHardCase`, `FiniteResourceFailure`, `CertificateDesignGap`, `CertifiedEmptyRealTargetImage`, `CertifiedNonFiniteTargetImage`, empty support, empty placeholder roots, or empty placeholder candidates in suite A fails P15.

**Closure evidence.**

- Full test output separating the three suites from `RGQ-061`.
- Static scan commands and outputs.
- Dynamic tamper/deletion test outputs.
- Reviewer notes proving stress cases exercise generic mechanisms, not hard-coded strings.

**Reviewer prompt.** Use `REVIEWER_PROMPTS.md#P15`.

---

## 17. Final closure

### P16 — Closure packet and final claim ladder

**Supports R-IDs:** all R-IDs, with explicit audit of `RGQ-057` through `RGQ-064`.  
**MECHs:** all MECHs required by the claimed label must be closed before this phase can pass.

**Implementation tasks.**

1. Read `BASE_SPEC.md`, `PLAN.md`, `SOURCE_MAP.md`, `REVIEW_ARCHIVE_SCHEMA.md`, `REVIEW_SUMMARY_SCHEMA.yaml`, all phase reviews, all evidence, and current git state.
2. Run full verification fresh after final code changes:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

3. Run static scans fresh:

```bash
rg -n "Unsupported|unsupported" geosolver-core/src || true
rg -n "circle|triangle|tangent|distance|area|incircle|circumcircle|orthic|mixtilinear|fixture|expected|answer|problem_id|official" geosolver-core/src || true
rg -n "todo!|unimplemented!|placeholder|dummy|fake|stub" geosolver-core/src || true
rg -n "CAD|QE|RCF|coordinate solution|coordinate roots|full coordinate RUR|RUR" geosolver-core/src || true
rg -n "ACCEPTANCE_COMPLETE.*candidate-cover solver core implementation" geosolver-core docs/ai/changes/RGDTPK-Q-v4-core/CLOSURE.md || true
```

4. Audit every review archive for schema validity, matching prompt/response hashes, byte-identical review schema mirrors, PASS-with-blocker rejection, and raw response/summary agreement.
5. Re-run and update `CONSISTENCY_AUDIT.md` with all checks from `RGQ-064`.
6. Produce `CLOSURE.md` with R-ID evidence table, MECH evidence table, support-producing candidate-cover table, exact-image semantics table, failure/nonfinite semantics table, verification evidence, review table, git state, residual risks, and exact final claim.
7. Use only the claim ladder from `RGQ-049`:

```text
SCAFFOLD_READY
PARTIAL_MECHANISM_READY:<MECH-ID>
CANDIDATE_COVER_CORE_READY
EXACT_IMAGE_CORE_READY
RGDTPK_Q_V4_ACCEPTANCE_COMPLETE
SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC
```

8. `RGDTPK_Q_V4_ACCEPTANCE_COMPLETE` is forbidden unless `EXACT_IMAGE_CORE_READY` is valid. If exact image is incomplete, the maximum final claim is `CANDIDATE_COVER_CORE_READY`, and `CLOSURE.md` must say full acceptance is not complete.
9. The old candidate-cover-only acceptance-complete wording is forbidden.
10. Run final reviewers with `REVIEWER_PROMPTS.md#P16`.

**Forbidden final claims.**

- benchmark superiority;
- universal finite-system completeness;
- geometry DSL support;
- natural language or diagram support;
- exact-image semantics from candidate-cover-only evidence;
- acceptance complete while exact image remains incomplete.

**Reviewer prompt.** Use `REVIEWER_PROMPTS.md#P16`.
