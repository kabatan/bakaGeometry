# P1-P13 Spec-Gap Fix Base Spec Delta

Purpose: controlling delta for the P1-P13 spec-gap blocker fix before P14 may start.
Status: admitted spec-gap fix authority after guardian boundary review on 2026-07-08.
Authority: this file converts `CW_ARC_DTP_Q_P1_P13_SPEC_GAP_FIX_INSTRUCTIONS.md` into local implementation authority after admission. `BASE_SPEC.md` remains controlling outside this delta.

## Source

- Source file: `C:\Users\bakat\Downloads\CW_ARC_DTP_Q_P1_P13_SPEC_GAP_FIX_INSTRUCTIONS.md`
- SHA-256: `2D646EFA570B45365618B7506FEA925B8412D8686651231ED12810B111C5FE59`
- Target checkpoint: `1b479ae07e92d31d017569e4ff974eb2a6353ba6`

## Non-Negotiable Correction

The previous scoped P7-P13 route-closure review is now evidence only. It is not authority for closing the newly identified spec gaps.

P14 must not start until all blockers in this delta are closed in production code and reviewed:

- F0: Authority reset and claim correction.
- F1: P5 top-level unbounded ideal execution integration.
- F2: P10 true `HiddenVariableSparseResultant` sparse resultant / eliminant template data-flow.
- F3: P11 generic affine `SliceSpecialization`.
- F4: P12 guarded-nonmonic `NormTraceTower` with nonconstant leading coefficients.
- F5: Cross-route exact-proof-gate and no-fallback regression.
- F6: Adversarial reviewer re-run over P1-P13 spec-gap blockers.

## F0 Requirements

F0-R1. Repository docs must stop claiming unconditional P7-P13 closure. They may state only that P7, P8, P9, and P13 are substantially implemented, while P5/P10/P11/P12 require corrective closure before P14.

F0-R2. `current_gap_inventory.md`, `p7_p13_route_closure_evidence.md`, this delta, and active context must record the P5/P10/P11/P12 blockers.

F0-R3. Evidence and review files remain pointers only. Production source control-flow, data-flow, and certificate replay decide PASS/FAIL.

## F1 Requirements

F1-R1. `max_window_degree = None` means unbounded fair window degree scheduling, not an empty window list or hidden finite cap.

F1-R2. `max_proof_weight = None` means unbounded fair proof tuple scheduling, not immediate `FiniteResourceFailure`.

F1-R3. Top-level `solve_target` must connect to a lazy global solve schedule. For any finite `window_degree` and finite proof tuple `(multiplier_degree, support_power, guard_power)`, unbounded execution must eventually attempt that combination unless an explicit resource limit is hit.

F1-R4. Bounded mode must remain bounded when both `max_window_degree` and `max_proof_weight` are `Some`.

F1-R5. Unbounded candidate/proof work and complete fallback budget work must not starve each other. Until P15 completes the fallback body, fallback work items may return explicit `ResourceFailure` or design-gap traces, but unbounded proof search must not be skipped.

F1-R6. Tests must include high radical power `T^9` success with both bounds `None`, bounded small-prefix non-success, arbitrary tuple reachability in the global schedule, and anti-hidden-default scans.

## F2 Requirements

F2-R1. The current Macaulay-style template/nullspace helper must not be used as the `HiddenVariableSparseResultant` PASS basis.

F2-R2. Production P10 must decompose each polynomial into eliminated-variable supports with target-coefficient polynomials, construct sparse/Newton/Cayley support-derived multiplier and row supports, and produce candidates through determinant/minor/eliminant-template data-flow.

F2-R3. `HiddenVariableSparseResultant` witnesses must record sparse template kind, support sums, row support size, multiplier support sizes, determinant/minor degree, and active multiplier supports derived from the relation/minor, not by copying the whole template.

F2-R4. Required conformance includes a 2-polynomial hidden resultant, a 3-equation 2-eliminated-variable non-chain sparse eliminant family, and a sparse support shape that is not total-degree Macaulay support.

F2-R5. Resultant candidates remain candidates only; adoption still requires fixed exact proof and verifier replay.

## F3 Requirements

F3-R1. Slice equations must be generic affine linear forms over non-target variables:

```text
L_j(X) = a_j,0 + sum_i a_j,i X_i = 0
```

with zero target coefficient.

F3-R2. Deterministic pseudo-generic slice scheduling must not generate only coordinate unit vectors. At least some slice forms must involve two or more non-target variables.

F3-R3. The slice schedule must be denominator-admissible for each prime used by the route. A prime-specific slice cannot silently ignore denominator, guard, replay, or reduction obstructions.

F3-R4. The sliced system must be `original equations + affine slice equations`.

F3-R5. Slice traces must record affine coefficients, constants, and equation indices. Coordinate constant assignments alone are insufficient.

F3-R6. P11 cannot close by depending on the weak pre-F2 P10 route.

## F4 Requirements

F4-R1. `NormTraceTower` must support tower levels with nonconstant lower-variable leading polynomial:

```text
A(Y_lower) * Z^d + lower_terms(Y_lower, Z) = 0
```

F4-R2. `TowerLevel` must carry a leading polynomial and verified guard certificate, not only a rational leading coefficient.

F4-R3. The leading polynomial must depend only on already available lower tower variables and must not depend on the target, current variable, or newer variables.

F4-R4. Guard certificates for nonconstant leading polynomials must be replay-verified, not accepted by equality check alone. The implementation must require `verify_guard_certificate(...) == Verified`; `DerivedProduct` must pass exact product replay; and `InputSemanticNonzero` must be backed by an identical input `semantic_guards` record.

F4-R5. Nonmonic denominator inversion must never become proof authority. The final route success must still pass fixed exact proof / guarded proof replay.

F4-R6. Required conformance family:

```text
variables = [x, y, T]
target = T
guard: x != 0
equations:
  x^2 - 2 = 0
  x*y^2 - 1 = 0
  T - y = 0
expected support: 2*T^4 - 1
```

## F5 Requirements

F5-R1. After F1-F4, every route must still have route forcing, complete fallback disabled, spurious candidate rejection, and tampered certificate rejection.

F5-R2. Finite-field traces, slice traces, resultant traces, Krylov traces, `origin_evidence`, prime count, and route count must never be adoption conditions.

F5-R3. `FactorizationResult::Partial` and `ResourceFailure` must never be treated as `Complete`.

F5-R4. Production safety scans must reject normal-path `Unsupported`, ordinary-route `ImplementationBug`, partial exact-image success, no-target eliminant success without verifier `Verified`, and description-only `ComponentUnionLcm` verification.

## F6 Requirements

F6-R1. Prior reviewer PASS results must not be reused as closure. A fresh adversarial review must inspect production source directly.

F6-R2. The final reviewer must inspect files/functions, public `solve_target` call chains, route-specific data-flow, forbidden-pattern searches, adversarial families, route-forcing/no-fallback/tamper tests, and remaining uncertainty.

## Claim Boundary

After this delta is admitted, the only allowed completion claim is:

```text
This closes the P1-P13 spec-gap blockers for top-level unbounded ideal proof execution, true HiddenVariableSparseResultant data-flow, generic affine SliceSpecialization, guarded-nonmonic NormTraceTower, and exact-proof-gated route adoption.
```

That claim is allowed only after F0-F6 production implementation, tests, spec review, quality review, and boundary review pass.

This delta does not close P14 exact elimination substrate, P15 complete target elimination fallback, P16 exact real image classification, final V3 completion, source-faithfulness, performance claims, readiness, acceptance-complete, production-safe, or any R-ID verified claim.
