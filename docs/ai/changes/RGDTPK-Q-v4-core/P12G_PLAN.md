# P12G Plan — Generality Remediation Before P13/P14

This plan is inserted after P12 and before P13. It is a mandatory remediation phase group. P13 and P14 are blocked until P12G-a through P12G-h have PASS reviews.

The goal is not to add more documentation. The goal is to verify and repair whether the current implementation is a real R-GDTPK general algebraic target-direct algorithm rather than a collection of narrow kernel-shaped slices.

---

## P12G-a — Current HEAD rebind, closure repair, and direct algorithm inventory

**Supports:** P12G-RGQ-073, P12G-RGQ-074, P12G-RGQ-082, P12G-RGQ-085.

### Files to inspect

```text
docs/ai/ACTIVE_CONTEXT.md
docs/ai/changes/RGDTPK-Q-v4-core/CLOSURE.md
docs/ai/changes/RGDTPK-Q-v4-core/PRIMITIVE_SCOPE_LEDGER.md
docs/ai/changes/RGDTPK-Q-v4-core/reviews/P6 through P12
geosolver-core/src/kernels/*
geosolver-core/src/planner/*
geosolver-core/src/compose/*
geosolver-core/src/verify/*
geosolver-core/src/roots/*
geosolver-core/src/solver/*
```

### Required tasks

1. Record current `git rev-parse --verify HEAD`.
2. Rerun:
   ```text
   cargo fmt --manifest-path geosolver-core/Cargo.toml -- --check
   cargo test --manifest-path geosolver-core/Cargo.toml -- --nocapture
   git diff --check
   ```
3. Update `CLOSURE.md` so it no longer describes P5R as the current state.
4. Create `docs/ai/changes/RGDTPK-Q-v4-core/P12G_DIRECT_ALGORITHM_INVENTORY.md`.
5. For each kernel and solver stage, record:
   ```text
   - actual production input class
   - actual production output class
   - whether it can produce target/separator support without pre-existing target-only relation
   - whether it uses local Groebner/resultant/interpolation/action during plan/admission
   - exact certificate route
   - limitation and whether limitation is honest or overclaimed
   ```
6. Mark any previously closed phase claim that depends on an overclaimed kernel as `REQUIRES_P12G_RECHECK`.

### Forbidden shortcuts

```text
- Do not rely on review_summary.yaml alone.
- Do not say "P13/P14 will fix this" unless the Base Spec amendment explicitly allows deferral.
- Do not hide stale CLOSURE wording as historical.
```

### Required evidence

```text
evidence/P12G-a/commands.txt
evidence/P12G-a/command_outputs.txt
evidence/P12G-a/direct_algorithm_inventory.md
evidence/P12G-a/claim_consistency_matrix.yaml
reviews/P12G-a/<timestamp>/{prompt.md,response.md,review_summary.yaml,evidence_manifest.yaml}
```

---

## P12G-b — TargetActionKrylov generalization or honest demotion

**Supports:** P12G-RGQ-075.

### Files to modify

```text
geosolver-core/src/kernels/action_krylov.rs
geosolver-core/src/algebra/quotient.rs
geosolver-core/src/algebra/krylov.rs
geosolver-core/src/verify/verify_message.rs
docs/ai/changes/RGDTPK-Q-v4-core/PRIMITIVE_SCOPE_LEDGER.md
docs/ai/changes/RGDTPK-Q-v4-core/ACTIVE_CONTEXT.md or docs/ai/ACTIVE_CONTEXT.md
docs/ai/changes/RGDTPK-Q-v4-core/CLOSURE.md
```

### Required decision

Choose exactly one route.

### Route A — implement generic finite quotient/action

Implement production TargetActionKrylov for at least the minimum non-target-only quotient case:

```text
x^2 - 2 = 0
T - x = 0
```

Required implementation outline:

1. Build a quotient basis from authorized local relations without computing coordinate roots.
2. For the minimum case, a valid basis is `[1, x]`; more general code must not hard-code variable IDs or expected relation strings.
3. Compute normal forms of `T * basis_i` using authorized relations:
   ```text
   T*1 - x ∈ J
   T*x - 2 ∈ J
   ```
4. Produce action columns with independent membership certificates.
5. Build a production provenanced quotient handle from those certificates.
6. Compute verified characteristic support coverage.
7. Export `T^2 - 2` as target support.
8. Add replay tests that tampering with:
   ```text
   - relation T - x
   - relation x^2 - 2
   - action column certificate
   - quotient authorization hash
   - output relation
   ```
   fails.

Required tests:

```text
p12g_action_krylov_non_target_only_quotient_produces_support
p12g_action_krylov_does_not_use_target_univariate_relation
p12g_action_krylov_rejects_tampered_authorized_relation
p12g_action_krylov_rejects_injected_action_columns
```

### Route B — demote and reopen

If Route A is not implemented:

