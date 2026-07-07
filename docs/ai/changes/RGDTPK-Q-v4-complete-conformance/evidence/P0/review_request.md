# P0 Review Request

Reviewer prompt: RP-P0 from `REVIEWER_PROMPTS.md`.

Scope:

- `docs/ai/changes/RGDTPK-Q-v4-complete-conformance/**`
- `geosolver-core/scripts/audit_v4_conformance.py`
- `docs/ai/sources/geosolver_core_r_gdtpk_q_algorithm_spec_v4.md`

Requested decision: PASS / FAIL / NEEDS_MORE_EVIDENCE for P0 only.

Review checks:

1. Base Spec and Plan are present in the repo.
2. Source path and blob sha are recorded.
3. Audit script scans production files and is not written to suppress known gaps.
4. Current gap inventory honestly lists known noncompliance.
5. Implementation authority was explicitly granted by the user before P0 edits.

Expected audit behavior:

```text
python geosolver-core/scripts/audit_v4_conformance.py --strict
```

currently exits nonzero and reports implementation findings. P0 may still pass if the harness is
strict and honest.

Implementation authority evidence:

```text
docs/ai/changes/RGDTPK-Q-v4-complete-conformance/evidence/P0/implementation_authority.md
```
