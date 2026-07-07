# P9 Source To Code Map

Phase: P9 TargetRelationSearchKernel production workhorse

Sources:

- Base Spec: BS-R093
- Mechanisms: MECH-01, MECH-03, MECH-07
- Reviewer prompt: RP-P9

Code anchors:

- `geosolver-core/src/kernels/target_relation_search.rs`
  - `admit_target_relation_search`: admits local and child-message relation inputs and records
    declared dense/sparse schedules.
  - `execute_target_relation_search`: executes only the declared direct/dense/sparse route.
  - `build_membership_matrix`: constructs coefficient comparison for `g - sum(q_i f_i) = 0`.
  - `reconstruct_and_verify_relation_from_builder`: treats modular nullspace output as candidates
    only and accepts only exact Q membership.
  - `verify_membership_exact`: exact identity oracle for successful candidates.
- `geosolver-core/src/planner/relation_schedule.rs`
  - `SupportDescriptor::DenseTotalDegree`
  - `SupportDescriptor::SparseFootprint`
  - `SupportDescriptor::SpecializedInterpolationFootprint`
  - dense/sparse schedule builders and specialized interpolation footprint descriptor helper.
- `geosolver-core/src/algebra/linear_solve.rs`
  - `solve_homogeneous_modular`
  - `ModularProofStatus::CandidateOnlyRequiresExactQCheck`
- `geosolver-core/src/verify/certificates.rs`
  - `TargetRelationSearchCertificate`
  - target relation hash helpers for variables, multipliers, and exact identity.
- `geosolver-core/src/verify/verify_message.rs`
  - `verify_target_relation_search_certificate`
  - exact membership replay and TRS field hash validation.

Claim boundary:

- This evidence supports P9 only.
- It does not claim full finite candidate-cover completion or exact-image equality.
