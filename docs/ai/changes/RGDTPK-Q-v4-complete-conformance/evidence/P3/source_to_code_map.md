# P3 Source-To-Code Map

Status: implementation evidence for Phase 3.

Relevant R-IDs:

- BS-R050 -> `algebra/monomial_order.rs`, `algebra/polynomial_ops.rs`
- BS-R051 -> `algebra/modular.rs`, `algebra/crt.rs`, `algebra/rational_reconstruction.rs`
- BS-R052 -> `algebra/sparse_matrix.rs`, `algebra/dense_matrix.rs`, `algebra/linear_solve.rs`, `types/matrix.rs`
- BS-R053 -> `algebra/normal_form.rs`

Mapping:

- Lex, grevlex, block, and elimination monomial orders are in `monomial_order.rs`.
- Leading term, S-polynomial, reduction by set with quotient evidence, and primitive content
  operations are in `polynomial_ops.rs`.
- Deterministic prime selection, Q-to-Fp reduction, modular arithmetic, and denominator/coefficient
  avoidance are in `modular.rs`.
- CRT scalar/vector combine and checked reject paths are in `crt.rs`.
- Rational and polynomial reconstruction with height bounds are in `rational_reconstruction.rs`.
- Sparse and dense modular row echelon, rank, and nullspace are in `sparse_matrix.rs` and
  `dense_matrix.rs`.
- `solve_homogeneous_modular` and `solve_inhomogeneous_modular` return candidate-only results with
  `ModularProofStatus::CandidateOnlyRequiresExactQCheck` in `linear_solve.rs`.
- Exact Q identity membership verification is in `normal_form.rs`.
