# P5 Source-To-Code Map

Status: implementation evidence for Phase 5.

Relevant R-IDs:

- BS-R060 -> `preprocess/compression.rs`
- BS-R061 -> `preprocess/definitional.rs`, `preprocess/linear_affine.rs`,
  `preprocess/binomial.rs`, `preprocess/saturation.rs`, `preprocess/independent.rs`

Mapping:

- `pre_kernel_compress` runs the source order exactly: definitional elimination, linear affine
  elimination, binomial simplification, explicit saturation, target-independent component marking,
  then coefficient-height/monomial trace finalization.
- `CompressionState` carries variables, relations, semantic encodings, substitutions, guards,
  rational affine certificates, saturations, feasibility obligations, diagnostics, and trace.
- Definitional elimination skips the target variable and records substitution hashes.
- Linear affine elimination only selects constant nonzero pivots or nonconstant denominators with
  recorded NonZero witness semantics; unsafe nonconstant pivots remain in the system and emit an
  `UnsafeAffinePivotRejected` diagnostic.
- Binomial simplification only primitive-normalizes and deduplicates reversible one/two-term
  relations. It does not factor-split or select union branches. Dedup preserves semantic
  provenance by retaining every relation in a duplicate group when any relation ID in that group is
  referenced by semantic encodings.
- Saturation records only semantic `RealConstraintKind::NonZero` encodings matching an
  `A*s - 1 = 0` witness relation and slack variable.
- Target-independent components are removed from candidate-cover construction only as
  disconnected non-target components and are preserved as feasibility obligations.
