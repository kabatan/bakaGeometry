use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Signed, Zero};

use crate::{Rational, UniPolynomialQ};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RationalInterval {
    pub lower: Rational,
    pub upper: Rational,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlgebraicRealRoot {
    pub polynomial: UniPolynomialQ,
    pub isolating_interval: RationalInterval,
    pub index: usize,
}

pub(crate) fn isolate_real_roots_squarefree(polynomial: &UniPolynomialQ) -> Vec<AlgebraicRealRoot> {
    let polynomial = polynomial.primitive_integer_normalized();
    let Some(degree) = polynomial.degree() else {
        return Vec::new();
    };
    if degree == 0 {
        return Vec::new();
    }

    let sequence = sturm_sequence(&polynomial);
    let (lower, upper) = root_search_interval(&polynomial);
    let count = roots_in_interval(&sequence, &lower, &upper);
    let mut intervals = Vec::new();
    isolate_intervals(&polynomial, &sequence, lower, upper, count, &mut intervals);
    intervals
        .into_iter()
        .enumerate()
        .map(|(index, isolating_interval)| AlgebraicRealRoot {
            polynomial: polynomial.clone(),
            isolating_interval,
            index,
        })
        .collect()
}

pub(crate) fn exact_root_count_in_interval(
    polynomial: &UniPolynomialQ,
    interval: &RationalInterval,
) -> usize {
    let sequence = sturm_sequence(&polynomial.primitive_integer_normalized());
    roots_in_interval(&sequence, &interval.lower, &interval.upper)
}

fn isolate_intervals(
    polynomial: &UniPolynomialQ,
    sequence: &[UniPolynomialQ],
    lower: Rational,
    upper: Rational,
    count: usize,
    intervals: &mut Vec<RationalInterval>,
) {
    if count == 0 {
        return;
    }
    if count == 1 {
        intervals.push(RationalInterval { lower, upper });
        return;
    }

    let split = non_root_split(polynomial, &lower, &upper);
    let left_count = roots_in_interval(sequence, &lower, &split);
    let right_count = roots_in_interval(sequence, &split, &upper);
    isolate_intervals(
        polynomial,
        sequence,
        lower,
        split.clone(),
        left_count,
        intervals,
    );
    isolate_intervals(polynomial, sequence, split, upper, right_count, intervals);
}

fn sturm_sequence(polynomial: &UniPolynomialQ) -> Vec<UniPolynomialQ> {
    let mut sequence = vec![polynomial.clone(), derivative(polynomial)];
    while !sequence.last().unwrap().is_zero() {
        let previous = sequence[sequence.len() - 2].clone();
        let current = sequence[sequence.len() - 1].clone();
        let (_, remainder) = div_rem(&previous, &current);
        if remainder.is_zero() {
            break;
        }
        sequence.push(neg(&remainder));
    }
    sequence
}

fn roots_in_interval(sequence: &[UniPolynomialQ], lower: &Rational, upper: &Rational) -> usize {
    let lower_variations = sign_variations(sequence, lower);
    let upper_variations = sign_variations(sequence, upper);
    lower_variations.saturating_sub(upper_variations)
}

fn sign_variations(sequence: &[UniPolynomialQ], point: &Rational) -> usize {
    let mut variations = 0;
    let mut previous = 0;
    for polynomial in sequence {
        let sign = sign_of(&evaluate(polynomial, point));
        if sign == 0 {
            continue;
        }
        if previous != 0 && sign != previous {
            variations += 1;
        }
        previous = sign;
    }
    variations
}

fn sign_of(value: &Rational) -> i8 {
    if value.is_zero() {
        0
    } else if value.is_positive() {
        1
    } else {
        -1
    }
}

fn root_search_interval(polynomial: &UniPolynomialQ) -> (Rational, Rational) {
    let degree = polynomial.degree().unwrap();
    let leading = polynomial.coefficients[degree].clone();
    let mut bound = BigRational::one();
    for coefficient in polynomial.coefficients.iter().take(degree) {
        if coefficient.is_zero() {
            continue;
        }
        let ratio = coefficient.clone().abs() / leading.clone().abs();
        if ratio > bound {
            bound = ratio;
        }
    }
    bound += BigRational::one();

    while evaluate(polynomial, &bound).is_zero()
        || evaluate(polynomial, &(-bound.clone())).is_zero()
    {
        bound += BigRational::one();
    }

    (-bound.clone(), bound)
}

fn non_root_split(polynomial: &UniPolynomialQ, lower: &Rational, upper: &Rational) -> Rational {
    let width = upper.clone() - lower.clone();
    for denominator in 2usize.. {
        for numerator in 1..denominator {
            let split = lower.clone()
                + width.clone()
                    * BigRational::new(BigInt::from(numerator), BigInt::from(denominator));
            if !evaluate(polynomial, &split).is_zero() {
                return split;
            }
        }
    }
    unreachable!()
}

fn evaluate(polynomial: &UniPolynomialQ, point: &Rational) -> Rational {
    polynomial
        .coefficients
        .iter()
        .rev()
        .fold(BigRational::zero(), |value, coefficient| {
            value * point.clone() + coefficient.clone()
        })
}

fn derivative(polynomial: &UniPolynomialQ) -> UniPolynomialQ {
    if polynomial.coefficients.len() <= 1 {
        return UniPolynomialQ::zero(polynomial.variable.clone());
    }
    let coefficients = polynomial
        .coefficients
        .iter()
        .enumerate()
        .skip(1)
        .map(|(degree, coefficient)| coefficient.clone() * BigRational::from_integer(degree.into()))
        .collect::<Vec<_>>();
    normalized(polynomial.variable.clone(), coefficients)
}

fn div_rem(left: &UniPolynomialQ, right: &UniPolynomialQ) -> (UniPolynomialQ, UniPolynomialQ) {
    assert_eq!(left.variable, right.variable);
    assert!(!right.is_zero(), "divisor must be nonzero");
    let divisor_degree = right.degree().unwrap();
    let divisor_leading = right.coefficients[divisor_degree].clone();
    let mut remainder = left.clone();
    let quotient_len = left
        .degree()
        .and_then(|degree| degree.checked_sub(divisor_degree))
        .map(|degree| degree + 1)
        .unwrap_or(0);
    let mut quotient = UniPolynomialQ {
        variable: left.variable.clone(),
        coefficients: vec![BigRational::zero(); quotient_len],
    };

    while let Some(remainder_degree) = remainder.degree() {
        if remainder_degree < divisor_degree {
            break;
        }
        let shift = remainder_degree - divisor_degree;
        let scale = remainder.coefficients[remainder_degree].clone() / divisor_leading.clone();
        if quotient.coefficients.len() <= shift {
            quotient
                .coefficients
                .resize_with(shift + 1, BigRational::zero);
        }
        quotient.coefficients[shift] += scale.clone();
        remainder = sub(&remainder, &shift_scale(right, shift, &scale));
    }

    quotient.normalize();
    remainder.normalize();
    (quotient, remainder)
}

fn shift_scale(polynomial: &UniPolynomialQ, shift: usize, scale: &Rational) -> UniPolynomialQ {
    if polynomial.is_zero() || scale.is_zero() {
        return UniPolynomialQ::zero(polynomial.variable.clone());
    }
    let mut coefficients = vec![BigRational::zero(); shift];
    coefficients.extend(
        polynomial
            .coefficients
            .iter()
            .map(|coefficient| coefficient.clone() * scale.clone()),
    );
    normalized(polynomial.variable.clone(), coefficients)
}

fn sub(left: &UniPolynomialQ, right: &UniPolynomialQ) -> UniPolynomialQ {
    assert_eq!(left.variable, right.variable);
    let len = left.coefficients.len().max(right.coefficients.len());
    let mut coefficients = vec![BigRational::zero(); len];
    for (index, coefficient) in left.coefficients.iter().enumerate() {
        coefficients[index] += coefficient.clone();
    }
    for (index, coefficient) in right.coefficients.iter().enumerate() {
        coefficients[index] -= coefficient.clone();
    }
    normalized(left.variable.clone(), coefficients)
}

fn neg(polynomial: &UniPolynomialQ) -> UniPolynomialQ {
    normalized(
        polynomial.variable.clone(),
        polynomial
            .coefficients
            .iter()
            .map(|coefficient| -coefficient.clone())
            .collect(),
    )
}

fn normalized(variable: crate::Variable, coefficients: Vec<Rational>) -> UniPolynomialQ {
    let mut polynomial = UniPolynomialQ {
        variable,
        coefficients,
    };
    polynomial.normalize();
    polynomial
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Variable;

    fn variable(symbol: &str) -> Variable {
        Variable {
            symbol: symbol.to_string(),
        }
    }

    fn rational(value: i64) -> Rational {
        BigRational::from_integer(BigInt::from(value))
    }

    fn uni(coefficients: &[i64]) -> UniPolynomialQ {
        let variable = variable("T");
        let mut polynomial = UniPolynomialQ {
            variable,
            coefficients: coefficients.iter().map(|value| rational(*value)).collect(),
        };
        polynomial.normalize();
        polynomial
    }

    #[test]
    fn squarefree_quadratic_has_two_exact_isolating_intervals() {
        let polynomial = uni(&[-2, 0, 1]);

        let roots = isolate_real_roots_squarefree(&polynomial);

        assert_eq!(roots.len(), 2);
        for root in &roots {
            assert_eq!(
                exact_root_count_in_interval(&polynomial, &root.isolating_interval),
                1
            );
            assert!(root.isolating_interval.lower < root.isolating_interval.upper);
            assert_eq!(root.polynomial, polynomial);
        }
    }
}
