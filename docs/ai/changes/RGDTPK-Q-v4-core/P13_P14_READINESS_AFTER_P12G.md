# P13/P14 Readiness After P12G

Status: P12G remediation artifact. This file is not authority beyond the Approved Base Spec and
P12G Base Spec Amendment.

Claim ceiling before P13: PARTIAL_MECHANISM_READY:MECH-011.

1. Is P8c generic TargetActionKrylov genuinely implemented? If not, which claim was reopened?

   No. Route A now covers the admitted alias-univariate finite quotient/action case without an
   initial target-only relation. Arbitrary target-relevant quotient bases are not claimed. The
   reopened claim is any broader "generic P8c TargetActionKrylov" claim.

2. Are plan-time computations classified and safe?

   Partially. CertifiedProbePlan now records authorization, source relation, probe output,
   resource, and cost trace hashes, and execute replay must match that certified probe. Final
   claims still require reviewer evidence.

3. Does candidate-cover handle non-real support correctly?

   Yes for the P12G case: support with no real roots yields an empty certified candidate cover with
   an `EmptyRealCandidateCover` diagnostic, not an AlgorithmicHardCase.

4. Are run certificate invariant flags truthful or still blocking final claims?

   They are truthful for P11/P12 replay scope, but final strong claims remain blocked unless
   explicit final invariant evidence is supplied.

5. Is actual DAG replay implemented or explicitly blocking P14/P16?

   Explicitly blocking. P14 cannot close until actual DAG replay replaces synthetic all-relations
   replay for final claims. P12G adds typed final DAG replay evidence, binds its hash into
   CoreRunCertificate, and adds regressions for block-local relation authorization and declared
   child edges. Caller-supplied structurally bound evidence still cannot close final claims in P12G.

6. Are nonfinite claims limited to positive proof kinds?

   Yes. Nonfinite certificates now carry a proof kind, and bounded/zero-target helpers do not
   satisfy broader generic nonfinite claims.

7. Did all P12G generality stress cases pass without geometry/problem/expected-answer dispatch?

   Yes for the P12G evidence scope. The stress manifest is
   `geosolver-core/tests/p12g_generality_stress.rs`; concrete cases G1-G8 are implemented in
   production module tests and the P12G rerun passed.

8. May P13 begin?

   Yes. P12G spec and quality reviewers passed after hard-block remediation, and the P12G evidence
   rerun is recorded.

9. May P14 begin?

   No. P14 remains blocked until actual DAG/block replay is implemented for final claims or the
   Approved Base Spec is amended.
