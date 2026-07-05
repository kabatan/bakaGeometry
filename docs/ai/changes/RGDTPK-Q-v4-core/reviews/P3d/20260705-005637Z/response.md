FAIL_FIXABLE

**Reviewed Scope**
Reviewed R-IDs: `RGQ-020`, `RGQ-025`, `RGQ-043`; also checked `RGQ-058`/`RGQ-059` where they constrain statuses and required primitive functions.  
Reviewed MECH: `MECH-007` continuation only. Claim ceiling remains `SCAFFOLD_READY`.

**Files Inspected**
`BASE_SPEC.md`, `PLAN.md`, `REVIEWER_PROMPTS.md`, `REVIEW_ARCHIVE_SCHEMA.md`, `REVIEW_SUMMARY_SCHEMA.yaml`, `schemas/evidence_manifest.schema.yaml`, `geosolver-core/src/algebra/resultant.rs`, `geosolver-core/src/algebra/interpolation.rs`, `geosolver-core/src/algebra/modular.rs`, `geosolver-core/src/types/polynomial.rs`, `geosolver-core/src/result/status.rs`, and all required P3d evidence files.

**Evidence Inspected**
Commands/evidence inspected: `commands.txt`, `command_outputs.txt`, `static_scans.txt`, `function_implementation_table.yaml`, `notes.md`.

Evidence records passing `cargo fmt`, targeted resultant/interpolation tests, full crate tests, and static scans. I did not rerun build/test commands because this was read-only review. `REVIEW_SUMMARY_SCHEMA.yaml` and `schemas/review_summary.schema.yaml` are byte-identical by hash.

**Algorithmic Sufficiency**
Insufficient for PASS. The P3d primitives are real implementations, not hook-only scaffolding, and they correctly keep outputs candidate-only. However, two boundary/certificate hardening issues prevent closure.

**Blockers**
1. `build_sparse_resultant_template` does not fully validate eliminate/keep boundaries.  
   In `resultant.rs:93-102`, `keep_variables` is converted to a set only for variable containment checks. There is no rejection when `input.eliminate` is also listed in `keep_variables`, and no duplicate/canonical keep-variable validation. That leaves malformed local/export boundaries admissible, contrary to the packet's boundary check for eliminate/keep validation.

2. `verify_resultant_certificate` can panic instead of rejecting tampered modular trace evidence.  
   `verify_resultant_certificate` reduces the recomputed relation at each supplied trace prime (`resultant.rs:175-178`). `reduce_q_to_fp` uses `expect("denominator must be nonzero modulo chosen prime")` in `modular.rs:104`. A tampered certificate can supply a trace prime dividing a rational denominator, causing panic rather than clean rejection. This fails the requirement that tampered modular trace evidence be rejected.

**Phase-Specific Checks**
`support_sets`: pass for deterministic monomial support extraction and hashing.  
`compute_resultant_relation`: pass for exact Sylvester resultant candidate generation and candidate-only proof status.  
`verify_resultant_certificate`: fail fixable due tampered modular-trace panic path.  
`choose_specialization_points` / `specialize_polynomials`: pass for deterministic exact specialization.  
`interpolate_sparse_coefficients`: pass within the implemented one-specialized-variable primitive scope, with invalid sample checks.  
`verify_interpolated_relation`: pass for exact re-specialization sample checking and bad-sample rejection.  
Candidate-only boundary: pass; P3d does not produce `CertifiedCandidateCover`, `CertifiedExactTargetImage`, or `CertifiedNonFiniteTargetImage`.

**Forbidden Scan Judgment**
The supplied scans and direct scan inspection show no coordinate roots, full coordinate RUR, global coordinate parametrization, solve-all-coordinates path, CAS/QE/CAD/homotopy fallback, ordinary `Unsupported`, or placeholder/stub markers in P3d-owned files.

**Forbidden Claims**
Do not claim P3d closes `SparseResultantProjectionKernel` or `SpecializationInterpolationKernel` admission/execution.  
Do not claim resultant/interpolation candidates are certified Q relations.  
Do not raise the claim ceiling above `SCAFFOLD_READY`.

**review_summary.yaml Implications**
For this review response, `review_status` must be `FAIL_FIXABLE`, `phase_closable: false`, `algorithmic_sufficiency.verdict: insufficient`, `raw_response_contains_blocker: true`, and `required_fixes` nonempty. A PASS summary would conflict with the raw response.

**Required Fixes**
Add eliminate/keep boundary validation, including reject overlap and duplicate/noncanonical keep variables as appropriate.  
Make modular reduction in certificate checking fallible, and have `verify_resultant_certificate` return `false` on invalid/tampered trace primes. Add negative tests for both issues.
