# Full Core Repair Plan — From P1–P12G Partial Implementation to Complete Candidate-Cover Core

This plan replaces the current continuation plan. P13, P14, P15, and P16 are blocked until this plan passes. The goal is not to add another evidence layer. The goal is to remove or generalize the existing partial implementations and complete the candidate-cover solver core defined by the v4 specification.



---

## FCR-P0A — Agent self-governance and failure-mode reset

### Required actions

1. Create `FULL_CORE_AGENT_FAILURE_MODE_RESET.md`.
2. In that file, the Agent must explicitly acknowledge, in its own words, the following implementation hazards:

```text
- passing gates is not implementation;
- documenting a limitation is not implementing the missing algorithm;
- a minimum example is not a generic mechanism;
- a module-only test is not a production pipeline proof;
- a reviewer PASS is invalid if the prompt did not force algorithmic sufficiency review;
- specialized kernels are optimizers inside the generic pipeline, not the whole solver;
- if the Agent cannot implement a v4 obligation, it must raise AlgorithmDefect instead of shrinking scope.
```

3. Add a local `PlanDefect / AlgorithmDefect trigger table` used by all later phases.
4. Require every FCR phase handoff to state whether any trigger fired.

### Reviewer must fail if

```text
- the Agent treats this file as a confession but not an execution constraint;
- the Agent says limitations are acceptable because they are documented;
- the Agent plans to make only the listed stress cases pass;
- the Agent does not identify at least one concrete prior failure mode that this phase must avoid.
```

---

## FCR-P1A — Source-spec compliance map against v4 sections 0–34

### Required actions

Create `FULL_CORE_SOURCE_SPEC_COMPLIANCE_MAP.md`.

The map must cover:

```text
- v4 sections 0–5: problem definition, non-negotiable principles, data invariants;
- v4 section 6: complete folder/module structure;
- v4 sections 7–29: every file/function/type listed in the specification;
- v4 sections 30–33: cost trace, nonfinite handling, geometry-derived algebraic footprint, completion conditions.
```

For every item, classify it using the schema from `FCR-018`.

### Required decisions

The map must explicitly separate:

```text
Candidate-cover mandatory now:
  functions and invariants needed to produce finite candidate covers through the production pipeline.

Exact-image mandatory later:
  fiber/hermite/thom/slack semantics required for exact image completion but not for this candidate-cover repair.

Never optional:
  no geometry dispatch, no expected-answer dispatch, no coordinate solution list/RUR, deterministic planner, actual DAG replay, exact-Q verification, cost trace.
```

### Reviewer must fail if

```text
- any v4 file/function is absent from the map;
- a candidate-cover-path function is marked later-phase without source justification;
- exact-image functions are silently ignored rather than explicitly P13-scoped;
- the map claims SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC before P13 exact-image work is complete.
```

---

## FCR-P11 — Adversarial red-team, final nonfinite gate, and closure preconditions

### Required actions

After FCR-P0A/FCR-P1A and FCR-P0 through FCR-P10, and before final closure, run a separate
red-team review that is not allowed to use previous phase PASS summaries as evidence.

The red-team reviewer must:

```text
1. Pick at least 10 non-fixture algebraic inputs not used in the acceptance suite.
2. Include at least:
   - a multivariate finite quotient/action case not reducible to local-univariate alias;
   - a separator-composition case with two exported separators;
   - a sparse resultant/eliminant case not reducible to target-univariate relation search at degree 1;
   - a guarded rational affine case where the denominator is nonconstant;
   - a one-large-block no-separator case;
   - a target-independent component case with feasibility obligation;
   - a final nonfinite case with positive certificate and a similar case without positive certificate.
3. Run the public or near-public pipeline for each.
4. Mutate relation order, variable ids, and syntactic polynomial forms.
5. Verify that success/failure comes from the v4 mechanism, not expected values or helper bypass.
```

Final nonfinite semantics are owned by this phase, not by FCR-P10 support-producing closure, unless
the implementation first provides a machine-readable, replay-bound nonfinite certificate that is
carried by the public result and accepted by `replay_run_certificate`.

