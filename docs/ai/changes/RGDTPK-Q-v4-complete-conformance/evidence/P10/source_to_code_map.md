# P10 Source To Code Map

Phase: P10 SparseResultantProjectionKernel

Sources:

- Base Spec: BS-R094
- Reviewer prompt: RP-P10

Code anchors:

- `geosolver-core/src/algebra/resultant.rs`
  - `ResultantInput`, `ResultantTemplate`, and `MonomialSupport` represent input polynomials,
    eliminated/keep variables, support sets, matrix dimensions, and template hash.
  - `compute_resultant_relation` computes a primitive exact relation and records modular traces.
  - `verify_resultant_certificate` rebuilds the template, recomputes the exact relation/backend,
    checks exact verification hash, and replays modular trace hashes.
- `geosolver-core/src/kernels/sparse_resultant.rs`
  - `probe_sparse_resultant_plan` performs finite template/swell estimation.
  - `build_sparse_resultant_trace` executes declared pairwise resultant-chain templates.
  - `execute_sparse_resultant` verifies exported-variable containment and emits
    `SparseResultantMatrix` / `CandidateCoverStrong`.
- `geosolver-core/src/verify/verify_message.rs`
  - `verify_sparse_resultant_payload` replays resultant certificates and exact relations.

Claim boundary:

- Pairwise resultant chains are represented as the implemented declared template kind.
- This phase does not claim a complete general sparse resultant backend beyond the represented
  template route.
