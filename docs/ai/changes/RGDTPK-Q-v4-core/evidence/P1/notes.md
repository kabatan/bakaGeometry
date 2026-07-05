---
purpose: phase-evidence-notes
status: active
authority: non-authoritative-evidence
---

# P1 Notes

P1 implements the Rust crate scaffold, public API shell, exact arithmetic base types, deterministic hashing, sparse polynomial normalization, univariate gcd/squarefree foundations, matrix shape/density/hash helpers, and exact rational intervals.

The public `SolverStatus` enum is closed to Appendix A variants. The pre-P14 API path uses `SolverStatus::ImplementationBug` plus diagnostic `TemporaryPipelineNotConnected`; this is recorded in the function implementation table and must be removed before P14 closes.

P1 does not claim problem validation semantics, projection planning, kernel execution, certificate verification, candidate-cover readiness, exact-image readiness, or any algorithmic MECH closure.
