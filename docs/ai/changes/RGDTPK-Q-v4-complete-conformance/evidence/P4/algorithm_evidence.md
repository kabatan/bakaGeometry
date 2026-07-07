# P4 Algorithm Evidence

Status: P4 implementation evidence.

Implemented changes:

- Promoted `algebra::f4` to a production module.
- Replaced the old test-only/non-production batch path with production `F4Options`,
  `F4BatchReductionResult`, `f4_reduce_batch`, and `f4_elimination_local`.
- Renamed dispatcher variants to the source-required production names.
- Added neutral `local_result_from_certified_basis` so Groebner and F4 share only the final
  certificate/export validation path.
- Added tests for F4 batch matrix construction, exact remainder certificate verification,
  F4/Groebner agreement on a small eliminant, dispatcher F4 execution, and production F4 reachability.
- Added P4 phase support to `audit_v4_conformance.py`, including a non-production F4 marker scan.
- Remediated the P4 reviewer finding that F4 batch reduction only wrapped Groebner:
  `f4_reduce_batch` now computes returned reductions from exact row reduction over the F4
  symbolic preprocessing matrix rows, after deterministic modular matrix trace construction.
- Remediated the P4 reviewer finding that Universal contained non-source internal stages:
  `UniversalStrategy` and Universal fixed planning now use exactly
  `EliminationGroebnerLocal`, `F4EliminationLocal`, `TargetRelationSearchEscalated`,
  `ResultantIfSquareOrOverdetermined`, and `SpecializeProjectInterpolateVerify`.
- Removed Universal internal use of ActionKrylov, RegularChain, and NormTrace. They remain
  separate ladder kernels only.
- Added direct exported-source relation handling to TargetRelationSearch so
  `TargetRelationSearchEscalated` can produce a membership-certified candidate cover without
  materializing high-footprint dense/sparse matrices when a source relation already lies in Q[Z].

Behavior evidence:

```text
cargo fmt --manifest-path geosolver-core\Cargo.toml --check
python geosolver-core\scripts\audit_v4_conformance.py --phase P4 --strict
cargo test --manifest-path geosolver-core\Cargo.toml --lib f4 -- --nocapture
cargo test --manifest-path geosolver-core\Cargo.toml --lib elimination -- --nocapture
cargo test --manifest-path geosolver-core\Cargo.toml --lib universal -- --nocapture
cargo test --manifest-path geosolver-core\Cargo.toml --no-run
```

All listed commands exited 0.

Known non-P4 blockers remain governed by later phases.
