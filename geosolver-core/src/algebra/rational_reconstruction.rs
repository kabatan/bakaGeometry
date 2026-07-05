use std::collections::BTreeSet;

use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{One, Signed, ToPrimitive, Zero};
use serde::{Deserialize, Serialize};

use crate::algebra::crt::{canonical_mod, ModInteger};
use crate::types::hash::hash_sequence;
use crate::types::monomial::Monomial;
use crate::types::polynomial::{normalize_poly, SparsePolynomialQ, TermQ};
use crate::types::rational::{new_q, zero_q, RationalQ};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModPolynomialTerm {
    pub coeff: BigInt,
    pub monomial: Monomial,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModPolynomialData {
    pub terms: Vec<ModPolynomialTerm>,
    pub height_bound: Option<usize>,
}

pub fn reconstruct_rational(
    a_mod_m: ModInteger,
    modulus: BigInt,
    height_bound: Option<usize>,
) -> Option<RationalQ> {
    if modulus <= BigInt::one() || a_mod_m.modulus != modulus {
        return None;
    }
    let a = canonical_mod(&a_mod_m.value, &modulus);
    if a.is_zero() {
        return Some(zero_q());
    }
    let cap = reconstruction_cap(&modulus, height_bound)?;
    let mut candidates = BTreeSet::new();
    for d in 1..=cap {
        let d_big = BigInt::from(d);
        if d_big.gcd(&modulus) != BigInt::one() {
            continue;
        }
        let n_mod = canonical_mod(&(a.clone() * &d_big), &modulus);
        let n = centered_residue(&n_mod, &modulus);
        if n.abs() > BigInt::from(cap) {
            continue;
        }
        if n.gcd(&d_big).abs() != BigInt::one() {
            continue;
        }
        candidates.insert(new_q(n, d_big));
    }
    if candidates.len() == 1 {
        candidates.into_iter().next()
    } else {
        None
    }
}

pub fn reconstruct_polynomial(
    mod_poly_data: ModPolynomialData,
    modulus: BigInt,
) -> Option<SparsePolynomialQ> {
    let mut terms = Vec::new();
    for term in mod_poly_data.terms {
        let coeff = reconstruct_rational(
            ModInteger {
                value: term.coeff,
                modulus: modulus.clone(),
            },
            modulus.clone(),
            mod_poly_data.height_bound,
        )?;
        terms.push(TermQ {
            coeff,
            monomial: term.monomial,
        });
    }
    Some(normalize_poly(SparsePolynomialQ {
        terms,
        hash: hash_sequence("poly", &[]),
    }))
}

fn reconstruction_cap(modulus: &BigInt, height_bound: Option<usize>) -> Option<u64> {
    if let Some(bits) = height_bound {
        if bits == 0 || bits >= 63 {
            return None;
        }
        return Some((1u64 << bits) - 1);
    }
    let half = modulus / 2;
    integer_sqrt(&half).to_u64()
}

fn integer_sqrt(n: &BigInt) -> BigInt {
    if *n <= BigInt::zero() {
        return BigInt::zero();
    }
    let mut lo = BigInt::zero();
    let mut hi = n.clone() + BigInt::one();
    while &hi - &lo > BigInt::one() {
        let mid = (&lo + &hi) / 2;
        if &mid * &mid <= *n {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}

fn centered_residue(value: &BigInt, modulus: &BigInt) -> BigInt {
    let half = modulus / 2;
    if value > &half {
        value - modulus
    } else {
        value.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ids::VariableId;
    use crate::types::monomial::normalize_monomial;
    use crate::types::polynomial::{constant_poly, variable_poly};

    #[test]
    fn rational_reconstruction_round_trip_is_exact() {
        let modulus = BigInt::from(101);
        let residue = BigInt::from(52);
        let q = reconstruct_rational(
            ModInteger {
                value: residue,
                modulus: modulus.clone(),
            },
            modulus,
            Some(3),
        )
        .unwrap();
        assert_eq!(q, new_q(BigInt::from(3), BigInt::from(2)));
    }

    #[test]
    fn unstable_reconstruction_fails_when_multiple_small_lifts_exist() {
        let modulus = BigInt::from(5);
        assert!(reconstruct_rational(
            ModInteger {
                value: BigInt::from(1),
                modulus: modulus.clone(),
            },
            modulus,
            Some(2),
        )
        .is_none());
    }

    #[test]
    fn polynomial_reconstruction_normalizes_terms() {
        let modulus = BigInt::from(101);
        let poly = reconstruct_polynomial(
            ModPolynomialData {
                terms: vec![ModPolynomialTerm {
                    coeff: BigInt::from(2),
                    monomial: normalize_monomial(vec![(VariableId(1), 1)]),
                }],
                height_bound: Some(2),
            },
            modulus,
        )
        .unwrap();
        assert_eq!(
            poly,
            crate::types::polynomial::poly_scale(
                &variable_poly(VariableId(1)),
                &new_q(BigInt::from(2), BigInt::one()),
            )
        );
        assert_ne!(poly, constant_poly(new_q(BigInt::from(2), BigInt::one())));
    }
}
