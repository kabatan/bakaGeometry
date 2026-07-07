# P0 Current Gap Inventory

Status: initial static inventory from `geosolver-core/scripts/audit_v4_conformance.py`.

Scope: finite candidate-cover repair only. Exact-image equality/classification is out of scope, but
any exposed exact-image request must be guarded and must not return exact-image success.

## Static Audit Findings

The first P0 strict audit found 12 blocking items for later phases:

| Code | Location | Current finding | Required later action |
|---|---|---|---|
| `missing_required_symbol` | `preprocess/compression.rs` | `compress_system` is not visible in production code. | Add the source-named production API or amend mapping if an equivalent source API exists. |
| `missing_required_symbol` | `planner/planner.rs` | `plan_projection_messages` is not visible in production code. | Add the source-named production API or amend mapping if an equivalent source API exists. |
| `missing_required_symbol` | `solver/pipeline.rs` | `run_pipeline` is not visible in production code. | Add the source-named production API or amend mapping if an equivalent source API exists. |
| `test_only_required_symbol` | `solver/orchestrator.rs` | `solve_target` is not visible in production code. | Ensure the public/source solve function is production-visible at the orchestrator/API boundary. |
| `possible_coordinate_rur_path` | `algebra/quotient.rs` | RUR/coordinate-root diagnostic strings remain in production code. | Inspect whether these are defensive diagnostics or evidence of a forbidden path; remove or justify through executable audit policy. |
| `possible_coordinate_rur_path` | `kernels/action_krylov.rs` | RUR/coordinate-root diagnostic string remains in production code. | Inspect whether this is defensive diagnostic text or a forbidden path; remove or justify through executable audit policy. |
| `kernel_not_ready_error` | `kernels/traits.rs` | A production `kernel_not_ready_error` helper remains reachable by name. | Remove from production route or prove unreachable and adapt audit only with reviewer approval. |
| `exact_image_success_in_scoped_repair` | `result/status.rs` | `CertifiedExactTargetImage` status exists in scoped repair. | Replace exposed exact-image success path with explicit out-of-scope guard/status for this repair. |
| `exact_image_success_in_scoped_repair` | `solver/orchestrator.rs` | exact-image mode can finalize `CertifiedExactTargetImage`. | Replace exact-image branch with required `ExactImageOutOfScope` guard. |
| `exact_image_success_in_scoped_repair` | `verify/replay.rs` | replay accepts exact-image success statuses. | Make replay reject or diagnose exact-image success in this scoped repair. |
| `descartes_delegates_to_sturm` | `algebra/real_root.rs` | `isolate_real_roots_descartes` delegates to Sturm. | Implement distinct exact Descartes/Vincent or remove Descartes as an advertised method until implemented. |

## Known Gap Categories Required by P0

- F4 test-only or non-production: not found by this first static audit, but P4 reviewer must inspect
  `algebra/f4.rs` directly.
- Descartes alias to Sturm: present and blocking for P15.
- Universal inner strategy list mismatch with source section 20.4: not established by this first
  static audit; P12 must inspect and test it.
- Sparse resultant two-polynomial-only limitation: not established by this first static audit; P10
  must inspect support/template generality.
- Regular-chain simplification: not established by this first static audit; P13 must inspect
  component/guard/projection semantics.
- Nonfinite small witness limitation: not established by this first static audit; P14 must inspect
  nonfinite certificate replay and evidence.
- Composition bounded heuristic limitation: not established by this first static audit; P14 must
  inspect final support composition and proof.
- Exact-image success/no-op paths: present and blocking for P16 scope guard.

No item in this inventory is marked fixed or verified.
