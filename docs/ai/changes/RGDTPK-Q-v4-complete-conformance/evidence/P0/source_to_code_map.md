# P0 Source-To-Code Map

Status: initial map for P0 harness only.

| Source / Base Spec item | Current implementation area | P0 status |
|---|---|---|
| BS-R000 scoped source fidelity | `docs/ai/changes/RGDTPK-Q-v4-complete-conformance/*` | Source and amended contract imported. |
| BS-R010 invariant scan | `geosolver-core/scripts/audit_v4_conformance.py` | Static scan added; strict mode currently reports findings. |
| BS-R150 finite candidate-cover completion conditions | `BASE_SPEC.md`, `PLAN.md`, `REVIEWER_PROMPTS.md`, audit script | Completion conditions are represented but not implemented/verified. |
| Required file layout | `geosolver-core/src/**` | Audit checks production file presence. |
| Required source-named APIs | `geosolver-core/src/**` | Audit checks an initial symbol set and currently reports missing/renamed APIs. |
| Exact-image scope guard | `result/status.rs`, `solver/orchestrator.rs`, `verify/replay.rs` | Audit reports exact-image success paths; P16 must fix. |

P0 does not claim code conformance. It only establishes the harness that later phases must satisfy.
