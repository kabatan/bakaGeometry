Purpose: P17 algorithm evidence
Status: evidence, non-authoritative
Authority: BASE_SPEC.md and original source remain authoritative

# P17 Algorithm Evidence

## Implemented Behavior

1. `SolverOptions` now carries `finite_candidate_cover_mode` with default `true`.
2. `hash_solver_options` includes `finite_candidate_cover_mode`, so mode changes alter the run certificate hash.
3. `solve_with_context` executes the finite candidate-cover pipeline in the source order:
   - validate,
   - canonicalize,
   - pre-kernel compression,
   - graph/DAG construction,
   - planning,
   - local projection execution,
   - projection message verification,
   - message composition,
   - support construction,
   - global support verification,
   - squarefree/root isolation/decode,
   - core certificate,
   - result finalization.
4. `verify_global_support` is called before `step_roots`; root isolation/decode is not reached when support verification fails.
5. `step_failure_cost_trace` records completed stage data and the failure step in `GlobalCostTrace`.
6. `step_cost_trace` populates the source cost-compression fields: total variable/relation/monomial/degree/height parameters, block traces, composition/verification traces, final support degree, and certificate size.
7. Exact-image requests are scope guarded. The solver returns `CertificateDesignGap`, includes `ExactImageOutOfScope`, and preserves support/root/candidate artifacts as unfiltered candidate-cover evidence.

## Behavior Evidence

Fresh commands:
- `cargo fmt --manifest-path geosolver-core\Cargo.toml --check`: PASS.
- `python geosolver-core\scripts\audit_v4_conformance.py --phase P17 --strict`: 0 findings.
- `cargo test --manifest-path geosolver-core\Cargo.toml --lib run_certificate -- --nocapture`: 6 passed.
- `cargo test --manifest-path geosolver-core\Cargo.toml --lib replay -- --nocapture`: 27 passed.
- `cargo test --manifest-path geosolver-core\Cargo.toml --lib pipeline -- --nocapture`: 13 passed.
- `cargo test --manifest-path geosolver-core\Cargo.toml --test p14_full_pipeline_integration -- --nocapture`: 10 passed.
- `cargo test --manifest-path geosolver-core\Cargo.toml --test p15_acceptance_stress -- --nocapture`: 6 passed.
- `cargo test --manifest-path geosolver-core\Cargo.toml --no-run`: PASS.

## Negative Shortcut Audit

P17 audit found 0 issues for:
- missing stage functions,
- stage reordering,
- root isolation before support verification,
- missing failure cost trace,
- missing cost fields,
- missing solver-options/candidate/root/support certificate binding.

Prior phase audits still cover:
- no Descartes-to-Sturm alias,
- no exact-image success in finite candidate-cover scope,
- no Universal hidden internal strategy outside the approved strategy list,
- no fixed small heuristic for composed eliminant proof.

## Claim Ceiling

This evidence supports P17 completion only after reviewer PASS.
It does not by itself authorize final `FINITE_CANDIDATE_COVER_COMPLETE` or `SOURCE_FAITHFUL_TO_V4_FINITE_CANDIDATE_COVER_LAYER` claims; those remain P18 closure claims requiring final reviewers.
