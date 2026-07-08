# P3 Re-review Blocker Fixes

Purpose: scoped evidence for the pre-P4 P3 re-review blocker checklist.
Status: scoped blocker fixes implemented; spec, quality, and boundary reviews passed.
Authority: evidence only. The checklist from the user, `BASE_SPEC.md`, and `PLAN.md` control correctness.

Date: 2026-07-08.

## Review Results

- `spec_verifier`: PASS after inspecting production and test files directly.
- `quality_reviewer`: PASS after `non_simplification_manifests.md` no-target replay claims were downgraded.
- `guardian_boundary_reviewer`: PASS for narrow pre-P4 blocker closure.

Local commands:

```text
cargo fmt --check
git diff --check
cargo test
```

Result:

```text
All passed; cargo test covered 109 tests.
```

## Checklist Mapping

### ComponentUnionLcm

Required behavior:
- Do not verify a component-union certificate from a nonempty description string alone.
- Return `CertificateDesignGap` unless source data is replay-verifiable.

Production source:
- `src/verifier.rs`, `verify_composite_target_certificate`, `CompositeRule::ComponentUnionLcm` branch.

Executable test:
- `tests/verifier_tests.rs::component_union_lcm_without_replay_source_is_design_gap`

Current scoped claim:
- The verifier checks child certificates and lcm support, rejects missing source, and returns `CertificateDesignGap` for description-only source data.

### NoTargetEliminant

Required behavior:
- Do not return `SolverStatus::CertifiedNoNonzeroTargetEliminant` while `verify_certificate` only returns `CertificateDesignGap`.

Production source:
- `src/solver.rs`, `CompleteFallbackResult::CertifiedNoTargetEliminant` branch.
- `src/verifier.rs`, `verify_no_target_eliminant_certificate`.

Executable tests:
- `tests/fallback_elimination_solver_tests.rs::solver_no_target_eliminant_is_design_gap_until_p15_replay`
- `tests/verifier_tests.rs::no_target_eliminant_is_p15_design_gap_not_monomial_acceptance`

Current scoped claim:
- Top-level solver returns `SolverStatus::CertificateDesignGap` with no success certificate for this path until P15 exact elimination-zero replay exists.

### Evidence Consistency

Required behavior:
- Reviewers must inspect actual test files and production source, not this evidence file as proof.
- Claims without executable tests or production citations are downgraded.

Production/test citations above are the intended review anchors.

Downgraded historical evidence:
- `p5_target_certificate_evidence.md` no longer claims verified ComponentUnionLcm acceptance from a marker string.
- `p11_complete_fallback_evidence.md` no longer claims top-level no-target solver success before P15 replay.
- `tamper_matrix.md` now records ComponentUnionLcm/no-target design-gap behavior.

## Claim Ceiling

This file supports only the scoped blocker-fix claim. It does not mark any R-ID verified, does not close P4+, and does not support final V3 completion claims.
