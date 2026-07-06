# P15 Review Packet - Acceptance Stress

Review target: Plan P15 only.

## Scope

Source anchors:

- `BASE_SPEC.md` RGQ-034, RGQ-035, RGQ-036, RGQ-037, RGQ-038, RGQ-039, RGQ-040,
  RGQ-048, RGQ-049, RGQ-050, RGQ-053, RGQ-055, RGQ-061, RGQ-062, RGQ-063.
- `PLAN.md` P15 implementation tasks.
- `REVIEWER_PROMPTS.md#P15`.
- Guardian Runtime Contract from `AGENTS.md`.

Changed implementation/test files:

- `geosolver-core/tests/p15_acceptance_stress.rs`

Changed evidence/docs:

- `docs/ai/changes/RGDTPK-Q-v4-core/evidence/P15/*`
- `docs/ai/changes/RGDTPK-Q-v4-core/reviews/P15/20260706-acceptance-stress/review_packet.md`

## Implementation Summary

P15 adds a dedicated acceptance stress integration test with the three-suite RGQ-061 partition:

- support-producing candidate-cover suite;
- exact-image semantics suite;
- failure and nonfinite semantics suite.

The support-producing suite uses the public `solve_target` path and renamed/permuted/scaled
algebraic inputs across all P15 acceptance categories. It requires candidate-cover success, nonzero
support, squarefree support, projection messages, core certificate, replay acceptance, cost
`delta`/`kappa`, and non-placeholder decoded candidates when real roots exist. The support cases now
include semantic guard/slack and branch/slack provenance in candidate-cover mode,
determinant/oriented-bilinear support, and dot/Gram-like bilinear support in addition to the earlier
kernel and pipeline categories.

The exact-image suite runs with `exact_image_mode=true` and checks nonempty exact image,
exact-empty, slack/guard filtering, branch semantics, and exact-image certificates.

The failure/nonfinite suite covers invalid input, bounded resource/hard/certificate result,
positive certified nonfinite, and exact-image nonfinite semantic proof gaps. These are not counted
as support-producing acceptance.

Anti-drift checks include dense relation-search schedule reproducibility on three local ideals with
different eliminated/exported variable counts and degrees, including support hashes, row monomial
hashes, matrix dimensions, and stage order under input permutation. They also include replay
rejection on DAG hash tamper, replay rejection on kernel plan hash tamper, replay rejection on final
DAG block authorization evidence tamper, replay rejection on kernel certificate binding tamper,
replay rejection on projection-message deletion, replay rejection on projection-message package hash
tamper, child-message removal changing/failing multiseparator composition, paired
renamed/permuted-structure mechanism checks, and static scan classifications.

## Verification

Commands run and passing:

```text
cargo fmt --manifest-path geosolver-core/Cargo.toml -- --check
cargo test --manifest-path geosolver-core/Cargo.toml --test p15_acceptance_stress -- --nocapture
cargo clippy --manifest-path geosolver-core/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path geosolver-core/Cargo.toml --all-targets --all-features -- --nocapture
rg -n --sort path --path-separator / "Unsupported|unsupported" geosolver-core/src
rg -n --sort path --path-separator / "circle|triangle|tangent|distance|area|incircle|circumcircle|orthic|mixtilinear|fixture|expected|answer|problem_id|official" geosolver-core/src
rg -n --sort path --path-separator / "todo!|unimplemented!|placeholder|dummy|fake|stub|temporary_pipeline_not_connected|NotProductionF4|for_tests" geosolver-core/src
rg -n --sort path --path-separator / "CAD|QE|RCF|coordinate solution|coordinate roots|full coordinate RUR|RUR|solve_all_coordinates" geosolver-core/src
git diff --check
```

P15 suite result: 6/6 PASS.

All-target result includes P15:

- lib tests: 212 passed;
- integration suites: fcr_final_nonfinite 2, fcr_p10 12, fcr_p11 10, fcr_p4 7,
  p12 1, p12g 1, p13 7, p14 10, p15 6, p3 2.

Static scan summary:

- ordinary `Unsupported`: 0 matches;
- geometry/problem/fixture/expected-answer dispatch scan: 139 classified matches, no new production
  dispatch added by P15;
- placeholder/test-only marker scan: 19 classified matches, no new production scaffold added by P15;
- QE/CAD/full-coordinate fallback scan: 3 classified rejection/error-text matches.

## Claim Boundary

P15 may close only generalized acceptance stress / anti-drift evidence. It must not approve P16
final closure, `EXACT_IMAGE_CORE_READY`, `RGDTPK_Q_V4_ACCEPTANCE_COMPLETE`,
`SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC`, benchmark readiness, final public replay-bound nonfinite
readiness, or any R-ID as `VERIFIED`.
