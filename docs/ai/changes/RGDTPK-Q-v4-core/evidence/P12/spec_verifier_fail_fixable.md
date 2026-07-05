# P12 Spec Verifier Result

The first P12 spec-verifier checkpoint returned `FAIL_FIXABLE`.

Blockers reported:

- P12 lacked required integration evidence proving support-producing cases return non-placeholder exact roots and decoded candidates.
- `verify_roots_and_candidates` validated present candidates but did not require exactly one decoded candidate per isolated real root, so omitted candidates could be accepted if the run certificate was rehashed consistently.
- P12 root/decode APIs were not wired into a production support-producing result path; `decode_candidates` appeared only in unit tests.

Required remediation:

- Add a support-producing integration/regression path that returns nonzero support, exact squarefree support, exact root isolation, and non-placeholder decoded candidates.
- Enforce one decoded candidate per isolated real root for candidate-cover success and reject duplicate or omitted candidates in replay.
- Add tests for omitted/duplicate candidate tamper and for support-producing candidate-cover output with exact roots/candidates.
