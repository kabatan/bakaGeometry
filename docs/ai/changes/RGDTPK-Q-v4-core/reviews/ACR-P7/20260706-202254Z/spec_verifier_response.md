# ACR-P7 Spec Verifier Response

RESULT: FAIL_FIXABLE

Blocking issue:

- The changed code added algebraic scoring fields, but the reviewed evidence did not include a
  generic large-footprint stress where arity/degree/monomial/height/projection-cost terms were
  necessary to select a separator that variable-count-only scoring would keep.

Inspected evidence:

- Graph weights and separator score fields included the required P7 cost terms.
- Selection was gated by width reduction and estimated-cost improvement.
- Unsplit decomposition diagnostics were surfaced into solver diagnostics.

Runtime evidence considered:

- Focused ACR-P7 tests passed.
- Graph tests passed.
- `cargo test --lib` passed before remediation with 243 tests.
- `cargo check` passed.
- Forbidden-marker scan had no matches.
- `git diff --check` reported only CRLF warnings.

Residual risks:

- Relation-duplication certificate cost was present, but not strongly stress-tested in the initial
  evidence.

Forbidden claims:

- Do not claim ACR-P7 PASS or closure from this response.
- Do not claim ACR-P8+, final closure, source fidelity, full acceptance, or candidate-cover
  readiness.

Remediation added after this response:

- Variable-count-only counterexample stress in graph and pipeline tests.
- Small high-cost retained-block diagnostic stress in graph and pipeline tests.
