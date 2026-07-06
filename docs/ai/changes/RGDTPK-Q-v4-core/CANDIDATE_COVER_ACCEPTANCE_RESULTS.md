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
| A1 no initial target-only relation | Existing P15/FCR support suites plus `ccc_p12_red_team_runs_twelve_fresh_public_inputs` cases 01-03. |
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
| A12 slack/guard spurious roots allowed | `ccc_p11_a12_spurious_roots_are_allowed_in_candidate_cover_mode` has four public spurious-root cases. |
| A13 bounded hard failure not nonfinite | Existing FCR/P11 bounded failure tests plus fresh red-team case 10. |

## Fresh Red-Team Inputs

`geosolver-core/tests/ccc_candidate_cover_completion.rs::ccc_p12_red_team_runs_twelve_fresh_public_inputs`
creates 12 new algebraic inputs through `api::solve_target`:

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
12. triangular cubic chain.

`ccc_p11_a12_spurious_roots_are_allowed_in_candidate_cover_mode` adds four explicit semantic
spurious-root cases. Candidate-cover mode keeps the roots and emits `ExactImageFilteringNotRequested`
and `CandidateCoverMayContainSpuriousRoots`.

## Focused Commands

```text
cargo test --manifest-path geosolver-core/Cargo.toml verify_support -- --nocapture: PASS
cargo test --manifest-path geosolver-core/Cargo.toml --test p13_exact_image_semantics -- --nocapture: PASS
cargo test --manifest-path geosolver-core/Cargo.toml --test ccc_candidate_cover_completion -- --nocapture: PASS
```