This phase must also bind closure preconditions that are easy to overclaim:

```text
- CoreInvariantFlags must be tied to fresh static scans plus replay/tamper evidence, not asserted
  as free booleans.
- no geometry dispatch, no fixture/problem-id/expected-answer dispatch, no hidden fallback, and
  no QE/CAD/RCF/full-coordinate fallback scans must be referenced from final closure.
- passing these scans is necessary but not sufficient; dynamic red-team and replay evidence are also
  required.
- CANDIDATE_COVER_CORE_READY, EXACT_IMAGE_CORE_READY, SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC, and
  RGDTPK_Q_V4_ACCEPTANCE_COMPLETE remain separate claim labels with separate evidence gates.
```

### Reviewer must fail if

```text
- the reviewer mainly checks YAML or evidence files;
- the reviewer does not construct any new algebraic counterexamples;
- the reviewer creates fewer than 10 new algebraic inputs;
- any new support-producing case fails because the implementation only supports the examples from FCR-P10;
- final nonfinite is accepted without machine-readable positive proof or without being clearly kept
  out of the candidate-cover readiness claim;
- CoreInvariantFlags are not connected to fresh scans and replay/tamper evidence;
- no-dispatch or no-QE/CAD evidence is only asserted in prose;
- the final claim depends on exact-image functions that are not implemented.
```


---

## FCR-P0 — Stop, freeze, and revoke unsafe continuation

### Required actions

1. Record current HEAD and create a repair branch.
2. Update `docs/ai/ACTIVE_CONTEXT.md`, `CLOSURE.md`, and `P12G_READINESS.md`:
   ```text
   P13/P14/P15/P16 are blocked.
   Current P12G implementation is insufficient as general R-GDTPK core.
   P8c/MECH-014 is not closed until generic TargetActionKrylov is implemented.
   P8b/P9 plan-time execution claims are reopened.
   ```
3. Add this repair pack under `docs/ai/changes/RGDTPK-Q-v4-core/`.
4. Run and archive:
   ```bash
   git rev-parse --verify HEAD
   cargo fmt --manifest-path geosolver-core/Cargo.toml -- --check
   cargo test --manifest-path geosolver-core/Cargo.toml -- --nocapture
   git diff --check
   ```

### Required evidence

```text
evidence/FCR-P0/commands.txt
evidence/FCR-P0/command_outputs.txt
evidence/FCR-P0/claim_reset_matrix.yaml
reviews/FCR-P0/<timestamp>/{prompt.md,response.md,review_summary.yaml,evidence_manifest.yaml}
```

### Reviewer must fail if

```text
- P13/P14 readiness remains open;
- P8c is still claimed as generic while TargetActionKrylov remains alias/univariate only;
- CLOSURE.md implies current code is close to full candidate-cover completion;
- old P12G review PASS is used as proof of generality.
```

---

## FCR-P1 — Full direct audit of P1–P12G production code

### Files to inspect

```text
geosolver-core/src/types/*
geosolver-core/src/problem/*
geosolver-core/src/algebra/*
geosolver-core/src/preprocess/*
geosolver-core/src/graph/*
geosolver-core/src/planner/*
geosolver-core/src/kernels/*
geosolver-core/src/compose/*
geosolver-core/src/verify/*
geosolver-core/src/roots/*
geosolver-core/src/result/*
geosolver-core/src/solver/*
```

### Required output

Create `FULL_CORE_PRODUCTION_AUDIT.md` with one row per public function and production path:

```yaml
path: <file::function>
production_reachable: true|false
algorithmic_role: <validate|compress|plan|execute|verify|root|finalize>
actual_input_class: <exact description>
actual_output_class: <exact description>
known_limitations: []
partial_slice_risk: none|low|medium|high|fatal
plan_time_execution: none|cost_probe_only|final_relation_construction
certificate_binding: exact|decorative|missing
required_action: keep|generalize|move_to_test|delete|replace
```

### Mandatory findings to confirm or disprove

The audit must explicitly inspect and classify:

