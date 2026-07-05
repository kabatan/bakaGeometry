# Primitive Scope Ledger - RGDTPK-Q-v4-core P12G

Status: active P12G anti-overclaim ledger.
Authority: tightening implementation ledger for `P5R-RGQ-070` and P12G-RGQ-073 through P12G-RGQ-085; it does not weaken `BASE_SPEC.md`, `P5R_BASE_SPEC_AMENDMENT.md`, or `P12G_BASE_SPEC_AMENDMENT.md`.
Current claim ceiling: `PARTIAL_MECHANISM_READY:MECH-011`.

This ledger prevents P6/P8/P9 and later reviewers from treating narrow P3/P4 primitives as completed generic target-direct solver kernels. It is not permission to keep a narrow solver. Every later phase reviewer must consult this file before accepting planner admission, kernel readiness, candidate-cover readiness, exact-image readiness, or final acceptance.

## algebra/resultant.rs

Current implemented capability:

Binary resultant support/template helpers with exact certificate checks for the current local algebra tests.

Exact limitations:

Not a generic sparse resultant projection kernel. It does not implement the full support-template search, all arities, determinant/resultant reconstruction ladder, or generic separator-target projection admission required by P8b.

Production use allowed before expansion:

Allowed only as a candidate-generation primitive under a later declared kernel plan and only when exact verification succeeds. It cannot by itself close `SparseResultantProjectionKernel`.

Required exact verification after use:

Every exported relation must be verified over Q by the resultant certificate or by exact membership/elimination verification before it becomes a projection message.

Allowed failure on exhaustion:

Admission false for this primitive or `AlgorithmicHardCase`, `FiniteResourceFailure`, or `CertificateDesignGap` from the owning kernel. It must not become ordinary `Unsupported`.

Forbidden claim:

Do not claim generic sparse resultant projection readiness, candidate-cover readiness, or performance readiness from the current binary primitive.

Later phase that must expand or replace this primitive:

P8b.

## algebra/interpolation.rs

Current implemented capability:

Deterministic specialization-point and one-variable sparse coefficient interpolation helpers with exact bad-sample rejection tests.

Exact limitations:

Not a generic specialization-interpolation projection kernel. It does not yet implement the full declared inner-kernel schedule, multi-separator sparse coefficient reconstruction, or global exact verification route required by P8b.

Production use allowed before expansion:

Allowed only as untrusted candidate generation under a declared later kernel plan.

Required exact verification after use:

Interpolated relations must be verified by exact Q membership or elimination proof. Sample agreement is not proof.

Allowed failure on exhaustion:

`AlgorithmicHardCase`, `FiniteResourceFailure`, or `CertificateDesignGap` with trace from the owning kernel.

Forbidden claim:

Do not claim generic `SpecializationInterpolationKernel` readiness or support-producing readiness from one-variable interpolation tests.

Later phase that must expand or replace this primitive:

P8b.

## algebra/regular_chain.rs

Current implemented capability:

Single-chain/component helper functions with guard-preserving local tests.

Exact limitations:

Not a generic regular-chain projection kernel. It does not yet cover compact component DAG construction, all branch/guard projection semantics, or generic target/separator relation export.

Production use allowed before expansion:

Allowed only where the owning later kernel can independently verify component semantics and exported relations.

Required exact verification after use:

Projected relations, guards, and component provenance must be verified by the P9 kernel certificate before use.

Allowed failure on exhaustion:

`AlgorithmicHardCase`, `FiniteResourceFailure`, or `CertificateDesignGap`.

Forbidden claim:

Do not claim generic `RegularChainProjectionKernel` readiness from the current single-chain helper.

Later phase that must expand or replace this primitive:

P9.

## algebra/norm_trace.rs

Current implemented capability:

Single-variable algebraic tower detection and norm relation verification helpers by algebraic form.

Exact limitations:

Not a generic norm/trace projection kernel. It does not yet implement multi-step tower plans, generic separator handling, or all P9 certificate/export requirements.

Production use allowed before expansion:

Allowed only when the later P9 kernel constructs the tower plan and verifies the norm relation exactly.

Required exact verification after use:

Norm/trace output must be recomputed and verified over Q against the authorized relations and exported variables.

Allowed failure on exhaustion:

Admission false for this primitive or hard/resource/certificate status from the owning kernel.

Forbidden claim:

Do not claim generic `NormTraceProjectionKernel` readiness from single-variable tower tests.

Later phase that must expand or replace this primitive:

P9.

## algebra/f4.rs

Current implemented capability:

Route B is chosen for P5R-b. The current file contains only `GroebnerBackedBatchOptions`, `F4ImplementationLevel::NotProductionF4`, `groebner_backed_batch_reduce_for_tests`, and `non_production_groebner_batch_elimination_for_tests`.

Exact limitations:

No real local F4-style symbolic row collection, Macaulay/F4 matrix construction, modular row reduction, rational reconstruction, or F4 matrix trace is implemented in P5R.