1. Rename current production path to a non-generic companion path.
2. Ensure `TargetActionKrylovKernel` is not admitted as generic P8c.
3. Update `PRIMITIVE_SCOPE_LEDGER.md`:
   ```text
   Current implementation: target-univariate companion action only.
   Generic TargetActionKrylov: not implemented.
   MECH-014: not closed.
   ```
4. Update active context and closure to reopen P8c/MECH-014.
5. Add a failing/ignored-with-blocker test showing the non-target-only quotient case is not solved by current TargetActionKrylov.

Route B may pass P12G only as a claim downgrade. It does not permit final source-faithful completion.

### Forbidden shortcuts

```text
- Do not pass by adding a target-only relation to the test input.
- Do not call TargetRelationSearch and then feed its output into TargetActionKrylov as proof that ActionKrylov is generic.
- Do not use coordinate roots, coordinate RUR, or expected answer dispatch.
```

---

## P12G-c — Plan/execute separation and declared probe model

**Supports:** P12G-RGQ-076, P12G-RGQ-077.

### Files to inspect/modify

```text
geosolver-core/src/planner/*
geosolver-core/src/kernels/sparse_resultant.rs
geosolver-core/src/kernels/specialization_interpolation.rs
geosolver-core/src/kernels/action_krylov.rs
geosolver-core/src/kernels/regular_chain_projection.rs
geosolver-core/src/kernels/norm_trace_projection.rs
geosolver-core/src/kernels/universal_elimination.rs
geosolver-core/src/result/cost_trace.rs
docs/ai/changes/RGDTPK-Q-v4-core/PRIMITIVE_SCOPE_LEDGER.md
```

### Required tasks

1. Create a `PlanWorkClassification` table for every kernel:
   ```text
   PurePlan
   CertifiedProbePlan
   InvalidHiddenExecution
   ```
2. Any `InvalidHiddenExecution` must be fixed.
3. If a kernel computes a relation during plan:
   - define a concrete `CertifiedProbePlan` object;
   - store probe hash, resource trace, authorization hash, source relation hashes;
   - require execute to replay the exact probe or recompute under the same declared probe;
   - require verifier to reject plan/execute mismatch.
4. Prefer refactoring so plan only declares:
   ```text
   schedules, bounds, template dimensions, sample points, inner kernel plans, resource caps
   ```
   and execute does the algebraic construction.

### Required tests

```text
p12g_plan_does_not_silently_execute_final_relation
p12g_certified_probe_plan_hash_tamper_fails
p12g_specialization_interpolation_inner_schedule_is_declared
p12g_sparse_resultant_template_plan_does_not_overclaim_binary_chain
```

### Forbidden shortcuts

```text
- Do not merely document that planning is heavy.
- Do not hide a final output relation in a plan hash without a typed probe object.
- Do not allow admission false to mean unsupported solver input.
```

---

## P12G-d — Candidate-cover no-real-root semantics

**Supports:** P12G-RGQ-078.

### Files to modify

```text
geosolver-core/src/compose/final_support.rs
geosolver-core/src/roots/isolate.rs
geosolver-core/src/roots/decode.rs
geosolver-core/src/result/output.rs
geosolver-core/tests/p12_roots_decode_integration.rs
```

### Required behavior

For support `S(T)=T^2+1`:

```text
finalize_candidate_cover_result(...)
  status == CertifiedCandidateCover
  support_polynomial == Some(S)
  squarefree_support_polynomial == Some(S)
  root_isolation == []
  decoded_candidates == []
  no AlgorithmicHardCase solely due to zero real roots
```

If exact-image mode later classifies this as empty, that is P13 responsibility.

### Required tests

```text
p12g_candidate_cover_no_real_roots_keeps_support_and_returns_empty_candidates
p12g_no_real_roots_replay_accepts_empty_root_and_candidate_lists_when_support_hashes_match
p12g_nonzero_support_with_no_real_roots_is_not_placeholder
```

---

## P12G-e — Truthful invariant flags and final-claim blockers

**Supports:** P12G-RGQ-079.

### Files to modify

```text
geosolver-core/src/verify/run_certificate.rs
geosolver-core/src/verify/replay.rs
docs/ai/changes/RGDTPK-Q-v4-core/CLOSURE.md
docs/ai/changes/RGDTPK-Q-v4-core/REVIEWER_PROMPTS.md
```

### Required tasks

1. Remove false fixed invariant derivation for final-claim flags.
2. Add an `InvariantEvidence` structure or explicit final-closure evidence hook for:
   ```text
   no_geometry_dispatch
   no_problem_id_dispatch
   no_expected_answer_dispatch
   no_qe_cad
   no_full_coordinate_solution_set
   no_full_coordinate_rur
   exact_q_verification
   no_hidden_fallback
   ```
3. Before P14/P16, replay/closure must fail if required invariant flags are false.
4. If runtime scan evidence is not implemented yet, mark P14/P16 as blocked until it is.

