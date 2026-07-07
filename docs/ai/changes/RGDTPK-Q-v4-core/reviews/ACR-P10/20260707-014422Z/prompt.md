# ACR-P10 Review Prompt

Scope: ACR-P10 final closure for alg-cost completion only.

Do not review unrelated P13+ work. Do not claim exact-image acceptance, full supplied-v4 source
fidelity, `SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC`, or `RGDTPK_Q_V4_ACCEPTANCE_COMPLETE`.

Required governing files:

- `ALG_COST_COMPLETION_REPAIR_BASE_SPEC.md`
- `ALG_COST_COMPLETION_REPAIR_PLAN.md`, ACR-P10 section
- `ALG_COST_COMPLETION_REVIEWER_PROMPTS.md`, ACR-P10 prompt and meta-protocol
- `ALG_COST_ACCEPTANCE_MATRIX.yaml`

Final artifacts:

- `ALG_COST_COMPLETION_CLOSURE.md`
- `ALG_COST_FINAL_RED_TEAM_RESULTS.md`
- `ALG_COST_ROUTE_BUDGET_AUDIT.md`
- `ALG_COST_DECOMPOSITION_AUDIT.md`
- `ALG_COST_NO_OVERFIT_AUDIT.md`
- `evidence/ACR-P10/commands.txt`
- `evidence/ACR-P10/command_outputs.txt`
- `evidence/ACR-P10/static_scan_results.md`

Previous PASS archives to verify:

- `reviews/ACR-P0/20260706-180127Z/`
- `reviews/ACR-P1/20260706-180629Z/`
- `reviews/ACR-P2/20260706-183208Z/`
- `reviews/ACR-P3-P4/20260707-040822Z/`
- `reviews/ACR-P5/20260706-194228Z/`
- `reviews/ACR-P6/20260706-200604Z/`
- `reviews/ACR-P7/20260706-203037Z/`
- `reviews/ACR-P8/20260706-210037Z/`
- `reviews/ACR-P9/20260707-012120Z/`

Fresh closure evidence:

- `cargo fmt --manifest-path geosolver-core\Cargo.toml --check`: PASS.
- CCC red-team 16 fresh public inputs: PASS.
- FCR-P11 red-team 10 tests: PASS.
- ACR-P9 stress 8 tests / 24 variants: PASS.
- Full cargo test: PASS.
- Source/test forbidden fixture scan: PASS, no matches.

Required checks:

1. All previous ACR phases have complete PASS archives.
2. Final artifacts are present, current, and not stale.
3. Route budgets, SparseResultant swell guards, dense TRS preflight, Universal internal budgets, and
   graph decomposition are backed by code and evidence, not prose alone.
4. Red-team evidence includes at least ten public or near-public algebraic inputs.
5. Reviewer constructs at least ten fresh algebraic challenge shapes and ties each to public or
   near-public evidence.
6. CoreInvariantFlags/static scans/no-dispatch/no-QE-CAD are closure-bound and not overclaimed.
7. Candidate-cover readiness is not confused with exact-image or full acceptance.

