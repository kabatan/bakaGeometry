Please re-review P8b after remediation. Do not edit files.

Workspace: C:\Users\bakat\OneDrive\ドキュメント\bakaGeometry

Prior FAIL_FIXABLE findings and remediation:

1. SparseResultantProjectionKernel ignored child message relations.
   Remediation: both admission/planning/execution now collect local relations plus child message relation generators, bind child package hashes into KernelExecutionPlan.child_message_hashes, verify child hash binding at execution, and include child-required/tamper regression `p8b_sparse_resultant_consumes_child_message_relations_and_rejects_tamper`.

2. SpecializationInterpolationKernel generated samples by specializing a precomputed exported Groebner generator rather than executing a declared inner target-only kernel.
   Remediation: sample generation now uses `execute_inner_target_only_kernel`, which constructs each specialized target-only context and executes admitted TargetRelationSearch via `admit_target_relation_search` plus `execute_target_relation_search`; local Groebner/elimination remains only final exact Q verification. Added child-required/tamper regression `p8b_specialization_interpolation_consumes_child_messages_and_rejects_tamper`.

Fresh evidence after remediation:

- cargo fmt --manifest-path geosolver-core/Cargo.toml --check: pass
- cargo test --manifest-path geosolver-core/Cargo.toml p8b_ -- --nocapture: pass, 7 passed
- cargo test --manifest-path geosolver-core/Cargo.toml interpolation -- --nocapture: pass, 9 passed
- cargo test --manifest-path geosolver-core/Cargo.toml resultant -- --nocapture: pass, 11 passed
- cargo test --manifest-path geosolver-core/Cargo.toml p6_ -- --nocapture: pass, 3 passed
- cargo test --manifest-path geosolver-core/Cargo.toml p7_ -- --nocapture: pass, 9 passed
- cargo test --manifest-path geosolver-core/Cargo.toml p8a_ -- --nocapture: pass, 6 passed
- cargo test --manifest-path geosolver-core/Cargo.toml: pass, 136 passed
- git diff --check: exit 0, CRLF warnings only

Inspect:

- geosolver-core/src/kernels/sparse_resultant.rs
- geosolver-core/src/kernels/specialization_interpolation.rs
- geosolver-core/src/algebra/interpolation.rs
- geosolver-core/src/algebra/resultant.rs
- geosolver-core/src/algebra/elimination.rs
- geosolver-core/src/planner/admission.rs
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/P8b/*
- docs/ai/changes/RGDTPK-Q-v4-core/PRIMITIVE_SCOPE_LEDGER.md

Review P8b only. Return RESULT PASS/FAIL with findings and exact PASS scope. Do not claim P8 umbrella, MECH-007 closure, P8c/P8d/P9, replay closure, final composition, exact-image semantics, public orchestration, or acceptance readiness.
