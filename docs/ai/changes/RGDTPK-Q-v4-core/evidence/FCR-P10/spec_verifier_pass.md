# FCR-P10 Spec Verifier Result

Status: PASS.

Verifier conclusion:
- No remaining FCR-P10 blockers.
- A1-A13 acceptance coverage runs through `api::solve_target` for public cases.
- Named-kernel cases assert the executed `ProjectionMessage.kernel_kind`.
- Shared support-producing assertions cover nonconstant support, squarefree support, projection messages, run certificate, candidate hash checks, replay acceptance, cost trace alignment, and verification relation checks.
- Anti-hack coefficient scaling is applied in the shared `problem()` path and asserts every nonzero relation changes.
- A3/A5 composition-removal checks are present.
- A13 asserts bounded failure plus nonempty matrix-dimension cost trace, with public `FiniteResourceFailure` trace retention.
- Advanced kernels are production-reachable in kernel registry and planner admission, and replay uses production compression.
- Final recheck after quality fix confirmed FCR-P10 remains spec-satisfied.
- Failure finalization now preserves the public problem target, TargetRelationSearch finite-resource errors carry `Some(target)`, and A13 asserts retained target plus TargetRelationSearch cost trace identity.
- Independent public-path A13 inspection returned `FiniteResourceFailure`, `target=VariableId(59)`, `kernel=TargetRelationSearch`, `rows=Some(7)`, and `cols=Some(5)`.

Verifier rerun evidence:
- `cargo test --manifest-path geosolver-core/Cargo.toml --test fcr_p10_acceptance_suite -- --nocapture`: PASS, 13/13.
- `cargo test --manifest-path geosolver-core/Cargo.toml --test fcr_p10_acceptance_suite fcr_p10_a13_public_resource_bounded_hard_case_has_spec_status -- --nocapture`: PASS.
- `cargo test --manifest-path geosolver-core/Cargo.toml verify::replay::tests -- --nocapture`: PASS, 16/16.
- `cargo test --manifest-path geosolver-core/Cargo.toml --test fcr_p4_pure_planning fcr_plan_sparse_resultant_does_not_construct_output_relation -- --nocapture`: PASS.
- `cargo check --manifest-path geosolver-core/Cargo.toml`: PASS.
- `git diff --check`: PASS with CRLF warnings only.
- Advanced-kernel cfg-test quarantine scan: no matches.
- Anti-hack/stub scans: no production fixture/expected-answer dispatch found.
