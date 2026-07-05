# P8a Reviewer FAIL_FIXABLE Remediation

Reviewer result before remediation: `FAIL_FIXABLE`.

Finding:

The implementation had working dense membership matrix construction and inline candidate verification, but it did not expose the exact RGQ-042 public functions/types:

- `MembershipMatrixBuilder`
- `VerifiedRelationSearchCandidate`
- `build_membership_matrix_builder`
- `reconstruct_and_verify_relation`

Fix:

- Added the required public structs.
- Added `build_membership_matrix_builder` and `build_membership_matrix_builder_for_variables`.
- Added `reconstruct_and_verify_relation`, routed through the same candidate reconstruction and exact Q identity verifier used by execution.
- Added `p8a_required_rgq042_public_api_reconstructs_only_verified_candidates`.

Verification after remediation:

- `cargo fmt --manifest-path geosolver-core/Cargo.toml --check`: pass
- `cargo test --manifest-path geosolver-core/Cargo.toml p8a_ -- --nocapture`: pass, 6 passed
- `cargo test --manifest-path geosolver-core/Cargo.toml p6_ -- --nocapture`: pass, 3 passed
- `cargo test --manifest-path geosolver-core/Cargo.toml p7_ -- --nocapture`: pass, 9 passed
- `cargo test --manifest-path geosolver-core/Cargo.toml`: pass, 127 passed
- `git diff --check`: pass, with CRLF warnings only
