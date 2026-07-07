# P4 Review Request

Reviewer prompt: RP-P4 from `REVIEWER_PROMPTS.md`.

Relevant R-IDs:

- BS-R054
- BS-R055
- BS-R096

Files to inspect:

- `geosolver-core/src/algebra/mod.rs`
- `geosolver-core/src/algebra/groebner.rs`
- `geosolver-core/src/algebra/f4.rs`
- `geosolver-core/src/algebra/elimination.rs`
- `geosolver-core/src/algebra/normal_form.rs`
- `geosolver-core/src/algebra/regular_chain.rs`
- `geosolver-core/src/planner/kernel_plan.rs`
- `geosolver-core/src/kernels/target_relation_search.rs`
- `geosolver-core/src/kernels/universal_elimination.rs`
- `geosolver-core/src/kernels/specialization_interpolation.rs`
- `geosolver-core/src/solver/pipeline.rs`
- `geosolver-core/src/verify/replay.rs`
- `geosolver-core/src/verify/verify_message.rs`
- `geosolver-core/scripts/audit_v4_conformance.py`

Remediation focus after prior P4 FAIL_FIXABLE:

- `f4_reduce_batch` no longer computes returned reductions by calling the Groebner reducer. It
  builds the F4 symbolic preprocessing matrix, records deterministic modular matrix evidence, then
  computes returned reductions by exact row reduction over those F4 matrix rows and verifies exact
  Q membership certificates.
- Universal internal strategies now match source section 20.4 exactly:
  `EliminationGroebnerLocal`, `F4EliminationLocal`, `TargetRelationSearchEscalated`,
  `ResultantIfSquareOrOverdetermined`, and `SpecializeProjectInterpolateVerify`.
- ActionKrylov, RegularChain, and NormTrace are no longer Universal internal stages. They remain
  separate ladder kernels only.

Evidence:

- `source_to_code_map.md`
- `algorithm_evidence.md`
- `commands.log`
- `static_audit.log`
- `changed_files.txt`

Requested decision: PASS / FAIL_FIXABLE / FAIL_BLOCKING / NEEDS_MORE_EVIDENCE for P4 only.