```text
solver/orchestrator.rs: temporary pipeline
algebra/f4.rs: NotProductionF4 and *_for_tests functions
kernels/action_krylov.rs: target-only and alias-univariate paths
kernels/sparse_resultant.rs: plan-time resultant trace
kernels/specialization_interpolation.rs: plan-time inner TargetRelationSearch and local Groebner verification
kernels/regular_chain_projection.rs: plan-time chain decomposition/projection
kernels/norm_trace_projection.rs: plan-time tower/norm construction
verify/replay.rs: synthetic all-relations replay
compose/final_support.rs: limited nonfinite proof kind
planner/kernel_plan.rs: default PurePlan path
```

### Reviewer must fail if

```text
- the audit says "limited but documented" and chooses keep without generalization;
- any production-reachable partial path is not assigned a repair action;
- any required function is skipped because it already has a PASS review.
```

---

## FCR-P2 — Delete, quarantine, or generalize inappropriate production code

### Required actions

For every row in `FULL_CORE_PRODUCTION_AUDIT.md` with `partial_slice_risk: high|fatal`, do one of:

```text
Generalize:
  implement the generic v4 algorithm and add exact replay evidence.

Move to test:
  put under #[cfg(test)] or test-support module and remove from production registry.

Delete:
  remove the function and all production references.
```

### Non-negotiable cleanup targets

```text
1. Non-production F4 code must not be reachable from production Universal or elimination strategies.
2. TargetActionKrylov alias-univariate path may remain only as a subcase of a generic quotient/action constructor, not as the whole kernel.
3. Binary resultant chain must not be the generic SparseResultantProjection implementation.
4. Explicit tower detection must not be the full NormTraceProjection claim unless the v4 tower/norm contract is actually implemented and not overclaimed.
5. Synthetic all-relations replay must not be the final replay path.
6. Any helper that constructs support without planner/kernel/message/certificate must be test-only.
```

### Required evidence

```text
evidence/FCR-P2/deletion_and_quarantine_log.yaml
evidence/FCR-P2/production_reachability_scan.txt
evidence/FCR-P2/forbidden_path_scan.txt
```

### Reviewer must fail if

```text
- code is simply renamed but still production-reachable;
- comments say "not generic" while the function remains in all_kernels or public production path;
- tests still pass only because helper functions bypass the production pipeline.
```

---

## FCR-P3 — Implement the full public candidate-cover pipeline

### Files to modify

```text
solver/pipeline.rs
solver/orchestrator.rs
api.rs
result/output.rs
result/cost_trace.rs
verify/run_certificate.rs
verify/replay.rs
```

### Required implementation

`solve_with_context` must implement the v4 top-level pipeline in order:

```rust
validated = step_validate(problem, &mut ctx)?;
canonical = step_canonicalize(validated, &mut ctx)?;
compressed = step_compress(canonical, &mut ctx)?;
graphs = step_build_graphs(&compressed, &mut ctx)?;
dag = step_build_dag(&graphs, &compressed, &mut ctx)?;
plans = step_plan(&dag, &compressed, &mut ctx)?;
messages = step_execute(&dag, &plans, &compressed, &mut ctx)?;
for msg in &messages { verify_projection_message_against_actual_block(msg, dag, compressed)?; }
composed = step_compose(&dag, messages, &mut ctx)?;
support = step_support(composed, compressed.target, &mut ctx)?;
support_cert = verify_global_support(&support, &composed, &ctx)?;
(squarefree, roots, candidates) = step_roots(&support, &mut ctx)?;
cert = finalize_core_run_certificate(...actual DAG evidence...)?;
return finalize_success_result(...);
```

### Required statuses

```text
CertifiedCandidateCover on finite candidate-cover success.
CertifiedNonFiniteTargetImage only on positive nonfinite certificate.
FiniteResourceFailure / AlgorithmicHardCase / CertificateDesignGap on bounded failure.
InvalidInput on invalid input.
```

### Reviewer must fail if

```text
- solve_with_context still returns temporary_pipeline_not_connected;
- orchestrator calls finalizers with hand-built messages;
- orchestrator skips DAG/planner/kernel execution;
- public success lacks certificate, projection messages, root isolation, or cost trace.
```

