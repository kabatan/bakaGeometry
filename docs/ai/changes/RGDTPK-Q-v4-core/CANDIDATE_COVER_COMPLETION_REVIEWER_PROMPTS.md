# Candidate-Cover Completion Reviewer Prompts v1

## Meta-protocol for every reviewer

Before issuing PASS, answer the following in the review response:

```text
1. What is the largest algebraic input class actually handled by the changed production code?
2. Does the reviewed phase prove true target values ⊆ roots(S), or only produce a plausible polynomial?
3. Are spurious roots being incorrectly treated as failures?
4. Does any success depend on exact-image filtering?
5. Does any success depend on geometry name, fixture, expected answer, or problem id?
6. Does any production path build full coordinate roots or full coordinate RUR?
7. Is every successful support replay-bound through actual DAG/messages/certificates?
8. Is this a generic candidate-cover mechanism, or a narrow slice with documentation?
```

A reviewer must not PASS merely because tests, YAML, hashes, or phase gates are present.

## Global pass/fail principles

### Must not fail

Do not fail candidate-cover merely because:

```text
- extra roots remain;
- a root has no real fiber under guard/slack semantics;
- exact_image_certificate is absent in candidate-cover mode;
- CertifiedExactTargetImage is not implemented.
```

### Must fail

Fail if:

```text
- true target values may be missing from returned candidates;
- S(T) is not exact Q verified;
- support is constructed from expected answer;
- support-producing case is helper-only and bypasses public pipeline;
- exact-image filtering is used to hide candidate-cover weakness;
- support-producing case returns AlgorithmicHardCase / FiniteResourceFailure / CertificateDesignGap / CertifiedNonFiniteTargetImage;
- narrow target-univariate/affine/alias-only implementation is treated as generic completion;
- nonfinite is returned without positive proof.
```

## Prompt: CCC-P0 reset review

```
Review CCC-P0. Confirm the Agent reset correctly states that candidate-cover means true target values are contained in roots(S), not equality. Confirm exact-image mixed fiber classification is no longer a blocker for candidate-cover readiness. Fail if the Agent still treats spurious roots as a defect, or if it tries to claim full exact-image/source-fidelity completion from this repair.
```

## Prompt: CCC-P1 source map review

```
Review CANDIDATE_COVER_SOURCE_MAP.md against v4. Confirm candidate-cover required functions are not deferred to exact-image later. Confirm section 27 exact-image functions are retained as later API/provenance, not deleted. Fail if support verification, root isolation, decode, DAG, planner, kernels, or replay are misclassified as later.
```

## Prompt: CCC-P2 candidate-cover semantics review

```
Inspect solver/orchestrator.rs, compose/final_support.rs, result/output.rs, and tests. Confirm candidate-cover mode does not filter roots by guard/slack semantics. Confirm at least one test intentionally permits spurious roots and still passes as CertifiedCandidateCover. Fail if exact-image mode is used to justify candidate-cover correctness.
```

## Prompt: CCC-P3 support verification review

```
Inspect verify/verify_support.rs, compose/final_support.rs, verify/replay.rs. Confirm both target-only product/lcm route and composed-ideal membership route are implemented when applicable. For membership route, require exact identity S(T)-Σq_i*r_i=0 over Q. Fail if membership is hash-only, sampled, numeric, or helper-only.
```

## Prompt: CCC-P4 generic projection integrity review

```
Inspect all kernels and planner/admission/ladder. Confirm specialized kernels are optimizers, not whole-solver slices. Confirm TargetRelationSearch and Universal remain generic routes. Fail if a declined optimizer can cause unsupported/slice failure when a generic route should continue.
```

## Prompt: CCC-P5 TRS/Universal review

```
Inspect target_relation_search.rs, universal_elimination.rs, planner/relation_schedule.rs, certificates. Confirm TRS uses deterministic schedule and exact membership. Confirm Universal is declared, local, resource-bounded, export-only, and never maps exhaustion to nonfinite. Fail if any hidden coordinate fallback exists.
```

## Prompt: CCC-P6 ActionKrylov review

```
Inspect action_krylov.rs, algebra/quotient.rs, algebra/krylov.rs. Confirm production TargetActionKrylov is not target-only or alias-univariate only. Confirm action columns are authorized-relation membership certified. Confirm coverage proof prevents missed eigenvalues. Fail if external action matrix self-consistency is accepted as proof.
```

## Prompt: CCC-P7 specialized optimizer review

```
Inspect sparse_resultant.rs, regular_chain_projection.rs, norm_trace_projection.rs, specialization_interpolation.rs. Confirm each optimizer has exact verification and does not define solver scope. If a kernel claims generic capability, verify the generic algorithm exists. Fail if limitations are used as completion boundaries.
```

## Prompt: CCC-P8 F4 decision review

```
Inspect algebra/f4.rs and all production references. Confirm either real production F4 exists with exact verification, or all F4 helpers are non-production and no readiness claim relies on F4. Fail if NotProductionF4 is used as production evidence.
```

## Prompt: CCC-P9 nonfinite safety review

```
Inspect final_support.rs, replay.rs, nonfinite tests. Confirm CertifiedNonFiniteTargetImage requires positive certificate and replay verifies it. Confirm relation-search exhaustion, sparse heuristic failure, Universal exhaustion, and composition failure do not become nonfinite. Fail if general nonfinite completeness is required for candidate-cover readiness.
```

## Prompt: CCC-P10 invariant review

```
Inspect run_certificate.rs, replay.rs, static scans, closure evidence. Confirm no-geometry/no-problem-id/no-expected-answer/no-QE/no-RUR/no-coordinate-solution evidence is hash-bound and reviewed. Fail if runtime flags claim more than they prove, or if static scan evidence is unbound.
```

## Prompt: CCC-P11 acceptance suite review

```
Run/inspect the public candidate-cover acceptance suite. Every support-producing case must use api::solve_target and return CertifiedCandidateCover with support, squarefree support, candidates, certificate, replay, and cost trace. Do not require exact target image equality. Fail if spurious roots are removed or treated as defects.
```

## Prompt: CCC-P12 red-team review

```
Construct at least 12 fresh algebraic inputs. At least 4 must permit spurious roots in candidate-cover mode. For each success, identify the proof of true target containment. Fail if support is expected-answer-based, if exact-image filtering is required for success, or if any success lacks exact containment proof.
```

## Prompt: CCC-P13 final closure review

```
Review CANDIDATE_COVER_CLOSURE.md and all evidence. Confirm final claim is only CANDIDATE_COVER_CORE_READY / SOURCE_FAITHFUL_TO_V4_CANDIDATE_COVER_LAYER. Confirm closure clearly states spurious roots are allowed. Fail if it claims SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC or RGDTPK_Q_V4_ACCEPTANCE_COMPLETE without separate exact-image completion.
```
