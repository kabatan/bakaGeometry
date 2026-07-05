RESULT: PASS

Blockers: none.

The reset file satisfies FCR-P0A as scoped:

- It is explicitly an active execution/runtime constraint, not a confession or retrospective.
- It acknowledges all listed hazards and expands them into concrete prevention rules.
- It includes a `PlanDefect / AlgorithmDefect Trigger Table`.
- It requires every later FCR handoff to state trigger status.
- It identifies concrete prior failure modes to avoid, including gate/review substitution, narrow
  algebraic slices, hidden fallback, plan-time execution, helper/module-test substitution,
  decorative certificates, and overclaiming exact-image/source fidelity.
- It does not say limitations are acceptable because documented; it requires partial paths to be
  generalized, quarantined, removed, or reported as defects.

Forbidden claims remain:

- `CANDIDATE_COVER_CORE_READY`
- `EXACT_IMAGE_CORE_READY`
- `SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC`
- `RGDTPK_Q_V4_ACCEPTANCE_COMPLETE`
- P13/P14/P15/P16 readiness

Next action: proceed only to the next scoped FCR phase under the reset constraint and require each
later handoff to report `PlanDefect` / `AlgorithmDefect` trigger status. No R-IDs marked VERIFIED.
