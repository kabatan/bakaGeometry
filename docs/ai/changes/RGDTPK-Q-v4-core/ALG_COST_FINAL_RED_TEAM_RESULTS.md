# Algebraic-Cost Final Red-Team Results

Scope: ACR-P10 final closure red-team evidence.

## Commands

```text
cargo test --manifest-path geosolver-core\Cargo.toml --test ccc_candidate_cover_completion ccc_p12_red_team_runs_sixteen_fresh_public_inputs -- --test-threads=1 --nocapture
PASS: 1 passed; 0 failed; finished in 0.36s

cargo test --manifest-path geosolver-core\Cargo.toml --test fcr_p11_red_team_suite -- --test-threads=1 --nocapture
PASS: 10 passed; 0 failed; finished in 3.71s

cargo test --manifest-path geosolver-core\Cargo.toml --test acr_p9_large_footprint_stress -- --test-threads=1 --nocapture
PASS: 8 passed; 0 failed; 24 variants executed
```

## Fresh Public Inputs

`ccc_p12_red_team_runs_sixteen_fresh_public_inputs` uses `solve_target` and rebuilt public or
near-public pipeline evidence. It covers these fresh algebraic shapes:

1. nonlinear alias-free target chain
2. two eliminated variables
3. higher-degree sparse eliminant
4. nonreal support producing empty candidate cover
5. target-independent feasible component retained
6. mixed bilinear target relation
7. additive target from two radicals
8. guarded affine denominator witness
9. positive nonfinite input with replayable nonfinite certificate
10. bounded resource hard case that is not reported as nonfinite
11. quartic support through helper variable
12. triangular cubic chain
13. positive slack keeps zero spurious root in candidate-cover mode
14. positive slack keeps negative spurious root in candidate-cover mode
15. nonnegative guard keeps shifted spurious root in candidate-cover mode
16. branch-choice slack keeps unfiltered branch root in candidate-cover mode

The test asserts `CertifiedCandidateCover` for support-producing cases, `replay_run_certificate`
for each run, public/near-public producer kernels, and global support evidence. Nonfinite and
bounded-hardcase cases are checked for their correct non-candidate-cover status boundaries.

## Independent Red-Team Suite

`fcr_p11_red_team_suite` ran ten additional public or near-public tests:

1. multivariate action is not reduced to target-univariate aliasing
2. two-separator public composition
3. sparse-resultant higher-degree eliminant
4. guarded rational affine nonconstant denominator
5. one-large-block Universal admission
6. target-independent feasibility component
7. positive nonfinite kept out of candidate-cover claim
8. target-free input without positive witness is not nonfinite
9. regular-chain fresh input
10. norm-trace fresh two-step tower

These tests call `solve_target`, assert replay, and rebuild composition/DAG evidence where needed.

## Reviewer Requirement

The ACR-P10 reviewer prompt must require the reviewer to construct at least ten fresh algebraic
challenge shapes and verify that public or near-public pipeline evidence covers them. Reviewer PASS
must not rely only on the existence of this file.

