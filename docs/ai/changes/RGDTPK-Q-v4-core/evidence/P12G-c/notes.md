P12G-c implemented for TargetActionKrylov through `PlanWorkClassification::CertifiedProbePlan`.
The probe binds authorization hash, source relation hashes, output hashes, resource trace hash,
cost trace hash, certificate/trace hash, and an execute replay flag. Regressions
`p12g_plan_does_not_silently_execute_final_relation`,
`p12g_action_krylov_plan_probe_classification_is_replayed_in_execute`, and
`p12g_certified_probe_plan_hash_tamper_fails` verify the model and tamper rejection.
