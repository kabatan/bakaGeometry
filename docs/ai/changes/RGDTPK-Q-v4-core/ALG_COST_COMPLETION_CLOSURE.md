# Algebraic-Cost Completion Closure

Status: ACR-P10 closure record after Guardian boundary and spec review PASS.

Scope: `RGDTPK-Q-v4-algebraic-cost-completion-repair-v1`.

## Closure Decision

The ACR-P0 through ACR-P9 repair phases now have PASS review archives:

| Phase | Closing review archive |
| --- | --- |
| ACR-P0 | `reviews/ACR-P0/20260706-180127Z/` |
| ACR-P1 | `reviews/ACR-P1/20260706-180629Z/` |
| ACR-P2 | `reviews/ACR-P2/20260706-183208Z/` |
| ACR-P3/P4 | `reviews/ACR-P3-P4/20260707-040822Z/` |
| ACR-P5 | `reviews/ACR-P5/20260706-194228Z/` |
| ACR-P6 | `reviews/ACR-P6/20260706-200604Z/` |
| ACR-P7 | `reviews/ACR-P7/20260706-203037Z/` |
| ACR-P8 | `reviews/ACR-P8/20260706-210037Z/` |
| ACR-P9 | `reviews/ACR-P9/20260707-012120Z/` |

Each listed archive contains `prompt.md`, `response.md`, `review_summary.yaml`, and
`evidence_manifest.yaml`. Historical FAIL or FAIL_FIXABLE archives remain as repair history only
and are superseded by the listed PASS archive.

## Restored Claim Boundary

After ACR-P10 reviewer PASS, this repair may restore only these claims:

```text
CANDIDATE_COVER_CORE_READY
SOURCE_FAITHFUL_TO_V4_CANDIDATE_COVER_LAYER
```

Meaning: for the algebraic-cost candidate-cover layer, the production pipeline has bounded
dominant algebraic costs on declared routes, non-monopolizing route execution, exact Q verification
of produced projection messages/support, replay-bound certificates, and generic support-producing
stress evidence.

Candidate-cover means every true finite target value is contained in `roots(S)`. It does not mean
`roots(S)` is exactly the target image.

## Explicit Non-Claims

This closure does not claim:

```text
SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC
RGDTPK_Q_V4_ACCEPTANCE_COMPLETE
exact-image acceptance
full source-fidelity for layers outside candidate-cover
spurious-root elimination in candidate-cover mode
```

Exact-image semantics remain governed by the separate exact-image tests and artifacts. The
candidate-cover layer is allowed to return extra real roots.

## Final Evidence Set

Final artifacts required by ACR-P10:

- `ALG_COST_FINAL_RED_TEAM_RESULTS.md`
- `ALG_COST_ROUTE_BUDGET_AUDIT.md`
- `ALG_COST_DECOMPOSITION_AUDIT.md`
- `ALG_COST_NO_OVERFIT_AUDIT.md`

Fresh closure commands:

```text
cargo test --manifest-path geosolver-core\Cargo.toml --test ccc_candidate_cover_completion ccc_p12_red_team_runs_sixteen_fresh_public_inputs -- --test-threads=1 --nocapture
PASS: 1 passed; 16 fresh public inputs exercised

cargo test --manifest-path geosolver-core\Cargo.toml --test fcr_p11_red_team_suite -- --test-threads=1 --nocapture
PASS: 10 passed

cargo test --manifest-path geosolver-core\Cargo.toml --test acr_p9_large_footprint_stress -- --test-threads=1 --nocapture
PASS: 8 passed; 24 variants; 69.13s

cargo test --manifest-path geosolver-core\Cargo.toml -- --test-threads=1
PASS: lib 253 passed and all integration/doc tests passed
```

Source/test forbidden-fixture scan:

```text
rg -n -i "mixtilinear|mixtilinear_candidate_cover_problem|circumcircle|incircle|tangent_solver|expected_cos|expected target value|expected support polynomial|known_support_polynomial|diagnostic_problem|problem hash|geometry name" geosolver-core\src geosolver-core\tests
PASS: no matches
```

## Acceptance Mapping

- Dense TRS materialization is bounded by preflight descriptors and sparse/lazy alternatives.
- SparseResultant has expression-swell preflight, runtime growth guards, and exact replayable
  backend evidence.
- Declared ladder execution enforces route budgets and records route-local failures without
  monopolizing the solver.
- UniversalTargetElimination uses bounded internal stages and records executed internal failures
  separately from skipped/prohibited stages.
- Graph decomposition uses algebraic-cost-aware separator scoring and records large-block
  explanations.
- P9 stress runs S1-S8 with deterministic anti-overfit variants, exact support verification, and
  replay acceptance.
