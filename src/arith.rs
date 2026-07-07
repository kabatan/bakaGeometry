use num_bigint::BigInt;
use num_integer::Integer;
use num_rational::BigRational;
use num_traits::{One, Signed, Zero};

pub(crate) fn rational_zero() -> BigRational {
    BigRational::zero()
}

pub(crate) fn rational_one() -> BigRational {
    BigRational::one()
}

pub(crate) fn lcm_bigint(values: impl IntoIterator<Item = BigInt>) -> BigInt {
    values
        .into_iter()
        .fold(BigInt::one(), |acc, value| acc.lcm(&value.abs()))
}
