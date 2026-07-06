# Closure Packet - RGDTPK-Q-v4-core

Status note for current work: historical closure only. The active algebraic-cost completion repair
has suspended `CANDIDATE_COVER_CORE_READY` and `SOURCE_FAITHFUL_TO_V4_CANDIDATE_COVER_LAYER` until
`ALG_COST_COMPLETION_REPAIR_PLAN.md` passes.

Status: superseded for active work by `v4_candidate_cover_completion_pack_v1`.

Active closure authority is now `CANDIDATE_COVER_CLOSURE.md`.

Active allowed claim after this repair:

```text
CANDIDATE_COVER_CORE_READY
SOURCE_FAITHFUL_TO_V4_CANDIDATE_COVER_LAYER
```

Not claimed from this repair:

```text
exact target-image completion
full supplied-v4 source-fidelity completion
full acceptance completion
benchmark superiority
universal finite-system completeness
geometry DSL support
natural-language or diagram support
any R-ID VERIFIED status
```

## Candidate-Cover Semantics

The active correctness statement is containment:

```text
true finite target values subset roots(S)
```

The reverse inclusion is not required for candidate-cover readiness. Candidate-cover mode must not
call exact-image filtering to remove real roots that are invalid only under later fiber, guard,
slack, or branch semantics.

## Supersession Note

Earlier P16 evidence remains historical implementation evidence, but it is no longer the active
claim authority for this thread. Any statement stronger than candidate-cover readiness requires a
separate exact-image/full-acceptance closure.

## Current Evidence Files

- `CANDIDATE_COVER_AGENT_RESET.md`
- `CANDIDATE_COVER_SOURCE_MAP.md`
- `CANDIDATE_COVER_ACCEPTANCE_RESULTS.md`
- `CANDIDATE_COVER_REPLAY_TAMPER_RESULTS.md`
- `CANDIDATE_COVER_COST_TRACE_SUMMARY.md`
- `CANDIDATE_COVER_CLOSURE.md`

## Residual Risks

- Exact-image equality is outside the active candidate-cover claim.
- Production F4 is not claimed; non-production Groebner-backed helpers remain test-only.
- Runtime invariant flags are not used as repo-wide static-scan proof by themselves.
- Geometry/natural-language/diagram input is out of scope.
