# P10 Notes

P10 implements composition and final support without rebuilding the original full coordinate system. Composition consumes only `ProjectionMessage` relation generators after checking the message package hash and exported-variable boundary.

Separator elimination is intentionally routed through a pseudo block/system built exclusively from message relations. The implementation uses the existing dense `TargetRelationSearchKernel` as the target-direct kernel. If the dense relation search exhausts without a target/separator relation, the status remains `AlgorithmicHardCase` and never becomes `CertifiedNonFiniteTargetImage`.

Final support construction builds the support polynomial only from composed root relations whose variables are contained in `{T}`. If no target-only relation exists, `build_global_support_polynomial` returns `AlgorithmicHardCase`. The separate `build_final_support_or_nonfinite` route returns `CertifiedNonFinite` only when `certify_nonfinite_target_image` produces and verifies a positive certificate. Public `CertifiedNonFiniteTargetImage` finalization re-verifies the certificate against the composed projection.

The implemented positive nonfinite certificate is deliberately narrow and exact: it certifies the structural case where no composed root generator contains the target variable, and an exact rational consistency witness satisfies every root relation. This proves the target is free for that composed ideal. Cases outside this certifiable route remain hard/certificate cases rather than nonfinite.

P10 does not claim P11 replay/certificate closure, P12 root isolation/decode completion, exact-image classification beyond the P10 real-nonfinite certificate function, public orchestration, performance readiness, final acceptance, or any R-ID as VERIFIED.
