# P5R-c Notes

The safe stress case `(x+1)*s-1`, `(x+1)*y-(T+x)`, `y-2` now uses a guarded rational affine pivot and clears denominators exactly. The no-witness variant leaves the unsafe affine relation in the system and does not return Unsupported or InvalidInput.

After P5R-c re-review feedback, `CompressedSystemQ` now persists `rational_affine_transformations`, with per-relation `RationalAffineTransformationCertificate` records covering pivot relation, eliminated variable, numerator, denominator, denominator-clearing power, denominator guard, witness relation ids, original/transformed relation hashes, and transformation hash.

After the exact-provenance re-review, the safe stress test now asserts exact original/transformed relation ids, exact pivot and witness ids, guard source ids, original and transformed relation hashes, recomputed transformation hash equality, and tamper hash inequality. `apply_rational_affine_substitution` now stores the final transformed canonical relation hash in `transformed_relation_hash`, not the intermediate polynomial hash.
