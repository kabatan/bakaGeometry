# P5R-f Notes

P5R-f is the final audit gate before P6. It depends on archived PASS reviews for P5R-a through P5R-e, fresh command evidence, final readiness wording, and a commit-bound remediation packet.

Current claim ceiling remains `PARTIAL_MECHANISM_READY:MECH-004`. P5R-f must not claim planner admission, kernel execution, candidate-cover construction, exact-image classification, public orchestration, performance readiness, or final acceptance.

Fresh final-audit commands after the last code change pass:

- `cargo fmt --manifest-path geosolver-core/Cargo.toml -- --check`
- full crate tests, 106 passed
- P5R-specific linear affine, compression, F4 demotion, elimination, quotient provenance, Krylov provenance, and graph tests

`schema_validation.txt` records PASS archives and schema-valid `review_summary.yaml` / `evidence_manifest.yaml` for P5R-a through P5R-e. `static_scans.txt` records the required P5R-f scans and classifies expected hits as negative no-export checks, enum definitions, demoted non-production F4 naming, or historical P0-P5 archive records.
