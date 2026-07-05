# Source Map for RGDTPK-Q-v4-core — v2.2 consistency-audited

## Context Packet

Spec ID: `RGDTPK-Q-v4-core`  
Type: Source Map  
Status: DRAFT FOR USER APPROVAL — v2.2 consistency-audited  
Parent: `BASE_SPEC.md`  
Scope: maps source documents, v2 hardening requirements, Plan phases, and reviewer prompts.  
Context Packet Authority: non-authoritative digest.

---

## Source documents

| Source ID | Path | SHA-256 | Classification | Purpose |
|---|---|---:|---|---|
| SRC-ALG-v4 | `docs/ai/sources/geosolver_core_r_gdtpk_q_algorithm_spec_v4.md` | `2dc2f950896ff3e60858b17bf3f1867667564ae773e0a71d6db8c0953143caed` | EXACT | Algorithm, types, folder layout, functions, pipeline, certificates, statuses, completion criteria |
| SRC-FAIL-20260704 | `docs/ai/sources/geosolver_failure_causes_generalized_2026_07_04.md` | `df0d9d525a022f1851fe8021c70fea97d10408425e7b2670bf991858723ae14e` | EXACT for failure-prevention; reference-only for historical names | Anti-drift, anti-slice, anti-fallback, reviewer behavior, generalized stress |
| HARDENING-v2 | `BASE_SPEC.md` §8A | N/A | NORMATIVE AMENDMENT | Removes interpretation freedom in Universal, TargetRelationSearch, ActionKrylov, nonfinite proof, phase closure, review archive, and P15/P16 claims |
| HARDENING-v2.2 | `BASE_SPEC.md` §8B | N/A | NORMATIVE CONSISTENCY AMENDMENT | Adds Appendix override map, closed status set, required-function strictness, schema mirror checks, suite partition, cost-compression evidence, fixture-free stress templates, and mandatory consistency audit |
| P5R-REMEDIATION | `P5R_BASE_SPEC_AMENDMENT.md` | `1118c64cdc8e7de288be81e77829af32d05f75fd8957852627975acd86dd1f84` | MANDATORY TIGHTENING AMENDMENT | Inserts a barrier after P5 and before P6; prevents fake F4 claims, polynomial-only affine narrowing, self-certifying quotient/action handles, primitive overclaim, and commit-unbound evidence |

---

## Algorithm source mapping

| Source section | Base Spec R-IDs | MECHs | Plan phase |
|---|---|---|---|
| SRC-ALG-v4 §0–§1 | RGQ-000, RGQ-001, RGQ-031, RGQ-033 | MECH-002 | P0, P14 |
| SRC-ALG-v4 §2 | RGQ-002, RGQ-003 | MECH-012 | P2, P13 |
| SRC-ALG-v4 §3 | RGQ-004, RGQ-005, RGQ-006 | MECH-005, MECH-008, MECH-010 | P2, P3c, P6, P8d, P11 |
| SRC-ALG-v4 §4 | RGQ-011 | MECH-002 | P14 |
| SRC-ALG-v4 §5 | RGQ-009, RGQ-030 | MECH-001, MECH-010 | P1, P2, P11, P14 |
| SRC-ALG-v4 §6–§8 | RGQ-007, RGQ-008, RGQ-009, RGQ-010 | MECH-001 | P1 |
| SRC-ALG-v4 §9 | RGQ-012 | MECH-001, MECH-002 | P2 |
| SRC-ALG-v4 §10 | RGQ-010, RGQ-019–RGQ-025, RGQ-028 | MECH-001, MECH-006, MECH-007, MECH-008, MECH-011, MECH-013, MECH-014 | P3a–P3f, P8a–P8d, P12 |
| SRC-ALG-v4 §11 | RGQ-013 | MECH-003 | P4 |
| SRC-ALG-v4 §12 | RGQ-014 | MECH-004 | P5 |
| SRC-ALG-v4 §13 | RGQ-015 | MECH-005 | P6 |
| SRC-ALG-v4 §14 | RGQ-016 | MECH-005, MECH-007 | P7 |
| SRC-ALG-v4 §15–§23 | RGQ-017–RGQ-025 | MECH-006, MECH-007, MECH-008, MECH-013, MECH-014 | P7, P8a–P8d, P9 |
| SRC-ALG-v4 §24 | RGQ-026 | MECH-009 | P10 |
| SRC-ALG-v4 §25 | RGQ-027 | MECH-010 | P11 |
| SRC-ALG-v4 §26 | RGQ-028 | MECH-011 | P12 |
| SRC-ALG-v4 §27 | RGQ-029 | MECH-012 | P13 |
| SRC-ALG-v4 §28–§29 | RGQ-030 | MECH-002 | P14 |
| SRC-ALG-v4 §30 | RGQ-031 | MECH-002 | P14, P16 |
| SRC-ALG-v4 §31 | RGQ-032 | MECH-009, MECH-015 | P10 |
| SRC-ALG-v4 §32 | RGQ-033 | MECH-003, MECH-004, MECH-007 | P4, P5, P9, P15 |
| SRC-ALG-v4 §33 | RGQ-034 | all MECHs | P15, P16 |

