Purpose: finite candidate-cover closure packet
Status: final reviewer PASS for finite candidate-cover scope

# RGDTPK-Q v4 Finite Candidate-Cover Closure

Scope: finite candidate-cover only. Exact-image equality, real-fiber filtering, Hermite/Thom/slack final classification, and full-v4 completion remain out of scope except for the explicit exact-image request guard required by BS-R003 and BS-R122.

Allowed final claim ceiling after final reviewers pass:

```text
FINITE_CANDIDATE_COVER_COMPLETE
SOURCE_FAITHFUL_TO_V4_FINITE_CANDIDATE_COVER_LAYER
VERIFIED_FOR_FINITE_CANDIDATE_COVER
```

Forbidden claims remain forbidden:

```text
SOURCE_FAITHFUL_TO_FULL_V4
ACCEPTANCE_COMPLETE_FOR_FULL_V4
EXACT_IMAGE_COMPLETE
CERTIFIED_EXACT_TARGET_IMAGE_COMPLETE
PRODUCTION_SAFE
BENCHMARK_PROVEN
```

## Fresh Evidence

- Final command log: `evidence/final/final_commands.log`
- Final source-to-code matrix: `evidence/final/source_to_code_conformance_matrix.md`
- Git status snapshot: `evidence/final/git_status_short.txt`
- Changed file snapshot: `evidence/final/changed_files_full_diff.txt`
- P18 evidence: `evidence/P18/`
- Prior phase reviewer results: `evidence/P0/review_result.md` through `evidence/P17/review_result.md`

Fresh command outcomes recorded in `final_commands.log`:

- `cargo fmt --manifest-path geosolver-core\Cargo.toml --check`: exit 0.
- `cargo clippy --manifest-path geosolver-core\Cargo.toml --all-targets -- -D warnings`: exit 0.
- `cargo test --manifest-path geosolver-core\Cargo.toml`: exit 0.
- `python geosolver-core\scripts\audit_v4_conformance.py --strict`: exit 0, findings 0.

## P18 Implementation Notes

P18 added `geosolver-core/tests/v4_candidate_cover_conformance.rs` and adjusted stale success-route expectations in large-footprint/generic planner tests. The final conformance tests exercise public API behavior under misleading roles/names/relation order, Descartes candidate decoding and hash binding, exact-image scope guard behavior, canonical/hash order-independence, and evidence-backed bounded failure.

P18 also corrected `audit_v4_conformance.py` strict all-phase scanning so required negative no-coordinate-export guard strings and exact-image rejection guards are not reported as production coordinate/RUR or exact-image success paths. The strict audit still scans production Rust files and now reports findings 0.

## BS-R150 Completion Conditions

| # | Condition | Evidence status |
|---|---|---|
| 1 | Every well-formed Q-polynomial target system enters the generic pipeline. | Public API and pipeline tests pass; P18 adversarial public API test passes. |
| 2 | No geometry-name dispatch exists. | Strict static audit findings 0; P18 misleading-name/role test passes. |
| 3 | No problem-id / fixture-id / expected-answer dispatch exists. | Strict static audit findings 0; P18 relation/name permutation test passes. |
| 4 | TargetProjectionDAG is built for every valid input. | P6/P14/P17 evidence and replay tests pass. |
| 5 | If no useful separator exists, one large block is sent to a generic target-direct kernel. | P6/P7/P9/P18 generic route tests pass. |
| 6 | Every block receives a deterministic KernelPlan. | P7/P17 plan hash tests and final full test pass. |
| 7 | UniversalTargetEliminationKernel exists and returns target/separator-only output. | P12 evidence and tests pass; strict audit checks Universal strategy list. |
| 8 | Production path does not construct full coordinate solution list. | Strict audit findings 0; quotient/action guards present. |
| 9 | Production path does not construct full coordinate RUR. | Strict audit findings 0; no-coordinate-RUR export guards present. |
| 10 | On success, S(T) is in Q[T] and passes exact Q verification. | P14 support verification tests and final full test pass. |
| 11 | Root isolation is exact. | P15 tests and P18 Descartes candidate-decode test pass. |
| 12 | Decoded candidates are bound to support hash and root index. | P15 tests and P18 candidate hash/root-index checks pass. |
| 13 | Exact-image/full-image requests cannot return silent success and are explicitly out of scope. | P16 tests and P18 exact-image scope guard test pass. |
| 14 | Failures return evidence-backed status, not Unsupported. | P14/P17 tests and P18 bounded failure test pass. |
| 15 | Cost trace records every algebraic-cost-compression parameter. | P17 reviewer PASS; P18 bounded failure and full tests pass. |
| 16 | Hidden fallback is impossible at the API level. | P7/P12 planner/Universal tests pass; strict audit findings 0. |

## QuestionDebt

No blocking QuestionDebt is admitted for the finite candidate-cover scope. Exact-image equality/classification remains explicitly out of scope and is not counted as unresolved debt for this repair.

## Reviewer Sequence

Required final reviewers passed in order:

1. `guardian_boundary_reviewer`: PASS, agent `019f3d2f-9078-7301-89f2-d266a98488a0`.
2. `spec_verifier`: PASS, agent `019f3d32-a1df-7760-909f-e0484619f241`.
3. `quality_reviewer`: PASS, agent `019f3d36-a793-71f0-9851-561b0c955289`.

Final reviewer record: `evidence/P18/review_result.md`.
