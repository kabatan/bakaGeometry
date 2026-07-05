# Issue Fix Map — v2.2 consistency-audited

This file maps the eight requested corrections to normative Base Spec requirements, Plan phases, and reviewer prompts.

| # | Problem identified | Fix in Base Spec | Fix in Plan | Reviewer enforcement |
|---|---|---|---|---|
| 1 | `UniversalTargetEliminationKernel` still had room to become heavy fallback. | `RGQ-041`, `RGQ-051`, `RGQ-056` make Universal bounded, local, authorization-bound, fixed-sequence including sparse-resultant, non-nonfinite local, and non-coordinate. | P6 builds the fixed plan; P8d implements Universal; P15/P16 test static and dynamic anti-fallback. | P6, P8d, P15, P16 prompts fail unbounded/global/coordinate fallback. |
| 2 | `TargetRelationSearch` degree/support bound policy remained implementation judgment. | `RGQ-042`, `RGQ-043`, `RGQ-055` define the exact dense total-degree schedule, support formulas, and reproducibility tests. | P6 plans the schedule; P8a implements it; P15 validates reproducibility. | P8a requires reviewer recomputation from `J,Y,Z,options`. |
| 3 | `TargetActionKrylov` coverage proof had multiple options, so a weak option could be chosen. | `RGQ-044`, `RGQ-054` allow only `VerifiedCharacteristicSupportCoverage`. | P3e builds action-matrix/charpoly/CH primitives; P8c implements kernel; P15 includes undercoverage regression. | P3e/P8c fail any single-vector/block-Wiedemann/trace-only coverage. |
| 4 | `CertifiedNonFiniteTargetImage` could be triggered by failure to find a relation. | `RGQ-045`, `RGQ-050`, `RGQ-051` require a positive exact elimination/dimension certificate, and real certificate in exact-image mode. | P10 implements nonfinite certificate; P13 exact-image real semantics; P15 separates hard-case from nonfinite. | P10/P13/P15 must fail any implementation where relation-search exhaustion, sparse heuristic failure, Universal stage failure, or composition failure is routed to nonfinite instead of hard/resource/certificate statuses. |
| 5 | P3/P8 were too large and could close with stubs/wrappers. | `RGQ-046` requires function implementation tables and forbids broad stub phases. | P3 is split into P3a–P3f; P8 is split into P8a–P8d. Each subphase needs separate evidence/review. | Separate prompts P3a–P3f and P8a–P8d inspect controlling functions. |
| 6 | Reviewer prompt/archive/schema was weak. | `RGQ-047`, `RGQ-052` define mandatory prompt/response archive and strict YAML schema. | P0 creates `REVIEW_ARCHIVE_SCHEMA.md`, `REVIEW_SUMMARY_SCHEMA.yaml`, `schemas/review_summary.schema.yaml`, and `schemas/evidence_manifest.schema.yaml`; every phase must archive prompt/response/summary/manifest; P16 audits hashes and schema validation. | Every prompt requires schema and prompt/response consistency checks. |
| 7 | P15 acceptance could be read as allowing hard-case to satisfy important stress. | `RGQ-048` splits support-producing acceptance from failure semantics and forbids hard-case in support-producing suite. | P15 requires success statuses, nonzero support, exact verification, exact roots, and candidates. | P15 prompt fails any hard-case/resource/certificate-gap in support-producing suite. |
| 8 | P16 final claim could look complete without exact image. | `RGQ-049`, `RGQ-050` define claim ladder and forbid acceptance complete without exact image. | P13 must close exact image; P16 may claim only `CANDIDATE_COVER_CORE_READY` if exact image incomplete. | P16 prompt forbids old `ACCEPTANCE_COMPLETE ... candidate-cover` phrase and overclaim. |

## v2.1 correction

`RGQ-051` has been corrected so that relation-search exhaustion, sparse heuristic failure, Universal stage failure, and composition failure to produce target-only support **must not** route to `CertifiedNonFiniteTargetImage`. These cases must route to `AlgorithmicHardCase`, `FiniteResourceFailure`, or `CertificateDesignGap` unless the positive nonfiniteness proof required by `RGQ-045` exists.


## v2.2 consistency audit fixes

- `RGQ-037` now separates support-producing success obligations from failure-semantics obligations, so nonfinite/hard-case evidence cannot be read as satisfying support-producing acceptance.
- `MECH-009` now routes missing target-only support to hard/resource/certificate statuses unless `MECH-015` supplies a positive nonfinite proof.
- Review-summary examples, machine schemas, and archive rules now consistently bind `evidence_manifest.yaml` and its schema hash.
- Universal's final local elimination stage name is consistently `LocalF4OrGroebnerEliminationToKeepZ`.

## v2.2 consistency fixes

| # | Consistency risk found in v2.1 | Fix |
|---|---|---|
| C1 | Appendix A contains weaker pseudocode in ActionKrylov, Universal, and final support that could be cited against v2 hardening. | Added `RGQ-057` and an Appendix override map in `SOURCE_MAP.md`. |
| C2 | P1 could be read as adding a non-spec `NotYetImplemented` status. | Added `RGQ-058`; P1 now uses only existing statuses plus diagnostic `TemporaryPipelineNotConnected`, removed by P14. |
| C3 | “hooks”, “helper foundations”, and “as needed” wording could let required files/functions remain thin or deferred. | Added `RGQ-059`; P3d/P3f/P12 wording now requires concrete Appendix A functions. |
| C4 | Review schema did not fully reject PASS summaries with blockers/fixes and had two schema paths without mirror authority. | Added `RGQ-060`; schemas now require byte-identical mirrors and reject PASS-with-blocker/fix. |
| C5 | P15 mixed support-producing, exact-image empty, and failure statuses too closely. | Added `RGQ-061`; P15 now has three separate suites. |
| C6 | Performance-first could be treated as late benchmark/preflight instead of design evidence. | Added `RGQ-062`; cost-compression evidence is required in plan/kernel/closure traces. |
| C7 | Generalized stress could still be hard-coded by fixture/string or become toy-only. | Added `RGQ-063`; stress cases must be algebraic templates with renaming/permutation tests. |
| C8 | No explicit final consistency audit artifact existed. | Added `RGQ-064` and `CONSISTENCY_AUDIT.md`. |
