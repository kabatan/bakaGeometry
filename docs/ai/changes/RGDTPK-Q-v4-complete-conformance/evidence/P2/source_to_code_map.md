# P2 Source-To-Code Map

Status: implementation evidence for Phase 2.

Relevant R-IDs:

- BS-R001 -> `problem/input.rs`, `problem/validate.rs`
- BS-R040 -> `problem/semantic.rs`, `problem/canonicalize.rs`, `preprocess/compression.rs`
- BS-R041 -> `problem/canonicalize.rs`
- BS-R042 -> `problem/context.rs`

Mapping:

- `RationalTargetProblem` source fields are implemented in `problem/input.rs`.
- `make_problem`, `make_problem_with_roles`, and `hash_problem_input` bind target, variables,
  equations, semantic encodings, and variable roles into the input hash.
- `RealConstraintKind`, `RealConstraintEncoding`, `register_slack_encoding`,
  `semantic_relations`, and `verify_semantic_references` are production APIs in
  `problem/semantic.rs`.
- `validate_input` rejects undeclared target, undeclared polynomial variables, non-normalized zero
  terms, and invalid semantic references, while accepting variable roles and slack/branch
  encodings when references are valid.
- `canonicalize_system` performs target-aware variable ordering, primitive denominator clearing,
  zero relation removal with diagnostics, nonzero constant contradiction failure, semantic
  consistency re-check after zero relation removal, and semantic-hash-bound canonical hashing.
- `preprocess/compression.rs` preserves semantic encodings and includes semantic hashes in
  compressed-state hashes.
- `SolverContext`, `ResourceMeter`, `ActiveRouteBudget`, and cooperative resource checks are in
  `problem/context.rs`.
