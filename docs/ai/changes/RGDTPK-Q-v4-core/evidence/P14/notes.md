# P14 Notes - Full Pipeline Integration

Status: PASS after spec, boundary, and quality reviewer re-checks.

P14 adds a dedicated public integration suite plus a manual Appendix 29 stage trace.

The stage trace executes:

1. `step_validate`
2. `step_canonicalize`
3. `step_compress`
4. `step_build_graphs`
5. `step_build_dag`
6. `step_plan`
7. `step_execute`
8. `step_verify_messages`
9. `step_compose`
10. `step_support`
11. `step_roots`
12. `step_core_certificate`
13. `step_cost_trace`

The trace compares support, squarefree support, roots, decoded candidates, projection messages,
cost trace, and run-certificate hash against `api::solve_target` for the same input.

Reviewer remediation added a second stage trace for the empty compressed-relation nonfinite branch.
That branch now runs `step_plan`, `step_execute`, `step_verify_messages`, and `step_compose` before
`step_support` finalizes `CertifiedNonFiniteTargetImage`; `solve_with_context` no longer has an
early nonfinite shortcut before planning/execution/composition.

P14 public suite coverage:

- candidate-cover success has populated support, squarefree support, exact roots, decoded candidates,
  projection messages, core certificate, replay acceptance, and non-dummy cost trace fields;
- empty-relation nonfinite still runs the Appendix 29 plan/execute/verify/compose stages before
  support finalization;
- exact-image success returns `CertifiedExactTargetImage`, not candidate-cover, and carries the
  exact-image certificate only after finite classification;
- exact empty returns `CertifiedEmptyRealTargetImage` with finite support retained and no roots or
  candidates;
- certified nonfinite is finalized as `TargetSolveResult` without panic and without a core run
  certificate over finite support;
- bounded hard/resource case returns an allowed failure status and retains global and block resource
  trace evidence without fabricated verification counts;
- invalid input maps to `InvalidInput` without panic and with empty result payloads.

P14 also removes the temporary public pipeline scaffold enum/diagnostic path. The static scan for
`TemporaryPipelineNotConnected`, `temporary_pipeline_not_connected`, and `NotYetImplemented` has no
matches under `geosolver-core/src` or `geosolver-core/tests`.

P14 reviewer remediation also corrected production kernel cost traces so coefficient-height fields
record coefficient height, not placeholder zero or monomial count:

- `TargetRelationSearch`, `TargetUnivariate`, `LinearAffine`, `SparseResultantProjection`,
  `TargetActionKrylov`, `UniversalTargetElimination`, `RegularChainProjection`,
  `NormTraceProjection`, and `SpecializationInterpolation` now populate success trace
  coefficient-height fields from input and output polynomials.
- `TargetRelationSearch`, `UniversalTargetElimination`, low-level Groebner pair limits, and sparse
  resultant template resource failures now carry observed `coefficient_height_bits`.
- Public failure finalization preserves the known post-compression global cost context and enriches
  finite-resource failure block traces from the DAG instead of synthesizing zero-filled traces.
- P14 tests assert nonzero coefficient-height fields for success and bounded-resource traces where
  coefficient data exists.
- P14 tests also directly cover Groebner and SparseResultant finite-resource coefficient-height
  errors, and the public bounded-resource test asserts retained global trace fields plus
  `checked_relation_count == 0` before verification has run.

Additional P14 spec remediation added Appendix 30 final trace parameters:

- `GlobalCostTrace::final_support_degree` records final support degree `delta` on finite success
  paths and is absent for nonfinite/failure paths.
- `GlobalCostTrace::certificate_size` records certificate size `kappa` as a deterministic
  core-certificate component count on finite success paths and is absent when no core certificate is
  present.
- P14 tests assert both fields on public finite success traces and assert absence on the nonfinite
  stage trace.

P14 does not claim P15 acceptance stress, P16 final closure, source fidelity, benchmark readiness,
final replay-bound exact-image readiness, final public replay-bound nonfinite readiness, or any R-ID
as `VERIFIED`.
