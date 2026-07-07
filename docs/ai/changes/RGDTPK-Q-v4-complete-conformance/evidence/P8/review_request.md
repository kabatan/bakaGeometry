# P8 Review Request

Reviewer prompt: RP-P8 from `REVIEWER_PROMPTS.md`.

Relevant R-IDs:

- BS-R091
- BS-R092

Files to inspect:

- `geosolver-core/src/kernels/target_univariate.rs`
- `geosolver-core/src/kernels/linear_affine.rs`
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

- TargetUnivariate scans local and child message relations.
- TargetUnivariate support uses primitive LCM/squarefree-compatible construction and emits
  PrincipalSupport.
- Source membership certificate binds source relation IDs/hashes and child message hashes.
- LinearAffine pivots are safe and guard-bound.
- Incomplete affine elimination returns correct failure; unsafe denominator division is impossible.
- Guarded affine replay rejects a nonconstant pivot step unless `denominator_guard_hash` matches an
  authorized `ctx.system.guards` record with the same pivot factor hash; regression tampering
  recomputes the certificate binding hash before verification.
- Message verification rejects relations outside exported variables.

Requested decision: PASS / FAIL_FIXABLE / FAIL_BLOCKING / NEEDS_MORE_EVIDENCE for P8 only.
