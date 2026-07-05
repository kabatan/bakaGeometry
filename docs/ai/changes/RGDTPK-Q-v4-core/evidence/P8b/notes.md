# P8b Notes

P8b implements SparseResultantProjectionKernel and SpecializationInterpolationKernel execution paths.

SparseResultant:

- admits only when a finite resultant template pair is available;
- consumes local relations plus child message relation generators;
- binds child message package hashes and source relation hashes into KernelExecutionPlan;
- plans a template trace and binds it through KernelExecutionPlan support hashes;
- recomputes the template trace at execution;
- verifies resultant certificates by exact recomputation before returning a ProjectionMessage;
- returns only exported-variable relations;
- treats "not sparse enough" as this-kernel admission decline, not solver unsupported.

SpecializationInterpolation:

- admits only when non-target exported separators exist;
- consumes local relations plus child message relation generators;
- binds child message package hashes and source relation hashes into KernelExecutionPlan;
- builds deterministic multiseparator coefficient support and specialization grids;
- executes a declared inner target-only TargetRelationSearch kernel for each specialization sample;
- uses modular inhomogeneous solving only to reconstruct candidate coefficient vectors from those inner-kernel samples;
- verifies those vectors over Q against all samples;
- then verifies the interpolated relation against exact local elimination/membership certificates before returning a ProjectionMessage.

Status boundaries:

- plan/source/authorization/template mismatch is `ImplementationBug`;
- child message package-hash mismatch is `ImplementationBug`;
- no exported relation within the declared route is `AlgorithmicHardCase`;
- missing certificate construction remains `CertificateDesignGap` before success;
- no P8b path returns `CertifiedNonFiniteTargetImage`;
- no `Unsupported`, `NotYetImplemented`, or phase-local public status is introduced.

Residual risks:

- P8b continues MECH-007 but does not close all optimized projection kernels; P8c, P8d, and P9 remain open.
- Universal, ActionKrylov, RegularChain, NormTrace, final composition, exact root decode, exact image semantics, public orchestration, replay closure, and final acceptance remain open.
