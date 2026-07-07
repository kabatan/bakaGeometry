Purpose: P17 source-to-code map
Status: evidence, non-authoritative
Authority: BASE_SPEC.md, SOURCE_MAP.md, and the original v4 source remain authoritative

# P17 Source-to-Code Map

## Scope

Phase: P17 - Orchestrator, pipeline, result finalization, cost trace

Relevant R-IDs:
- BS-R130: result status and output
- BS-R131: algebraic cost compression trace
- BS-R140: top-level solver pipeline
- MECH-03: declared ladder and no hidden fallback
- MECH-04: Projection DAG and composition
- MECH-05: exact root isolation and candidate binding
- MECH-07: algebraic cost trace

Source sections:
- 4.1 top-level pipeline
- 4.2 invariants
- 25.2 projection message verification
- 25.3 global support verification
- 26 root isolation and candidate decode
- 28 result and diagnostics
- 29 solver orchestrator
- 30 cost trace
- 33 completion conditions

## Code Map

| Requirement | Code evidence | Notes |
|---|---|---|
| Solver options expose finite candidate-cover mode, exact-image mode, limits, root isolation method, certificate level | `geosolver-core/src/solver/options.rs::SolverOptions` | `finite_candidate_cover_mode` defaults true. `exact_image_mode` remains exposed but guarded by orchestrator. Resource and root/certificate options are present. |
| Solver options are certificate-bound | `geosolver-core/src/verify/run_certificate.rs::hash_solver_options` | Hash includes `finite_candidate_cover_mode`, `exact_image_mode`, resource limits, root isolation method, certificate level, and kernel priority. |
| Pipeline exposes source stage functions | `geosolver-core/src/solver/pipeline.rs::step_validate` through `step_core_certificate` | P17 audit checks required step names and function existence. |
| Orchestrator calls stages in source order | `geosolver-core/src/solver/orchestrator.rs::solve_with_context` | Order is validate, canonicalize, compress, graphs, DAG, plan, execute, verify messages, compose, support, verify support, roots, certificate, cost trace, finalize. Exact-image scope guard is only applied after candidate-cover evidence is built. |
| Message verification before composition | `geosolver-core/src/solver/orchestrator.rs::solve_with_context`; `geosolver-core/src/solver/pipeline.rs::step_verify_messages` | `step_verify_messages` is called before `step_compose`. Failure finalizes with cost trace and no root decode. |
| Support verification before roots | `geosolver-core/src/solver/orchestrator.rs::solve_with_context` | `verify_global_support` is executed after `step_support` and before `step_roots`. |
| Failure results preserve cost evidence | `geosolver-core/src/solver/orchestrator.rs::finalize_pipeline_error`; `geosolver-core/src/solver/pipeline.rs::step_failure_cost_trace`; `geosolver-core/src/result/output.rs` | Completed stage data and failure diagnostics flow into `TargetSolveResult`. |
| Cost trace records source fields | `geosolver-core/src/result/cost_trace.rs::GlobalCostTrace`; `geosolver-core/src/solver/pipeline.rs::step_cost_trace` | Includes total n/m/d/s/h, max block width, max separator width, block traces, composition/verification traces, final support degree, certificate size. |
| Final certificate binds in-scope fields | `geosolver-core/src/solver/pipeline.rs::step_core_certificate`; `geosolver-core/src/verify/run_certificate.rs::CoreRunCertificate` | Certificate binds input/options/canonical/compression/hypergraph/DAG/plan/message/support/squarefree/root/candidate/global support certificate/final DAG replay evidence. |
| Exact-image request cannot produce success status | `geosolver-core/src/solver/orchestrator.rs::exact_image_out_of_scope_diagnostic`; status branch in success path | Exact-image mode returns `CertificateDesignGap` with `ExactImageOutOfScope` diagnostic and preserves unfiltered candidate-cover artifacts. |

## Static Audit Binding

`geosolver-core/scripts/audit_v4_conformance.py --phase P17 --strict` checks:
- solver option field presence,
- pipeline stage function presence,
- orchestrator stage order,
- support verification before roots,
- failure path cost trace construction,
- cost trace fields,
- run-certificate binding of finite candidate-cover mode, exact-image mode, support/root/candidate/final DAG evidence.
