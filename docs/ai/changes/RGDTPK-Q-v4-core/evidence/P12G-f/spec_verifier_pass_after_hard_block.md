RESULT: PASS

Blocking findings: none.

Spec verifier inspected the current working tree against P12G-RGQ-080 and P12G-f/P12G-h after the
hard-block fix. `require_final_claim_dag_replay_evidence` structurally checks hash-bound DAG
evidence and then always returns a `CertificateDesignGap` P14 blocker in P12G. The actual-block
replay helper is runtime-callable within the crate via `pub(crate)` and is no longer test-only. The
required P12G-f tests are present and updated to assert blocker behavior.

Runtime evidence cited by reviewer:

- `cargo test --manifest-path geosolver-core\Cargo.toml p12g_ -- --nocapture`: pass, 20 unit tests
  plus 1 integration test.
- `git diff --check`: pass, CRLF warnings only.
