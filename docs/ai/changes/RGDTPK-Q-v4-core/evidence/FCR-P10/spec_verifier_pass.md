# FCR-P10 Spec Verifier Result

Status: PASS.

Verifier conclusion:
- No remaining FCR-P10 blockers.
- A1-A11 support-producing acceptance coverage plus B1 bounded failure semantics run through `api::solve_target` for public cases.
- Named-kernel cases assert the executed `ProjectionMessage.kernel_kind`.
- Shared support-producing assertions cover nonconstant support, squarefree support, projection messages, run certificate, candidate hash checks, replay acceptance, cost trace alignment, and verification relation checks.
- Anti-hack coefficient scaling is applied in the shared `problem()` path and asserts every nonzero relation changes.
- A3/A5 composition-removal checks are present.
- B1 asserts bounded failure plus nonempty matrix-dimension cost trace, with public `FiniteResourceFailure` trace retention.
- Advanced kernels are production-reachable in kernel registry and planner admission, and replay uses production compression.
- Final recheck after quality fix confirmed FCR-P10 remains spec-satisfied.
- Failure finalization now preserves the public problem target, TargetRelationSearch finite-resource errors carry `Some(target)`, and B1 asserts retained target plus TargetRelationSearch cost trace identity.
- Independent public-path B1 inspection returned `FiniteResourceFailure`, `target=VariableId(59)`, `kernel=TargetRelationSearch`, `rows=Some(7)`, and `cols=Some(5)`.

Verifier rerun evidence:
- Pre-P11 correction moves certified nonfinite out of P10 and into the final nonfinite gate.
- B1 asserts bounded failure plus nonempty matrix-dimension cost trace, with public `FiniteResourceFailure` trace retention.
- `cargo test --manifest-path geosolver-core/Cargo.toml --test fcr_p10_acceptance_suite -- --nocapture`: PASS, 12/12 after pre-P11 correction.
- `cargo test --manifest-path geosolver-core/Cargo.toml --test fcr_final_nonfinite_semantics -- --nocapture`: PASS, 2/2.
- `cargo test --manifest-path geosolver-core/Cargo.toml --test fcr_p10_acceptance_suite fcr_p10_b1_public_resource_bounded_hard_case_has_spec_status -- --nocapture`: PASS.
- `cargo test --manifest-path geosolver-core/Cargo.toml verify::replay::tests -- --nocapture`: PASS, 16/16.
- `cargo test --manifest-path geosolver-core/Cargo.toml --test fcr_p4_pure_planning fcr_plan_sparse_resultant_does_not_construct_output_relation -- --nocapture`: PASS.
- `cargo check --manifest-path geosolver-core/Cargo.toml`: PASS.

Pre-P11 correction recheck:
- Initial spec_verifier result was `FAIL_FIXABLE` because two P10 evidence lines still used a stale
  red-team phase label.
- After updating those lines to `FCR-P11 red-team/final-nonfinite gate` and reserving FCR-P12 for
  final closure, the spec_verifier returned `PASS`.
- The stale-label scan only returns the explicitly superseded historical prompt heading.
- No P11/P12 closure, readiness, exact-image, source-fidelity, full acceptance, or R-ID verification
  is granted by this recheck.
- `git diff --check`: PASS with CRLF warnings only.
- Advanced-kernel cfg-test quarantine scan: no matches.
- Anti-hack/stub scans: no production fixture/expected-answer dispatch found.