---

## FCR-P4 — Enforce pure planning and move algebraic output construction to execute

### Files to modify

```text
planner/kernel_plan.rs
kernels/sparse_resultant.rs
kernels/specialization_interpolation.rs
kernels/regular_chain_projection.rs
kernels/norm_trace_projection.rs
kernels/action_krylov.rs
kernels/universal_elimination.rs
```

### Required changes

1. Remove the default assumption that `KernelExecutionPlan::new` is `PurePlan` if the kernel computed output relations while planning.
2. For every kernel plan function:
   ```text
   plan_* may declare schedules/templates/bounds/sample points/inner plan ids only.
   plan_* must not call functions that compute final relation_generators.
   ```
3. Move relation-producing logic to `execute_*`.
4. Make `PlanWorkClassification::PurePlan` verifiable by static code inspection and unit tests.
5. Delete or refactor these plan-time constructions:
   ```text
   build_sparse_resultant_trace from plan_sparse_resultant_with_messages
   build_specialization_interpolation_trace from plan_specialization_interpolation_with_messages
   build_regular_chain_trace from plan_regular_chain_projection
   build_norm_trace_trace from plan_norm_trace_projection
   build_target_action_krylov_trace from plan_target_action_krylov_with_messages unless it is non-final cost probe only
   ```

### Required tests

```text
fcr_plan_sparse_resultant_does_not_construct_output_relation
fcr_plan_specialization_interpolation_does_not_run_inner_kernel
fcr_plan_regular_chain_does_not_project_chain
fcr_plan_norm_trace_does_not_construct_norm_relation
fcr_plan_target_action_does_not_construct_final_support
fcr_execute_recomputes_and_certifies_declared_plan_outputs
```

### Reviewer must fail if

```text
- any plan_* function calls compute_resultant_relation, execute_target_relation_search,
  eliminate_to_keep_variables, local_regular_chain_decomposition, norm_relation_for_tower_plan,
  or a function that returns final relation_generators;
- CertifiedProbePlan is used to hide final output construction during planning.
```

---

## FCR-P5 — Implement generic TargetActionKrylov

### Files to modify

```text
algebra/quotient.rs
algebra/krylov.rs
algebra/normal_form.rs
algebra/groebner.rs
algebra/f4.rs
kernels/action_krylov.rs
verify/verify_message.rs
verify/certificates.rs
```

### Required algorithm

Implement a production quotient/action constructor that is not limited to univariate relations or linear aliases.

Required steps:

```text
1. Input: authorized local relations J ⊂ Q[V] and target T.
2. Construct or verify a finite-dimensional target-relevant quotient basis B from J.
   - Use standard monomials from a verified Groebner/F4 normal form basis, or another exact finite quotient certificate.
   - Do not use coordinate roots or full coordinate RUR.
3. For every b ∈ B, compute NF(T*b) in span(B).
4. For every action column, produce exact membership certificate:
   T*b - Σ_j c_j b_j ∈ <authorized relations>.
5. Build the target action matrix M_T.
6. Compute characteristic polynomial or a proven target-support annihilator.
7. Prove coverage by deterministic full-basis/block coverage, not a single Krylov vector.
8. Verify S(M_T)=0 exactly.
9. Export S(T) as a target-only relation.
```

### Required non-slice tests

All tests must go through planner/admission/execute/message verification, not direct helper functions.

```text
fcr_action_multivariate_quotient_no_target_relation
  relations: x^2 + y - 1 = 0, y^2 - x = 0, T - x - y = 0
  target: T
  required: TargetActionKrylov admitted and produces verified target support without TargetRelationSearch first.

fcr_action_target_is_nonlinear_expression
  relations: x^2 - 2 = 0, y^2 - 3 = 0, T - x*y - x = 0
  required: action columns use quotient basis containing x,y monomials; no alias-univariate shortcut.

fcr_action_rejects_injected_basis_or_column
  tamper basis, action columns, membership, authorization hash, output support; replay fails.
```

### Reviewer must fail if

