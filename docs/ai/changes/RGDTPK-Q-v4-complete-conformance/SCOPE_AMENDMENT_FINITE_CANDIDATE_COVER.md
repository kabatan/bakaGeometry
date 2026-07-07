# Scope Amendment: Finite Candidate-Cover Completion

Purpose: active scope amendment for the imported Guardian packet.  
Status: approved for implementation by user on 2026-07-07.  
Authority: records the user-approved target that finite candidate-cover output is sufficient for this repair. The amended `BASE_SPEC.md`, `PLAN.md`, `SOURCE_MAP.md`, and `REVIEWER_PROMPTS.md` are the executable contract.  

The imported packet originally targeted full v4 conformance including exact-image classification.
This repair is narrowed to the v4 finite candidate-cover layer:

```text
Given a well-formed Q-polynomial target system, produce a finite exact candidate cover:
S(T) in Q[T], S(T) != 0,
such that every true finite target value is a root of S.
```

Spurious roots are allowed. Exact target-image equality, real-fiber filtering, Hermite/Thom/slack
classification as a final image proof, and `CertifiedExactTargetImage` are out of scope for this
repair.

The solver must not silently treat exact-image mode as implemented. If an exact-image request is
exposed through the API during this scoped repair, it must return an explicit evidence-backed scope
diagnostic or failure status and must not be counted as candidate-cover success.

Allowed final claims after all scoped phases and reviewers pass:

```text
FINITE_CANDIDATE_COVER_COMPLETE
SOURCE_FAITHFUL_TO_V4_FINITE_CANDIDATE_COVER_LAYER
VERIFIED_FOR_FINITE_CANDIDATE_COVER
```

Forbidden final claims for this scoped repair:

```text
SOURCE_FAITHFUL_TO_FULL_V4
ACCEPTANCE_COMPLETE_FOR_FULL_V4
EXACT_IMAGE_COMPLETE
CERTIFIED_EXACT_TARGET_IMAGE_COMPLETE
PRODUCTION_SAFE
BENCHMARK_PROVEN
```
