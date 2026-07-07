# P4 Source-To-Code Map

Status: implementation evidence for Phase 4.

Relevant R-IDs:

- BS-R054 -> `algebra/groebner.rs`, `algebra/f4.rs`, `algebra/normal_form.rs`
- BS-R055 -> `algebra/elimination.rs`
- BS-R096 -> `planner/kernel_plan.rs`, `kernels/universal_elimination.rs`,
  `kernels/target_relation_search.rs`, `verify/verify_message.rs`

Mapping:

- Local Groebner elimination with exact membership certificates remains in `groebner.rs`.
- Production F4 module is exported from `algebra/mod.rs` without `#[cfg(test)]`.
- `f4_reduce_batch` constructs symbolic preprocessing rows, builds an F4 matrix, records a
  deterministic modular matrix trace, computes returned reductions by exact row reduction over
  those matrix rows, and verifies every remainder certificate by exact Q membership.
- `f4_elimination_local` runs an F4-style batched S-pair loop, adds only exact certificate-verified
  remainders, and returns `LocalEliminationResult`.
- `EliminationStrategy` includes `EliminationGroebnerLocal`, `F4EliminationLocal`,
  `TargetRelationSearchEscalated`, `ResultantIfSquareOrOverdetermined`, and
  `SpecializeProjectInterpolateVerify`.
- `eliminate_to_keep_variables` dispatches only the declared strategy and validates Q[keep]
  generators plus membership certificates.
- `UniversalStrategy` now contains exactly the source section 20.4 strategies:
  `EliminationGroebnerLocal`, `F4EliminationLocal`, `TargetRelationSearchEscalated`,
  `ResultantIfSquareOrOverdetermined`, and `SpecializeProjectInterpolateVerify`.
- `UniversalTargetEliminationKernel` no longer plans or executes ActionKrylov, RegularChain, or
  NormTrace as Universal internal stages.
- `TargetRelationSearchEscalated` can wrap an already-exported source relation as an exact
  membership certificate, avoiding high-footprint dense/sparse matrix materialization for that
  cheap case.
