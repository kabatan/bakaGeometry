# P3b Evidence Notes

P3b implements deterministic modular arithmetic, CRT, rational reconstruction, finite-field matrix rank/nullspace, and modular linear solve traces. It does not certify target relations or close candidate-cover behavior.

Important proof boundary:

- `solve_homogeneous_modular` and `solve_inhomogeneous_modular` return `ModularProofStatus::CandidateOnlyRequiresExactQCheck`.
- The modular solve prime sequence screens every selected prime against all relevant matrix coefficients, and `solve_inhomogeneous_modular` also screens RHS coefficients.
- The modular solve trace records pivot-column rank profiles and waits for stable rank profile, not rank alone.
- The modular solve layer resets sample buffers on rank-profile changes and performs CRT plus bounded rational reconstruction into candidate `VectorQ` values using only the final stable-profile suffix.
- If the configured stable rank-profile threshold is not reached before `max_primes` is exhausted, the modular solve layer returns no rational reconstruction candidate.
- No P3b function returns a certificate that a modular relation is valid over Q.
- Exact relation acceptance remains the responsibility of later callers, using P3a/P8a/P11 exact identity verification.

Negative coverage includes incompatible CRT, nonunique rational reconstruction, multi-prime avoidance, RHS-denominator avoidance, homogeneous same-rank/different-pivot-profile regression, inhomogeneous stable-suffix reconstruction coverage, and homogeneous/inhomogeneous no-reconstruction regressions when configured stability is not reached.
