# P9 Notes

P9 implements `RegularChainProjectionKernel` and `NormTraceProjectionKernel` for algebraically detected applicable structures. The implemented admissions are intentionally not broad catch-alls: unsupported systems are declined so that Universal and later composition remain responsible for generic fallback behavior.

The first P9 Guardian review returned `FAIL_FIXABLE` and did not close P9 or `MECH-007`. That review identified two overclaim risks against `PRIMITIVE_SCOPE_LEDGER.md`: regular-chain guard/component semantics were too narrow, and norm/trace tower detection was single-variable only.

The remediation expands the regular-chain helper used by P9 from a single-chain helper into a component-DAG builder for the admitted structures. Component chains bind relation hashes, main/free variables, union semantics, and guards. `RegularChainProjectionKernel` now passes `CompressedSystemQ.guards` into planning and execution, and the support trace/certificate hash changes when guards are deleted or mutated.

The remediation also expands norm/trace planning from a single algebraic variable into `TowerPlanDescription` with ordered `TowerStep` entries. The P9 kernel detects tower plans by polynomial form, computes the target relation by eliminating tower variables with sparse resultants, verifies the relation by exact recomputation, checks exported-only output, and emits `NormTraceTower`.

P9 still does not claim performance readiness, final support composition, root isolation/decode, exact-image semantics, public orchestration, nonfinite certification, final acceptance, or any R-ID as VERIFIED. `MECH-007` closure is claimed only after a P9 Guardian PASS archive.
