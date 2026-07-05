# P3d Evidence Notes

P3d implements algebra-level sparse resultant and specialization/interpolation primitives. These are candidate-generation support primitives and do not accept target support or solver results by themselves.

Important proof boundary:

- `ResultantRelation` and `InterpolationCertificate` use `CandidateOnlyRequiresExactMembership`.
- `compute_resultant_relation` records modular trace hashes but the returned relation remains a candidate requiring later exact membership/elimination verification.
- `verify_resultant_certificate` verifies template/relation/modular-trace consistency for the resultant primitive; it does not certify final target support in the full solver pipeline.
- Resultant template construction rejects eliminate/keep overlap, duplicate keep variables, noncanonical keep-variable ordering, and non-keep variables.
- Resultant modular trace verification validates trace moduli are prime and denominator-safe before modular reduction and returns `false` for invalid/tampered trace primes instead of panicking.
- `verify_interpolated_relation` verifies every sample by exact re-specialization and rejects bad samples; it does not replace later membership/elimination verification.
- P3d does not implement SparseResultantProjectionKernel or SpecializationInterpolationKernel integration. Later P8b owns kernel admission/execution and ProjectionMessage production.
- P3d does not produce coordinate roots, coordinate parametrizations, full coordinate RURs, exact-image statuses, candidate-cover statuses, or nonfinite target-image certification.

Negative coverage includes non-keep variable rejection for resultant templates, eliminate/keep overlap, duplicate keep rejection, noncanonical keep-order rejection, tampered resultant relation-hash rejection, tampered modular-trace-prime rejection without panic, nonprime trace-modulus rejection, deterministic specialization, exact sparse coefficient interpolation, and a bad interpolation sample regression that fails final verification.
