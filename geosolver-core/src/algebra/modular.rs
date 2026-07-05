use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{Signed, ToPrimitive, Zero};
use serde::{Deserialize, Serialize};

use crate::types::hash::hash_sequence;
use crate::types::monomial::Monomial;
use crate::types::polynomial::SparsePolynomialQ;

pub type Prime = u64;
pub type Fp = u64;
pub type IntegerRepresentative = BigInt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TermFp {
    pub coeff: Fp,
    pub monomial: Monomial,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SparsePolynomialFp {
    pub prime: Prime,
    pub terms: Vec<TermFp>,
    pub hash: crate::types::hash::Hash,
}

pub fn choose_prime_avoiding_denominators(polys: &[SparsePolynomialQ], seed: u64) -> Prime {
    let mut p = next_prime_at_or_after(seed.max(2));
    while prime_divides_any_forbidden_coeff_part(polys, p) {
        p = next_prime_after(p);
    }
    p
}

pub fn reduce_q_to_fp(p: &SparsePolynomialQ, prime: Prime) -> SparsePolynomialFp {
    let mut terms = Vec::new();
    for term in &p.terms {
        let coeff = reduce_rational_coeff(&term.coeff.num, &term.coeff.den, prime);
        if coeff != 0 {
            terms.push(TermFp {
                coeff,
                monomial: term.monomial.clone(),
            });
        }
    }
    terms.sort_by(|a, b| a.monomial.cmp(&b.monomial));
    let hash = hash_fp_poly(prime, &terms);
    SparsePolynomialFp { prime, terms, hash }
}

pub fn lift_fp_coeff(c: Fp, prime: Prime) -> IntegerRepresentative {
    let c = c % prime;
    if c > prime / 2 {
        BigInt::from(c) - BigInt::from(prime)
    } else {
        BigInt::from(c)
    }
}

pub fn next_prime_at_or_after(n: u64) -> Prime {
    let mut candidate = n.max(2);
    while !is_prime(candidate) {
        candidate = candidate.saturating_add(1);
    }
    candidate
}

pub fn next_prime_after(p: Prime) -> Prime {
    next_prime_at_or_after(p.saturating_add(1))
}

pub fn add_mod(a: u64, b: u64, prime: Prime) -> u64 {
    ((a as u128 + b as u128) % prime as u128) as u64
}

pub fn sub_mod(a: u64, b: u64, prime: Prime) -> u64 {
    ((a as u128 + prime as u128 - (b % prime) as u128) % prime as u128) as u64
}

pub fn mul_mod(a: u64, b: u64, prime: Prime) -> u64 {
    ((a as u128 * b as u128) % prime as u128) as u64
}

pub fn inv_mod_u64(a: u64, prime: Prime) -> Option<u64> {
    let a = a % prime;
    if a == 0 {
        return None;
    }
    Some(pow_mod(a, prime - 2, prime))
}

pub fn reduce_bigint_mod(value: &BigInt, prime: Prime) -> u64 {
    let p = BigInt::from(prime);
    let mut reduced = value.mod_floor(&p);
    if reduced.is_negative() {
        reduced += &p;
    }
    reduced.to_u64().unwrap()
}

pub fn reduce_rational_coeff(num: &BigInt, den: &BigInt, prime: Prime) -> u64 {
    let n = reduce_bigint_mod(num, prime);
    let d = reduce_bigint_mod(den, prime);
    let inv = inv_mod_u64(d, prime).expect("denominator must be nonzero modulo chosen prime");
    mul_mod(n, inv, prime)
}

fn prime_divides_any_forbidden_coeff_part(polys: &[SparsePolynomialQ], prime: Prime) -> bool {
    let p = BigInt::from(prime);
    polys.iter().any(|poly| {
        poly.terms.iter().any(|term| {
            term.coeff.den.mod_floor(&p).is_zero()
                || term.coeff.num.mod_floor(&p).is_zero() && !term.coeff.num.is_zero()
        })
    })
}

fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 {
        return true;
    }
    if n % 2 == 0 {
        return false;
    }
    let mut d = 3;
    while d <= n / d {
        if n % d == 0 {
            return false;
        }
        d += 2;
    }
    true
}

fn pow_mod(mut base: u64, mut exp: u64, prime: Prime) -> u64 {
    let mut result = 1 % prime;
    base %= prime;
    while exp > 0 {
        if exp & 1 == 1 {
            result = mul_mod(result, base, prime);
        }
        base = mul_mod(base, base, prime);
        exp >>= 1;
    }
    result
}

fn hash_fp_poly(prime: Prime, terms: &[TermFp]) -> crate::types::hash::Hash {
    let chunks: Vec<Vec<u8>> = terms
        .iter()
        .map(|term| {
            let mut bytes = Vec::new();
            bytes.extend_from_slice(&prime.to_be_bytes());
            bytes.extend_from_slice(&term.coeff.to_be_bytes());
            bytes.extend_from_slice(&crate::types::monomial::monomial_to_bytes(&term.monomial));
            bytes
        })
        .collect();
    hash_sequence("poly-fp", &chunks)
}

#[cfg(test)]
mod tests {
    use num_bigint::BigInt;

    use super::*;
    use crate::types::hash::hash_sequence;
    use crate::types::ids::VariableId;
    use crate::types::monomial::normalize_monomial;
    use crate::types::polynomial::{normalize_poly, TermQ};
    use crate::types::rational::{int_q, new_q};

    #[test]
    fn prime_choice_is_deterministic_and_avoids_denominators_and_coefficients() {
        let p = normalize_poly(SparsePolynomialQ {
            terms: vec![TermQ {
                coeff: new_q(BigInt::from(5), BigInt::from(2)),
                monomial: normalize_monomial(vec![(VariableId(1), 1)]),
            }],
            hash: hash_sequence("poly", &[]),
        });
        assert_eq!(choose_prime_avoiding_denominators(&[p.clone()], 2), 3);
        assert_eq!(choose_prime_avoiding_denominators(&[p], 5), 7);
    }

    #[test]
    fn rational_coefficients_reduce_and_lift_exactly() {
        let p = normalize_poly(SparsePolynomialQ {
            terms: vec![
                TermQ {
                    coeff: new_q(BigInt::from(1), BigInt::from(2)),
                    monomial: normalize_monomial(vec![(VariableId(1), 1)]),
                },
                TermQ {
                    coeff: new_q(BigInt::from(-1), BigInt::from(3)),
                    monomial: normalize_monomial(Vec::new()),
                },
            ],
            hash: hash_sequence("poly", &[]),
        });
        let fp = reduce_q_to_fp(&p, 7);
        assert_eq!(fp.terms[0].coeff, 2);
        assert_eq!(fp.terms[1].coeff, 4);
        assert_eq!(lift_fp_coeff(6, 7), BigInt::from(-1));
        assert_eq!(reduce_rational_coeff(&int_q(3).num, &int_q(2).den, 7), 3);
    }
}