### Required tests

```text
p12g_run_certificate_cannot_claim_final_invariants_with_false_flags
p12g_replay_or_closure_rejects_false_no_geometry_dispatch_for_final_claim
p12g_invariant_flags_are_not_ignored_by_p11_replay_enforced
```

---

## P12G-f — DAG/block authorization replay preparation

**Supports:** P12G-RGQ-080.

### Files to modify

```text
geosolver-core/src/verify/run_certificate.rs
geosolver-core/src/verify/replay.rs
geosolver-core/src/graph/projection_dag.rs
geosolver-core/src/compose/message.rs
docs/ai/changes/RGDTPK-Q-v4-core/PLAN.md
```

### Required tasks

1. Add certificate fields or a typed structure for per-block authorization binding.
2. Add replay support for verifying messages against actual DAG block authorization.
3. Add a regression where a message can verify against synthetic all-relations replay but must fail under actual block authorization.
4. If full DAG replay is deferred to P14, add a P14 blocking TODO with schema-visible status:
   ```text
   P14 cannot close until actual DAG replay replaces synthetic all-relations replay for final claims.
   ```

### Required tests

```text
p12g_replay_rejects_message_using_relation_outside_original_block
p12g_replay_rejects_child_message_not_on_declared_dag_edge
p12g_dag_authorization_hash_bound_into_run_certificate
```

---

## P12G-g — Nonfinite claim tightening

**Supports:** P12G-RGQ-081.

### Files to modify

```text
geosolver-core/src/compose/final_support.rs
geosolver-core/src/result/status.rs
docs/ai/changes/RGDTPK-Q-v4-core/PRIMITIVE_SCOPE_LEDGER.md
docs/ai/changes/RGDTPK-Q-v4-core/PLAN.md
```

### Required tasks

1. Rename or document current bounded rational witness certificate as limited.
2. Add explicit certificate enum/field that identifies proof kind:
   ```text
   BoundedRationalWitness
   DimensionCertificate
   AlgebraicDependenceCertificate
   RegularChainDimensionCertificate
   GroebnerEliminationZeroCertificate
   ```
3. Only proof kinds with sufficient semantics may satisfy P15 generic nonfinite cases.
4. Add tests that failure to find a bounded witness returns `AlgorithmicHardCase` or `CertificateDesignGap`, not `CertifiedNonFiniteTargetImage`.

### Required tests

```text
p12g_nonfinite_no_relation_search_failure_is_not_nonfinite
p12g_nonfinite_bounded_witness_failure_is_not_nonfinite
p12g_nonfinite_certificate_declares_proof_kind
```

---

## P12G-h — General algebraic stress battery and P13/P14 readiness

**Supports:** P12G-RGQ-074, P12G-RGQ-083, P12G-RGQ-084, P12G-RGQ-085.

### Files to create/modify

```text
geosolver-core/tests/p12g_generality_stress.rs
docs/ai/changes/RGDTPK-Q-v4-core/P13_P14_READINESS_AFTER_P12G.md
docs/ai/changes/RGDTPK-Q-v4-core/CLOSURE.md
docs/ai/ACTIVE_CONTEXT.md
```

### Required stress cases

Implement all cases from `P12G-RGQ-083`:

```text
G1 projection without initial target-only relation
G2 finite quotient/action without initial target-only relation
G3 bilinear determinant-like structure
G4 dot/Gram-like quadratic structure
G5 guarded rational affine preprocessing
G6 multi-separator projection and composition
G7 algebraic tower/norm trace
G8 non-real support
```

Each support-producing case must prove:

```text
- no geometry names;
- no expected-answer dispatch;
- production code receives only algebraic polynomials;
- output support is nonzero;
- support is exact-verified;
- squarefree/root/decode behavior is correct;
- relation-order and variable-renaming variants still pass where applicable;
- failures, if any, are treated as blockers unless the case is explicitly tied to a downgraded claim.
```

### Required readiness file

`P13_P14_READINESS_AFTER_P12G.md` must answer:

```text
1. Is P8c generic TargetActionKrylov genuinely implemented? If not, which claim was reopened?
2. Are plan-time computations classified and safe?
3. Does candidate-cover handle non-real support correctly?
4. Are run certificate invariant flags truthful or still blocking final claims?
5. Is actual DAG replay implemented or explicitly blocking P14/P16?
6. Are nonfinite claims limited to positive proof kinds?
7. Did all P12G generality stress cases pass without geometry/problem/expected-answer dispatch?
8. May P13 begin?
9. May P14 begin?
```

### Forbidden shortcuts

```text
- Do not mark a stress case as hard-case and still pass P12G.
- Do not add target-only relations to make a non-target-only case pass.
- Do not use helper functions that bypass planner/kernel/compose where the case is meant to test them.
- Do not claim P14 readiness if solve_with_context is still temporary without an explicit P14 plan blocker.
```
