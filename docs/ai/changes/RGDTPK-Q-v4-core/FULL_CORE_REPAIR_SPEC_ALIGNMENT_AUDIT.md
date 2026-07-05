# Full Core Repair v2 — Spec Alignment Audit

## Verdict

The v1 repair pack was directionally correct but not strict enough. It still allowed several dangerous readings:

```text
- candidate-cover completion could be mistaken for full v4 source-fidelity;
- exact-image obligations could be forgotten rather than explicitly deferred;
- the Agent mindset was not sufficiently constrained against gate-seeking;
- reviewer prompts still trusted evidence too much;
- plan/execute separation was identified but not made an all-kernel source-fidelity blocker;
- source-to-code coverage of every v4 file/function was not mandatory;
- specialized kernel limitations could still be documented rather than repaired.
```

## Source-aligned interpretation

The supplied v4 specification requires a Q-polynomial target solver core, not a geometry-family handler. It requires all well-formed inputs to enter a generic pipeline, no geometry/problem/expected-answer dispatch, deterministic planning, target/separator-only projection messages, exact-Q verification, exact root isolation, cost trace, and no narrow-slice completion.

The finite-candidate layer is `CertifiedCandidateCover`. This full-core repair may target that layer. It must not claim exact-image completion or full source-fidelity until the v4 real-fiber/slack/guard semantics are implemented.

## Over- and under-specification fixes in v2

```text
1. Added Agent operating doctrine and failure-mode reset.
2. Added source-spec compliance map for all v4 sections 0–34.
3. Added explicit candidate-cover vs exact-image scope boundary.
4. Added no-specialized-kernel-stands-in-for-generic-pipeline rule.
5. Strengthened reviewer meta-protocol to require adversarial algorithmic review.
6. Added red-team review after repair to generate new algebraic counterexamples.
7. Kept final allowed claim at CANDIDATE_COVER_CORE_READY only.
8. Kept EXACT_IMAGE_CORE_READY / SOURCE_FAITHFUL / ACCEPTANCE_COMPLETE blocked until P13+P16.
```

## Remaining responsibility

This pack cannot be satisfied by preserving current P12G code and adding explanations. The Agent must either generalize or remove/quarantine production partial implementations. If this proves impossible under the current design, the Agent must declare AlgorithmDefect instead of passing the phase.
