use std::panic;

use geosolver_core::{Monomial, PolynomialQ, Rational, UniPolynomialQ, Variable};
use num_bigint::BigInt;
use num_rational::BigRational;

fn var(symbol: &str) -> Variable {
    Variable {
        symbol: symbol.to_string(),
    }
}

fn rat(numerator: i64, denominator: i64) -> Rational {
    BigRational::new(BigInt::from(numerator), BigInt::from(denominator))
}

fn int(value: i64) -> Rational {
    rat(value, 1)
}

fn uni(variable: &Variable, coefficients: &[i64]) -> UniPolynomialQ {
    let mut polynomial = UniPolynomialQ {
        variable: variable.clone(),
        coefficients: coefficients.iter().map(|value| int(*value)).collect(),
    };
    polynomial.normalize();
    polynomial
}

#[test]
fn zero_terms_are_removed_from_sparse_polynomials() {
    let x = var("X");
    let polynomial = PolynomialQ::from_term(vec![x], int(0), Monomial { exponents: vec![3] });

    assert!(polynomial.is_zero());
    assert!(polynomial.support().is_empty());
}

#[test]
fn polynomial_arithmetic_rejects_variable_order_mismatch() {
    let x = var("X");
    let y = var("Y");
    let left = PolynomialQ::one(vec![x.clone(), y.clone()]);
    let right = PolynomialQ::one(vec![y, x]);

    assert!(panic::catch_unwind(|| left.add(&right)).is_err());
}

#[test]
fn monomial_divisibility_and_quotient_are_exact() {
    let left = Monomial {
        exponents: vec![4, 1, 3],
    };
    let right = Monomial {
        exponents: vec![1, 1, 2],
    };

    assert!(left.is_divisible_by(&right));
    assert_eq!(
        left.quotient_if_divisible_by(&right),
        Some(Monomial {
            exponents: vec![3, 0, 1]
        })
    );
    assert_eq!(
        left.multiply(&right),
        Monomial {
            exponents: vec![5, 2, 5]
        }
    );
    assert!(!right.is_divisible_by(&left));
}

#[test]
fn primitive_integer_normalization_clears_denominators_content_and_sign() {
    let t = var("T");
    let mut polynomial = UniPolynomialQ {
        variable: t.clone(),
        coefficients: vec![rat(-2, 3), rat(-4, 9)],
    };
    polynomial.normalize();

    let normalized = polynomial.primitive_integer_normalized();

    assert_eq!(
        normalized,
        UniPolynomialQ {
            variable: t,
            coefficients: vec![int(3), int(2)]
        }
    );
}

#[test]
fn univariate_gcd_and_lcm_are_exact_over_q() {
    let t = var("T");
    let t_squared_minus_one = uni(&t, &[-1, 0, 1]);
    let t_squared_minus_t = uni(&t, &[0, -1, 1]);

    assert_eq!(
        t_squared_minus_one.gcd(&t_squared_minus_t),
        uni(&t, &[-1, 1])
    );
    assert_eq!(
        t_squared_minus_one.lcm(&t_squared_minus_t),
        uni(&t, &[0, -1, 0, 1])
    );
}

#[test]
fn squarefree_part_is_exact_and_does_not_mutate_original() {
    let t = var("T");
    let repeated = uni(&t, &[1, -2, 1]);
    let squarefree = repeated.squarefree_part();

    assert_eq!(squarefree, uni(&t, &[-1, 1]));
    assert_eq!(repeated, uni(&t, &[1, -2, 1]));
}
