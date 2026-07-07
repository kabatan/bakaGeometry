# P1 Source-To-Code Map

Status: P1 implementation evidence.

| R-ID / Plan item | Implementation file/function | Evidence |
|---|---|---|
| BS-R020 module exports | `geosolver-core/src/lib.rs` | `lib.rs` contains module exports only. |
| P1 API solve target | `geosolver-core/src/api.rs::solve_target`, `solver/orchestrator.rs::solve_target` | Public API delegates to orchestrator and returns `TargetSolveResult` without panicking on solver errors. |
| BS-R030 stable IDs | `types/ids.rs` | Added production `fresh_block_id`, `fresh_package_id`, `fresh_kernel_plan_id`; existing stable namespace hashing retained. |
| BS-R031 rational/poly/univariate normalization | `types/rational.rs`, `types/monomial.rs`, `types/polynomial.rs`, `types/univariate.rs` | Existing exact arithmetic/normalization tests pass under `cargo test ... types`. |
| BS-R032 matrix primitives | `types/matrix.rs` | Existing exact shape/density/hash test passes. |
| BS-R032 interval validation | `types/interval.rs` | `interval_new` now enforces `lo < hi`; tests reject `lo == hi`. |
| BS-R130 result status/output | `result/status.rs`, `result/output.rs` | Result tests pass; failure conversion preserves diagnostics and cost trace. |
| P1 audit harness | `geosolver-core/scripts/audit_v4_conformance.py` | Added `--phase P1`; strict P1 audit reports zero findings. |

P1 does not claim global candidate-cover completion.
