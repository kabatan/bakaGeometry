# P12G Patch Notes

This pack was created after direct inspection of the P6–P12 implementation.

The key conclusion is:

```text
P6–P12 contain real mechanisms, but the current implementation is not yet safe to treat as a complete general R-GDTPK algorithm.
```

The most important direct finding is that `TargetActionKrylovKernel` currently behaves like a companion-matrix wrapper around an already target-only univariate relation. That is not the intended generic quotient/action kernel.

The second most important finding is that planning and execution are blurred in several kernels. This must be resolved before public orchestration and final claims.

P12G is intentionally inserted before P13/P14 so that exact-image and public-pipeline work does not build on overclaimed P8/P11/P12 assumptions.
