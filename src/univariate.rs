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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FactorizationResult {
    pub status: FactorizationStatus,
    pub squarefree_part: UniPolynomialQ,
    pub factors: Vec<UniPolynomialQ>,
    pub remaining: Option<UniPolynomialQ>,
    pub failure: Option<FactorizationFailure>,
    pub trace: FactorizationTrace,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FactorizationStatus {
    Complete,
    Partial,
    ResourceFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FactorizationFailure {
    CoefficientHeightTooLarge,
    DivisorEnumerationLimitExceeded,
    ExactDivisionFailed,
    InterpolationPointExhausted,
    ProductReconstructionFailed,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FactorizationTrace {
    pub original_degree: Option<usize>,
    pub squarefree_degree: Option<usize>,
    pub searched_factor_degrees: Vec<usize>,
    pub evaluated_points: usize,
    pub divisor_enumerations: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FactorizationLimits {
    pub max_divisor_enumerations: Option<usize>,
    pub max_evaluation_abs: u64,
}

impl Default for FactorizationLimits {
    fn default() -> Self {
        Self {
            max_divisor_enumerations: Some(200_000),
            max_evaluation_abs: 100_000,
        }
    }
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

    pub fn factor_squarefree_over_q(&self) -> FactorizationResult {
        self.factor_squarefree_over_q_with_limits(FactorizationLimits::default())
    }

    pub fn factor_squarefree_over_q_with_divisor_limit(
        &self,
        max_divisor_enumerations: usize,
    ) -> FactorizationResult {
        self.factor_squarefree_over_q_with_limits(FactorizationLimits {
            max_divisor_enumerations: Some(max_divisor_enumerations),
            ..FactorizationLimits::default()
        })
    }

    pub fn factor_squarefree_over_q_with_limits(
        &self,
        limits: FactorizationLimits,
    ) -> FactorizationResult {
        factor_squarefree_over_q_with_limits(self, limits)
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

    fn add(&self, rhs: &Self) -> Self {
        assert_eq!(self.variable, rhs.variable);
        let len = self.coefficients.len().max(rhs.coefficients.len());
        let mut coefficients = vec![crate::arith::rational_zero(); len];
        for (index, coefficient) in self.coefficients.iter().enumerate() {
            coefficients[index] += coefficient.clone();
        }
        for (index, coefficient) in rhs.coefficients.iter().enumerate() {
            coefficients[index] += coefficient.clone();
        }
        let mut result = Self {
            variable: self.variable.clone(),
            coefficients,
        };
        result.normalize();
        result
    }
}

fn factor_squarefree_over_q_with_limits(
    polynomial: &UniPolynomialQ,
    limits: FactorizationLimits,
) -> FactorizationResult {
    let squarefree_part = polynomial.squarefree_part().primitive_integer_normalized();
    let mut trace = FactorizationTrace {
        original_degree: polynomial.degree(),
        squarefree_degree: squarefree_part.degree(),
        searched_factor_degrees: Vec::new(),
        evaluated_points: 0,
        divisor_enumerations: 0,
    };
    if squarefree_part.is_zero() {
        return FactorizationResult {
            status: FactorizationStatus::Complete,
            squarefree_part,
            factors: Vec::new(),
            remaining: None,
            failure: None,
            trace,
        };
    }

    let mut factors = Vec::new();
    let mut remaining = squarefree_part.clone();

    loop {
        let Some(degree) = remaining.degree() else {
            break;
        };
        if degree == 0 {
            factors.push(remaining.primitive_integer_normalized());
            break;
        }

        let mut found_factor = None;
        for factor_degree in 1..=degree / 2 {
            trace.searched_factor_degrees.push(factor_degree);
            match find_factor_of_degree(&remaining, factor_degree, &limits, &mut trace) {
                Ok(Some(factor)) => {
                    found_factor = Some(factor);
                    break;
                }
                Ok(None) => {}
                Err(failure) => {
                    return FactorizationResult {
                        status: FactorizationStatus::ResourceFailure,
                        squarefree_part,
                        factors,
                        remaining: Some(remaining.primitive_integer_normalized()),
                        failure: Some(failure),
                        trace,
                    };
                }
            }
        }

        let Some(factor) = found_factor else {
            factors.push(remaining.primitive_integer_normalized());
            break;
        };
        let (quotient, remainder) = remaining.div_rem(&factor);
        if !remainder.is_zero() || quotient.is_zero() {
            return FactorizationResult {
                status: FactorizationStatus::Partial,
                squarefree_part,
                factors,
                remaining: Some(remaining.primitive_integer_normalized()),
                failure: Some(FactorizationFailure::ExactDivisionFailed),
                trace,
            };
        }
        factors.push(factor.primitive_integer_normalized());
        remaining = quotient.primitive_integer_normalized();
    }

    let product = product_of_factors(&squarefree_part.variable, &factors);
    if product != squarefree_part {
        return FactorizationResult {
            status: FactorizationStatus::Partial,
            squarefree_part,
            factors,
            remaining: None,
            failure: Some(FactorizationFailure::ProductReconstructionFailed),
            trace,
        };
    }

    FactorizationResult {
        status: FactorizationStatus::Complete,
        squarefree_part,
        factors,
        remaining: None,
        failure: None,
        trace,
    }
}

fn find_factor_of_degree(
    polynomial: &UniPolynomialQ,
    factor_degree: usize,
    limits: &FactorizationLimits,
    trace: &mut FactorizationTrace,
) -> Result<Option<UniPolynomialQ>, FactorizationFailure> {
    let points = select_nonzero_evaluation_points(polynomial, factor_degree + 1, trace)?;
    let divisor_lists = points
        .iter()
        .map(|(_, value)| signed_divisors(value, limits, trace))
        .collect::<Result<Vec<_>, _>>()?;
    search_divisor_assignments(
        polynomial,
        factor_degree,
        &points,
        &divisor_lists,
        0,
        &mut Vec::new(),
        limits,
        trace,
    )
}

fn select_nonzero_evaluation_points(
    polynomial: &UniPolynomialQ,
    count: usize,
    trace: &mut FactorizationTrace,
) -> Result<Vec<(i64, BigInt)>, FactorizationFailure> {
    let degree = polynomial.degree().unwrap_or(0);
    let max_probe_count = degree
        .saturating_mul(2)
        .saturating_add(count.saturating_mul(4))
        .saturating_add(16);
    let mut points = Vec::new();
    for index in 0..max_probe_count {
        let point = integer_evaluation_point(index);
        let value =
            evaluate_univariate(polynomial, &BigRational::from_integer(BigInt::from(point)));
        trace.evaluated_points += 1;
        if value.is_zero() {
            continue;
        }
        if value.denom() != &BigInt::one() {
            return Err(FactorizationFailure::CoefficientHeightTooLarge);
        }
        points.push((point, value.numer().clone()));
        if points.len() == count {
            return Ok(points);
        }
    }
    Err(FactorizationFailure::InterpolationPointExhausted)
}

fn integer_evaluation_point(index: usize) -> i64 {
    if index == 0 {
        0
    } else {
        let magnitude = index.div_ceil(2) as i64;
        if index % 2 == 1 {
            magnitude
        } else {
            -magnitude
        }
    }
}

fn signed_divisors(
    value: &BigInt,
    limits: &FactorizationLimits,
    trace: &mut FactorizationTrace,
) -> Result<Vec<BigInt>, FactorizationFailure> {
    let abs = value.abs();
    let limit = abs
        .to_u64()
        .ok_or(FactorizationFailure::CoefficientHeightTooLarge)?;
    if limit > limits.max_evaluation_abs {
        return Err(FactorizationFailure::CoefficientHeightTooLarge);
    }

    let mut divisors = Vec::new();
    for candidate in 1..=limit {
        let divisor = BigInt::from(candidate);
        if &abs % &divisor == BigInt::zero() {
            divisors.push(divisor.clone());
            divisors.push(-divisor);
        }
    }
    spend_divisor_budget(trace, limits, divisors.len())?;
    Ok(divisors)
}

fn search_divisor_assignments(
    polynomial: &UniPolynomialQ,
    factor_degree: usize,
    points: &[(i64, BigInt)],
    divisor_lists: &[Vec<BigInt>],
    index: usize,
    current: &mut Vec<BigInt>,
    limits: &FactorizationLimits,
    trace: &mut FactorizationTrace,
) -> Result<Option<UniPolynomialQ>, FactorizationFailure> {
    if index == divisor_lists.len() {
        spend_divisor_budget(trace, limits, 1)?;
        let candidate = interpolate_polynomial(&polynomial.variable, points, current)
            .primitive_integer_normalized();
        if candidate.degree() != Some(factor_degree)
            || candidate == polynomial.primitive_integer_normalized()
        {
            return Ok(None);
        }
        let (_, remainder) = polynomial.div_rem(&candidate);
        if remainder.is_zero() {
            return Ok(Some(candidate));
        }
        return Ok(None);
    }

    for divisor in &divisor_lists[index] {
        current.push(divisor.clone());
        if let Some(factor) = search_divisor_assignments(
            polynomial,
            factor_degree,
            points,
            divisor_lists,
            index + 1,
            current,
            limits,
            trace,
        )? {
            return Ok(Some(factor));
        }
        current.pop();
    }
    Ok(None)
}

fn spend_divisor_budget(
    trace: &mut FactorizationTrace,
    limits: &FactorizationLimits,
    amount: usize,
) -> Result<(), FactorizationFailure> {
    trace.divisor_enumerations = trace.divisor_enumerations.saturating_add(amount);
    if limits
        .max_divisor_enumerations
        .is_some_and(|max| trace.divisor_enumerations > max)
    {
        return Err(FactorizationFailure::DivisorEnumerationLimitExceeded);
    }
    Ok(())
}

fn interpolate_polynomial(
    variable: &Variable,
    points: &[(i64, BigInt)],
    values: &[BigInt],
) -> UniPolynomialQ {
    let mut result = UniPolynomialQ::zero(variable.clone());
    for (index, (point, _)) in points.iter().enumerate() {
        let xi = BigInt::from(*point);
        let mut basis = UniPolynomialQ::one(variable.clone());
        let mut denominator = BigInt::one();
        for (other_index, (other_point, _)) in points.iter().enumerate() {
            if index == other_index {
                continue;
            }
            let xj = BigInt::from(*other_point);
            let linear = UniPolynomialQ {
                variable: variable.clone(),
                coefficients: vec![BigRational::from_integer(-xj.clone()), BigRational::one()],
            };
            basis = basis.mul(&linear);
            denominator *= xi.clone() - xj;
        }
        let scale = BigRational::new(values[index].clone(), denominator);
        result = result.add(&basis.shift_scale(0, &scale));
    }
    result
}

fn product_of_factors(variable: &Variable, factors: &[UniPolynomialQ]) -> UniPolynomialQ {
    factors
        .iter()
        .fold(UniPolynomialQ::one(variable.clone()), |product, factor| {
            product.mul(factor)
        })
        .primitive_integer_normalized()
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