```text
- implementation still only handles target-only or local-univariate + linear alias;
- basis is hard-coded for tests;
- expected support is used by production code;
- full coordinate roots/RUR are computed;
- undercoverage is accepted.
```

---

## FCR-P6 — Implement production F4/Groebner target/separator elimination

### Files to modify

```text
algebra/f4.rs
algebra/groebner.rs
algebra/elimination.rs
kernels/universal_elimination.rs
```

### Required implementation

1. Replace `NotProductionF4` production reachability with either:
   ```text
   - real exact local F4/F5-like sparse matrix reduction with certificates; or
   - no F4 production route, with Universal using certified local Groebner/TargetRelationSearch instead.
   ```
2. `eliminate_to_keep_variables` must return generators in Q[keep] with exact membership certificates.
3. Local elimination may be heavy but must be:
   ```text
   - block-local;
   - target/separator-only export;
   - declared before execution;
   - resource bounded;
   - certificate replayable;
   - not coordinate root enumeration.
   ```

### Required tests

```text
fcr_universal_one_large_block_multivariate_projection
fcr_universal_keep_only_exports_no_coordinate_roots
fcr_elimination_membership_certificates_replay
fcr_nonproduction_f4_not_reachable
```

### Reviewer must fail if

```text
- any `*_for_tests` F4/Groebner batch path is production reachable;
- Universal relies on a fake F4 claim;
- elimination exports local coordinates;
- relation generators lack exact membership certificates.
```

---

## FCR-P7 — Implement generic SparseResultant, SpecializationInterpolation, RegularChain, and NormTrace contracts

### SparseResultant requirements

```text
- plan declares support/template dimensions only;
- execute constructs resultant/eliminant relation;
- not limited to one hard-coded pair or a pairwise-only completion claim;
- exact certificate verifies the relation;
- failure does not become Unsupported.
```

### SpecializationInterpolation requirements

```text
- plan declares separator variables, sample schedule, interpolation support, inner kernel plans;
- execute runs inner computations and interpolates;
- exact Q verification is mandatory;
- sample agreement alone is insufficient.
```

### RegularChain requirements

```text
- plan does not decompose/project;
- execute performs local regular-chain decomposition and projection;
- component/guard semantics are preserved;
- output relation_generators are exported-only and replayable.
```

### NormTrace requirements

```text
- plan does not construct norm relation;
- execute detects algebraic tower and computes norm/trace relation;
- two-step and multi-step towers must be handled if the kernel is claimed complete;
- explicit tower support is an optimization, not the entire generic solver claim.
```

### Reviewer must fail if

```text
- any of these kernels remains a narrow slice while claimed complete;
- plan-time output construction remains;
- module-only tests are the only evidence.
```

---

## FCR-P8 — Actual DAG/block replay and certificate finalization

### Files to modify

```text
graph/projection_dag.rs
verify/replay.rs
verify/run_certificate.rs
verify/verify_message.rs
compose/compose.rs
solver/orchestrator.rs
```

### Required implementation

1. CoreRunCertificate must bind the actual `TargetProjectionDAG`:
   ```text
   dag_hash
   block ids
   block authorization hashes
   block relation ids/hashes
   child edges
   message package hashes
   plan hashes
   support certificate hash
   root/candidate hashes
   ```
2. `replay_run_certificate` must use actual blocks, not synthetic all-relations blocks.
3. Replay must reject:
   ```text
   - message uses relation outside block;
   - child message not on DAG edge;
   - block authorization changed;
   - plan hash changed;
   - package hash changed;
   - support changed;
   - root/candidate omitted or duplicated.
   ```

### Reviewer must fail if

```text
- synthetic all-relations replay is still the main accepted replay path;
- actual DAG replay is only a helper or final-claim blocker;
- CoreRunCertificate lacks actual block authorization evidence.
```

---

## FCR-P9 — Full support composition, root isolation, and result construction

### Required implementation

1. `compose_projection_messages` must use message ideals and separator elimination without reconstructing the full coordinate system.
2. `build_global_support_polynomial` must build nonzero S(T) from verified target-only root relations or verified membership in the composed message ideal.
3. `verify_global_support` must replay exact proof.
4. `squarefree_support`, root isolation, and decode must run in public success path.
5. Empty real root set remains `CertifiedCandidateCover` with empty candidates.
6. Cost trace must include all parameters required by the v4 spec.

