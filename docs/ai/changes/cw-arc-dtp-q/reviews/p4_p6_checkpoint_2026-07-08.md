# P4-P6 Checkpoint Review

Purpose: record scoped Guardian review results for the P4-P6 implementation checkpoint.
Status: review record.
Authority: review evidence only. This file does not verify any V3 requirement.

Date: 2026-07-08.

## Scope

The reviewed scope is P4 through P6 of `CW-ARC-DTP-Q-FULL-V3`:

- P4 certificate windows, modular reduction, residual oracle, and ResidualCyclic prime/support data-flow.
- P5 fixed proof schedule, proof learning, obstruction expansion, and bounded behavior for `max_window_degree = None`.
- P6 candidate normalization, modular merge, CRT/rational reconstruction, ranking, and factor schedule.

Out of scope:

- P7 and later candidate route closure.
- P15 exact no-target eliminant replay.
- P16 exact real-image replay.
- Final V3 completion or source-faithfulness claims.

## Review Results

`spec_verifier`: PASS after fixable findings were addressed.

Fixes addressed before the passing spec review:

- Single-prime modular candidates no longer populate `reconstructed`; they remain modular-only.
- Modular candidate merge now requires route-family correspondence evidence and preserves duplicate-prime alternatives.
- ResidualCyclic now solves `M_p,W u = N_p,d c` and records active multiplier support from nonzero solution entries.
- ResidualCyclic now explicitly rejects denominator-bad primes across equations, guard certificates, compression replay data, and target-power vectors.
- Later quality-review fixes were re-reviewed by `spec_verifier` and passed.

`quality_reviewer`: PASS after two fixable findings were addressed.

Fixes addressed before the passing quality review:

- `max_window_degree = None` now passes the effective finite bound into fixed proof obstruction expansion.
- Duplicate-prime alternatives are preserved while distinct-prime combinations still reach CRT/rational reconstruction.

`guardian_boundary_reviewer`: PASS.

Boundary accepted:

- The checkpoint claim is limited to scoped P4-P6 implementation that was locally tested and reviewed.
- Evidence cites current production/test anchors.
- Reviewer PASS is a review-process record, not executable proof.

## Local Evidence

Commands run by the main agent after the final fixes:

```text
cargo fmt --check
cargo test
git diff --check
```

Result:

```text
All passed. cargo test covered 121 tests.
```

## Post-Checkpoint P5/P6 Blocker-Fix Addendum

Source:
- `C:\Users\bakat\Downloads\CW_ARC_DTP_Q_P5_P6_BLOCKER_FIX_INSTRUCTIONS.md`
- SHA-256: `460EACD1C644E32E62094ECBF13E7D111346F23FF430BA151EDE8A6314557C32`

Base Spec admission:
- Local delta: `docs/ai/changes/cw-arc-dtp-q/P5_P6_BLOCKER_FIX_BASE_SPEC.md`
- `guardian_boundary_reviewer`: PASS after fixable documentation omissions were corrected.

Implemented blocker fixes:
- P5: lazy unbounded fair proof schedule plus explicit bounded prefix; no hidden `max_proof_weight.unwrap_or(6)` or solver-side `max_window_degree.unwrap_or(6)` cap; missing proof bound fail-closes with `resource:unbounded_proof_requires_bound`.
- P6: status-bearing exact Q squarefree factorization with `Complete`, `Partial`, and `ResourceFailure`; Kronecker-style factor search splits reducible squarefree polynomials without rational roots; product reconstruction and exact division are checked.
- P6: factor schedule surfaces factorization status and every factor trial still requires its own fixed exact proof.
- P6: origin evidence is carried and merged for same primitive reconstructed support; origin count affects trial order only and is not certificate authority.
- P3 regression preservation: description-only `ComponentUnionLcm` remains a design gap, and no-target-eliminant solver output remains a design gap until P15 replay.

Local evidence:

```text
cargo test
```

Result:

```text
All passed. cargo test covered 132 tests.
```

Reviewer status for this addendum:
- `spec_verifier`: PASS.
- `quality_reviewer`: PASS after ComponentUnionLcm missing-source behavior was corrected to return `CertificateDesignGap`.
- `guardian_boundary_reviewer`: PASS after an initial FAIL_FIXABLE. The fix removed hidden `max_window_degree.unwrap_or(...)` caps from fallback/repair paths and added fail-closed tests for omitted window bounds.

## Claim Boundary

This review supports only the scoped P4-P6 checkpoint closure. It does not mark any R-ID verified, does not close P7+, and does not support final V3 completion, source-faithfulness, production-safety, readiness, or acceptance-complete claims.
