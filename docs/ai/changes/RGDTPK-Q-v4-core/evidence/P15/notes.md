# P15 Notes - Acceptance Stress And Anti-Drift

Status: evidence prepared; P15 reviewer pass is required before P15 is closed.

P15 adds `geosolver-core/tests/p15_acceptance_stress.rs` with the RGQ-061 three-suite
partition:

1. Support-producing candidate-cover suite.
2. Exact-image semantics suite.
3. Failure and nonfinite semantics suite.

The support-producing suite runs the public `api::solve_target` path with renamed variables,
permuted relation order, and nonzero rational scaling. It asserts finite candidate-cover result
fields, replay acceptance, nonzero support, squarefree support, projection messages, core
certificate, populated `delta`/`kappa` cost fields, and non-placeholder decoded candidates when real
roots exist. The covered structures are the same acceptance categories required by P15: no initial
target-only relation, nonlinear multivariate action, multi-separator composition, sparse resultant,
specialization/interpolation, guarded rational affine preprocessing, semantic guard/slack and
branch/slack provenance in candidate-cover mode, determinant/oriented-bilinear support,
dot/Gram-like bilinear support, target-independent component, one-large-block Universal projection,
regular-chain projection, norm/trace tower, and no-real-root candidate cover.

The exact-image suite is separate and does not compensate for support-suite failures. It covers
nonempty exact image, exact empty real image, slack/guard filtering, branch semantics, and
exact-image certificate presence.

The failure/nonfinite suite is separate and does not count toward support-producing acceptance. It
covers invalid input, bounded resource/hard/certificate failure, positive certified nonfinite, and
exact-image nonfinite with unproved semantic obligations returning `CertificateDesignGap`.

P15 also adds direct anti-drift checks:

- a paired stress-renaming/relation-permutation test proves the same nonlinear action quotient
  structure still uses TargetActionKrylov and has the same final support degree under fresh ids,
  relation permutation, and rational relation scaling;
- dense relation-search schedule reproducibility on three local ideals with different eliminated
  variable counts, exported variable counts, and degrees, including support hashes, row monomial
  hashes, matrix dimensions, and stage order under input permutation;
- DAG hash tamper rejected by replay;
- kernel plan hash tamper rejected by replay;
- final DAG block authorization evidence tamper rejected by replay;
- kernel certificate binding tamper rejected by replay;
- projection message deletion rejected by replay;
- projection message package hash tamper rejected by replay;
- child projection message removal in a multiseparator case fails or changes composed support;
- static scans for ordinary Unsupported, geometry/problem/fixture/expected-answer dispatch,
  placeholder/test-only markers, and QE/CAD/full-coordinate fallback terms.

P15 does not claim P16 final closure, source fidelity, benchmark readiness, final public
replay-bound nonfinite readiness, full acceptance, or any R-ID as `VERIFIED`.
