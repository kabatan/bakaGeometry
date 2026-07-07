# P11 Algorithm Evidence

Implemented/verified behavior:

- Production quotient handles are provenance-bound to authorized relations and reject coordinate-root
  or full-coordinate-RUR export flags.
- Production quotient input validates the standard target-relevant monomial basis against the
  authorized relations and verifies normal-form/action-column certificates.
- TargetActionKrylov builds deterministic block Krylov sequences from all unit basis probes, not a
  single start vector.
- Coverage is accepted only when the recovered recurrence equals the exact characteristic polynomial
  of the target action matrix.
- `verify_annihilator` recomputes the exact target action matrix, characteristic polynomial, and
  Cayley-Hamilton verification hash.
- The kernel emits `MessageRepresentation::QuotientAction` only after coverage and annihilator
  verification succeed.
- Message replay reconstructs the production quotient handle and replays sequence, coverage, and
  annihilator checks exactly.

Regression coverage:

- `verified_characteristic_support_coverage_is_accepted`
- `single_vector_krylov_undercoverage_is_rejected`
- `debug_explicit_handle_is_rejected_by_production_krylov_boundary`
- `p8c_quotient_handle_rejects_coordinate_root_or_rur_export`
- `p8c_undercovered_single_vector_recurrence_cannot_escape_as_support`
- `p11_action_krylov_replay_rejects_tampered_coverage_after_rehash`
- `fcr_action_rejects_injected_basis_or_column`
- `fcr_action_multivariate_quotient_no_target_relation`

Static audit:

- `audit_v4_conformance.py --phase P11 --strict` checks P11 files, required symbols, no-coordinate
  export boundary markers, coverage/annihilator replay markers, and the P11 forged-coverage
  regression marker.

