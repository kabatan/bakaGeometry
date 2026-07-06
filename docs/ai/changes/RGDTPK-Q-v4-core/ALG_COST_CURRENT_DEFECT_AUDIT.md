# Algebraic-Cost Completion Current Defect Audit v1

## Defect class

The latest diagnostic report shows a post-GSR timeout in projection execution, not planning. The first executable route for the large block is `SparseResultantProjection`; it enters an exact sparse-resultant chain with severe expression swell.

## Direct evidence to account for

The report records:
- validation/canonicalization/compression/graph/DAG/planning complete in about 0.2 seconds;
- block 5 has 33 local variables, 29 relations, 2 exports;
- declared ladder begins with SparseResultantProjection;
- step 5 produces a 628,925-term intermediate relation;
- step 15 starts a 3x3 Sylvester resultant on 1,588-term and 16,662-term inputs;
- no compute end line appears before timeout.

## Why previous PASS was invalid

The previous closure focused on candidate-cover semantic correctness and some generic stress. It did not require every production route to be bounded by its real dominant algebraic cost.

The missing requirement was:

```text
A production route must not be admitted, prioritized, or executed solely because a shallow proxy
(matrix dimension, route name, or template existence) looks small.
```

## Required correction

This repair must introduce:
- expression-swell-aware SparseResultant planning;
- runtime growth guards;
- bounded alternate resultant backend;
- non-monopolizing ladder route budgets;
- graph/decomposition cost awareness;
- sparse/lazy TRS;
- Universal success-route evidence;
- generic large-footprint support-producing stress.

## Not acceptable

- Adding the diagnostic problem as a regression gate.
- Returning a faster failure and calling that success.
- Moving SparseResultant after another route without bounding it.
- Disabling SparseResultant entirely without providing generic success routes.
- Keeping recursive symbolic determinant as an unbounded production backend.
