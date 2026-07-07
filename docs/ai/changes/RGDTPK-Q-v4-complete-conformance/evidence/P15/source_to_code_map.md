# P15 Source To Code Map

Phase: P15 - exact root isolation, Descartes/Vincent, candidate decode.

## BS-R120 - Squarefree Support And Exact Root Isolation

- `geosolver-core/src/roots/squarefree.rs`
  - `squarefree_support`
  - rejects zero support
  - delegates exact derivative/gcd squarefree computation to `squarefree_part_uni`
- `geosolver-core/src/algebra/real_root.rs`
  - `sturm_sequence`
  - `isolate_real_roots_sturm`
  - exact Cauchy bound
  - exact Sturm root count through rational sign variation
  - `isolate_real_roots_descartes`
  - `isolate_descartes_interval`
  - `descartes_variations_on_interval`
  - Descartes/Vincent path uses exact Mobius interval transform coefficients and does not call Sturm
- `geosolver-core/src/roots/isolate.rs`
  - dispatches `RootIsolationMethod::Sturm` and `RootIsolationMethod::Descartes`
  - binds each `RealRootRecord` to support hash and deterministic root index

## BS-R121 - Candidate Decode

- `geosolver-core/src/roots/decode.rs`
  - `decode_candidates`
  - `hash_target_candidate`
  - candidate hash binds target variable, support hash, root index, and rational interval endpoints
- `geosolver-core/src/roots/algebraic_number.rs`
  - algebraic root records bind support hash/root index/interval hash
  - refinement and comparison operate on exact rational isolating intervals
- `geosolver-core/src/algebra/sign.rs`
  - exact sign and Thom helpers remain available for semantic/exact-image code paths

## P15 Remediation Notes

- `isolate_real_roots_descartes` no longer aliases `isolate_real_roots_sturm`.
- `choose_nonroot_split` no longer gives up at a fixed `..=128` cap; it deterministically increases rational subdivision denominators until a non-root split is found.
- No float-only root isolation path is used in P15 files.
