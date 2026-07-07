use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::Rational;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PrimeModulus {
    value: u64,
}

impl PrimeModulus {
    pub(crate) fn new(value: u64) -> Option<Self> {
        is_prime(value).then_some(Self { value })
    }

    pub(crate) fn value(self) -> u64 {
        self.value
    }

    pub(crate) fn normalize(self, value: u64) -> u64 {
        value % self.value
    }

    pub(crate) fn add(self, left: u64, right: u64) -> u64 {
        (((left % self.value) as u128 + (right % self.value) as u128) % self.value as u128) as u64
    }

    pub(crate) fn sub(self, left: u64, right: u64) -> u64 {
        (((left % self.value) as u128 + self.value as u128 - (right % self.value) as u128)
            % self.value as u128) as u64
    }

    pub(crate) fn neg(self, value: u64) -> u64 {
        self.sub(0, value)
    }

    pub(crate) fn mul(self, left: u64, right: u64) -> u64 {
        (((left % self.value) as u128 * (right % self.value) as u128) % self.value as u128) as u64
    }

    pub(crate) fn pow(self, mut base: u64, mut exponent: u64) -> u64 {
        base %= self.value;
        let mut result = 1;
        while exponent > 0 {
            if exponent % 2 == 1 {
                result = self.mul(result, base);
            }
            base = self.mul(base, base);
            exponent /= 2;
        }
        result
    }

    pub(crate) fn inv(self, value: u64) -> Option<u64> {
        let value = value % self.value;
        if value == 0 {
            return None;
        }
        Some(self.pow(value, self.value - 2))
    }
}

fn is_prime(value: u64) -> bool {
    const SMALL_PRIMES: [u64; 12] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];
    if value < 2 {
        return false;
    }
    for prime in SMALL_PRIMES {
        if value == prime {
            return true;
        }
        if value % prime == 0 {
            return false;
        }
    }

    let mut odd_part = value - 1;
    let mut powers_of_two = 0;
    while odd_part % 2 == 0 {
        odd_part /= 2;
        powers_of_two += 1;
    }

    [2, 325, 9_375, 28_178, 450_775, 9_780_504, 1_795_265_022]
        .into_iter()
        .all(|base| !is_miller_rabin_witness(base, value, odd_part, powers_of_two))
}

fn is_miller_rabin_witness(base: u64, modulus: u64, odd_part: u64, powers_of_two: u32) -> bool {
    let base = base % modulus;
    if base == 0 {
        return false;
    }
    let mut value = mod_pow(base, odd_part, modulus);
    if value == 1 || value == modulus - 1 {
        return false;
    }
    for _ in 1..powers_of_two {
        value = mod_mul(value, value, modulus);
        if value == modulus - 1 {
            return false;
        }
    }
    true
}

fn mod_pow(mut base: u64, mut exponent: u64, modulus: u64) -> u64 {
    let mut result = 1;
    while exponent > 0 {
        if exponent % 2 == 1 {
            result = mod_mul(result, base, modulus);
        }
        base = mod_mul(base, base, modulus);
        exponent /= 2;
    }
    result
}

fn mod_mul(left: u64, right: u64, modulus: u64) -> u64 {
    ((left as u128 * right as u128) % modulus as u128) as u64
}

pub(crate) fn rational_to_mod_prime(coefficient: &Rational, modulus: PrimeModulus) -> Option<u64> {
    let numerator = big_int_to_mod_prime(coefficient.numer(), modulus);
    let denominator = big_int_to_mod_prime(coefficient.denom(), modulus);
    let denominator_inverse = modulus.inv(denominator)?;
    Some(modulus.mul(numerator, denominator_inverse))
}

fn big_int_to_mod_prime(value: &BigInt, modulus: PrimeModulus) -> u64 {
    let prime = BigInt::from(modulus.value());
    let mut reduced = value % &prime;
    if reduced < BigInt::from(0) {
        reduced += prime;
    }
    reduced.to_u64().expect("reduced residue must fit into u64")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prime_modulus_rejects_composite_values() {
        assert!(PrimeModulus::new(1).is_none());
        assert!(PrimeModulus::new(15).is_none());
        assert_eq!(PrimeModulus::new(17).unwrap().value(), 17);
    }

    #[test]
    fn finite_field_operations_are_modular() {
        let modulus = PrimeModulus::new(17).unwrap();

        assert_eq!(modulus.add(16, 3), 2);
        assert_eq!(modulus.sub(3, 5), 15);
        assert_eq!(modulus.mul(8, 9), 4);
        assert_eq!(modulus.pow(3, 4), 13);
        assert_eq!(modulus.inv(5), Some(7));
    }

    #[test]
    fn finite_field_operations_avoid_large_prime_overflow() {
        let modulus = PrimeModulus::new(18_446_744_073_709_551_557).unwrap();
        let value = modulus.value();

        assert_eq!(modulus.add(value - 1, value - 1), value - 2);
        assert_eq!(modulus.sub(1, value - 1), 2);
        assert_eq!(modulus.mul(value - 1, value - 1), 1);
    }
}
