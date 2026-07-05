RESULT: FAIL_FIXABLE

Exact blocker:

- Invalid semantic references are only checked for relation IDs, not semantic variable references. `RealConstraintEncoding` includes `slack_variables`, but `verify_semantic_references` only validates `encoded_relation_ids`: [semantic.rs](</Users/bakat/OneDrive/ドキュメント/bakaGeometry/geosolver-core/src/problem/semantic.rs:19>) and [semantic.rs](</Users/bakat/OneDrive/ドキュメント/bakaGeometry/geosolver-core/src/problem/semantic.rs:63>). `validate_input` passes only relation IDs into that check: [validate.rs](</Users/bakat/OneDrive/ドキュメント/bakaGeometry/geosolver-core/src/problem/validate.rs:38>). This narrows RGQ-012/Plan P2’s “invalid semantic references” requirement: [BASE_SPEC.md](</Users/bakat/OneDrive/ドキュメント/bakaGeometry/docs/ai/changes/RGDTPK-Q-v4-core/BASE_SPEC.md:185>), [PLAN.md](</Users/bakat/OneDrive/ドキュメント/bakaGeometry/docs/ai/changes/RGDTPK-Q-v4-core/PLAN.md:157>).

Checks that did not block:

- Invalid target and undeclared polynomial variables are rejected: [validate.rs](</Users/bakat/OneDrive/ドキュメント/bakaGeometry/geosolver-core/src/problem/validate.rs:13>), [validate.rs](</Users/bakat/OneDrive/ドキュメント/bakaGeometry/geosolver-core/src/problem/validate.rs:28>).
- Coordinate roles and branch/slack semantics are not used as rejection/dispatch criteria in the inspected validation path.
- Canonicalization calls denominator clearing, removes zero relations with diagnostics, detects nonzero constant contradiction, and carries semantic encodings forward: [canonicalize.rs](</Users/bakat/OneDrive/ドキュメント/bakaGeometry/geosolver-core/src/problem/canonicalize.rs:51>), [canonicalize.rs](</Users/bakat/OneDrive/ドキュメント/bakaGeometry/geosolver-core/src/problem/canonicalize.rs:59>), [canonicalize.rs](</Users/bakat/OneDrive/ドキュメント/bakaGeometry/geosolver-core/src/problem/canonicalize.rs:84>), [canonicalize.rs](</Users/bakat/OneDrive/ドキュメント/bakaGeometry/geosolver-core/src/problem/canonicalize.rs:90>).
- Public statuses remain closed; `TemporaryPipelineNotConnected` maps to public `ImplementationBug` plus diagnostic and is tracked for P14: [status.rs](</Users/bakat/OneDrive/ドキュメント/bakaGeometry/geosolver-core/src/result/status.rs:9>), [status.rs](</Users/bakat/OneDrive/ドキュメント/bakaGeometry/geosolver-core/src/result/status.rs:84>), [diagnostics.rs](</Users/bakat/OneDrive/ドキュメント/bakaGeometry/geosolver-core/src/result/diagnostics.rs:31>), [function_implementation_table.yaml](</Users/bakat/OneDrive/ドキュメント/bakaGeometry/docs/ai/changes/RGDTPK-Q-v4-core/evidence/P2/function_implementation_table.yaml:51>).

Forbidden claims until fixed:

- P2 closure passed.
- RGQ-012 semantic-reference validation is complete.
- Any R-ID is VERIFIED.
- Candidate-cover, exact-image, or algorithmic MECH closure.

Next action:

- Extend semantic validation to reject undeclared `slack_variables` and any other semantic variable references, add a targeted regression test, then rerun and archive P2 evidence.
