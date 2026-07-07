# P10 Review Request

Reviewer prompt: RP-P10 from `REVIEWER_PROMPTS.md`.

Relevant R-IDs:

- BS-R094

Files to inspect:

- `geosolver-core/src/algebra/resultant.rs`
- `geosolver-core/src/kernels/sparse_resultant.rs`
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

- Template represents source support-set resultant/eliminant semantics for the declared pairwise
  chain route.
- Two-polynomial and pairwise-chain templates are not claimed as a full general sparse resultant
  implementation.
- Resource caps return `FiniteResourceFailure`.
- Resultant relation variables are subset of exported variables.
- Certificates recompute exact relation/backend or exact membership and do not accept modular trace
  alone.
- Remediation check: clearing `modular_traces` and recomputing the wrapper hashes is rejected by
  both direct resultant certificate verification and sparse-resultant message replay.

Requested decision: PASS / FAIL_FIXABLE / FAIL_BLOCKING / NEEDS_MORE_EVIDENCE for P10 only.
