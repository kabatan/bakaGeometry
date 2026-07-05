---
purpose: phase-evidence-notes
status: active
authority: non-authoritative-evidence
---

# P2 Notes

P2 implements problem input records, semantic encoding records, validation, canonicalization, solver context records, public status/error mapping, diagnostics, cost trace records, and result finalization helpers.

Validation does not inspect geometry names and does not reject because a variable has a coordinate role or because branch/slack semantics are present. Semantic encodings are checked for encoded relation IDs and declared semantic variable references, including `slack_variables`.

Canonicalization clears denominators through primitive normalization, removes zero relations with diagnostics, preserves semantic encodings, and returns `InvalidInput` for a nonzero constant contradiction.

The `TemporaryPipelineNotConnected` path remains reachable because P14 has not connected the full orchestrator. It is still mapped to public `ImplementationBug` with a diagnostic and remains tracked for P14 removal.
