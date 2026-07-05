FAIL_FIXABLE

**Reviewed Scope**
Reviewed R-IDs: `RGQ-020`, `RGQ-025`, `RGQ-043`, with `RGQ-058`/`RGQ-059` checked for status/function drift.  
Reviewed MECH: `MECH-007` continuation only. Claim ceiling remains `SCAFFOLD_READY`.

**Files Inspected**
Inspected all required packet files: `BASE_SPEC.md`, `PLAN.md`, reviewer/schema files, prior P3d review archive files, P3d evidence files, and source files:
`geosolver-core/src/algebra/resultant.rs`, `interpolation.rs`, `modular.rs`, `types/polynomial.rs`, `result/status.rs`.

**Evidence And Commands Inspected**
Inspected `commands.txt`, `command_outputs.txt`, `static_scans.txt`, `function_implementation_table.yaml`, `notes.md`, prior `review_summary.yaml`, and prior `evidence_manifest.yaml`.

Evidence reports:
`cargo fmt --check` passed, targeted `algebra::resultant` passed 5 tests, targeted `algebra::interpolation` passed 3 tests, full crate tests passed 64 tests.

I also performed read-only schema/hash/static-scan checks. Prior `review_summary.yaml` and `evidence_manifest.yaml` validate against their schemas. `REVIEW_SUMMARY_SCHEMA.yaml` and `schemas/review_summary.schema.yaml` are byte-identical by SHA-256.

**Algorithmic Sufficiency Judgment**
Insufficient for PASS. The implementation is real algebra-level P3d primitive work and does not overclaim final solver certification, but one required boundary fix remains incomplete under the packet's exact wording.

**Blocker**
`build_sparse_resultant_template` still accepts noncanonical `keep_variables` ordering.

`validate_keep_variables` in `geosolver-core/src/algebra/resultant.rs:138` rejects eliminate/keep overlap and duplicate variables, but it does not reject a noncanonical order such as `[VariableId(3), VariableId(1)]`. `hash_template` in `resultant.rs:408` hash-binds `input.keep_variables` in caller-provided order, so noncanonical keep-variable boundaries can produce distinct templates for the same keep set. The packet explicitly required "reject overlap and duplicate/noncanonical keep variables"; this is not fully satisfied.

**Prior Fix Status**
Overlap rejection: fixed and tested by `resultant_template_rejects_overlapping_or_duplicate_keep_variables`.

Duplicate keep rejection: fixed and tested by the same regression.

Tampered denominator-dividing trace prime panic: fixed for the tested case. `verify_resultant_certificate` checks `trace_prime_is_valid_for_poly` before `reduce_q_to_fp`, and the regression `resultant_certificate_rejects_tampered_trace_prime_without_panic` covers the prior panic path.

Residual trace-prime risk: `trace_prime_is_valid_for_poly` only checks `prime >= 2` and denominator nonzero modulo that value. It does not check actual primality. Since the packet speaks of invalid/tampered trace primes, the next fix should either reject non-prime moduli or otherwise make certificate-time modular reduction explicitly fallible for all invalid moduli.

**Phase-Specific Checks**
`support_sets`: pass for deterministic support extraction and support hashing.

`build_sparse_resultant_template`: fail fixable due noncanonical keep-variable acceptance; otherwise validates non-keep variables and resource caps.

`compute_resultant_relation`: pass for exact Sylvester resultant candidate generation, modular trace hashes, and candidate-only proof status.

`verify_resultant_certificate`: mostly pass for recomputing template/resultant and rejecting tampered relation/hash evidence; residual trace-prime validity hardening noted above.

`choose_specialization_points` / `specialize_polynomials`: pass for deterministic exact specialization.

`interpolate_sparse_coefficients`: pass for implemented one-specialized-variable exact sparse coefficient interpolation and invalid sample rejection.

`verify_interpolated_relation`: pass for exact re-specialization checks and bad-sample rejection.

Candidate-only boundary: pass. P3d does not produce `CertifiedCandidateCover`, `CertifiedExactTargetImage`, or `CertifiedNonFiniteTargetImage`.

**Forbidden Scan Judgment**
Direct scans and supplied evidence show no coordinate roots, full coordinate RUR, global coordinate parametrization, solve-all-coordinates path, global CAS/QE/CAD/homotopy, ordinary `Unsupported`, or placeholder/stub markers in P3d-owned files.

**Forbidden Claims**
Do not claim P3d closes `SparseResultantProjectionKernel` or `SpecializationInterpolationKernel` admission/execution.  
Do not claim resultant/interpolation candidates are certified Q relations.  
Do not raise the claim ceiling above `SCAFFOLD_READY`.

**review_summary.yaml Implications**
For this review response, `review_status` must be `FAIL_FIXABLE`, `phase_closable: false`, `algorithmic_sufficiency.verdict: insufficient`, `raw_response_contains_blocker: true`, and `required_fixes` nonempty. A PASS summary would conflict with this raw response.

**Required Fixes**
Reject noncanonical keep-variable ordering in `validate_keep_variables`, and add a negative test for it.

Strengthen trace-prime validation to reject non-prime or otherwise invalid moduli during certificate checking, or make certificate-time modular reduction explicitly fallible for all invalid trace moduli.
