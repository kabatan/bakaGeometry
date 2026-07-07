# P3 Algorithm Evidence

Status: P3 implementation evidence.

Implemented changes:

- Added explicit monomial-order tests proving eliminated variables dominate keep variables, block
  order precedence, and grevlex behavior.
- Added `try_crt_vector_combine` so vector CRT has a production checked reject path for length
  mismatch or incompatible congruences.
- Added P3 phase support to `audit_v4_conformance.py`.

Existing implementation evidence retained:

- Modular solve results are marked candidate-only and require exact Q checks.
- Sparse/dense row reduction uses deterministic pivot search.
- Rational reconstruction returns `None` for ambiguous/out-of-bound cases.
- Membership certificates verify exact Q identities and reject out-of-range relation IDs.

Behavior evidence:

```text
cargo fmt --manifest-path geosolver-core\Cargo.toml --check
python geosolver-core\scripts\audit_v4_conformance.py --phase P3 --strict
cargo test --manifest-path geosolver-core\Cargo.toml --lib algebra -- --nocapture
cargo test --manifest-path geosolver-core\Cargo.toml --no-run
```

All listed commands exited 0.

Known non-P3 blockers remain governed by later phases.
