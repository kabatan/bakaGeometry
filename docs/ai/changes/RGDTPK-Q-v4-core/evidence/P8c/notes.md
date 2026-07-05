# P8c Notes

P8c implements the production TargetActionKrylov kernel path for certifiable target-only finite quotient/action cases. It does not claim a generic quotient-basis constructor for arbitrary local ideals; the implemented admission is deliberately narrower and declines when the production quotient/action certificates cannot be built.

The kernel uses only `VerifiedCharacteristicSupportCoverage` as the support-producing proof. A recovered Krylov recurrence is treated as candidate generation only, and a relation is emitted only after the exact characteristic polynomial and exact Cayley-Hamilton verification pass through `certify_krylov_coverage` and `verify_annihilator`.

The implemented quotient handle is `ProductionProvenancedTargetQuotientHandle`. It is built from authorized relation hashes and per-action-column normal-form certificates. The P8c kernel file does not construct or accept `DebugExplicitTargetQuotientHandle`.

Claim ceiling after P8c review should be limited to P8c / MECH-014. This does not close P8 as an umbrella, does not implement UniversalTargetElimination, and does not claim final composition, root isolation, exact-image semantics, or acceptance readiness.
