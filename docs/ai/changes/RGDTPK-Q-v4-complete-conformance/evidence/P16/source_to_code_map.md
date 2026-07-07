# P16 Source To Code Map

Phase: P16 - exact-image scope guard and semantic boundary.

## BS-R003 / BS-R040 / BS-R122 - Exact-Image Out Of Scope

- `geosolver-core/src/solver/options.rs`
  - `SolverOptions::exact_image_mode` remains an exposed request flag and is bound into solver options hash.
- `geosolver-core/src/solver/orchestrator.rs`
  - exact-image requests no longer call `classify_real_target_image`.
  - exact-image requests no longer filter `root_isolation` or `decoded_candidates`.
  - exact-image requests return `SolverStatus::CertificateDesignGap`.
  - `exact_image_out_of_scope_diagnostic` emits `ExactImageOutOfScope` with input hash, support hash, squarefree support hash, candidate hashes, and candidate count.
  - `exact_image_out_of_scope_nonfinite_diagnostic` maps exact-image nonfinite outcomes to `CertificateDesignGap` and binds the nonfinite certificate hash in diagnostics.
- `geosolver-core/src/verify/run_certificate.rs`
  - solver options hash includes `exact_image_mode`.
  - `exact_image_certificate_hash` remains bound in the run certificate and remains `None` for the scoped repair path.
- `geosolver-core/src/verify/replay.rs`
  - replay rejects non-`None` exact-image certificate hash.
  - replay rejects `CertifiedExactTargetImage` and `CertifiedEmptyRealTargetImage`.
- `geosolver-core/src/problem/semantic.rs`
  - semantic provenance remains represented in input/compression data for future exact-image work; candidate-cover mode does not classify by it.

## P16 Remediation Notes

- Removed reachable exact-image success path from orchestrator.
- Removed candidate filtering by semantic/exact-image classifier from exact-image requests.
- Exact-image requests now preserve candidate-cover support/root/candidate evidence but report `CertificateDesignGap` and `ExactImageOutOfScope`.
- Exact-image nonfinite requests now report `CertificateDesignGap` and `ExactImageOutOfScope`; `CertifiedNonFiniteTargetImage` remains available only outside exact-image mode.
- Candidate-cover mode remains unchanged and still diagnoses possible spurious roots.
