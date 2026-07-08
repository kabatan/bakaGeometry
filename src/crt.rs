use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{One, Zero};

use crate::univariate::UniPolynomialFp;
use crate::Variable;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum CrtError {
    EmptyInput,
    VariableMismatch,
    NonCoprimeModuli,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ModularUniPolynomialLift {
    pub variable: Variable,
    pub modulus: BigInt,
    pub residues: Vec<BigInt>,
}

pub(crate) fn combine_univariate_polynomial_residues(
    polynomials: &[UniPolynomialFp],
) -> Result<ModularUniPolynomialLift, CrtError> {
    let first = polynomials.first().ok_or(CrtError::EmptyInput)?;
    let variable = first.variable.clone();
    if polynomials
        .iter()
        .any(|polynomial| polynomial.variable != variable)
    {
        return Err(CrtError::VariableMismatch);
    }
    let max_len = polynomials
        .iter()
        .map(|polynomial| polynomial.coefficients.len())
        .max()
        .unwrap_or(0);
    let mut residues = vec![BigInt::zero(); max_len];
    let mut modulus = BigInt::one();

    for polynomial in polynomials {
        let next_modulus = BigInt::from(polynomial.modulus);
        for degree in 0..max_len {
            let residue = BigInt::from(*polynomial.coefficients.get(degree).unwrap_or(&0));
            let (combined, _) =
                combine_coprime_residues(&residues[degree], &modulus, &residue, &next_modulus)?;
            residues[degree] = combined;
        }
        modulus *= next_modulus;
    }

    Ok(ModularUniPolynomialLift {
        variable,
        modulus,
        residues,
    })
}

pub(crate) fn combine_coprime_residues(
    left_residue: &BigInt,
    left_modulus: &BigInt,
    right_residue: &BigInt,
    right_modulus: &BigInt,
) -> Result<(BigInt, BigInt), CrtError> {
    if left_modulus.gcd(right_modulus) != BigInt::one() {
        return Err(CrtError::NonCoprimeModuli);
    }
    let inverse = modular_inverse(left_modulus, right_modulus).ok_or(CrtError::NonCoprimeModuli)?;
    let delta = (right_residue - left_residue).mod_floor(right_modulus);
    let adjustment = (delta * inverse).mod_floor(right_modulus);
    let modulus = left_modulus * right_modulus;
    let residue = (left_residue + left_modulus * adjustment).mod_floor(&modulus);
    Ok((residue, modulus))
}

fn modular_inverse(value: &BigInt, modulus: &BigInt) -> Option<BigInt> {
    let (gcd, x, _) = extended_gcd(value, modulus);
    (gcd == BigInt::one()).then(|| x.mod_floor(modulus))
}

fn extended_gcd(a: &BigInt, b: &BigInt) -> (BigInt, BigInt, BigInt) {
    if b.is_zero() {
        return (a.clone(), BigInt::one(), BigInt::zero());
    }
    let (gcd, x1, y1) = extended_gcd(b, &(a % b));
    let x = y1.clone();
    let y = x1 - (a / b) * y1;
    (gcd, x, y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::univariate::UniPolynomialFp;

    fn variable(symbol: &str) -> Variable {
        Variable {
            symbol: symbol.to_string(),
        }
    }

    #[test]
    fn crt_combines_matching_modular_univariate_polynomials() {
        let t = variable("T");
        let lift = combine_univariate_polynomial_residues(&[
            UniPolynomialFp {
                variable: t.clone(),
                modulus: 5,
                coefficients: vec![2, 1],
            },
            UniPolynomialFp {
                variable: t.clone(),
                modulus: 7,
                coefficients: vec![0, 1],
            },
        ])
        .unwrap();

        assert_eq!(lift.variable, t);
        assert_eq!(lift.modulus, BigInt::from(35));
        assert_eq!(lift.residues, vec![BigInt::from(7), BigInt::from(1)]);
    }
}
