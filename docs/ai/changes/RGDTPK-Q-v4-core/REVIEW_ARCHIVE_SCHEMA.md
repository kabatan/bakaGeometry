# Review Archive Schema for RGDTPK-Q-v4-core — v2.2

This file is normative under `RGQ-047`, `RGQ-060`, and `MECH-016`. The machine-checkable schemas are:

```text
docs/ai/changes/RGDTPK-Q-v4-core/REVIEW_SUMMARY_SCHEMA.yaml
docs/ai/changes/RGDTPK-Q-v4-core/schemas/review_summary.schema.yaml
docs/ai/changes/RGDTPK-Q-v4-core/schemas/evidence_manifest.schema.yaml
```

`REVIEW_SUMMARY_SCHEMA.yaml` and `schemas/review_summary.schema.yaml` must be byte-identical. If this markdown and any YAML schema differ, the stricter rule wins and the Agent must stop for `BaseSpecConflict` rather than choosing the easier rule.

Every review invocation must be archived exactly as:

```text
docs/ai/changes/RGDTPK-Q-v4-core/reviews/<phase-id>/<YYYYMMDD-HHMMSSZ>/
  prompt.md
  response.md
  review_summary.yaml
  evidence_manifest.yaml
```

`prompt.md` must be the exact prompt sent to the reviewer. `response.md` must be the exact reviewer response. The Agent may not replace either with a summary.

## `review_summary.yaml` requirements

The authoritative machine-checkable form is `REVIEW_SUMMARY_SCHEMA.yaml`, byte-identical to `schemas/review_summary.schema.yaml`. Every summary must use schema version `rgdtpk-review-v2.2` and include all fields required by that schema, including the v2.2 hardening fields `schema_mirror_sha256`, `appendix_override_checks`, `status_mapping_checks`, and `suite_partition_checks`.

Example shape:

