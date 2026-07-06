# ACR-P5 Guardian Boundary Review Response

Status: FAIL_FIXABLE

Findings:

1. Non-monopolization is not satisfied. The implementation checks route budget only before and after a synchronous `kernel.execute` call, so an executing route can still monopolize the ladder until the kernel returns.
2. `max_elapsed_steps` is not enforceable during kernel execution. The budget field is present, but there are no cooperative in-flight checkpoints that consume the step budget.
3. The no-failure aggregate path can drop route attempt summaries. If a ladder produces no message and does not collect a route failure, aggregate diagnostics do not include all attempted summaries.
4. The near-public P5 stress proves a preflight budget yield, not interruption while a route is executing.

Required fixes:

- Add cooperative in-flight route budget metering shared by production route execution.
- Ensure `max_elapsed_steps` and `max_work_units` can stop a route before it finishes.
- Preserve all attempted route summaries in aggregate ladder failures.
- Add a near-public stress that passes preflight, stops inside a route through a cooperative budget checkpoint, then continues to a later successful route.

Forbidden claim: This FAIL response does not close ACR-P5 and does not authorize P6 or any readiness claim.