Production use allowed before expansion:

Not allowed as production F4. `EliminationStrategy::NonProductionGroebnerBatchForTests` is rejected by production dispatch with `CertificateDesignGap`. Local Groebner remains separately named as `LocalGroebner`.

Required exact verification after use:

Low-level helper tests still verify exported generators by exact Q membership, but this does not prove F4 semantics.

Allowed failure on exhaustion:

Production attempts to select the non-production batch route must fail as `CertificateDesignGap`.

Forbidden claim:

Do not claim `LocalF4`, F4-style sparse linear algebra, Universal local F4 readiness, or performance readiness from the Groebner-backed helper.

Later phase that must expand or replace this primitive:

P8d if the Universal local route needs real F4; otherwise it must choose an honestly named local Groebner route under the fixed plan and resource/certificate bounds.

## preprocess/linear_affine.rs

Current implemented capability:

Guarded affine preprocessing supports constant pivots, guarded polynomial pivots, and guarded rational pivots by exact denominator clearing with recorded denominator guard and rational substitution provenance.

Exact limitations:

This is a preprocessing primitive, not a full LinearAffineKernel. It does not by itself prove all later exact-image feasibility obligations or public pipeline readiness.

Production use allowed before expansion:

Allowed in pre-kernel compression for non-target variables only. Safe nonconstant denominator substitution requires explicit nonzero witness. Unsafe nonconstant denominator candidates are left in the system and do not produce ordinary `Unsupported` or `InvalidInput`.

Required exact verification after use:

The denominator guard, source witness relation, rational expression hash, and transformed relations must remain in compression provenance. Later exact-image/fiber code must enforce guard semantics.

Allowed failure on exhaustion:

No ordinary unsupported status for a well-formed Q-polynomial input. Resource exhaustion may become `FiniteResourceFailure`; invariant violation may become `ImplementationBug`.

Forbidden claim:

Do not claim full guarded-affine kernel readiness, exact-image readiness, or planner readiness from preprocessing alone.

Later phase that must expand or replace this primitive:

P7 for `LinearAffineKernel`; P13/P16 for exact-image semantics.

## algebra/quotient.rs and kernels/action_krylov.rs

Current implemented capability:

Production quotient handles are split from debug explicit handles. `ProductionProvenancedTargetQuotientHandle` is built from authorized relations, an authorization hash, a normal-form basis certificate, and per-action-column normal-form membership certificates. P12G Route A adds a production TargetActionKrylov path for a local univariate relation plus a linear target alias relation, for example `x^2 - 2 = 0` and `T - x = 0`, without calling TargetRelationSearch or Universal first.

Exact limitations:

The production builder verifies supplied certificates and the P12G TargetActionKrylov constructor builds the alias/univariate quotient-action certificates. It still does not implement a generic quotient-basis construction algorithm for all local block shapes or arbitrary ideals.

Production use allowed before expansion:

Allowed when authorized block relations are target-only univariate or match the P12G local-univariate plus target-alias quotient pattern and independent normal-form certificates are built from those authorized relations. `DebugExplicitTargetQuotientHandle` is not production-provenanced.

Required exact verification after use:

Every action column must verify that `var * basis_element - represented_normal_form` is in the authorized relation ideal by exact membership certificate. Authorization hash tampering must fail.

Allowed failure on exhaustion:

`CertificateDesignGap` for missing or unverifiable certificates; `FiniteResourceFailure` or `AlgorithmicHardCase` for construction exhaustion in the owning phase.

Forbidden claim:

Do not claim production `TargetActionKrylovKernel` readiness from externally injected action matrices, debug explicit handles, target-univariate companion-only behavior, or arbitrary local-ideal quotient basis construction.

Later phase that must expand or replace this primitive:

P8c plus P12G-b.

## algebra/krylov.rs

Current implemented capability:

Krylov coverage accepts only `VerifiedCharacteristicSupportCoverage`. Production Krylov functions now accept only `ProductionProvenancedTargetQuotientHandle` and build the target action matrix from verified action-column certificates.

Exact limitations:

This is still the algebra primitive layer. The full P8c kernel admission, planning, replay, and projection-message export path is not implemented yet.

Production use allowed before expansion:

Allowed only with production-provenanced quotient handles. Debug explicit handles are excluded by type from production Krylov functions.

Required exact verification after use:

The characteristic polynomial and Cayley-Hamilton check must be exact, and coverage must remain `VerifiedCharacteristicSupportCoverage`.

Allowed failure on exhaustion:

`CertificateDesignGap` for undercoverage or unverified action provenance; resource/hard-case statuses from the owning kernel when applicable.

Forbidden claim:

Do not claim support-producing TargetActionKrylov readiness from a single-vector sequence, block Wiedemann-like coverage without characteristic support proof, trace powers, or externally injected self-certifying columns.

Later phase that must expand or replace this primitive:

P8c.
