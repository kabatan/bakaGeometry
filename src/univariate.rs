use num_bigint::BigInt;
use num_integer::Integer;
use num_rational::BigRational;
use num_traits::{One, Signed, ToPrimitive, Zero};

use crate::{Monomial, PolynomialQ, Rational, Variable};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct UniPolynomialQ {
    pub variable: Variable,
    pub coefficients: Vec<Rational>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct UniPolynomialFp {
    pub variable: Variable,
    pub modulus: u64,
    pub coefficients: Vec<u64>,
}

impl UniPolynomialFp {
    pub fn normalize(&mut self) {
        while self
            .coefficients
            .last()
            .is_some_and(|coefficient| *coefficient == 0)
        {
            self.coefficients.pop();
        }
    }

    pub fn is_zero(&self) -> bool {
        self.coefficients.is_empty()
    }
}

impl UniPolynomialQ {
    pub fn zero(variable: Variable) -> Self {
        Self {
            variable,
            coefficients: Vec::new(),
        }
    }

    pub fn one(variable: Variable) -> Self {
        Self {
            variable,
            coefficients: vec![crate::arith::rational_one()],
        }
    }

    pub fn degree(&self) -> Option<usize> {
        self.coefficients
            .iter()
            .rposition(|coefficient| !coefficient.is_zero())
    }

    pub fn is_zero(&self) -> bool {
        self.degree().is_none()
    }

    pub fn normalize(&mut self) {
        while self
            .coefficients
            .last()
            .is_some_and(num_traits::Zero::is_zero)
        {
            self.coefficients.pop();
        }
    }

    pub fn primitive_integer_normalized(&self) -> Self {
        if self.is_zero() {
            return Self::zero(self.variable.clone());
        }

        let denominator_lcm = crate::arith::lcm_bigint(
            self.coefficients
                .iter()
                .map(|coefficient| coefficient.denom().clone()),
        );
        let mut integer_coefficients = self
            .coefficients
            .iter()
            .map(|coefficient| coefficient.numer() * (&denominator_lcm / coefficient.denom()))
            .collect::<Vec<BigInt>>();

        let integer_content = integer_coefficients
            .iter()
            .filter(|coefficient| !coefficient.is_zero())
            .fold(BigInt::zero(), |accumulator, coefficient| {
                if accumulator.is_zero() {
                    coefficient.abs()
                } else {
                    accumulator.gcd(&coefficient.abs())
                }
            });

        if !integer_content.is_zero() && integer_content != BigInt::one() {
            for coefficient in &mut integer_coefficients {
                *coefficient /= integer_content.clone();
            }
        }

        if integer_coefficients
            .iter()
            .rposition(|coefficient| !coefficient.is_zero())
            .is_some_and(|degree| integer_coefficients[degree].is_negative())
        {
            for coefficient in &mut integer_coefficients {
                *coefficient = -coefficient.clone();
            }
        }

        let mut result = Self {
            variable: self.variable.clone(),
            coefficients: integer_coefficients
                .into_iter()
                .map(BigRational::from_integer)
                .collect(),
        };
        result.normalize();
        result
    }

    pub fn squarefree_part(&self) -> Self {
        if self.degree().is_none_or(|degree| degree == 0) {
            return self.primitive_integer_normalized();
        }
        let derivative = self.derivative();
        if derivative.is_zero() {
            return self.primitive_integer_normalized();
        }
        let repeated_part = self.gcd(&derivative);
        let (quotient, remainder) = self.div_rem(&repeated_part);
        assert!(remainder.is_zero());
        quotient.primitive_integer_normalized()
    }

    pub fn gcd(&self, rhs: &Self) -> Self {
        assert_eq!(self.variable, rhs.variable);
        let mut left = self.primitive_integer_normalized();
        let mut right = rhs.primitive_integer_normalized();

        while !right.is_zero() {
            let (_, remainder) = left.div_rem(&right);
            left = right;
            right = remainder.primitive_integer_normalized();
        }

        left.primitive_integer_normalized()
    }

    pub fn lcm(&self, rhs: &Self) -> Self {
        assert_eq!(self.variable, rhs.variable);
        if self.is_zero() || rhs.is_zero() {
            Self::zero(self.variable.clone())
        } else {
            let gcd = self.gcd(rhs);
            let (quotient, remainder) = self.div_rem(&gcd);
            assert!(remainder.is_zero());
            quotient.mul(rhs).primitive_integer_normalized()
        }
    }

    pub fn factor_squarefree_over_q(&self) -> Vec<Self> {
        let mut remaining = self.squarefree_part().primitive_integer_normalized();
        if remaining.is_zero() {
            return Vec::new();
        }
        if remaining.degree().is_none_or(|degree| degree == 0) {
            return vec![remaining];
        }

        let mut factors = Vec::new();
        while remaining.degree().is_some_and(|degree| degree > 1) {
            let Some(root) = rational_root(&remaining) else {
                break;
            };
            let factor = linear_factor_for_root(&remaining.variable, &root);
            let (quotient, remainder) = remaining.div_rem(&factor);
            if !remainder.is_zero() {
                break;
            }
            factors.push(factor.primitive_integer_normalized());
            remaining = quotient.primitive_integer_normalized();
        }

        if !remaining.is_zero() && remaining.degree().is_some_and(|degree| degree > 0) {
            factors.push(remaining.primitive_integer_normalized());
        }
        factors
    }

    pub fn pow(&self, exponent: usize) -> Self {
        let mut result = Self::one(self.variable.clone());
        for _ in 0..exponent {
            result = result.mul(self);
        }
        result
    }

    pub fn to_multivariate(&self, variables: &[Variable]) -> PolynomialQ {
        let target_index = variables
            .iter()
            .position(|variable| variable == &self.variable)
            .expect("target variable must appear in multivariate order");
        let mut result = PolynomialQ::zero(variables.to_vec());
        for (degree, coefficient) in self.coefficients.iter().enumerate() {
            if coefficient.is_zero() {
                continue;
            }
            let mut exponents = vec![0; variables.len()];
            exponents[target_index] = degree as u32;
            result = result.add(&PolynomialQ::from_term(
                variables.to_vec(),
                coefficient.clone(),
                Monomial { exponents },
            ));
        }
        result
    }

    fn mul(&self, rhs: &Self) -> Self {
        assert_eq!(self.variable, rhs.variable);
        if self.is_zero() || rhs.is_zero() {
            return Self::zero(self.variable.clone());
        }
        let mut coefficients = vec![
            crate::arith::rational_zero();
            self.coefficients.len() + rhs.coefficients.len() - 1
        ];
        for (left_degree, left_coefficient) in self.coefficients.iter().enumerate() {
            for (right_degree, right_coefficient) in rhs.coefficients.iter().enumerate() {
                coefficients[left_degree + right_degree] +=
                    left_coefficient.clone() * right_coefficient.clone();
            }
        }
        let mut result = Self {
            variable: self.variable.clone(),
            coefficients,
        };
        result.normalize();
        result
    }

    fn derivative(&self) -> Self {
        if self.coefficients.len() <= 1 {
            return Self::zero(self.variable.clone());
        }
        let coefficients = self
            .coefficients
            .iter()
            .enumerate()
            .skip(1)
            .map(|(degree, coefficient)| {
                coefficient.clone() * BigRational::from_integer(BigInt::from(degree))
            })
            .collect();
        let mut result = Self {
            variable: self.variable.clone(),
            coefficients,
        };
        result.normalize();
        result
    }

    fn div_rem(&self, divisor: &Self) -> (Self, Self) {
        assert_eq!(self.variable, divisor.variable);
        assert!(!divisor.is_zero(), "divisor must be nonzero");

        let divisor_degree = divisor.degree().expect("nonzero divisor has degree");
        let divisor_leading = divisor.coefficients[divisor_degree].clone();
        let mut remainder = self.clone();
        remainder.normalize();
        let quotient_len = self
            .degree()
            .and_then(|degree| degree.checked_sub(divisor_degree))
            .map(|degree| degree + 1)
            .unwrap_or(0);
        let mut quotient = Self {
            variable: self.variable.clone(),
            coefficients: vec![crate::arith::rational_zero(); quotient_len],
        };

        while let Some(remainder_degree) = remainder.degree() {
            if remainder_degree < divisor_degree {
                break;
            }
            let degree_shift = remainder_degree - divisor_degree;
            let scale = remainder.coefficients[remainder_degree].clone() / divisor_leading.clone();
            if quotient.coefficients.len() <= degree_shift {
                quotient
                    .coefficients
                    .resize_with(degree_shift + 1, crate::arith::rational_zero);
            }
            quotient.coefficients[degree_shift] += scale.clone();
            let subtractor = divisor.shift_scale(degree_shift, &scale);
            remainder = remainder.sub(&subtractor);
        }

        quotient.normalize();
        remainder.normalize();
        (quotient, remainder)
    }

    fn shift_scale(&self, degree_shift: usize, scale: &Rational) -> Self {
        if self.is_zero() || scale.is_zero() {
            return Self::zero(self.variable.clone());
        }
        let mut coefficients = vec![crate::arith::rational_zero(); degree_shift];
        coefficients.extend(
            self.coefficients
                .iter()
                .map(|coefficient| coefficient.clone() * scale.clone()),
        );
        let mut result = Self {
            variable: self.variable.clone(),
            coefficients,
        };
        result.normalize();
        result
    }

    fn sub(&self, rhs: &Self) -> Self {
        assert_eq!(self.variable, rhs.variable);
        let len = self.coefficients.len().max(rhs.coefficients.len());
        let mut coefficients = vec![crate::arith::rational_zero(); len];
        for (index, coefficient) in self.coefficients.iter().enumerate() {
            coefficients[index] += coefficient.clone();
        }
        for (index, coefficient) in rhs.coefficients.iter().enumerate() {
            coefficients[index] -= coefficient.clone();
        }
        let mut result = Self {
            variable: self.variable.clone(),
            coefficients,
        };
        result.normalize();
        result
    }
}

fn rational_root(polynomial: &UniPolynomialQ) -> Option<Rational> {
    let normalized = polynomial.primitive_integer_normalized();
    let degree = normalized.degree()?;
    if degree == 0 {
        return None;
    }
    let leading = normalized.coefficients[degree].numer().clone();
    let constant = normalized
        .coefficients
        .first()
        .map(|coefficient| coefficient.numer().clone())
        .unwrap_or_else(BigInt::zero);

    if constant.is_zero() {
        return Some(BigRational::zero());
    }

    let numerators = bounded_positive_divisors(&constant.abs())?;
    let denominators = bounded_positive_divisors(&leading.abs())?;
    for numerator in numerators {
        for denominator in &denominators {
            for sign in [1, -1] {
                let signed = if sign == 1 {
                    numerator.clone()
                } else {
                    -numerator.clone()
                };
                let candidate = BigRational::new(signed, denominator.clone());
                if evaluate_univariate(&normalized, &candidate).is_zero() {
                    return Some(candidate);
                }
            }
        }
    }
    None
}

fn bounded_positive_divisors(value: &BigInt) -> Option<Vec<BigInt>> {
    let limit = value.to_u64()?;
    if limit > 1_000_000 {
        return None;
    }
    let mut divisors = Vec::new();
    for candidate in 1..=limit {
        if value % BigInt::from(candidate) == BigInt::zero() {
            divisors.push(BigInt::from(candidate));
        }
    }
    Some(divisors)
}

fn evaluate_univariate(polynomial: &UniPolynomialQ, point: &Rational) -> Rational {
    polynomial
        .coefficients
        .iter()
        .rev()
        .fold(BigRational::zero(), |value, coefficient| {
            value * point.clone() + coefficient.clone()
        })
}

fn linear_factor_for_root(variable: &Variable, root: &Rational) -> UniPolynomialQ {
    let mut factor = UniPolynomialQ {
        variable: variable.clone(),
        coefficients: vec![
            -BigRational::from_integer(root.numer().clone()),
            BigRational::from_integer(root.denom().clone()),
        ],
    };
    factor.normalize();
    factor.primitive_integer_normalized()
}
