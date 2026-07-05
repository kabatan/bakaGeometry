# P3a Evidence Notes

P3a implements exact polynomial operation primitives and a membership verifier foundation only. It does not close the full solver pipeline, candidate cover, exact image, root isolation, global support verification, or any P3b+ algebra phase.

Implemented behavior:

- `leading_term` selects the leading term using the declared monomial order.
- `s_polynomial` uses LCM leading monomials and exact leading-coefficient division over Q.
- `reduce_by_set` performs exact reduction with coefficient-aware quotients and a remainder satisfying `f = Σ q_i g_i + r`.
- `content_primitive_part` returns exact rational content and primitive integer part.
- `normal_form` returns exact reduction remainder.
- `verify_membership_by_certificate` reconstructs `Σ multiplier_i * relation_i - g` over Q and accepts only if the normalized polynomial is zero.

Negative tests cover incorrect multiplier certificates and out-of-range relation references. The implementation does not accept membership by hash equality.
