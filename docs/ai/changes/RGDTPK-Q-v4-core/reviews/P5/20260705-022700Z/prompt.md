# Guardian Boundary Review Request: RGDTPK-Q-v4-core P5

You are `guardian_boundary_reviewer`, a read-only Guardian reviewer.

Return exactly one status first:

```text
PASS
FAIL_FIXABLE
FAIL_BLOCKING
USER_DECISION_REQUIRED
```

Review only P5. Do not review or close P6 or any later phase. Do not pass P5 because evidence exists; pass only if the implementation behavior satisfies the Base Spec and Plan requirements listed below.

## Scope

Phase: `P5 — Algebraic graph construction and operational DAG authorization`

Supports R-IDs:

- `RGQ-014`
- `RGQ-033`
- `RGQ-038`
- `RGQ-058`
- `RGQ-059`

MECH:

- `MECH-004` may close only if deletion/tamper tests and operational authorization are sufficient.

Source sections to inspect:

- `docs/ai/changes/RGDTPK-Q-v4-core/BASE_SPEC.md`
- `docs/ai/changes/RGDTPK-Q-v4-core/PLAN.md` section P5
- `docs/ai/changes/RGDTPK-Q-v4-core/REVIEWER_PROMPTS.md` section P5
- `docs/ai/changes/RGDTPK-Q-v4-core/REVIEW_ARCHIVE_SCHEMA.md`
- `docs/ai/changes/RGDTPK-Q-v4-core/REVIEW_SUMMARY_SCHEMA.yaml`
- `docs/ai/changes/RGDTPK-Q-v4-core/schemas/evidence_manifest.schema.yaml`

Stable source and schema hashes:

- algorithm source sha256: `2dc2f950896ff3e60858b17bf3f1867667564ae773e0a71d6db8c0953143caed`
- failure source sha256: `df0d9d525a022f1851fe8021c70fea97d10408425e7b2670bf991858723ae14e`
- Base Spec sha256: `dfd6832c211af0928270cfbaa98dcf73e50cd37e6155534703b1217636038f6c`
- Plan sha256: `e78cfebc3cce75fbe632c1d0384f59eee7168a27c81d7326c1f02362584b26bc`
- Source map sha256: `c3fc89dd76d1bf68684ad5359f6e2c4accf28e8db87c0e725b36aba21a2362a2`
- review archive schema sha256: `53ed17e5416a0a98a3e058ad20c37191050e3ea649e79a97cdbf92dc05771bad`
- review summary schema sha256: `ca9a11d4e5511218222d1cd5b675223d3e3017a989cfb776795ac6ef1b352ec0`
- evidence manifest schema sha256: `70fd1c72382f5ab847e3eb2eb40f135abb628f46c2a1e5ebf006acdba81c8e0f`

Commit state:

- `unborn-master-no-commit`
- `git rev-parse --is-inside-work-tree` returned true.
- `git rev-parse --verify HEAD` returned `fatal: Needed a single revision`.
- Treat explicit file hashes and fresh command outputs as the evidence anchor.

## P5 Implementation Files to Inspect

- `geosolver-core/src/graph/hypergraph.rs`
- `geosolver-core/src/graph/influence.rs`
- `geosolver-core/src/graph/weighted_primal.rs`
- `geosolver-core/src/graph/separators.rs`
- `geosolver-core/src/graph/tree_decomposition.rs`
- `geosolver-core/src/graph/projection_dag.rs`
- `geosolver-core/src/graph/metrics.rs`
- `geosolver-core/src/graph/mod.rs`

## Required Behavior to Check

Use `REVIEWER_PROMPTS.md#P5`:

> Check every polynomial occurrence in hypergraph, algebraic-only weights, no-separator one-large-block path, authorization hashes, duplication certificates, and mismatch tests. Fail if DAG is decorative or execution can read arbitrary relations.

Also check Appendix A section 12 required functions:

- `build_relation_variable_hypergraph`
- `connected_components`
- `relation_variables`
- `variable_relations`
- `build_target_influence_graph`
- `build_weighted_primal_graph`
- `variable_weight`
- `edge_weight`
- `articulation_variable_candidates`
- `min_fill_separator_candidates`
- `score_separator`
- `build_target_rooted_decomposition`
- `build_target_projection_dag`
- `authorize_block_relations`
- `validate_projection_dag`
- `structural_metrics`
- `estimate_local_quotient_rank`
- `estimate_sparse_template_size`
- `estimate_coefficient_growth`

