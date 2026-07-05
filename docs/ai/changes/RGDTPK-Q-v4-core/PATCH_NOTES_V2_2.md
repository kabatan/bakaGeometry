# Patch Notes — v2.2 final consistency hardening

This patch is a consistency pass over v2.1. It does not change the mathematical target of R-GDTPK-Q / ACCTP-Q. It removes remaining places where the Agent could choose a weaker interpretation.

## Added normative R-IDs

- `RGQ-057`: Appendix A hardening overrides are explicit and non-optional.
- `RGQ-058`: `SolverStatus` is a closed public set; internal errors must map to statuses explicitly.
- `RGQ-059`: Required files/functions are mandatory, not “as needed”, hook-only, helper-only, or deferred after their owning phase.
- `RGQ-060`: Review schema mirrors are byte-identical and PASS cannot contain blockers or required fixes.
- `RGQ-061`: P15 acceptance is partitioned into support-producing, exact-image semantics, and failure/nonfinite semantics suites.
- `RGQ-062`: Performance-first requires design-time cost-compression evidence, not late benchmark wording.
- `RGQ-063`: Generalized stress must be algebraic templates, not problem fixtures or hard-coded strings.
- `RGQ-064`: `CONSISTENCY_AUDIT.md` is mandatory before implementation and at final closure.

## Corrected weak plan wording

- P1 no longer permits a non-spec “not-yet-implemented status”. Temporary scaffold behavior must use existing statuses plus diagnostic `TemporaryPipelineNotConnected`, and P14 must remove it.
- P3d no longer says “verification hooks”; it requires exact verification functions.
- P3f no longer says “helper foundations”; it requires the concrete Appendix A functions.
- P12 no longer says `roots/algebraic_number.rs` is “as needed”; it is mandatory.
- P15 now has three separate suites, so hard cases and exact empty image cannot satisfy support-producing acceptance.

## Schema hardening

- `REVIEW_SUMMARY_SCHEMA.yaml` and `schemas/review_summary.schema.yaml` must be byte-identical.
- PASS review summaries with nonempty `blocking_findings` or `required_fixes` are schema-invalid.
- PASS requires all `pass_conditions` values to be true, raw prompt/response consistency checks to be clean, and the phase-specific reviewer checks required by the schema. Non-P0 phases require `algorithmic_sufficiency.verdict: sufficient`.
- The schema requires P15/P16 forbidden-pattern scans to pass and requires semantic-deletion/tamper challenges for the high-risk phases P5, P8d, P11, P15, and P16.
- P15/P16 PASS summaries require all three suite-partition rows to be present and pass: support-producing candidate cover, exact-image semantics, and failure/nonfinite semantics.
- For PASS, phase-specific, Appendix-override, status-mapping, and suite-partition check rows cannot contain `fail`; `not_applicable` must be justified by path in the archive contract.
- `evidence_manifest.schema.yaml` requires phase-id validation, source/pack/schema hashes, command-output records with exit codes, static scans, function implementation tables, claim ceilings, freshness flags, and an assertion that no untracked evidence is omitted.


## Appendix conflict resolution

`SOURCE_MAP.md` now contains an Appendix A override table. In particular, Appendix A §20.3 and §24.4 cannot be used to route relation-search exhaustion, local Universal failure, or composition failure to `CertifiedNonFiniteTargetImage`. `RGQ-045`, `RGQ-051`, and `RGQ-057` control those cases.

## Final consistency audit fix

During the final audit, the review-summary YAML schema was found to lag behind the v2.2 prose requirements: it did not yet contain `schema_mirror_sha256`, `appendix_override_checks`, `status_mapping_checks`, `suite_partition_checks`, or the new v2.2 forbidden-scan fields. This has been corrected in both schema mirrors, and the mirrors were rechecked as byte-identical.

The final audit also validates that PASS-with-blocker, PASS-with-required-fix, insufficient-algorithm PASS, missing v2.2 scan fields, false pass conditions, and underspecified P15 suite evidence are schema-invalid.
