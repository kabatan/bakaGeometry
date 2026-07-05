Guardian boundary review for RGDTPK-Q-v4-core P3f.

Workspace: C:\Users\bakat\OneDrive\ドキュメント\bakaGeometry

Review target: Plan P3f only. Do not review or require later kernel integration, root decode, exact-image fiber classification, acceptance suites, or MECH closure beyond the P3f primitive-layer claim.

Required source anchors:

- `BASE_SPEC.md` `RGQ-023`, `RGQ-024`, `RGQ-028`, `RGQ-029`, `RGQ-058`, `RGQ-059`.
- Appendix A sections 10.16 through 10.19.
- `PLAN.md` P3f.
- `REVIEWER_PROMPTS.md#P3f`.
- Review archive and evidence schemas.

Changed files to inspect:

- `geosolver-core/src/algebra/regular_chain.rs`.
- `geosolver-core/src/algebra/norm_trace.rs`.
- `geosolver-core/src/algebra/real_root.rs`.
- `geosolver-core/src/algebra/sign.rs`.

Evidence to inspect:

- `docs/ai/changes/RGDTPK-Q-v4-core/evidence/P3f/commands.txt`.
- `docs/ai/changes/RGDTPK-Q-v4-core/evidence/P3f/command_outputs.txt`.
- `docs/ai/changes/RGDTPK-Q-v4-core/evidence/P3f/static_scans.txt`.
- `docs/ai/changes/RGDTPK-Q-v4-core/evidence/P3f/function_implementation_table.yaml`.
- `docs/ai/changes/RGDTPK-Q-v4-core/evidence/P3f/notes.md`.

Evidence hashes:

- `commands.txt`: `7485f3373fb8a0242d92b6f754057cffc21cc18ae41f691755134aa17097e331`.
- `command_outputs.txt`: `12749805a2e6463ca6f6488ac2de774716ea5f3fb4034c380fe96c6c8ee620a3`.
- `static_scans.txt`: `13575ef93b75627beb5d674f83e36c4df06e64a31231104994af58a257895112`.
- `function_implementation_table.yaml`: `1fdba67f27658218b6db3e3eb48e92f43026a728ce92a0eb780b06b226fd9764`.
- `notes.md`: `ae298b4282d72bdfde519b8f321bd9b6f586154dded587c657f02300dc7af203`.

Expected implementation summary:

- `regular_chain.rs` defines `RegularChainInput`, `RegularChainDAG`, `RegularChain`, `ProjectionGenerators`, and `UnionSemantics`; implements `local_regular_chain_decomposition`, `project_chain_to_variables`, and `combine_chain_projections`.
- Regular-chain decomposition normalizes relations/guards, validates canonical variable lists and declared-variable coverage, rejects duplicate triangular main variables, stores guard/component semantics and hashes, and returns a compact single-chain DAG for already triangular local input.
- Regular-chain projection validates keep variables, calls exact local Groebner elimination when non-keep variables must be eliminated, rechecks output is in `Q[keep]`, and preserves guards/component semantics.
- Projection combination uses generator union for intersection semantics and products for component-union semantics.
- `norm_trace.rs` defines `TowerDescription` and `UniOrMultiPolynomialQ`; implements `detect_explicit_tower`, `norm_of_target_minus_expression`, and `verify_norm_relation`.
- Tower detection is by algebraic form only: exactly one non-exported algebraic variable, a univariate minimal-polynomial relation in that variable, and a target-minus-expression over exported variables plus that algebraic variable. It rejects noncanonical exported-variable lists.
- Norm computation validates variable boundaries and computes the exact Sylvester resultant of minimal polynomial and target-minus-expression through the P3d resultant primitive, primitive-normalized over Q. Verification recomputes and compares primitive-normalized relations, so nonzero resultant scalar convention is not treated as failure.
- `real_root.rs` implements `sturm_sequence`, `isolate_real_roots_sturm`, and `isolate_real_roots_descartes` using exact RationalQ arithmetic, exact Cauchy bounds, Sturm sign-variation counts, and rational isolating intervals. It rejects zero support as `AlgorithmicHardCase`.
- `sign.rs` implements `sign_at_algebraic_root` and `thom_encoding`; it returns `RefinementRequired` rather than an asserted sign when the current interval cannot certify sign constancy.

Commands/evidence summary:

- `cargo fmt --manifest-path geosolver-core\Cargo.toml -- --check`: pass.
- Focused P3f tests: `algebra::regular_chain` 3 passed; `algebra::norm_trace` 4 passed after scalar-normalization test fix; `algebra::real_root` 2 passed; `algebra::sign` 2 passed.
- Full crate tests: 80 passed.
- Static scans: no forbidden markers, geometry dispatch, floating paths, coordinate-solution/full-RUR paths, QE/CAD/homotopy, or certified final solver status overclaims in P3f-owned files.

Failure/fix loop to verify:

- Initial `algebra::norm_trace` test failed because the test compared a resultant relation against a fixed sign convention.
- Fix was targeted: compare both sides after `clear_denominators_primitive`.
- Re-run evidence after the fix passes.

Review instructions:

- Report `PASS`, `FAIL_FIXABLE`, or `FAIL_BLOCKING`.
- PASS is allowed only if the P3f required functions are real implementations, not empty foundations, hook-only wrappers, unconditional verifiers, or deferred shells.
- Fail if regular-chain component/guard/projection semantics are dropped, if tower detection uses geometry names, if root/sign code is floating-only, if sign classification asserts success without exact evidence, if norm verification is unconditional, or if any P3f path claims kernel integration/candidate-cover/exact-image completion.
- If PASS, state claim ceiling remains `SCAFFOLD_READY`; P3f starts executable primitive support for `MECH-007`, `MECH-011`, and `MECH-012` but does not close them.
