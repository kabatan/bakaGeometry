Purpose: P18 reviewer request
Status: evidence, non-authoritative

# P18 Review Request

Review phase: P18 final finite candidate-cover conformance.

Use:
- `docs/ai/changes/RGDTPK-Q-v4-complete-conformance/BASE_SPEC.md`
- `docs/ai/changes/RGDTPK-Q-v4-complete-conformance/PLAN.md`
- `docs/ai/changes/RGDTPK-Q-v4-complete-conformance/SOURCE_MAP.md`
- `docs/ai/changes/RGDTPK-Q-v4-complete-conformance/REVIEWER_PROMPTS.md`, prompt RP-P18
- `docs/ai/changes/RGDTPK-Q-v4-complete-conformance/evidence/final/source_to_code_conformance_matrix.md`
- `docs/ai/changes/RGDTPK-Q-v4-complete-conformance/CLOSURE.md`
- `docs/ai/changes/RGDTPK-Q-v4-complete-conformance/evidence/final/final_commands.log`
- `docs/ai/changes/RGDTPK-Q-v4-complete-conformance/evidence/P0` through `evidence/P18`

Required checks:

1. Source-to-code matrix covers every in-scope source section and every Base Spec R-ID. Exact-image-only sections must be marked `OUT_OF_SCOPE`.
2. Strict static audit passes and does not hide required checks.
3. Fresh fmt/clippy/test/audit evidence exists.
4. All P0-P17 reviewer findings are resolved or non-blocking by scope.
5. No blocking QuestionDebt remains.
6. Each of the 16 BS-R150 finite candidate-cover completion conditions has code/evidence.
7. Final claim ceiling remains finite candidate-cover only.

Expected result format:

```text
Decision: PASS or FAIL
Scope reviewed:
Evidence inspected:
R-IDs / MECHs checked:
Findings:
Blocking items:
Residual risk:
```
