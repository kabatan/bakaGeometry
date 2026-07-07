# P15 Review Result

Reviewer: spec_verifier

Decision: PASS

Scope: P15 only, BS-R120 / BS-R121 / MECH-05.

Accepted evidence:

- `squarefree_support` rejects zero support and delegates exact normalized squarefree computation.
- Sturm path uses exact rational Cauchy bound, Sturm sequence, root counts, and rational subdivision.
- Descartes/Vincent is a distinct exact path using rational interval transforms and sign variations, not a Sturm alias.
- Non-root split search has no fixed `..=128` cap.
- Root records bind squarefree support hash and deterministic root index.
- Candidate hashes bind target, support hash, root index, and rational interval endpoints.
- Replay checks root/candidate support, target, index, interval equality, duplicate/omission, and candidate hash recomputation.
- P15 static audit covers required files/symbols, Descartes aliasing, float markers, fixed cap, and hash bindings.

Fresh checks observed by reviewer:

- fmt
- P15 audit findings 0
- `--lib real_root`
- `--lib roots`
- `--lib sign`
- `--test p12_roots_decode_integration`
- `--test p15_acceptance_stress`
- `--no-run`

Missing evidence: none for P15 scope.

Blockers: none.
