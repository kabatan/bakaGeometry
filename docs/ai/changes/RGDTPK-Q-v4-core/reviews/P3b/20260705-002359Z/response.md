FAIL_FIXABLE

**Reviewed R-IDs and MECHs**
Reviewed `RGQ-010`, `RGQ-019`, `RGQ-020`, `RGQ-025`, Appendix A §10.3-§10.7, Plan P3b/general execution rules, and P3b MECH scope: continues `MECH-001`, starts `MECH-006`. No R-ID is marked verified.

**Algorithmic Sufficiency Judgment**
Insufficient but fixable.

The prior required fix is mostly addressed: traces include `pivot_columns`, loops compare pivot profiles, sample buffers reset when profile changes, and regression tests cover same-rank/different-pivot and inhomogeneous stable-suffix reconstruction.

Remaining blocker:
- `solve_homogeneous_modular` and `solve_inhomogeneous_modular` still reconstruct when the loop exits by exhausting `max_primes` before `stable_rank >= stable_rank_after`.
- This violates the required rank-profile stabilization before CRT/reconstruction handoff.

**Required Fixes**
1. Track whether `stable_rank >= stable_rank_after.max(1)` was achieved.
2. Return no reconstructed basis/solution candidates, or an explicit non-handoff result, when `max_primes` is exhausted before stability.
3. Add homogeneous and inhomogeneous regressions where profiles do not stabilize before `max_primes`, asserting no CRT/reconstruction candidate is produced.
