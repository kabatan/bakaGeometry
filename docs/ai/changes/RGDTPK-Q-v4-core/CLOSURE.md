# Closure Packet - RGDTPK-Q-v4-core

Status: FCR-P12 final closure passed spec, boundary, and quality review. FCR-P0A/FCR-P1A and
FCR-P0 through FCR-P12 are complete for the candidate-cover layer. P13 exact-image implementation
passed spec, boundary, and quality review for MECH-012 only.

Approved FCR claim:

```text
CANDIDATE_COVER_CORE_READY
```

## Current State

P0 through P12G have historical PASS/review evidence in the current worktree history for their
original scopes. `FULL_CORE_REPAIR_BASE_SPEC.md` and `FULL_CORE_REPAIR_PLAN.md` are now mandatory
corrective overlays inserted before P13/P14/P15/P16.

Full Core Repair reopens the current generality and public-pipeline claims. FCR-P12 may close only
the candidate-cover layer. It does not close exact-image semantics, source fidelity, full
acceptance, final nonfinite readiness with a public replay-bound nonfinite certificate, or any
benchmark/performance/universal-completeness claim.

## P12G Remediation Scope

P12G addressed a direct algorithm audit before FCR, but FCR now classifies the remaining gaps as
larger core-repair blockers:

- TargetActionKrylov now has a provenance-bound non-target-only quotient/action path for local
  univariate relation plus target alias relation, in addition to the target-only companion path.
- Kernel plans can carry a typed `CertifiedProbePlan`; TargetActionKrylov binds plan-time probe
  source hashes, output hash, and trace hash, then replays them during execute.
- Candidate-cover finalization keeps nonzero support with zero real roots as
  `CertifiedCandidateCover` with empty roots/candidates.
- Final invariant evidence and final DAG replay evidence are hash-bound blockers. P14/P16 final
  claims remain blocked until actual DAG/block replay replaces synthetic all-relations replay for
  final claims.
- Nonfinite certificates carry an explicit proof kind and reject proof-kind/evidence mismatch.
- P12G G1-G8 stress tests are present at direct module or pipeline-fragment level.

## Full Core Repair Overlay

FCR requires repair or removal of narrow production paths, including alias/univariate-only
TargetActionKrylov, module-only stress proof, synthetic replay substitutes, plan-time execution
paths, fake or non-generic kernel claims, and any public pipeline gap in `api::solve_target`.

FCR-P10/P11 evidence is not final closure evidence. FCR-P11 selected the conservative final
nonfinite route: public nonfinite results do not yet carry a machine-readable replay-bound
certificate, so nonfinite readiness is excluded from `CANDIDATE_COVER_CORE_READY`. FCR-P11 also ran
the red-team suite with 10 fresh non-fixture algebraic inputs through the public or near-public
pipeline.

Required final FCR claim target is `CANDIDATE_COVER_CORE_READY`, not another partial mechanism
label. FCR-P12 spec, boundary, and quality reviews passed for that candidate-cover-only claim. P13
spec, boundary, and quality reviews passed for MECH-012 exact-image semantics only.

Final closure binds `CoreInvariantFlags` to fresh static scans and replay/tamper evidence in
`FULL_CORE_INVARIANT_SCAN_BINDING.md`. Static scans are necessary but not sufficient; the claim also
depends on FCR-P10 acceptance, FCR-P11 red-team inputs, replay/tamper tests, and final invariant/DAG
gate tests.

## Explicit Negative Claims

The following are still not complete:

- P14/P15/P16 historical exact-image/final-acceptance phases
- final nonfinite readiness with public replay-bound certificate
- performance claim
- `EXACT_IMAGE_CORE_READY`
- `SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC`
- `RGDTPK_Q_V4_ACCEPTANCE_COMPLETE`
- any R-ID marked `VERIFIED`

The following claim is approved by FCR-P12 reviewers for the candidate-cover layer only:

- `CANDIDATE_COVER_CORE_READY`

## FCR-P12 Evidence

| Artifact | Status |
| --- | --- |
| `FULL_CORE_ACCEPTANCE_RESULTS.md` | prepared |
| `FULL_CORE_REPLAY_TAMPER_RESULTS.md` | prepared |
| `FULL_CORE_COST_TRACE_SUMMARY.md` | prepared |
| `FULL_CORE_RED_TEAM_RESULTS.md` | prepared by FCR-P11 |
| `FULL_CORE_NONFINITE_RESULTS.md` | prepared by FCR-P11 route-2 exclusion |
| `FULL_CORE_INVARIANT_SCAN_BINDING.md` | prepared |
| `evidence/FCR-P12/command_outputs.txt` | prepared |
| `evidence/FCR-P12/scan_outputs.txt` | prepared |
| `evidence/FCR-P12/reviewer_results.md` | PASS |

## P13 Evidence Pending Review

| Artifact | Status |
| --- | --- |
| `geosolver-core/src/fiber/exact_image.rs` | implemented |
| `geosolver-core/src/fiber/hermite.rs` | implemented |
| `geosolver-core/src/fiber/thom.rs` | implemented |
| `geosolver-core/src/fiber/slack_semantics.rs` | implemented |
| `geosolver-core/tests/p13_exact_image_semantics.rs` | 7/7 PASS |
| `evidence/P13/command_outputs.txt` | prepared |
| `evidence/P13/reviewer_results.md` | PASS |

P13 evidence covers finite exact-image classification, candidate-cover/exact-image status
separation, slack/guard/branch filtering, exact-empty classification, and exact-image-mode
nonfinite requiring the real nonfinite certificate path. Semantic nonfinite cases without a real
semantic/guard/saturation proof return `CertificateDesignGap`. It does not close P14/P15/P16,
final public replay-bound nonfinite readiness, source fidelity, benchmark readiness, or full
acceptance.

Fresh FCR-P12 commands:

```text
cargo fmt --manifest-path geosolver-core/Cargo.toml -- --check: PASS
cargo clippy --manifest-path geosolver-core/Cargo.toml --all-targets --all-features -- -D warnings: PASS
cargo test --manifest-path geosolver-core/Cargo.toml --all-targets --all-features -- --nocapture: PASS
git diff --check: PASS
```

## Commit Binding

The current FCR import is bound by `FULL_CORE_REPAIR_PACK_MANIFEST.sha256`, the source zip SHA256
recorded in FCR-P0 evidence, and local evidence under
`docs/ai/changes/RGDTPK-Q-v4-core/evidence/FCR-P0/`. Historical P0-P12G archives remain evidence
for their original scopes only and do not override FCR.