### Reviewer must fail if

```text
- support is hand-built for tests;
- global support verification only multiplies already target-only helper relations while other support routes are unverified;
- roots/candidates are absent on success;
- no-real-root support becomes hard-case.
```

---

## FCR-P10 — Full algebraic support-producing acceptance suite

### Required suite categories

All support-producing cases must go through the public or near-public pipeline. For final readiness, use `api::solve_target`.

```text
A1. no initial target-only relation, one block
A2. multivariate finite quotient/action, target nonlinear in quotient variables
A3. multiple eliminated variables and multiple separators
A4. sparse resultant/eliminant with no target-only relation
A5. specialization-interpolation with exact verification
A6. guarded rational affine preprocessing through final support
A7. target-independent components with feasibility obligations
A8. no useful separator one-large-block Universal path
A9. regular-chain projection with component/guard semantics
A10. norm/trace two-step tower
A11. non-real support empty candidate cover
B1. resource-bounded hard case with cost trace
```

Certified nonfinite target image is intentionally not a P10 support-producing acceptance case. It is
owned by FCR-P11 final nonfinite semantics unless a later implementation carries a machine-readable,
replay-bound nonfinite certificate in the public result.

### Mandatory anti-hack transformations

For each support-producing case:

```text
- variable ids are permuted;
- relation order is permuted;
- coefficients are scaled by nonzero rationals where semantics allow;
- target variable id is not always 0;
- production code is scanned for case names, expected answers, and fixture ids.
```

### Failure rules

A support-producing suite case fails the repair if it returns:

```text
AlgorithmicHardCase
FiniteResourceFailure
CertificateDesignGap
CertifiedNonFiniteTargetImage
InvalidInput
empty support
placeholder roots/candidates
no certificate
no projection messages
```

### Reviewer must fail if

```text
- stress cases are module-only helpers;
- support-producing cases do not use planner/kernel/compose;
- expected answers are used in production code;
- only simple univariate/alias examples pass.
- P10 evidence is used to claim final nonfinite readiness.
```

---

## FCR-P12 — Final closure and claim

### Required commands

Run fresh after all code changes:

```bash
cargo fmt --manifest-path geosolver-core/Cargo.toml -- --check
cargo clippy --manifest-path geosolver-core/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path geosolver-core/Cargo.toml --all-targets --all-features -- --nocapture
rg -n "Unsupported|unsupported" geosolver-core/src || true
rg -n "circle|triangle|tangent|distance|area|incircle|circumcircle|orthic|mixtilinear|fixture|expected|answer|problem_id|official" geosolver-core/src || true
rg -n "todo!|unimplemented!|placeholder|dummy|fake|stub|temporary_pipeline_not_connected|NotProductionF4|for_tests" geosolver-core/src || true
rg -n "CAD|QE|RCF|coordinate solution|coordinate roots|full coordinate RUR|RUR|solve_all_coordinates" geosolver-core/src || true
```

### Required final artifacts

```text
CLOSURE.md
FULL_CORE_PRODUCTION_AUDIT.md
FULL_CORE_ACCEPTANCE_RESULTS.md
FULL_CORE_REPLAY_TAMPER_RESULTS.md
FULL_CORE_COST_TRACE_SUMMARY.md
FULL_CORE_RED_TEAM_RESULTS.md
FULL_CORE_NONFINITE_RESULTS.md
FULL_CORE_INVARIANT_SCAN_BINDING.md
review archive for every FCR phase
```

### Final allowed claim

Only if all FCR phases pass, including FCR-P11 red-team and final nonfinite/invariant-scan gates:

```text
CANDIDATE_COVER_CORE_READY
```

Forbidden unless P13 exact-image also passes:

```text
EXACT_IMAGE_CORE_READY
RGDTPK_Q_V4_ACCEPTANCE_COMPLETE
SOURCE_FAITHFUL_TO_SUPPLIED_V4_SPEC
```

