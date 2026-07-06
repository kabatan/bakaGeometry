# Candidate-Cover Replay And Tamper Results

Status: candidate-cover replay evidence.

## Replay-Bound Objects

Candidate-cover replay recomputes and checks:

- input hash;
- canonical and compressed system hashes;
- target projection DAG;
- kernel plans and projection messages;
- composed projection;
- global support certificate hash;
- squarefree support;
- real root isolation;
- decoded candidates;
- exact-image certificate hash when exact-image mode is used.

Nonfinite replay uses the public `NonFiniteCertificate`, recomputes composition, and binds replay to
`hash_nonfinite_certificate`.

## New Support-Certificate Route B

`compose/final_support.rs` can now construct support from the composed message ideal when Route A is
unavailable, and `verify_support.rs` has a machine-readable
`ComposedIdealMembershipSupportCertificate` for the verification route:

```text
S(T) = sum q_i r_i
```

The certificate records:

- target;
- support hash;
- composed projection hash;
- composed relation hashes;
- exact rational multipliers;
- exact identity hash;
- certificate hash.

Focused tests:

```text
ccc_route_b_final_support_uses_composed_ideal_membership_when_route_a_unavailable: PASS
composed_ideal_membership_route_verifies_support_without_target_only_root_relation: PASS
composed_ideal_membership_route_rejects_multiplier_tamper: PASS
composed_ideal_membership_route_rejects_removed_relation: PASS
```

Boundary note: multiplier and removed-relation tamper are verifier-level checks on the
machine-readable `ComposedIdealMembershipSupportCertificate`. Public replay does not serialize those
multipliers separately; it replay-binds the recomputed support certificate through
`CoreRunCertificate.global_support_certificate_hash`.

Replay-level checks for support evidence are covered by:

```text
p11_replay_fails_on_input_canonical_dag_plan_and_squarefree_tamper: PASS
ccc_p12_red_team_runs_sixteen_fresh_public_inputs: PASS
```

The first rejects tampered global support certificate hashes through `replay_run_certificate`. The
second uses `api::solve_target`, replays every support-producing fresh red-team success, and checks
that the run certificate's global support certificate hash equals the exact certificate recomputed
from actual DAG/messages/composition.

## Candidate-Cover Semantic Separation

`p13_candidate_cover_mode_does_not_claim_exact_image_for_semantic_problem` verifies candidate-cover
mode keeps real roots of `S(T)`, does not attach an exact-image certificate, emits the candidate-cover
diagnostics, and replays successfully.

`ccc_p11_a12_spurious_roots_are_allowed_in_candidate_cover_mode` verifies four public semantic
spurious-root inputs retain the extra roots in candidate-cover mode.

## Tamper/Fallback Residual Boundary

Existing P11/P15/P16 replay tamper suites remain relevant for input/message/support/root/candidate,
exact-image classification hash, and nonfinite certificate tamper. This repair adds Route B
verifier-level support-certificate tamper coverage, plus replay-level support-certificate hash
binding checks, and does not broaden the final claim beyond candidate-cover readiness.
