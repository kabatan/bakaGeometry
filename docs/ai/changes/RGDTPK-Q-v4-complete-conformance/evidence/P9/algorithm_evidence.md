# P9 Algorithm Evidence

Implemented/verified behavior:

- TargetRelationSearch gathers local block relations and child projection-message relations through
  `collect_relation_search_inputs`.
- Admission is possible when either local or child relations exist, and the plan binds source
  relation hashes plus child message hashes.
- Dense and sparse relation-search schedules construct export support, multiplier supports, row
  monomials, and membership matrix dimensions deterministically.
- `SupportDescriptor` declares DenseTotalDegree, SparseFootprint, and
  SpecializedInterpolationFootprint strategies.
- Modular nullspace output is labelled candidate-only and must pass exact Q membership before any
  `ProjectionMessage` is returned.
- Successful TRS messages carry `TargetRelationSearchCertificate` fields for exported/eliminated
  variable hashes, support hashes, membership matrix hash, primes used, rational reconstruction
  hash, relation hash, multipliers hash, and exact identity hash.
- `verify_projection_message` replays membership and recomputes/binds source relation IDs, export
  support hash, multiplier support hashes, membership matrix hash, modular prime trace, accepted
  candidate vector, rational reconstruction hash, multipliers hash, and exact identity hash.
- AlgorithmicHardCase for exhausted schedules includes accumulated matrix trace hash.

Regression coverage:

- `p9_target_relation_search_uses_child_message_relations`
- `p9_target_relation_search_certificate_fields_reject_tamper`
- `p9_target_relation_search_rejects_source_matrix_reconstruction_and_prime_tamper`
- `p9_specialized_interpolation_footprint_descriptor_is_declared`
- Existing TRS dense/sparse, exact membership, exhaustion, and sparse-cost-prohibited tests.

Static audit:

- `audit_v4_conformance.py --phase P9 --strict` checks P9 file/symbol presence and markers for
  child-message inputs, candidate-only modular status, exact Q verification, certificate fields,
  support strategies, and matrix-trace hard cases.
