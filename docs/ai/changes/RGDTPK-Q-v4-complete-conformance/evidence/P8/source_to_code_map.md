# P8 Source-To-Code Map

Status: implementation evidence for Phase 8.

Relevant R-IDs:

- BS-R091 -> `kernels/target_univariate.rs`, `verify/certificates.rs`,
  `verify/verify_message.rs`
- BS-R092 -> `kernels/linear_affine.rs`, `verify/certificates.rs`,
  `verify/verify_message.rs`

Mapping:

- `admit_target_univariate_with_messages` scans local block relations and child message relations
  for nonzero target-only relations.
- `execute_target_univariate` converts all target-only inputs to univariate support, clears
  denominators, applies primitive/squarefree-compatible support construction, and emits
  `MessageRepresentation::PrincipalSupport`.
- TargetUnivariate certificates use `CertificateRoute::SourceMembershipCertificate` and bind source
  relation IDs/hashes and child message hashes.
- `find_triangular_affine_order` and `choose_safe_affine_pivot` select triangular affine eliminations
  only when pivots are constant nonzero or have recorded nonzero denominator guards.
- `plan_linear_affine` stores affine elimination order, pivot hashes, and denominator guard hashes.
- `execute_linear_affine` rechecks authorization/source hashes/pivot hashes/guards, performs exact
  substitutions, and emits a guarded affine projection certificate.
- `verify_message.rs` rejects exported relations containing non-exported variables, mismatched
  certificate export sets, missing nonconstant affine guard hashes, and guarded affine replay
  mismatches.
