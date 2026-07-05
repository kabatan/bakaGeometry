use num_bigint::{BigInt, Sign};
use num_integer::Integer;
use num_traits::{One, Signed, ToPrimitive, Zero};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RationalQ {
    pub num: BigInt,
    pub den: BigInt,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("division by zero rational")]
pub struct DivisionByZero;

pub fn new_q(num: BigInt, den: BigInt) -> RationalQ {
    assert!(!den.is_zero(), "zero denominator");
    normalize_q(RationalQ { num, den })
}

pub fn normalize_q(mut q: RationalQ) -> RationalQ {
    assert!(!q.den.is_zero(), "zero denominator");
    if q.num.is_zero() {
        return RationalQ {
            num: BigInt::zero(),
            den: BigInt::one(),
        };
    }
    if q.den.sign() == Sign::Minus {
        q.num = -q.num;
        q.den = -q.den;
    }
    let g = q.num.abs().gcd(&q.den);
    RationalQ {
        num: q.num / &g,
        den: q.den / g,
    }
}

pub fn zero_q() -> RationalQ {
    RationalQ {
        num: BigInt::zero(),
        den: BigInt::one(),
    }
}

pub fn one_q() -> RationalQ {
    RationalQ {
        num: BigInt::one(),
        den: BigInt::one(),
    }
}

pub fn int_q(n: i64) -> RationalQ {
    RationalQ {
        num: BigInt::from(n),
        den: BigInt::one(),
    }
}

pub fn add_q(a: &RationalQ, b: &RationalQ) -> RationalQ {
    new_q(&a.num * &b.den + &b.num * &a.den, &a.den * &b.den)
}

pub fn sub_q(a: &RationalQ, b: &RationalQ) -> RationalQ {
    new_q(&a.num * &b.den - &b.num * &a.den, &a.den * &b.den)
}

pub fn neg_q(a: &RationalQ) -> RationalQ {
    RationalQ {
        num: -a.num.clone(),
        den: a.den.clone(),
    }
}

pub fn mul_q(a: &RationalQ, b: &RationalQ) -> RationalQ {
    new_q(&a.num * &b.num, &a.den * &b.den)
}

pub fn div_q(a: &RationalQ, b: &RationalQ) -> Result<RationalQ, DivisionByZero> {
    if b.num.is_zero() {
        return Err(DivisionByZero);
    }
    Ok(new_q(&a.num * &b.den, &a.den * &b.num))
}

pub fn bit_height_q(q: &RationalQ) -> usize {
    let num_bits = q.num.abs().to_biguint().map_or(0, |n| n.bits() as usize);
    let den_bits = q.den.abs().to_biguint().map_or(0, |n| n.bits() as usize);
    num_bits.max(den_bits)
}

pub fn rational_to_bytes(q: &RationalQ) -> Vec<u8> {
    let mut out = Vec::new();
    let num = q.num.to_signed_bytes_be();
    let den = q.den.to_signed_bytes_be();
    out.extend_from_slice((num.len() as u64).to_be_bytes().as_slice());
    out.extend_from_slice(&num);
    out.extend_from_slice((den.len() as u64).to_be_bytes().as_slice());
    out.extend_from_slice(&den);
    out
}

pub fn lcm_denominators<'a>(values: impl IntoIterator<Item = &'a RationalQ>) -> BigInt {
    values
        .into_iter()
        .fold(BigInt::one(), |acc, q| acc.lcm(&q.den))
}

pub fn is_zero_q(q: &RationalQ) -> bool {
    q.num.is_zero()
}

pub fn sign_q(q: &RationalQ) -> i8 {
    q.num.signum().to_i8().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_sign_and_gcd() {
        let q = new_q(BigInt::from(6), BigInt::from(-8));
        assert_eq!(q.num, BigInt::from(-3));
        assert_eq!(q.den, BigInt::from(4));
    }

    #[test]
    fn zero_is_canonical() {
        let q = new_q(BigInt::from(0), BigInt::from(99));
        assert_eq!(q.num, BigInt::zero());
        assert_eq!(q.den, BigInt::one());
    }

    #[test]
    fn exact_arithmetic_identity() {
        let a = new_q(BigInt::from(1), BigInt::from(2));
        let b = new_q(BigInt::from(1), BigInt::from(3));
        let sum = add_q(&a, &b);
        assert_eq!(sum, new_q(BigInt::from(5), BigInt::from(6)));
        assert_eq!(sub_q(&sum, &b), a);
        assert_eq!(mul_q(&a, &b), new_q(BigInt::from(1), BigInt::from(6)));
        assert_eq!(
            div_q(&a, &b).unwrap(),
            new_q(BigInt::from(3), BigInt::from(2))
        );
    }
}
