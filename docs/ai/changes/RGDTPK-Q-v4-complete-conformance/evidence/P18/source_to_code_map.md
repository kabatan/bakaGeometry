Purpose: P18 source-to-code map
Status: evidence, non-authoritative

# P18 Source-to-Code Map

P18 is the final conformance suite, static audit, closure, and reviewer phase. It does not replace the phase maps from P0-P17; it cross-checks them against the finite candidate-cover completion conditions in BS-R150.

| Plan task | Base Spec / acceptance | Code and evidence |
|---|---|---|
| Add final conformance suite | BS-R000 through BS-R150, Acceptance A3 | `geosolver-core/tests/v4_candidate_cover_conformance.rs` covers generic public API behavior under misleading roles/names/relation order, Descartes candidate decoding and hash binding, exact-image scope guard, normalization/hash order-independence, and evidence-backed bounded failure. |
| Property/adversarial checks | BS-R001, BS-R010, BS-R030, BS-R041, BS-R121, BS-R150 | `v4_candidate_cover_conformance.rs`; updated large-footprint and generic success-route tests in `geosolver-core/tests/acr_p9_large_footprint_stress.rs` and `geosolver-core/tests/generic_success_route_planner.rs`. |
| Static forbidden-path audit | BS-R010, BS-R150, Acceptance A2 | `geosolver-core/scripts/audit_v4_conformance.py --strict`; final output recorded in `evidence/final/final_commands.log` and `evidence/P18/static_audit.log`. |
| Strict clippy/fmt/test evidence | Acceptance A4 | `cargo fmt --manifest-path geosolver-core\Cargo.toml --check`, `cargo clippy --manifest-path geosolver-core\Cargo.toml --all-targets -- -D warnings`, `cargo test --manifest-path geosolver-core\Cargo.toml`; all exit 0 in `evidence/final/final_commands.log`. |
| Final matrix and closure | Acceptance A1, A4 | `evidence/final/source_to_code_conformance_matrix.md` and `CLOSURE.md`. |

Audit script correction:
- P18 adjusted `audit_v4_conformance.py` so strict all-phase scanning does not flag negative no-coordinate-export guard strings or the replay/status exact-image scope rejection as production coordinate/RUR/exact-image success paths.
- The correction is narrow: strict mode still scans production Rust files for forbidden coordinate solver markers and exact-image success references; it only permits explicit no-export markers and finite-scope rejection references.
