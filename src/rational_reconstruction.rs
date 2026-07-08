use num_bigint::BigInt;
use num_integer::Integer;
use num_rational::BigRational;
use num_traits::{One, Signed, ToPrimitive};

use crate::crt::{combine_univariate_polynomial_residues, CrtError};
use crate::univariate::{UniPolynomialFp, UniPolynomialQ};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RationalReconstructionBounds {
    pub numerator_abs: BigInt,
    pub denominator_abs: BigInt,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum RationalReconstructionError {
    InvalidModulus,
    SearchBoundTooLarge,
    NoCandidate,
    NonUnique,
    Crt(CrtError),
}

pub(crate) fn reconstruct_univariate_q_from_modular(
    polynomials: &[UniPolynomialFp],
    bounds: &RationalReconstructionBounds,
) -> Result<UniPolynomialQ, RationalReconstructionError> {
    let lift = combine_univariate_polynomial_residues(polynomials)
        .map_err(RationalReconstructionError::Crt)?;
    let coefficients = lift
        .residues
        .iter()
        .map(|residue| reconstruct_rational(residue, &lift.modulus, bounds))
        .collect::<Result<Vec<_>, _>>()?;
    let mut polynomial = UniPolynomialQ {
        variable: lift.variable,
        coefficients,
    };
    polynomial.normalize();
    Ok(polynomial)
}

pub(crate) fn reconstruct_rational(
    residue: &BigInt,
    modulus: &BigInt,
    bounds: &RationalReconstructionBounds,
) -> Result<BigRational, RationalReconstructionError> {
    if modulus <= &BigInt::one() {
        return Err(RationalReconstructionError::InvalidModulus);
    }
    let denominator_bound = bounds
        .denominator_abs
        .to_u64()
        .ok_or(RationalReconstructionError::SearchBoundTooLarge)?;
    if denominator_bound > 1_000_000 {
        return Err(RationalReconstructionError::SearchBoundTooLarge);
    }

    let mut found = None;
    for denominator_u64 in 1..=denominator_bound.max(1) {
        let denominator = BigInt::from(denominator_u64);
        if denominator.gcd(modulus) != BigInt::one() {
            continue;
        }
        let numerator = centered_residue(&(residue * &denominator), modulus);
        if numerator.abs() > bounds.numerator_abs {
            continue;
        }
        if numerator.gcd(&denominator) != BigInt::one() {
            continue;
        }
        if found.is_some() {
            return Err(RationalReconstructionError::NonUnique);
        }
        found = Some(BigRational::new(numerator, denominator));
    }

    found.ok_or(RationalReconstructionError::NoCandidate)
}

fn centered_residue(value: &BigInt, modulus: &BigInt) -> BigInt {
    let residue = value.mod_floor(modulus);
    if &residue * 2 > *modulus {
        residue - modulus
    } else {
        residue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::univariate::UniPolynomialFp;
    use crate::Variable;

    fn variable(symbol: &str) -> Variable {
        Variable {
            symbol: symbol.to_string(),
        }
    }

    #[test]
    fn rational_reconstruction_recovers_integer_larger_than_one_prime() {
        let t = variable("T");
        let bounds = RationalReconstructionBounds {
            numerator_abs: BigInt::from(100),
            denominator_abs: BigInt::one(),
        };
        let reconstructed = reconstruct_univariate_q_from_modular(
            &[
                UniPolynomialFp {
                    variable: t.clone(),
                    modulus: 5,
                    coefficients: vec![2],
                },
                UniPolynomialFp {
                    variable: t.clone(),
                    modulus: 11,
                    coefficients: vec![9],
                },
                UniPolynomialFp {
                    variable: t.clone(),
                    modulus: 13,
                    coefficients: vec![3],
                },
            ],
            &bounds,
        )
        .unwrap();

        assert_eq!(
            reconstructed.coefficients,
            vec![BigRational::from_integer(BigInt::from(42))]
        );
    }

    #[test]
    fn rational_reconstruction_has_explicit_failure_for_ambiguous_bounds() {
        let bounds = RationalReconstructionBounds {
            numerator_abs: BigInt::from(4),
            denominator_abs: BigInt::from(4),
        };

        assert_eq!(
            reconstruct_rational(&BigInt::from(1), &BigInt::from(5), &bounds),
            Err(RationalReconstructionError::NonUnique)
        );
    }
}
