# Candidate-Cover Cost Trace And Static Scan Summary

Status: candidate-cover cost/static evidence.

## Cost Trace

The public candidate-cover pipeline records:

- total variable count;
- total relation count;
- total monomial count;
- maximum total degree;
- coefficient height;
- maximum block and separator width;
- final support degree;
- certificate size;
- per-block projection traces;
- composition trace;
- verification trace.

Fresh red-team and P15/FCR acceptance tests assert cost trace fields on support-producing paths.
Bounded hard/failure cases retain failure cost traces and are not converted to nonfinite success.

## Static Scans

Commands run for the active candidate-cover closure:

```text
rg -n "problem_id|expected_answer|expected[_ -]?root|fixture|golden|answer dispatch|dispatch.*answer" geosolver-core/src -S
rg -n "QE|CAD|RCF|RUR|full coordinate|coordinate solution|coordinate root|CoordinateRUR" geosolver-core/src -S
rg -n "unimplemented!|todo!|TODO|placeholder|dummy|fake|stub|Unsupported|unsupported|NotYetImplemented" geosolver-core/src -S
```

Observed classification after final verification:

- expected-answer/problem-id scan: only `CoreInvariantFlags` and final invariant evidence field names;
- QE/CAD/RUR/coordinate scan: only rejection/error messages forbidding coordinate roots or RUR export;
- TODO/stub/unsupported scan: no matches in `geosolver-core/src`.

## Runtime Flags Boundary

`CoreInvariantFlags` are run-local replay evidence and are not treated as repo-wide static proof by
themselves. The static scans above are bound to this closure by command text, observed
classification, final verification commands, and the final commit containing this document.

## No-F4 Claim

Candidate-cover readiness does not rely on production F4. Existing Groebner-backed batch helpers
remain non-production/test-only and production dispatch rejects the non-production strategy.
