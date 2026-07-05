# Full Core Invariant and Static-Scan Binding

Status: pending FCR-P12 final closure.

This file is a required final-closure artifact. It is not evidence yet.

`CoreInvariantFlags` must not be treated as free booleans. Final closure must bind each invariant
flag to fresh static scans and to dynamic replay/tamper or semantic evidence where applicable.

Required binding:

| Closure invariant | Required static evidence | Required dynamic evidence | Status |
| --- | --- | --- | --- |
| no geometry dispatch | scan production source for geometry-family dispatch terms | red-team inputs are algebraic-only and do not use geometry labels | pending |
| no problem-id/fixture/expected-answer dispatch | scan source and tests for problem id, fixture, expected answer, official answer terms | variable/relation/order mutation does not change algorithmic path | pending |
| no hidden fallback | scan for fallback/temporary/stub/nonproduction markers | tampering plan/DAG/message/certificate causes replay or execution failure | pending |
| no QE/CAD/RCF/full-coordinate fallback | scan source for QE/CAD/RCF, coordinate roots, full coordinate RUR, solve-all-coordinates | success paths expose target candidates only, not coordinate solutions | pending |
| actual DAG replay | scan/review replay path and certificate fields | replay rejects DAG, block authorization, child message, plan, support, root, and candidate tamper | pending |

Final closure must fail if this file remains pending, if scans are only summarized without command
outputs, or if static scans are used as a substitute for dynamic replay/red-team evidence.
