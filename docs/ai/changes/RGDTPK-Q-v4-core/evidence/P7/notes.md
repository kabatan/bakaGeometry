# P7 Notes

P7 implements the kernel trait and registry, TargetUnivariate, and LinearAffine only. The registry returns all nine Appendix A kernels, but kernels owned by later phases remain admission-declining placeholders with explicit CertificateDesignGap execution errors.

TargetUnivariate uses target-only relations selected from the authorized block relation set and from child projection messages bound by package hash, converts them to univariate form, multiplies squarefree parts, squarefree-normalizes the product, and exports one PrincipalSupport generator over the target variable. The final primitive integerization preserves positive highest-degree coefficient for the univariate LCM.

LinearAffine admits only when a complete safe triangular affine elimination order is found. Constant nonzero pivots are allowed directly. Nonconstant pivots require a recorded guard hash matching the pivot factor. Execution revalidates the plan hash, pivot hash, and guard hash, clears denominators after substitution, exports only relations over the declared exported variables, and returns AlgorithmicHardCase when a stale/incomplete plan leaves a nonzero local-variable relation.

Guardian reviewer remediation:

- TargetUnivariate now includes child-message target-only relation generators in admission and execution and binds their message package hashes in the plan.
- Planner TargetUnivariate admission now requires variables subset `{T}`, not merely subset of all exported variables.
- TargetUnivariate and LinearAffine now reject block authorization and source hash tampering during execution.
- Replay checks recomputed package identity and context block identity before accepting a message.

Residual risks before later phases:

- P8a still owns TargetRelationSearch execution and membership verification.
- P8b-P9 still own execution for the remaining registered kernels.
- P7 replay methods are structural and do not close final public orchestration, exact-image acceptance, or MECH-013.
