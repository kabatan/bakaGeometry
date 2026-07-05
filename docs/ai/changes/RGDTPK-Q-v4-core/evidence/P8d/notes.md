# P8d Notes

P8d implements `UniversalTargetEliminationKernel` as the bounded local target/separator projection route required by `RGQ-041`. The kernel is not a hidden heavy fallback: execution is limited to the plan-hash-bound fixed sequence of TargetRelationSearchEscalated, SparseResultantIfSquareOrOverdetermined, SpecializeProjectInterpolateVerify, and LocalGroebnerEliminationToKeepZ.

The local final stage uses `EliminationStrategy::LocalGroebner` with exported variables as keep variables and explicit caps. This is intentional: `PRIMITIVE_SCOPE_LEDGER.md` says the existing F4 code is non-production and must not be claimed as production F4. P8d therefore closes the Universal local route, if reviewer accepts it, as an honestly named local Groebner route under the fixed plan/resource/certificate bounds, not as F4 readiness.

The implementation verifies plan hash, block authorization hash, child message hashes, source hash coverage, certificate route, local nonfinite policy, support hash, and fixed strategy order before execution. Returned generators must lie in the exported variable set and pass exact local elimination validation.

Exhaustion is routed only to AlgorithmicHardCase, FiniteResourceFailure, or CertificateDesignGap. P8d does not return local CertifiedNonFiniteTargetImage; final-support nonfinite semantics remain owned by later phases.

Claim ceiling after P8d review should be limited to P8d / MECH-008. This does not close P8 as an umbrella, P9 optimized kernels, P10 final support composition, root isolation, exact-image semantics, public orchestration, performance readiness, or final acceptance.
