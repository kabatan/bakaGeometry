# P8b Reviewer FAIL_FIXABLE Remediation

Reviewer result before remediation: `FAIL_FIXABLE`.

Findings:

1. `SparseResultantProjectionKernel` ignored child message relations even though the spec execution input is local relations plus child messages.
2. `SpecializationInterpolationKernel` generated interpolation samples by specializing a precomputed exported Groebner generator instead of executing a declared inner target-only kernel on each specialized context.

Fix:

- Added child-message relation inputs to both P8b kernels.
- Bound child message package hashes into `KernelExecutionPlan.child_message_hashes`.
- Added execution-time child package-hash binding checks.
- Added source-hash coverage checks that work for mixed local and child-message relations.
- Changed SpecializationInterpolation candidate generation to execute `TargetRelationSearch` as the declared inner target-only kernel for each specialization sample.
- Kept local Groebner/elimination only as final exact Q verification for the interpolated candidate.
- Added child-required and child-tamper regression tests for both kernels.

Verification after remediation:

- `cargo fmt --manifest-path geosolver-core/Cargo.toml --check`: pass
- `cargo test --manifest-path geosolver-core/Cargo.toml p8b_ -- --nocapture`: pass, 7 passed
- `cargo test --manifest-path geosolver-core/Cargo.toml interpolation -- --nocapture`: pass, 9 passed
- `cargo test --manifest-path geosolver-core/Cargo.toml resultant -- --nocapture`: pass, 11 passed
- `cargo test --manifest-path geosolver-core/Cargo.toml p6_ -- --nocapture`: pass, 3 passed
- `cargo test --manifest-path geosolver-core/Cargo.toml p7_ -- --nocapture`: pass, 9 passed
- `cargo test --manifest-path geosolver-core/Cargo.toml p8a_ -- --nocapture`: pass, 6 passed
- `cargo test --manifest-path geosolver-core/Cargo.toml`: pass, 136 passed
- `git diff --check`: pass, with CRLF warnings only
