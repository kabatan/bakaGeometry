# P1 Algorithm Evidence

Status: P1 implementation evidence.

Implemented changes:

- Added missing source-named production entry points:
  - `preprocess::compression::compress_system`
  - `planner::planner::plan_projection_messages`
  - `solver::pipeline::run_pipeline`
  - `solver::orchestrator::solve_target`
- Kept `api::solve_target(problem, options) -> TargetSolveResult` as the public panic-free API.
- Added fresh ID helpers for block, package, and kernel plan IDs.
- Tightened `interval_new` to reject degenerate intervals, matching BS-R032 `lo < hi`.
- Updated replay tests that used degenerate intervals to use strict rational intervals.
- Added phase-scoped audit support for P1.
- Removed the out-of-source public `TargetSolveResult.exact_image_certificate` and
  `TargetSolveResult.nonfinite_certificate` fields. Existing tests now verify finite-scope
  bindings through `TargetSolveResult.certificate`/status/replay instead of extra public fields.
- Made sparse matrix density and hash canonical over duplicate entries and zero-valued entries,
  and avoided shape-product overflow in density denominators. `SparseMatrixQ` duplicate
  coordinates are summed exactly; `SparseMatrixFp` has no modulus field, so duplicate
  coordinates are encoded as a sorted nonzero residue multiset to keep hash/density
  order-independent.

Behavior evidence:

```text
cargo fmt --manifest-path geosolver-core\Cargo.toml --check
python geosolver-core\scripts\audit_v4_conformance.py --phase P1 --strict
cargo test --manifest-path geosolver-core\Cargo.toml types -- --nocapture
cargo test --manifest-path geosolver-core\Cargo.toml result -- --nocapture
cargo test --manifest-path geosolver-core\Cargo.toml --no-run
```

All listed commands exited 0. Full command output or concise command evidence is archived in
`commands.log`.

Known non-P1 blockers remain from P0, including exact-image scope guard, Descartes/Vincent aliasing,
and global audit findings. P1 does not close those items.