```yaml
schema_version: "rgdtpk-review-v2.2"
phase_id: "P8a"
phase_kind: "algorithmic"
review_status: "PASS"
phase_closable: true
claim_ceiling_after_phase: "PARTIAL_MECHANISM_READY:MECH-013"
base_spec_sha256: "0000000000000000000000000000000000000000000000000000000000000000"
plan_sha256: "0000000000000000000000000000000000000000000000000000000000000000"
source_map_sha256: "0000000000000000000000000000000000000000000000000000000000000000"
review_archive_schema_sha256: "0000000000000000000000000000000000000000000000000000000000000000"
review_summary_schema_sha256: "0000000000000000000000000000000000000000000000000000000000000000"
evidence_manifest_schema_sha256: "0000000000000000000000000000000000000000000000000000000000000000"
schema_mirror_sha256: "0000000000000000000000000000000000000000000000000000000000000000"
commit: "current-commit-or-worktree-id"
prompt_sha256: "0000000000000000000000000000000000000000000000000000000000000000"
response_sha256: "0000000000000000000000000000000000000000000000000000000000000000"
evidence_manifest_sha256: "0000000000000000000000000000000000000000000000000000000000000000"
prompt_path: "docs/ai/changes/RGDTPK-Q-v4-core/reviews/P8a/20260704-120000Z/prompt.md"
response_path: "docs/ai/changes/RGDTPK-Q-v4-core/reviews/P8a/20260704-120000Z/response.md"
evidence_manifest_path: "docs/ai/changes/RGDTPK-Q-v4-core/reviews/P8a/20260704-120000Z/evidence_manifest.yaml"
reviewer_identity: "guardian-reviewer"
reviewed_rids: ["RGQ-020", "RGQ-057", "RGQ-060"]
reviewed_mechs: ["MECH-013"]
source_sections_checked: ["Appendix A §17", "RGQ-057 override table"]
files_inspected: ["geosolver-core/src/kernels/target_relation_search.rs"]
evidence_inspected: ["docs/ai/changes/RGDTPK-Q-v4-core/evidence/P8a/commands.txt"]
commands_inspected: ["cargo test --workspace"]
function_implementation_table: "docs/ai/changes/RGDTPK-Q-v4-core/evidence/P8a/function_implementation_table.yaml"
algorithmic_sufficiency:
  verdict: "sufficient"
  rationale: "Reviewer verified real algorithmic behavior, not only types or wrappers."
forbidden_pattern_scan:
  heavy_fallback: "pass"
  narrow_slice_escape: "pass"
  hidden_fallback: "pass"
  decorative_certificate: "pass"
  placeholder_candidate: "pass"
  geometry_dispatch: "pass"
  unsupported_for_well_formed_input: "pass"
  exact_image_overclaim: "pass"
  target_relation_schedule_discretion: "pass"
  weak_action_krylov_coverage: "pass"
  nonfinite_without_positive_proof: "pass"
  non_spec_status: "pass"
  required_function_deferred: "pass"
  appendix_override_bypass: "pass"
  stress_fixture_dispatch: "pass"
not_applicable_justifications:
  appendix_override_checks[0]: "P8a does not implement Universal or final support; the override table was still checked for drift."
  suite_partition_checks[0]: "P15-only suite partition is not active in P8a; no acceptance claim is made."
raw_response_consistency:
  raw_response_contains_blocker: false
  raw_response_contains_required_fix: false
  summary_overrides_raw_response: false
  reviewer_declared_files_inspected: true
phase_specific_checks:
  - check_id: "P8a-membership-identity"
    required_by: "RGQ-020"
    verdict: "pass"
    evidence: "Exact identity g - Σq_i f_i is reconstructed over Q."
    reviewer_rationale: "Hash-only proof would fail this check."
appendix_override_checks:
  - appendix_section: "Appendix A §20.3"
    controlling_requirement: "RGQ-057"
    verdict: "not_applicable"
    evidence: "This phase does not implement Universal."
    reviewer_rationale: "Override still checked for drift."
status_mapping_checks:
  - error_or_internal_condition: "TargetRelationSearchExhausted"
    mapped_solver_status: "AlgorithmicHardCase"
    verdict: "pass"
    evidence: "No path maps exhaustion to CertifiedNonFiniteTargetImage."
    reviewer_rationale: "Matches RGQ-051/RGQ-058."
suite_partition_checks:
  - suite_name: "not_applicable"
    verdict: "not_applicable"
    evidence: "P15-only suite partition is not active in P8a."
    reviewer_rationale: "Reviewed for premature acceptance wording."
semantic_deletion_challenges: []
tamper_challenges: []
unresolved_risks: []
blocking_findings: []
required_fixes: []
pass_conditions:
  schema_valid: true
  schema_mirror_byte_identical: true
  no_blocking_findings: true
  no_required_fixes: true
  raw_response_consistent: true
  algorithmic_sufficiency_satisfied: true
  all_forbidden_scans_pass: true
  phase_specific_checks_pass: true
  appendix_overrides_respected: true
  status_mapping_respected: true
  suite_partition_respected: true
  reviewer_read_code_not_only_summary: true
  no_controlling_stub_or_placeholder: true
  exact_verification_present_where_required: true
  no_failures_overridden_by_summary: true
  fresh_evidence_after_last_code_change: true
  pass_matches_raw_response: true
  schema_hashes_match_current_files: true
  prompt_and_response_archived: true
  evidence_manifest_archived_and_valid: true
```

A `PASS` is invalid unless all of the following hold:

```text
review_status == PASS
phase_closable == true
blocking_findings == []
required_fixes == []
all relevant forbidden_pattern_scan entries are pass or justified not_applicable
all `not_applicable` verdicts name the exact field path in `not_applicable_justifications` with a nonempty rationale
P15/P16 forbidden_pattern_scan entries are all pass, never not_applicable
all pass_conditions are true
algorithmic_sufficiency.verdict == sufficient, except P0 where not_applicable is allowed
prompt_sha256 matches prompt.md
response_sha256 matches response.md
evidence_manifest_sha256 matches evidence_manifest.yaml
REVIEW_SUMMARY_SCHEMA.yaml and schemas/review_summary.schema.yaml are byte-identical
the raw response does not contain unresolved blockers or required fixes
```

A non-PASS review must set `phase_closable: false`. The Agent must not close a phase from `review_summary.yaml` alone; the prompt, response, evidence manifest, command outputs, function implementation table, and code inspection evidence are required.

## `evidence_manifest.yaml` requirements

The evidence manifest must validate against `schemas/evidence_manifest.schema.yaml`, bind the phase, bind the current source/spec/schema hashes, list every command with an exit code, and list every evidence file used by the reviewer. Evidence from before the last relevant code change is stale.
