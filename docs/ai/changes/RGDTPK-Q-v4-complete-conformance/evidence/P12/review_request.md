# P12 Review Request

Reviewer prompt: RP-P12 from `REVIEWER_PROMPTS.md`.

Relevant R-IDs:

- BS-R096

Files to inspect:

- `geosolver-core/src/kernels/universal_elimination.rs`
- `geosolver-core/src/algebra/elimination.rs`
- `geosolver-core/src/algebra/f4.rs`
- `geosolver-core/src/kernels/target_relation_search.rs`
- `geosolver-core/src/kernels/sparse_resultant.rs`
- `geosolver-core/src/kernels/specialization_interpolation.rs`
- `geosolver-core/src/planner/kernel_plan.rs`
- `geosolver-core/src/verify/certificates.rs`
- `geosolver-core/src/verify/verify_message.rs`
- `geosolver-core/scripts/audit_v4_conformance.py`

Evidence:

- `source_to_code_map.md`
- `algorithm_evidence.md`
- `commands.log`
- `static_audit.log`
- `changed_files.txt`

Requested checks:

- Universal admission is true for Q-polynomial relation blocks.
- Internal strategies exactly match source section 20.4:
  `EliminationGroebnerLocal`, `F4EliminationLocal`, `TargetRelationSearchEscalated`,
  `ResultantIfSquareOrOverdetermined`, `SpecializeProjectInterpolateVerify`.
- NormTrace, RegularChain, and ActionKrylov are not Universal internal stages.
- Strategy is declared before execution and certificate-bound.
- Every exported generator is in Q[Z] and exactly verified.
- Empty generator path either certifies nonfinite or returns AlgorithmicHardCase; current finite
  candidate-cover scope uses AlgorithmicHardCase and makes no local nonfinite claim.

Requested decision: PASS / FAIL_FIXABLE / FAIL_BLOCKING / NEEDS_MORE_EVIDENCE for P12 only.

