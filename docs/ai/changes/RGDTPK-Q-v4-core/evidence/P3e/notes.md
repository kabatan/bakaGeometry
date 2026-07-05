# P3e Evidence Notes

P3e implements target-relevant quotient/action handles and verified characteristic support coverage primitives. It does not implement the full ActionKrylov kernel integration.

Important proof boundary:

- `TargetQuotientHandle` exposes no coordinate roots or full coordinate RUR APIs.
- `certify_krylov_coverage` accepts only `VerifiedCharacteristicSupportCoverage`.
- The target-action matrix is materialized column by column using `handle.multiply_by_variable`.
- Every target-action column is checked against exact `normal_form(T * basis_j)`.
- The characteristic polynomial is computed exactly over Q and Cayley-Hamilton is checked exactly as a matrix identity.
- A recurrence recovered from weak Krylov data is not accepted unless it equals the verified characteristic polynomial.
- `verify_annihilator` accepts only the verified characteristic support polynomial.

Negative coverage includes a quotient/action system with characteristic polynomial `T^2 - 3T + 2` where a single eigenvector Krylov sequence recovers only `T - 1`; certification rejects it with `CertificateDesignGap` instead of returning a candidate relation.