---

## Failure-prevention source mapping

| Source section | Base Spec R-IDs | Plan/review effect |
|---|---|---|
| SRC-FAIL §0–§1 | RGQ-035, RGQ-036 | Objective remains generic algebraic target candidate enumeration with no heavy fallback and no narrow-scope escape. |
| SRC-FAIL §2.1–§2.5 | RGQ-035, RGQ-036, RGQ-033 | Prevent purpose swap, geometry-layer confusion, partial support. |
| SRC-FAIL §2.6–§2.7 | RGQ-037, RGQ-048 | Generalized real-problem structures are split into support-producing success obligations and separate failure-semantics obligations; hard cases cannot satisfy success acceptance. |
| SRC-FAIL §2.8–§2.13 | RGQ-040, RGQ-046, RGQ-047, RGQ-028 | Prevent gate dependence, evidence-only review, placeholder roots/candidates. |
| SRC-FAIL §2.14–§2.18 | RGQ-036, RGQ-038, RGQ-041, RGQ-056 | Prevent heavy fallback, CoverageMatrix escape, projector portfolio without main algorithm. |
| SRC-FAIL §2.19–§2.21 | RGQ-038, RGQ-039, RGQ-040, RGQ-052, RGQ-053 | Require operational DAG/certificates and algorithmic reviewer inspection. |
| SRC-FAIL §2.22–§2.26 | RGQ-035, RGQ-031, RGQ-049 | Docs/reviews/preflight/performance claims do not prove completion; final claim ladder is limited. |
| SRC-FAIL §2.27–§2.35 | RGQ-019, RGQ-026, RGQ-037, RGQ-042, RGQ-044, RGQ-045 | Specify integrated generic algebraic algorithm and generalized structures. |
| SRC-FAIL §3–§6 | RGQ-035–RGQ-064 | Reviewer prompts, Plan closure rules, consistency audit, and final claim ceiling. |

---

## v2 hardening mapping

| User issue | New R-IDs | MECHs | Plan phase | Reviewer prompt |
|---|---|---|---|---|
| 1. Universal can collapse into heavy fallback | RGQ-041, RGQ-051, RGQ-056 | MECH-008 | P6, P8d, P15, P16 | P6, P8d, P15, P16 |
| 2. TargetRelationSearch degree/support bounds left to implementer | RGQ-042, RGQ-043, RGQ-055 | MECH-006, MECH-013 | P6, P8a, P15 | P6, P8a, P15 |
| 3. ActionKrylov coverage has multiple options | RGQ-044, RGQ-054 | MECH-014 | P3e, P8c, P15 | P3e, P8c, P15 |
| 4. Nonfinite proof too weak | RGQ-045, RGQ-050, RGQ-051 | MECH-015 | P10, P13, P15 | P10, P13, P15 |
| 5. P3/P8 too large | RGQ-046 | all affected | P3a–P3f, P8a–P8d | P3a–P3f, P8a–P8d |
| 6. Reviewer schema/archive weak | RGQ-047, RGQ-052 | MECH-016 | P0, every phase, P16 | all prompts |
| 7. P15 stress can be passed by hard-case | RGQ-048 | all | P15 | P15 |
| 8. P16 final claim can overclaim without exact image | RGQ-049, RGQ-050 | MECH-012, MECH-016 | P13, P16 | P13, P16 |

## Review schema artifacts

