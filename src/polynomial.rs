use std::collections::BTreeMap;

use num_bigint::BigInt;
use num_integer::Integer;
use num_rational::BigRational;
use num_traits::{Signed, Zero};

use crate::{Monomial, UniPolynomialQ, Variable};

pub type Rational = BigRational;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PolynomialQ {
    pub variables: Vec<Variable>,
    pub terms: BTreeMap<Monomial, Rational>,
}

impl PolynomialQ {
    pub fn zero(variables: Vec<Variable>) -> Self {
        Self {
            variables,
            terms: BTreeMap::new(),
        }
    }

    pub fn one(variables: Vec<Variable>) -> Self {
        let monomial = Monomial {
            exponents: vec![0; variables.len()],
        };
        Self::from_term(variables, crate::arith::rational_one(), monomial)
    }

    pub fn from_term(variables: Vec<Variable>, coefficient: Rational, monomial: Monomial) -> Self {
        assert_eq!(monomial.exponents.len(), variables.len());
        let mut terms = BTreeMap::new();
        if coefficient != crate::arith::rational_zero() {
            terms.insert(monomial, coefficient);
        }
        Self { variables, terms }
    }

    pub fn normalize(&mut self) {
        let variable_count = self.variables.len();
        self.terms.retain(|monomial, coefficient| {
            assert_eq!(monomial.exponents.len(), variable_count);
            !coefficient.is_zero()
        });
    }

    pub fn is_zero(&self) -> bool {
        self.terms.is_empty()
    }

    pub fn support(&self) -> Vec<Monomial> {
        self.terms.keys().cloned().collect()
    }

    pub fn degree(&self) -> u32 {
        self.terms
            .keys()
            .map(Monomial::total_degree)
            .max()
            .unwrap_or(0)
    }

    pub fn add(&self, rhs: &Self) -> Self {
        self.assert_same_variables(rhs);
        let mut result = self.clone();
        for (monomial, coefficient) in &rhs.terms {
            let entry = result
                .terms
                .entry(monomial.clone())
                .or_insert_with(crate::arith::rational_zero);
            *entry += coefficient.clone();
        }
        result.normalize();
        result
    }

    pub fn sub(&self, rhs: &Self) -> Self {
        self.assert_same_variables(rhs);
        let mut result = self.clone();
        for (monomial, coefficient) in &rhs.terms {
            let entry = result
                .terms
                .entry(monomial.clone())
                .or_insert_with(crate::arith::rational_zero);
            *entry -= coefficient.clone();
        }
        result.normalize();
        result
    }

    pub fn mul(&self, rhs: &Self) -> Self {
        self.assert_same_variables(rhs);
        let mut result = Self::zero(self.variables.clone());
        for (left_monomial, left_coefficient) in &self.terms {
            for (right_monomial, right_coefficient) in &rhs.terms {
                let exponents = left_monomial
                    .exponents
                    .iter()
                    .zip(&right_monomial.exponents)
                    .map(|(left, right)| left + right)
                    .collect();
                let entry = result
                    .terms
                    .entry(Monomial { exponents })
                    .or_insert_with(crate::arith::rational_zero);
                *entry += left_coefficient.clone() * right_coefficient.clone();
            }
        }
        result.normalize();
        result
    }

    pub fn pow(&self, exponent: usize) -> Self {
        let mut result = Self::one(self.variables.clone());
        for _ in 0..exponent {
            result = result.mul(self);
        }
        result
    }

    pub fn scale(&self, factor: &Rational) -> Self {
        let mut result = self.clone();
        for coefficient in result.terms.values_mut() {
            *coefficient *= factor.clone();
        }
        result.normalize();
        result
    }

    pub(crate) fn primitive_integer_normalized_with_multiplier(&self) -> (Self, Rational) {
        if self.is_zero() {
            return (
                Self::zero(self.variables.clone()),
                crate::arith::rational_one(),
            );
        }
        let denominator_lcm = crate::arith::lcm_bigint(
            self.terms
                .values()
                .map(|coefficient| coefficient.denom().clone()),
        );
        let integer_coefficients = self
            .terms
            .values()
            .map(|coefficient| coefficient.numer() * (&denominator_lcm / coefficient.denom()))
            .collect::<Vec<BigInt>>();
        let content =
            integer_coefficients
                .iter()
                .fold(BigInt::zero(), |accumulator, coefficient| {
                    if accumulator.is_zero() {
                        coefficient.abs()
                    } else {
                        accumulator.gcd(&coefficient.abs())
                    }
                });
        let multiplier = if content.is_zero() {
            BigRational::from_integer(denominator_lcm)
        } else {
            BigRational::new(denominator_lcm, content)
        };
        (self.scale(&multiplier), multiplier)
    }

    pub fn substitute_variable(&self, variable: &Variable, replacement: &PolynomialQ) -> Self {
        assert_eq!(self.variables, replacement.variables);
        let variable_index = self
            .variables
            .iter()
            .position(|candidate| candidate == variable)
            .expect("variable must be present for substitution");
        let mut result = Self::zero(self.variables.clone());
        for (monomial, coefficient) in &self.terms {
            let mut base_exponents = monomial.exponents.clone();
            let exponent = base_exponents[variable_index] as usize;
            base_exponents[variable_index] = 0;
            let base = Self::from_term(
                self.variables.clone(),
                coefficient.clone(),
                Monomial {
                    exponents: base_exponents,
                },
            );
            result = result.add(&base.mul(&replacement.pow(exponent)));
        }
        result
    }

    pub fn depends_only_on(&self, allowed: &[Variable]) -> bool {
        self.terms.keys().all(|monomial| {
            monomial
                .exponents
                .iter()
                .zip(&self.variables)
                .all(|(exponent, variable)| *exponent == 0 || allowed.contains(variable))
        })
    }

    pub fn to_univariate_in(&self, target: &Variable) -> Option<UniPolynomialQ> {
        let target_index = self
            .variables
            .iter()
            .position(|variable| variable == target)?;
        let mut coefficients = Vec::new();
        for (monomial, coefficient) in &self.terms {
            for (index, exponent) in monomial.exponents.iter().enumerate() {
                if index != target_index && *exponent != 0 {
                    return None;
                }
            }
            let degree = monomial.exponents[target_index] as usize;
            if coefficients.len() <= degree {
                coefficients.resize_with(degree + 1, crate::arith::rational_zero);
            }
            coefficients[degree] += coefficient.clone();
        }
        let mut result = UniPolynomialQ {
            variable: target.clone(),
            coefficients,
        };
        result.normalize();
        Some(result)
    }

    fn assert_same_variables(&self, rhs: &Self) {
        assert_eq!(self.variables, rhs.variables);
    }
}
