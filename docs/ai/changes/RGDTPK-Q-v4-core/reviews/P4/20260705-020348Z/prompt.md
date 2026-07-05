Guardian boundary review for RGDTPK-Q-v4-core P4.

Workspace: C:\Users\bakat\OneDrive\ドキュメント\bakaGeometry

Review target: Plan P4 only. Scope is preprocessing/compression with provenance. Do not require P5 graph construction, P6 planner, P7+ kernel integration, candidate-cover readiness, exact-image readiness, acceptance suites, or final solver completion.

Required source anchors:

- `BASE_SPEC.md` `RGQ-013`, `RGQ-003`, `RGQ-037`, `RGQ-058`, `RGQ-059`.
- Appendix A section 11.
- `PLAN.md` P4.
- `REVIEWER_PROMPTS.md#P4`.
- Review archive and evidence schemas.

Changed files to inspect:

- `geosolver-core/src/preprocess/compression.rs`.
- `geosolver-core/src/preprocess/definitional.rs`.
- `geosolver-core/src/preprocess/linear_affine.rs`.
- `geosolver-core/src/preprocess/binomial.rs`.
- `geosolver-core/src/preprocess/saturation.rs`.
- `geosolver-core/src/preprocess/independent.rs`.

Evidence to inspect:

- `docs/ai/changes/RGDTPK-Q-v4-core/evidence/P4/commands.txt`.
- `docs/ai/changes/RGDTPK-Q-v4-core/evidence/P4/command_outputs.txt`.
- `docs/ai/changes/RGDTPK-Q-v4-core/evidence/P4/static_scans.txt`.
- `docs/ai/changes/RGDTPK-Q-v4-core/evidence/P4/function_implementation_table.yaml`.
- `docs/ai/changes/RGDTPK-Q-v4-core/evidence/P4/notes.md`.

Evidence hashes:

- `commands.txt`: `af650eac6a8912930397b144ed5fde7dfdb030730d64055e7e88d497dd12661a`.
- `command_outputs.txt`: `5261fa948b18ae5e8820d6af2e0ada689cacc479cc92244def7e252de9b17e14`.
- `static_scans.txt`: `8ff86eb13cb0691a6dce4bd6a1ebfcd1bb59df460c817c95fdafe61b6403b166`.
- `function_implementation_table.yaml`: `d79971dc0919f4ee52809172ecabab3b608f277343fbfd28c4f65252a868fb36`.
- `notes.md`: `a478706be958c8ab4ecb8ded548017c50e48dd34f427a9fec475f1585382c46a`.

Expected implementation summary:

- `compression.rs` defines `CompressedSystemQ`, `CompressionState`, `CompressionTrace`, substitution/guard/saturation/component/feasibility-obligation records, and implements `pre_kernel_compress`.
- `pre_kernel_compress` runs exactly this order: definitional elimination, linear-affine elimination, binomial simplification, explicit saturation, target-independent component marking, then monomial/coefficient trace finalization.
- `definitional.rs` implements `find_definitional_relations` and `apply_definitional_elimination`, accepting only exact `y - p(X)` / `c*y - p(X)` with `c in Q\\{0}`, and never eliminating the target variable.
- `linear_affine.rs` implements `find_linear_affine_candidates`, `select_safe_affine_pivots`, and `eliminate_linear_affine_variables`, accepting constant nonzero pivots directly and nonconstant pivots only when `A*s - 1 = 0` proves the denominator guard and `-b/A` is an exact Q-polynomial quotient.
- `binomial.rs` implements `detect_binomial_relations` and `simplify_binomial_relations`, primitive-normalizing and deduplicating monomial/binomial relations without factor splitting or union semantics.
- `saturation.rs` implements `apply_explicit_saturations` and `is_explicit_nonzero_factor`; nonconstant saturation proof is from explicit `A*s - 1 = 0` witness extraction, not unbound semantic labels.
- `independent.rs` implements `mark_target_independent_components` and `compute_component_feasibility_obligations`; target-independent components are removed from candidate-cover relations only after relation-variable component computation and are retained as hash-bound feasibility obligations.

Commands/evidence summary:

- `cargo fmt --manifest-path geosolver-core\Cargo.toml -- --check`: pass.
- Focused `preprocess` tests: 8 passed after correcting a test oracle for a second safe affine substitution.
- Full crate tests: 88 passed.
- Static scans: no forbidden markers, geometry/fixture/expected-answer dispatch, unsafe factor-selection wording, full-coordinate paths, QE/CAD/homotopy fallback, or final solver status overclaim in P4-owned files.

Failure/fix loop to verify:

- Initial focused preprocess test failed because it expected one substitution where the safe pipeline produced two (`y=x`, then `x=t`).
- The implementation did not eliminate the target.
- The test oracle was corrected to expect two substitutions and no remaining relation in that algebraically free-target case.
- Re-run evidence after the fix passes.

Review instructions:

- Report `PASS`, `FAIL_FIXABLE`, or `FAIL_BLOCKING`.
- PASS is allowed only if the P4 required functions are real implementations and preserve guard/component/provenance semantics.
- Fail if ordered compression steps are wrong, if nonconstant affine substitution occurs without recorded nonzero guard, if explicit saturation uses anything other than an explicit witness or nonzero constant, if binomial simplification performs unsafe factor/component split, if target-independent components are deleted without exact-image feasibility obligations, if geometry names/expected answers are used, or if P4 claims later graph/kernel/candidate-cover/exact-image completion.
- If PASS, state whether P4 closes `MECH-003` or only starts/continues it. If MECH-003 closure is not justified, keep claim ceiling `SCAFFOLD_READY`; if closure is justified under the Plan, use the schema-allowed claim ceiling `PARTIAL_MECHANISM_READY:MECH-003`.
