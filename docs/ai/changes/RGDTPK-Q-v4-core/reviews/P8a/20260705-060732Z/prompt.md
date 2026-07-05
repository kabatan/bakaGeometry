Please re-review P8a after remediation. Do not edit files.

Workspace: C:\Users\bakat\OneDrive\ドキュメント\bakaGeometry

The prior FAIL_FIXABLE finding was missing RGQ-042 public API surface. Remediation added:
- `pub struct MembershipMatrixBuilder`
- `pub struct VerifiedRelationSearchCandidate`
- `pub fn build_membership_matrix_builder(...)`
- `pub fn build_membership_matrix_builder_for_variables(...)`
- `pub fn reconstruct_and_verify_relation(...)`
- test `p8a_required_rgq042_public_api_reconstructs_only_verified_candidates`

The execute path and public wrappers now share the same matrix-building and exact-verification helpers.

Fresh evidence after remediation:
- cargo fmt --manifest-path geosolver-core/Cargo.toml --check: pass
- cargo test --manifest-path geosolver-core/Cargo.toml p8a_ -- --nocapture: pass, 6 passed
- cargo test --manifest-path geosolver-core/Cargo.toml p6_ -- --nocapture: pass, 3 passed
- cargo test --manifest-path geosolver-core/Cargo.toml p7_ -- --nocapture: pass, 9 passed
- cargo test --manifest-path geosolver-core/Cargo.toml: pass, 127 passed
- git diff --check: pass except CRLF warnings

Please inspect:
- geosolver-core/src/kernels/target_relation_search.rs
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P8a/*

Review P8a only. Return RESULT PASS/FAIL with findings and exact PASS scope.
