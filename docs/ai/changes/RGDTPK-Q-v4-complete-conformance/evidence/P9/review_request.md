# P9 Review Request

Reviewer prompt: RP-P9 from `REVIEWER_PROMPTS.md`.

Relevant R-IDs:

- BS-R093

MECHs:

- MECH-01
- MECH-03
- MECH-07

Files to inspect:

- `geosolver-core/src/kernels/target_relation_search.rs`
- `geosolver-core/src/planner/relation_schedule.rs`
- `geosolver-core/src/algebra/linear_solve.rs`
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

- Kernel implements coefficient comparison for `g - sum(q_i f_i) = 0`.
- Dense, sparse, and specialized interpolation footprint support strategies are represented.
- Modular solve output is candidate-only and cannot certify success by itself.
- Every success passes exact Q membership verification.
- Certificate contains exported/eliminated variable hashes, support hashes, membership matrix hash,
  primes used, rational reconstruction hash, relation hash, multipliers hash, and exact identity
  hash.
- `verify_message` recomputes/binds source relation IDs, support hashes, membership matrix hash,
  modular prime trace, accepted candidate vector, rational reconstruction hash, multipliers hash,
  and exact identity hash; regressions tamper each reviewer-identified field after recomputing the
  certificate binding hash.
- Child projection-message relations are included only when bound by child message hashes.
- AlgorithmicHardCase for exhausted bounds includes accumulated matrix trace evidence.

Requested decision: PASS / FAIL_FIXABLE / FAIL_BLOCKING / NEEDS_MORE_EVIDENCE for P9 only.
