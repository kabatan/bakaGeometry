# P13 Notes - Exact Image Semantics

Status: PASS after spec, boundary, and quality reviewer remediation.

Implemented finite exact-image mode by inserting `classify_real_target_image` after exact root
isolation and candidate decode. In candidate-cover mode the solver still returns
`CertifiedCandidateCover` and leaves `exact_image_certificate` absent.

In exact-image mode each candidate receives:

- an algebraic target condition bound to support hash, root index, and candidate hash;
- applied semantic hashes from `RealConstraintEncoding`;
- equality decisions for target-only equations;
- guard/slack/branch semantic decisions with Thom/sign certificates;
- a Hermite-style real-fiber count certificate for realizable target-only/slack-semantic fibers,
  including exact support-root count for the candidate root and semantic slack real-root factors.

The implemented real-fiber proof is conservative: mixed target/coordinate fibers that are not
covered by the supported target-only/slack-semantic certificate patterns return
`CertificateDesignGap`, not an exact-image success. This avoids using candidate-cover evidence as
exact real-image proof.

P13 exact-image suite coverage:

- candidate-cover mode does not claim exact image;
- positive square-slack semantics filters the spurious `t = 0` support root and keeps `t = 1`;
- negative square guard semantics yields `CertifiedEmptyRealTargetImage`;
- branch-choice semantics affects exact classification;
- exact-image-mode nonfinite requires the real nonfinite certificate path and exposes proof-kind
  evidence in diagnostics;
- exact-image-mode nonfinite with semantic encodings but no real semantic proof returns
  `CertificateDesignGap`, not `CertifiedNonFiniteTargetImage`.
- exact-image-mode nonfinite with guard or saturation obligations but no real proof returns
  `CertificateDesignGap`, not `CertifiedNonFiniteTargetImage`.

Reviewer remediation after initial P13 review:

- `hermite_real_root_count_for_fiber` no longer hard-codes count `1`; it counts the exact support
  root by root index and multiplies recognized semantic real-root factors such as positive
  square-slack equations.
- The P13 suite asserts a positive square-slack exact candidate has real fiber count `2`.
- Exact-image nonfinite with semantic encodings now returns `CertificateDesignGap` unless a real
  semantic nonfinite certificate path is available.
- `solver/pipeline.rs::step_support` now passes `CompressedSystemQ` to final support so nonfinite
  certification sees semantic, guard, and saturation obligations.

P13 does not claim full acceptance, source fidelity, benchmark readiness, final public
replay-bound nonfinite readiness, or any R-ID as `VERIFIED`.
