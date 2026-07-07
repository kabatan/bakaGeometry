# Source Map: R-GDTPK-Q / ACCTP-Q v4 有限 Candidate-Cover 準拠修正

Original Source:
- Path: `docs/ai/sources/geosolver_core_r_gdtpk_q_algorithm_spec_v4.md`
- Blob sha: `ef108f0dc95880d2e3030c96872b9073be995274`

This file maps source sections to Base Spec R-IDs for the finite candidate-cover repair. It is an index for review. The Base Spec body and the original source are the normative authorities within the scoped candidate-cover layer. Exact-image-only sections are marked `OUT_OF_SCOPE`.

| Source Section | Source Topic | Base Spec R-IDs | Implementation Area |
|---|---|---|---|
| 0 | self-contained fixed implementation spec | BS-R000 | all files |
| 1.1-1.3 | research strength, target-direct, genericity | BS-R000, BS-R001, BS-R010, BS-R150 | solver, planner, kernels |
| 2.1 | input Q-polynomial system | BS-R001, BS-R040, BS-R041 | problem/* |
| 2.2 | S(T), squarefree, root isolation | BS-R002, BS-R111, BS-R120, BS-R121 | compose/final_support.rs, roots/* |
| 2.3 | candidate cover vs exact image | BS-R002, BS-R003, BS-R122 | result/*; exact-image classification OUT_OF_SCOPE |
| 2.4 | slack/guard encodings | BS-R003, BS-R040, BS-R122 | problem/semantic.rs; exact-image classification OUT_OF_SCOPE |
| 3.1 | prohibited coordinate/RUR/QE/CAD/geometry fallback | BS-R010, BS-R150, section 17 forbidden simplifications | all production paths |
| 3.2 | allowed heavy local algebra | BS-R054-BS-R056, BS-R093-BS-R099 | algebra/*, kernels/* |
| 3.3 | failure statuses | BS-R011, BS-R130 | result/status.rs, result/output.rs |
| 4.1 | top-level pipeline | BS-R140 | solver/pipeline.rs, solver/orchestrator.rs |
| 4.2 | invariants I1-I10 | BS-R010 | run certificate and all stages |
| 5.1-5.3 | IDs, RationalQ, Monomial/Polynomial | BS-R030, BS-R031 | types/* |
| 5.4 | RationalTargetProblem and semantics | BS-R001, BS-R040 | problem/input.rs, problem/semantic.rs |
| 5.5 | CanonicalSystemQ | BS-R041 | problem/canonicalize.rs |
| 5.6 | ProjectionMessage | BS-R090, BS-R110 | compose/message.rs, kernels/* |
| 5.7 | TargetSolveResult and SolverStatus | BS-R011, BS-R130 | result/* |
| 6 | folder layout | BS-R020 | geosolver-core/src/** |
| 7.1 | lib.rs | BS-R020 | src/lib.rs |
| 7.2 | api.rs | BS-R130, BS-R140 | src/api.rs |
| 8.1-8.7 | types functions | BS-R030-BS-R032 | types/* |
| 9.1-9.5 | problem functions | BS-R040-BS-R042 | problem/* |
| 10.1-10.8 | algebra foundations | BS-R050-BS-R053 | algebra/monomial_order.rs through normal_form.rs |
| 10.9 | Groebner | BS-R054 | algebra/groebner.rs |
| 10.10 | F4 | BS-R054 | algebra/f4.rs |
| 10.11 | elimination dispatcher | BS-R055 | algebra/elimination.rs |
| 10.12 | resultant | BS-R056, BS-R094 | algebra/resultant.rs, kernels/sparse_resultant.rs |
| 10.13 | interpolation | BS-R056, BS-R099 | algebra/interpolation.rs |
| 10.14 | quotient handle | BS-R056, BS-R095 | algebra/quotient.rs |
| 10.15 | Krylov | BS-R056, BS-R095 | algebra/krylov.rs |
| 10.16 | regular chain | BS-R056, BS-R097 | algebra/regular_chain.rs |
| 10.17 | norm/trace | BS-R056, BS-R098 | algebra/norm_trace.rs |
| 10.18 | real roots | BS-R120 | algebra/real_root.rs |
| 10.19 | sign | BS-R122 | OUT_OF_SCOPE unless used by candidate-cover code |
| 11.1-11.6 | preprocessing | BS-R060, BS-R061 | preprocess/* |
| 12.1-12.7 | graph and DAG | BS-R070, BS-R071 | graph/* |
| 13.1-13.6 | planner | BS-R080-BS-R082 | planner/* |
| 14.1-14.2 | kernel trait and registry | BS-R090 | kernels/traits.rs, kernels/mod.rs |
| 15 | TargetUnivariate | BS-R091 | kernels/target_univariate.rs |
| 16 | LinearAffine | BS-R092 | kernels/linear_affine.rs |
| 17 | TargetRelationSearch | BS-R093 | kernels/target_relation_search.rs |
| 18 | SparseResultant | BS-R094 | kernels/sparse_resultant.rs |
| 19 | TargetActionKrylov | BS-R095 | kernels/action_krylov.rs |
| 20 | UniversalTargetElimination | BS-R096 | kernels/universal_elimination.rs |
| 21 | RegularChainProjection | BS-R097 | kernels/regular_chain_projection.rs |
| 22 | NormTraceProjection | BS-R098 | kernels/norm_trace_projection.rs |
| 23 | SpecializationInterpolation | BS-R099 | kernels/specialization_interpolation.rs |
| 24.1-24.4 | message composition and final support | BS-R110, BS-R111 | compose/* |
| 25.1-25.5 | certificates, verify, replay | BS-R112, BS-R113 | verify/* |
| 26.1-26.3 | squarefree, roots, decode | BS-R120, BS-R121 | roots/*, algebra/real_root.rs |
| 27.1-27.4 | exact image / hermite / thom / slack | BS-R003, BS-R122 | OUT_OF_SCOPE except exact-image request scope guard |
| 28.1-28.3 | result and cost trace | BS-R011, BS-R130, BS-R131 | result/* |
| 29.1-29.3 | solver options/pipeline/orchestrator | BS-R140 | solver/* |
| 30.1-30.3 | algebraic cost design | BS-R131 | result/cost_trace.rs, planner/* |
| 31 | nonfinite target image | BS-R111, BS-R112, BS-R113 | compose/final_support.rs, verify/* |
| 32 | algebraic footprint not geometry dispatch | BS-R001, BS-R010, BS-R150 | all production code |
| 33 | completion conditions | BS-R150 | final finite candidate-cover conformance review |
| 34 | final summary | BS-R000, BS-R150 | candidate-cover closure |

Reviewer use:
- For every implementation phase, reviewers must use this map to select source sections and code files.
- If an in-scope source section has no implementation file evidence, the phase fails.
- If a source section is implemented only by tests or by a placeholder, the phase fails.