Specific pass/fail checks:

1. Hypergraph must record every variable occurrence from compressed relation polynomials.
2. Weights and separators must be algebraic graph/polynomial data only; no geometry labels, fixture ids, coordinates, expected answers, or fallback solver dispatch.
3. If no useful separator exists, target projection must create one large target block rather than an empty/decorative DAG.
4. `TargetProjectionDAG` blocks must be operationally authorization-bound: a block cannot read arbitrary compressed relations outside its authorization hash.
5. Authorization hash must bind at least block identity, parent/child structure, local/export variables, relation ids, and relation hashes.
6. Validation must reject authorization hash mismatch.
7. Validation must reject deletion/omission of a compressed relation.
8. Validation must reject relation duplication without a duplication certificate.
9. Validation must reject unknown relation ids and relations whose variables are outside the local block.
10. P5 must not claim planner/kernel/candidate-cover/exact-image/replay readiness.

## Evidence to Inspect

Evidence files:

- `docs/ai/changes/RGDTPK-Q-v4-core/evidence/P5/commands.txt`
  - sha256 `a7e7098b4ecba2222718bd2000385cf91720b2bfb2a7383805691153afa78b7a`
- `docs/ai/changes/RGDTPK-Q-v4-core/evidence/P5/command_outputs.txt`
  - sha256 `df6399c5f44cd0a07be49b0fc8b0d53f36b05268c18ddae396dc5b4b5082aa6d`
- `docs/ai/changes/RGDTPK-Q-v4-core/evidence/P5/static_scans.txt`
  - sha256 `cd15b69060a0149266d50425885fb95dfc2b7d1161374b69aabfb543535d6d6a`
- `docs/ai/changes/RGDTPK-Q-v4-core/evidence/P5/function_implementation_table.yaml`
  - sha256 `18db5570447fb4b454db030e2963db700b6ae3be02481bd9b9df723df9c1b984`
- `docs/ai/changes/RGDTPK-Q-v4-core/evidence/P5/notes.md`
  - sha256 `fad96d97aea7f53a88189f9b5701cf86c8a31ca319130f21d6cc1945a2ad4a48`

Commands claimed fresh after the last P5 code change:

- `$env:CARGO_INCREMENTAL='0'; cargo fmt --manifest-path geosolver-core\Cargo.toml -- --check`
  - exit code 0
- `$env:CARGO_INCREMENTAL='0'; cargo test --manifest-path geosolver-core\Cargo.toml graph -- --nocapture`
  - exit code 0
  - graph tests: 9 passed
- `$env:CARGO_INCREMENTAL='0'; cargo test --manifest-path geosolver-core\Cargo.toml -- --nocapture`
  - exit code 0
  - full crate tests: 97 passed
- forbidden marker / coordinate / geometry / randomness scan over graph and preprocess
  - exit code 1, no matches
- geometry/fixture/coordinate/CAS/final-status overclaim scan over graph
  - exit code 1, no matches
- positive required function scan over graph
  - exit code 0, matches all required P5 function names

Failure/fix loop to assess:

- Initial `graph::separators::tests::articulation_candidate_is_algebraic_incidence_based` failed.
- Root cause: the test oracle used `t*x - y`, which creates a triangle in the primal graph, so `x` is not an articulation variable.
- Targeted fix: use the pure algebraic incidence chain `t*x`, `x*y` and build compressed system directly from `CompressionState` to avoid P4 rewriting the graph oracle.
- Fresh graph and full crate tests passed after the fix.

## Required Reviewer Output Details

If PASS, explicitly state:

- reviewed R-IDs and `MECH-004`;
- files inspected;
- commands inspected;
- evidence inspected;
- algorithmic sufficiency judgment;
- semantic deletion challenge considered, including `relation_deletion_fails_validation`;
- tamper challenge considered, including `authorization_mismatch_fails_validation`;
- whether relation duplication without certificate is rejected;
- whether no-separator one-large-block path is real;
- whether `MECH-004` is closable;
- residual risks and claim ceiling after P5.

If FAIL, include:

- exact R-ID/MECH;
- code/evidence location;
- why this is algorithmic rather than paperwork only;
- minimal required fix.

Do not claim any status stronger than `PARTIAL_MECHANISM_READY:MECH-004` for P5.
