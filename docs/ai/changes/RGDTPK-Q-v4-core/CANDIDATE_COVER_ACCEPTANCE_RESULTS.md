# Candidate-Cover Acceptance Results

Status: candidate-cover completion evidence.

## Claim Boundary

Allowed after all listed checks pass:

```text
CANDIDATE_COVER_CORE_READY
SOURCE_FAITHFUL_TO_V4_CANDIDATE_COVER_LAYER
```

Not claimed: exact-image equality, full supplied-v4 source-fidelity completion, full acceptance
completion, benchmark superiority, or universal finite-system completeness.

## Acceptance Matrix Coverage

| Matrix item | Evidence |
| --- | --- |
| A1 no initial target-only relation | Existing P15/FCR support suites plus `ccc_p12_red_team_runs_sixteen_fresh_public_inputs` cases 01-03. |
| A2 multivariate action not alias-only | Existing FCR/P15 action suites remain in force. |
| A3 multiple eliminated variables/separators | Existing FCR/P10/P11 composition tests plus fresh red-team case 02. |
| A4 sparse resultant/generic route | Existing P15 sparse resultant case plus fresh red-team case 03. |
| A5 specialization interpolation exact-Q verification | Existing P15 specialization case. |
| A6 guarded rational affine denominator | Existing FCR/P15 guarded-affine cases plus fresh red-team case 08. |
| A7 target-independent component | Existing FCR/P15 target-independent cases plus fresh red-team case 05. |
| A8 one large block Universal | Existing FCR/P11/P15 Universal cases. |
| A9 regular chain optimizer | Existing FCR/P11/P15 regular-chain cases. |
| A10 norm trace optimizer | Existing FCR/P11/P15 norm-trace cases. |
| A11 non-real support empty candidate cover | Existing P12G/P15 cases plus fresh red-team case 04. |
| A12 slack/guard spurious roots allowed | `ccc_p11_a12_spurious_roots_are_allowed_in_candidate_cover_mode` has four public spurious-root cases, and P12 fresh cases 13-16 add four new spurious-root cases not reused from P11. |
| A13 bounded hard failure not nonfinite | Existing FCR/P11 bounded failure tests plus fresh red-team case 10. |

## Route B Support Construction

`compose/final_support.rs::build_final_support_or_nonfinite_with_system` now constructs finite
support through composed-ideal elimination when Route A target-only root relations are unavailable.
`compose/compose.rs::compose_projection_messages` admits this fallback only when the composed message
ideal has a target eliminant; incomplete message deletion still fails.

Focused production support-stage test:

```text
ccc_route_b_final_support_uses_composed_ideal_membership_when_route_a_unavailable: PASS
```

## Fresh Red-Team Inputs

`geosolver-core/tests/ccc_candidate_cover_completion.rs::ccc_p12_red_team_runs_sixteen_fresh_public_inputs`
creates 16 new algebraic inputs through `api::solve_target`:

1. nonlinear alias-free target chain;
2. two eliminated variables;
3. higher-degree sparse eliminant;
4. nonreal support empty candidate cover;
5. target-independent feasible component;
6. mixed bilinear target relation;
7. additive target from two radicals;
8. guarded affine denominator witness;
9. positive nonfinite certificate;
10. bounded no-positive-proof case not certified nonfinite;
11. helper-variable support case;
12. triangular cubic chain;
13. positive slack keeps zero spurious root;
14. positive slack keeps negative spurious root;
15. nonnegative shifted guard keeps a spurious root;
16. branch-choice slack keeps an unfiltered branch root.

For every support-producing candidate-cover success, `assert_candidate_cover` rebuilds the public
validate/canonicalize/compress/DAG/compose flow, verifies `verify_global_support`, checks the exact
global support certificate hash bound in `CoreRunCertificate`, and records the production kernel
kinds that emitted the projection messages. This identifies the exact containment proof route
(`TargetOnlyRootRelationProduct` or `ComposedIdealMembership`) instead of accepting a plausible
polynomial by shape alone.

Per-success containment route and producer kernel assertions:

| Case | Containment proof route | Producer kernel(s) |
| --- | --- | --- |
| 01 | `TargetOnlyRootRelationProduct` | `TargetUnivariate` |
| 02 | `TargetOnlyRootRelationProduct` | `TargetUnivariate` |
| 03 | `TargetOnlyRootRelationProduct` | `TargetRelationSearch` |
| 04 | `TargetOnlyRootRelationProduct` | `TargetUnivariate` |
| 05 | `TargetOnlyRootRelationProduct` | `TargetUnivariate` |
| 06 | `TargetOnlyRootRelationProduct` | `TargetUnivariate` |
| 07 | `TargetOnlyRootRelationProduct` | `TargetRelationSearch` |
| 08 | `TargetOnlyRootRelationProduct` | `TargetUnivariate` |
| 11 | `TargetOnlyRootRelationProduct` | `TargetRelationSearch` |
| 12 | `TargetOnlyRootRelationProduct` | `TargetRelationSearch` |
| 13 | `TargetOnlyRootRelationProduct` | `TargetUnivariate` |
| 14 | `TargetOnlyRootRelationProduct` | `TargetUnivariate` |
| 15 | `TargetOnlyRootRelationProduct` | `TargetUnivariate` |
| 16 | `TargetOnlyRootRelationProduct` | `TargetUnivariate` |

Cases 09 and 10 are safety cases rather than support-producing candidate-cover successes: 09 checks
positive nonfinite certification and replay, and 10 checks bounded resource failure does not become
nonfinite without proof.

`ccc_p11_a12_spurious_roots_are_allowed_in_candidate_cover_mode` adds four explicit semantic
spurious-root cases. Candidate-cover mode keeps the roots and emits `ExactImageFilteringNotRequested`
and `CandidateCoverMayContainSpuriousRoots`.

## Focused Commands

```text
cargo test --manifest-path geosolver-core/Cargo.toml verify_support -- --nocapture: PASS
cargo test --manifest-path geosolver-core/Cargo.toml ccc_route_b_final_support -- --nocapture: PASS
cargo test --manifest-path geosolver-core/Cargo.toml --test p13_exact_image_semantics -- --nocapture: PASS
cargo test --manifest-path geosolver-core/Cargo.toml --test ccc_candidate_cover_completion -- --nocapture: PASS
```