`REVIEW_ARCHIVE_SCHEMA.md` defines the archive contract. `REVIEW_SUMMARY_SCHEMA.yaml`, `schemas/review_summary.schema.yaml`, and `schemas/evidence_manifest.schema.yaml` are machine-readable YAML-form JSON Schema files required by RGQ-047/RGQ-052/RGQ-060. The two review-summary schema copies must be byte-identical. The evidence manifest schema binds source hashes, pack/schema hashes, command outputs with exit codes, static scans, function implementation tables, claim ceilings, and freshness flags.

---

## v2.2 consistency-hardening mapping

| Consistency issue | Base Spec R-ID | Plan phase(s) | Reviewer prompt(s) |
|---|---|---|---|
| Appendix A pseudocode can be read as weaker than v2 hardening | `RGQ-057` | P0, P8c, P8d, P10, P16 | P0, P8c, P8d, P10, P16 |
| Public status set and `SolverError` mapping could drift or add non-spec statuses | `RGQ-058` | P1, P10, P14, P16 | P1, P10, P14, P16 |
| Required files/functions could be weakened by “as needed”, hook, foundation, helper-only, or deferred wording | `RGQ-059` | P3d, P3f, P12, P13, P16 | P3d, P3f, P12, P13, P16 |
| Review summary schema mirrors and PASS-with-blocker rejection were not fully machine-checked | `RGQ-060` | P0, all review archives, P16 | P0, all prompts, P16 |
| P15 exact-image empty cases and failure cases could be confused with support-producing acceptance | `RGQ-061` | P15, P16 | P15, P16 |
| Performance-first could be reduced to late benchmark wording instead of cost-compression evidence | `RGQ-062` | P6, P8a–P8d, P10, P14, P15, P16 | P6, P8a–P8d, P10, P14, P15, P16 |
| Generalized stress could still become fixture/string dispatch or toy-only coverage | `RGQ-063` | P15, P16 | P15, P16 |
| The instruction pack needs an explicit consistency audit artifact | `RGQ-064` | P0, P16 | P0, P16 |

## P5R remediation mapping

| P5R issue | P5R R-ID | Plan phase(s) | Reviewer prompt(s) |
|---|---|---|---|
| P5R must block P6 until closed | `P5R-RGQ-065` | P5R-a, P5R-f, P6 | P5R-a, P5R-f, P6 |
| P0-P5 evidence and claims must be rebound to a current commit | `P5R-RGQ-066` | P5R-a, P5R-f | P5R-a, P5R-f |
| Groebner-backed wrapper must not be claimed as production F4 | `P5R-RGQ-067` | P5R-b, P8d | P5R-b, P8d |
| Guarded affine preprocessing must support rational affine semantics | `P5R-RGQ-068` | P5R-c, P7 | P5R-c, P7 |
| TargetActionKrylov production handles must be provenance-bound | `P5R-RGQ-069` | P5R-d, P8c | P5R-d, P8c |
| Narrow primitives must be recorded and blocked from overclaim | `P5R-RGQ-070` | P5R-e, P6, P8a-P8d, P9 | P5R-e, P6, P8a-P8d, P9 |
| P6 readiness must be explicit and support-producing | `P5R-RGQ-071` | P5R-a, P5R-f, P6 | P5R-a, P5R-f, P6 |
| P5R review must inspect algorithmic drift, not just evidence format | `P5R-RGQ-072` | P5R-a-P5R-f | P5R-a-P5R-f |

## Appendix A hardening override map required by RGQ-057

| Appendix A section | Source behavior to preserve | v2.2 controlling rule |
|---|---|---|
| §19.4 | ActionKrylov must not return a candidate without coverage proof | `RGQ-044`, `RGQ-054`: only `VerifiedCharacteristicSupportCoverage` is accepted |
| §20.3 | Universal is planned target/separator projection | `RGQ-041`, `RGQ-051`: local Universal cannot return `CertifiedNonFiniteTargetImage` |
| §20.4 | Universal internal strategies are allowed target/separator algebraic tools | `RGQ-041`: exact fixed strategy sequence only |
| §24.4 | Final support may call nonfinite certification only at final support stage | `RGQ-045`, `RGQ-051`: no target-only support is not nonfiniteness proof |
| §31 | Nonfinite target image requires proof of `I ∩ Q[T] = {0}` | `RGQ-045`: exact positive elimination/dimension/regular-chain certificate required |
