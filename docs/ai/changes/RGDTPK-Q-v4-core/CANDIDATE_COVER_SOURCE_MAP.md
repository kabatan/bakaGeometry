# Candidate-Cover Source Map

Status: active source map for `v4_candidate_cover_completion_pack_v1`.

## Candidate-Cover Required Surface

| Requirement area | Implementation files | Status |
| --- | --- | --- |
| Public algebraic input pipeline | `geosolver-core/src/api.rs`, `solver/orchestrator.rs`, `solver/pipeline.rs` | Public `solve_target` validates, canonicalizes, compresses, builds graphs/DAG, plans, executes kernels, composes messages, verifies support, isolates roots, decodes candidates, and builds replay evidence. |
| DAG/messages/replay binding | `graph/*`, `compose/message.rs`, `verify/replay.rs`, `verify/run_certificate.rs` | Candidate-cover replay recomputes input, compression, DAG, messages, support certificate, roots, and decoded candidates. |
| Support containment certificate | `verify/verify_support.rs` | Route A verifies target-only root relation product. Route B creates `ComposedIdealMembershipSupportCertificate` with relation hashes, multipliers, exact identity hash, and certificate hash for `S(T) = sum q_i r_i`. |
| Final support/nonfinite gate | `compose/final_support.rs`, `solver/orchestrator.rs` | Finite candidate cover uses support verification. Nonfinite success requires positive nonfinite certificate and public replay binding. |
| Root isolation/decode | `roots/isolate.rs`, `roots/squarefree.rs`, `roots/decode.rs`, `solver/pipeline.rs` | Exact squarefree support, real root isolation, and decoded candidates are replay-bound. |
| Candidate-cover semantics | `solver/orchestrator.rs`, `compose/final_support.rs` | Default candidate-cover mode does not invoke exact-image filtering and emits `ExactImageFilteringNotRequested` plus `CandidateCoverMayContainSpuriousRoots`. |
| Generic kernels | `kernels/target_relation_search.rs`, `kernels/universal_elimination.rs`, `kernels/action_krylov.rs`, `kernels/sparse_resultant.rs`, `kernels/regular_chain_projection.rs`, `kernels/norm_trace_projection.rs`, `kernels/specialization_interpolation.rs` | Public support-producing cases and existing FCR/P15 suites exercise generic routes; limitations do not define the candidate-cover claim boundary. |
| Anti-fallback invariants | `verify/run_certificate.rs`, `verify/replay.rs` | Runtime flags stay run-local; static scans are recorded in closure evidence for no expected-answer/problem-id/QE/CAD/RUR/coordinate fallback. |

## Later Or Separate Scope

| Area | Classification |
| --- | --- |
| Exact target image equality | Later/separate closure. Candidate-cover allows extra roots. |
| Full supplied-v4 source fidelity | Not claimed by this repair. |
| Production F4/F5 sparse linear algebra | Not claimed. Existing Groebner-backed batch helpers are non-production/test-only and production dispatch rejects them. |
| Geometry DSL, diagrams, natural language | Out of scope. |

## Source-Fidelity Boundary

This source map supports only the candidate-cover layer claim. It must not be cited as proof of
exact-image completion, full supplied-v4 source fidelity, benchmark superiority, or universal
finite-system completeness.
