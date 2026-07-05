Guardian boundary review request: FCR-P8 actual DAG/block replay and certificate finalization.

Review scope:
- FCR-P8 only. Do not grant P13, final source-fidelity, exact-image, acceptance, or full production-readiness claims.
- Determine whether this phase can close under the approved Full Core Repair Base Spec and Plan.
- PASS only if replay_run_certificate verifies projection messages against actual TargetProjectionDAG blocks and not a synthetic all-relations block, and CoreRunCertificate binds actual DAG replay evidence.

Source requirements:
- docs/ai/changes/RGDTPK-Q-v4-core/FULL_CORE_REPAIR_BASE_SPEC.md
  - FCR-010: real DAG/block replay is mandatory for core readiness.
  - FCR-011/FCR-012 as affected by support/root/candidate binding and nonfinite limits.
- docs/ai/changes/RGDTPK-Q-v4-core/FULL_CORE_REPAIR_PLAN.md
  - FCR-P8.
- docs/ai/changes/RGDTPK-Q-v4-core/FULL_CORE_REPAIR_REVIEWER_PROMPTS.md
  - FCR-P8 reviewer prompt.
- docs/ai/changes/RGDTPK-Q-v4-core/BASE_SPEC.md
  - MECH-010 certificates, verification, and replay.

Changed code to inspect:
- geosolver-core/src/verify/mod.rs
- geosolver-core/src/verify/replay.rs
- geosolver-core/src/verify/run_certificate.rs
- geosolver-core/src/solver/pipeline.rs
- geosolver-core/tests/p3_public_pipeline_integration.rs

Evidence to inspect:
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/FCR-P8/commands.txt
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/FCR-P8/command_outputs.txt
- docs/ai/changes/RGDTPK-Q-v4-core/evidence/FCR-P8/actual_dag_replay_static_scan.txt

Hashes:
- FULL_CORE_REPAIR_BASE_SPEC.md: 1f73bd7a26b2566b08f53d31b14785110466e8ae6170b6769d1af17d50a36047
- FULL_CORE_REPAIR_PLAN.md: b836599108c1db4a4e828e672780e5074049045fe5fa590e94252cdc91850bb9
- FULL_CORE_SOURCE_SPEC_COMPLIANCE_MAP.md: 892b75221a3f724ab864e0214e3a0559e99331cda0b8fba4976f5a21372f08ec
- FULL_CORE_REPAIR_REVIEWER_PROMPTS.md: 28075ea55b02213c4b38f5304735f87b163ab7eddb15121eefe072878a1f387b
- BASE_SPEC.md: dfd6832c211af0928270cfbaa98dcf73e50cd37e6155534703b1217636038f6c
- REVIEW_ARCHIVE_SCHEMA.md: d3db02e4abc8cc3065e2be8e82614b3703266ad0c0ce6cf3c0ce85ad662d364c
- REVIEW_SUMMARY_SCHEMA.yaml: e078bd86caf3efabfe056da1de9b80194e79e4756bdd3af3bacb938cc2f206be
- schemas/evidence_manifest.schema.yaml: 71b1285a18392f61868b947628aac0a591de29f975d3df5a69ae3ac44217af76
- verify/replay.rs: 25e7f77bbebc4e3ff25eab209df944f7247e4e281417fa70e401af08cfd47fca
- verify/run_certificate.rs: 20384cf8e276046ebf772c9a182761a55589882a812b0772f00560e62d5ec902
- solver/pipeline.rs: 7683cf2f40283f9ec52a766eaba5b65b96f2106a67c96d7f6f66b8d816c92a7c
- verify/mod.rs: 1d4b59c28d52b09aee15eb071531e0e9b22eed5345e349f5c242f3ea0e8e8d6f
- p3_public_pipeline_integration.rs: 5dc2c682cb90bfffe99e6e3bf63ab6aece9fff8658a792b1d13bd229b43ebee7

Fresh command evidence:
- cargo fmt: PASS
- cargo check: PASS
- cargo test --lib fcr_p8_: PASS, 2 passed
- cargo test --test p3_public_pipeline_integration: PASS, 1 passed
- cargo test: PASS, 207 lib tests plus all listed integrations
- git diff --check: PASS, CRLF warnings only

Specific checks requested:
1. Confirm verify/replay.rs is production-exported through verify/mod.rs and not cfg(test)-only.
2. Confirm replay_run_certificate reconstructs actual TargetProjectionDAG from the compressed system and compares cert.target_projection_dag_hash to actual_dag.dag_hash.
3. Confirm replay_run_certificate recomputes final DAG replay evidence from actual DAG blocks and rejects mismatch with both final_dag_replay_evidence_hash and final_dag_replay_evidence.
4. Confirm message dependencies come from actual DAG child edges, not relation-hash inference or synthetic all-relations blocks.
5. Confirm message verification uses actual_dag.blocks and rejects missing/duplicate messages, unauthorized source relations, non-edge child dependencies, and block authorization evidence tamper.
6. Confirm CoreRunCertificate/FinalDagReplayEvidence binds block ids/authorization/relation ids/relation hashes/child edges/message hashes/plan hashes/support/root/candidate hashes.
7. Confirm mandatory FCR-P8 tamper coverage exists for relation authorization removal/tamper, relation outside block, child not on DAG edge, plan hash tamper, package hash tamper, support tamper, and root/candidate omission or duplication.
8. Confirm any remaining synthetic all-relations references are confined to cfg(test) negative tests or test-only helpers and are not accepted production replay paths.

Return:
- RESULT: PASS or FAIL
- Blocking findings if any, with file/line references.
- Residual risks and claim ceiling.
- Do not mark any R-ID VERIFIED.
