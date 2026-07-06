# P14 Review Packet - Full Pipeline Integration

Review target: Plan P14 only.

## Scope

Source anchors:

- `BASE_SPEC.md` RGQ-008, RGQ-011, RGQ-030, RGQ-031, RGQ-034, RGQ-049, Appendix A sections 28-30.
- `PLAN.md` P14 implementation tasks.
- `REVIEWER_PROMPTS.md#P14`.
- Guardian Runtime Contract from `AGENTS.md`.

Changed implementation files:

- `geosolver-core/src/compose/compose.rs`
- `geosolver-core/src/algebra/groebner.rs`
- `geosolver-core/src/algebra/resultant.rs`
- `geosolver-core/src/kernels/action_krylov.rs`
- `geosolver-core/src/kernels/linear_affine.rs`
- `geosolver-core/src/kernels/norm_trace_projection.rs`
- `geosolver-core/src/kernels/regular_chain_projection.rs`
- `geosolver-core/src/kernels/sparse_resultant.rs`
- `geosolver-core/src/kernels/specialization_interpolation.rs`
- `geosolver-core/src/kernels/target_relation_search.rs`
- `geosolver-core/src/kernels/target_univariate.rs`
- `geosolver-core/src/kernels/universal_elimination.rs`
- `geosolver-core/src/preprocess/compression.rs`
- `geosolver-core/src/result/diagnostics.rs`
- `geosolver-core/src/result/cost_trace.rs`
- `geosolver-core/src/result/output.rs`
- `geosolver-core/src/result/status.rs`
- `geosolver-core/src/solver/orchestrator.rs`
- `geosolver-core/src/solver/pipeline.rs`
- `geosolver-core/src/types/polynomial.rs`
- `geosolver-core/tests/p14_full_pipeline_integration.rs`

P14 relies on already-implemented pipeline files and verifies them:

- `geosolver-core/src/api.rs`
- `geosolver-core/src/solver/options.rs`
- `geosolver-core/src/solver/orchestrator.rs`
- `geosolver-core/src/solver/pipeline.rs`
- `geosolver-core/src/result/output.rs`
- `geosolver-core/src/result/cost_trace.rs`

Changed evidence/docs:

- `docs/ai/ACTIVE_CONTEXT.md`
- `docs/ai/changes/RGDTPK-Q-v4-core/FULL_CORE_ACCEPTANCE_RESULTS.md`
- `docs/ai/changes/RGDTPK-Q-v4-core/FULL_CORE_SOURCE_SPEC_COMPLIANCE_MAP.md`
- `docs/ai/changes/RGDTPK-Q-v4-core/CLOSURE.md`
- `docs/ai/changes/RGDTPK-Q-v4-core/evidence/P14/*`

## Implementation Summary

P14 adds a dedicated public integration suite and manual Appendix 29 stage trace. The public suite
covers candidate-cover success, exact-image success, exact-image empty, certified nonfinite,
bounded hard/resource failure, and invalid input. Result-field checks cover status, target, support,
squarefree support, roots, decoded candidates, projection messages, core certificate,
exact-image certificate, diagnostics, and global cost trace as applicable.

Reviewer remediation removed the empty compressed-relation early shortcut in `solve_with_context`.
The empty-relation nonfinite branch now runs planning, execution, message verification, and
composition before final support/nonfinite finalization. Composition admits an empty
`ComposedProjection` only when every DAG block has no relation and no projection messages are
present.

Reviewer remediation also corrected production kernel coefficient-height cost trace fields.
Successful production kernel traces now derive `coefficient_height_before_bits` from input
relations and `coefficient_height_after_bits` from emitted output relations. TargetRelationSearch
and Universal resource failures now carry observed coefficient-height evidence. Additional
remediation fixed low-level Groebner pair-limit and sparse resultant template finite-resource
errors so they carry observed coefficient-height evidence, and public post-compression failure
finalization preserves known global cost context while enriching finite-resource block traces from
the authorized DAG instead of fabricating zero-filled traces or verification counts.

Additional reviewer remediation added Appendix 30 final trace parameters:
`GlobalCostTrace::final_support_degree` for final support degree `delta` and
`GlobalCostTrace::certificate_size` for certificate size `kappa` as a deterministic core-certificate
component count. Finite success paths populate both; nonfinite/failure paths leave them absent.

The temporary public pipeline scaffold has been removed from the result status/diagnostic path.
A static scan for `TemporaryPipelineNotConnected`, `temporary_pipeline_not_connected`, and
`NotYetImplemented` has no matches in `geosolver-core/src` or `geosolver-core/tests`.

## Verification

Commands run and passing:

```text
cargo fmt --manifest-path geosolver-core/Cargo.toml -- --check
cargo test --manifest-path geosolver-core/Cargo.toml --test p14_full_pipeline_integration -- --nocapture
rg -n "TemporaryPipelineNotConnected|temporary_pipeline_not_connected|NotYetImplemented" geosolver-core/src geosolver-core/tests
cargo clippy --manifest-path geosolver-core/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path geosolver-core/Cargo.toml --all-targets --all-features -- --nocapture
git diff --check
```

P14 suite cases:

- ordered Appendix 29 stage trace matches public `solve_target`;
- empty compressed-relation nonfinite branch still runs Appendix 29 plan/execute/verify/compose
  before support finalization;
- candidate-cover success has all public result fields and cost trace;
- Groebner pair-limit finite-resource errors carry observed coefficient height;
- sparse resultant template finite-resource errors carry observed coefficient height;
- exact-image success is not candidate-cover;
- exact-image empty keeps support but no real candidates;
- certified nonfinite finalizes without panic;
- bounded hard/resource case carries global and block resource trace without a synthetic
  verification count;
- invalid input maps to result status without panic.

Initial P14 reviewers found fixable blockers for dummy coefficient-height trace fields, missing
final support degree/certificate size cost-trace fields, and an empty-relation nonfinite branch that
bypassed part of Appendix 29. This packet includes remediation for all blockers and updated tests.

## Claim Boundary

Before reviewer pass, the approved claim remains:

```text
CANDIDATE_COVER_CORE_READY
```

If P14 passes, it may close the P14 full-pipeline-integration checkpoint and support MECH-002 full
stage-trace evidence. This review must not approve P15/P16, final public replay-bound nonfinite
readiness, benchmark readiness, `EXACT_IMAGE_CORE_READY`,
`RGDTPK_Q_V4_ACCEPTANCE_COMPLETE`, `SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC`, or any R-ID as
`VERIFIED`.
