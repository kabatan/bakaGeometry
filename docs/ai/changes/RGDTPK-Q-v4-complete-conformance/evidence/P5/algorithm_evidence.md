# P5 Algorithm Evidence

Status: P5 implementation evidence.

Implemented changes:

- Tightened explicit saturation so an `A*s - 1 = 0` shape is accepted only when it is bound to a
  `RealConstraintKind::NonZero` semantic encoding for the same relation and slack variable.
- Added a regression showing unencoded nonzero-shaped relations do not create saturation or guard
  records.
- Added unsafe affine pivot diagnostics for nonconstant denominators without a recorded nonzero
  guard, while leaving those relations in the system.
- Remediated the P5 reviewer finding that binomial dedup could drop a semantically referenced
  duplicate relation ID. Duplicate primitive/binomial groups now retain the full group if any member
  is referenced by semantic encodings, and a regression verifies that later explicit saturation still
  finds a semantic NonZero witness on the duplicate relation ID.
- Added P5 static audit support for required files/symbols, compression order, semantic saturation
  binding, unsafe affine diagnostics, and independent-component obligations.

Behavior evidence:

```text
cargo fmt --manifest-path geosolver-core\Cargo.toml --check
python geosolver-core\scripts\audit_v4_conformance.py --phase P5 --strict
cargo test --manifest-path geosolver-core\Cargo.toml --lib preprocess -- --nocapture
cargo test --manifest-path geosolver-core\Cargo.toml --lib semantic -- --nocapture
cargo test --manifest-path geosolver-core\Cargo.toml --no-run
```

All listed commands exited 0.

Known non-P5 blockers remain governed by later phases.
