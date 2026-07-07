use std::collections::{BTreeMap, BTreeSet};

use num_traits::{One, Signed, Zero};

use crate::candidates::{
    CandidateOracle, CandidateOrigin, CandidateTrace, RouteWitnessTrace, TargetCandidate,
};
use crate::compression::CertifiedSystemQ;
use crate::window::CertificateWindow;
use crate::{Monomial, PolynomialQ, Rational, UniPolynomialQ, Variable};

pub(crate) struct NormTraceTowerOracle;

#[derive(Clone, Debug)]
struct TowerLevel {
    variable_index: usize,
    equation_index: usize,
    degree: u32,
    lower: PolynomialQ,
}

impl CandidateOracle for NormTraceTowerOracle {
    fn generate(
        &self,
        system: &CertifiedSystemQ,
        _window: &CertificateWindow,
    ) -> Vec<TargetCandidate> {
        norm_trace_tower_candidates(system)
    }
}

pub(crate) fn norm_trace_tower_candidates(system: &CertifiedSystemQ) -> Vec<TargetCandidate> {
    let Some((target_equation, target_expression)) = target_expression(system) else {
        return Vec::new();
    };
    let Some(levels) = detect_monic_tower(system, target_equation) else {
        return Vec::new();
    };
    if levels.is_empty() || !target_expression_depends_on_tower(system, &target_expression, &levels)
    {
        return Vec::new();
    }

    let basis = tower_basis(system.variables.len(), &levels);
    let columns = basis
        .iter()
        .map(|basis_monomial| {
            let basis_polynomial = PolynomialQ::from_term(
                system.variables.clone(),
                crate::arith::rational_one(),
                basis_monomial.clone(),
            );
            let reduced = reduce_by_tower(&basis_polynomial.mul(&target_expression), &levels);
            vector_from_polynomial(&reduced, &basis, system)
        })
        .collect::<Vec<_>>();
    let support =
        characteristic_polynomial(system.target.clone(), &columns).primitive_integer_normalized();
    if support.is_zero() {
        return Vec::new();
    }

    let mut equation_indices = levels
        .iter()
        .map(|level| level.equation_index)
        .collect::<Vec<_>>();
    equation_indices.push(target_equation);

    vec![TargetCandidate {
        support_mod_primes: Vec::new(),
        reconstructed: Some(support),
        origin: CandidateOrigin::NormTraceTower,
        traces: vec![CandidateTrace::RouteWitness(RouteWitnessTrace {
            origin: CandidateOrigin::NormTraceTower,
            equation_indices,
            support_size: basis.len(),
        })],
    }]
}

fn target_expression(system: &CertifiedSystemQ) -> Option<(usize, PolynomialQ)> {
    let target_index = system
        .variables
        .iter()
        .position(|candidate| candidate == &system.target)?;
    for (equation_index, equation) in system.equations.iter().enumerate() {
        let mut target_coefficient = crate::arith::rational_zero();
        let mut rest = PolynomialQ::zero(system.variables.clone());
        let mut valid = true;
        for (monomial, coefficient) in &equation.terms {
            let target_exponent = monomial.exponents[target_index];
            match target_exponent {
                0 => {
                    rest = rest.add(&PolynomialQ::from_term(
                        system.variables.clone(),
                        coefficient.clone(),
                        monomial.clone(),
                    ));
                }
                1 if monomial.total_degree() == 1 => {
                    target_coefficient += coefficient.clone();
                }
                _ => {
                    valid = false;
                    break;
                }
            }
        }
        if valid && !target_coefficient.is_zero() && target_coefficient.abs().is_one() {
            let scale = -crate::arith::rational_one() / target_coefficient;
            return Some((equation_index, rest.scale(&scale)));
        }
    }
    None
}

fn detect_monic_tower(
    system: &CertifiedSystemQ,
    target_equation: usize,
) -> Option<Vec<TowerLevel>> {
    let target_index = variable_index(&system.variables, &system.target)?;
    let mut available = BTreeSet::from([target_index]);
    let mut used_equations = BTreeSet::from([target_equation]);
    let mut levels = Vec::new();

    loop {
        let mut selected = None;
        for variable_index in 0..system.variables.len() {
            if available.contains(&variable_index) || variable_index == target_index {
                continue;
            }
            for (equation_index, equation) in system.equations.iter().enumerate() {
                if used_equations.contains(&equation_index) {
                    continue;
                }
                if let Some(level) =
                    monic_level(system, equation, equation_index, variable_index, &available)
                {
                    selected = Some(level);
                    break;
                }
            }
            if selected.is_some() {
                break;
            }
        }

        let Some(level) = selected else {
            break;
        };
        available.insert(level.variable_index);
        used_equations.insert(level.equation_index);
        levels.push(level);
    }

    Some(levels)
}

fn monic_level(
    system: &CertifiedSystemQ,
    equation: &PolynomialQ,
    equation_index: usize,
    variable_index: usize,
    available: &BTreeSet<usize>,
) -> Option<TowerLevel> {
    let degree = equation
        .terms
        .keys()
        .map(|monomial| monomial.exponents[variable_index])
        .max()
        .unwrap_or(0);
    if degree == 0 {
        return None;
    }

    let mut leading_coefficient = crate::arith::rational_zero();
    let mut lower = PolynomialQ::zero(system.variables.clone());
    for (monomial, coefficient) in &equation.terms {
        if monomial
            .exponents
            .iter()
            .enumerate()
            .any(|(index, exponent)| {
                index != variable_index && *exponent != 0 && !available.contains(&index)
            })
        {
            return None;
        }
        if monomial.exponents[variable_index] == degree {
            if monomial
                .exponents
                .iter()
                .enumerate()
                .any(|(index, exponent)| index != variable_index && *exponent != 0)
            {
                return None;
            }
            leading_coefficient += coefficient.clone();
        } else {
            lower = lower.add(&PolynomialQ::from_term(
                system.variables.clone(),
                coefficient.clone(),
                monomial.clone(),
            ));
        }
    }
    if !leading_coefficient.is_one() {
        return None;
    }

    Some(TowerLevel {
        variable_index,
        equation_index,
        degree,
        lower: lower.scale(&-crate::arith::rational_one()),
    })
}

fn target_expression_depends_on_tower(
    system: &CertifiedSystemQ,
    expression: &PolynomialQ,
    levels: &[TowerLevel],
) -> bool {
    let tower_variables = levels
        .iter()
        .map(|level| level.variable_index)
        .collect::<BTreeSet<_>>();
    let target_index = variable_index(&system.variables, &system.target).unwrap();
    expression.terms.keys().any(|monomial| {
        monomial
            .exponents
            .iter()
            .enumerate()
            .any(|(index, exponent)| *exponent != 0 && index != target_index)
    }) && expression.terms.keys().all(|monomial| {
        monomial.exponents[target_index] == 0
            && monomial
                .exponents
                .iter()
                .enumerate()
                .all(|(index, exponent)| *exponent == 0 || tower_variables.contains(&index))
    })
}

fn tower_basis(variable_count: usize, levels: &[TowerLevel]) -> Vec<Monomial> {
    let mut basis = Vec::new();
    let mut exponents = vec![0; variable_count];
    enumerate_tower_basis(levels, 0, &mut exponents, &mut basis);
    basis
}

fn enumerate_tower_basis(
    levels: &[TowerLevel],
    index: usize,
    exponents: &mut [u32],
    basis: &mut Vec<Monomial>,
) {
    if index == levels.len() {
        basis.push(Monomial {
            exponents: exponents.to_vec(),
        });
        return;
    }
    let level = &levels[index];
    for exponent in 0..level.degree {
        exponents[level.variable_index] = exponent;
        enumerate_tower_basis(levels, index + 1, exponents, basis);
    }
    exponents[level.variable_index] = 0;
}

fn reduce_by_tower(polynomial: &PolynomialQ, levels: &[TowerLevel]) -> PolynomialQ {
    let mut current = polynomial.clone();
    loop {
        let mut changed = false;
        'term_scan: for (monomial, coefficient) in current.terms.clone() {
            for level in levels.iter().rev() {
                if monomial.exponents[level.variable_index] >= level.degree {
                    let mut quotient_exponents = monomial.exponents.clone();
                    quotient_exponents[level.variable_index] -= level.degree;
                    let quotient = PolynomialQ::from_term(
                        current.variables.clone(),
                        coefficient.clone(),
                        Monomial {
                            exponents: quotient_exponents,
                        },
                    );
                    let original =
                        PolynomialQ::from_term(current.variables.clone(), coefficient, monomial);
                    current = current.sub(&original);
                    current = current.add(&quotient.mul(&level.lower));
                    changed = true;
                    break 'term_scan;
                }
            }
        }
        if !changed {
            return current;
        }
    }
}

fn vector_from_polynomial(
    polynomial: &PolynomialQ,
    basis: &[Monomial],
    system: &CertifiedSystemQ,
) -> Vec<Vec<Rational>> {
    let target_index = variable_index(&system.variables, &system.target).unwrap();
    let index = basis
        .iter()
        .map(|basis_monomial| {
            let mut without_target = basis_monomial.clone();
            without_target.exponents[target_index] = 0;
            without_target
        })
        .enumerate()
        .map(|(index, monomial)| (monomial, index))
        .collect::<BTreeMap<_, _>>();
    let mut vector = vec![Vec::new(); basis.len()];
    for (monomial, coefficient) in &polynomial.terms {
        let mut basis_part = monomial.clone();
        let target_degree = basis_part.exponents[target_index] as usize;
        basis_part.exponents[target_index] = 0;
        if let Some(row) = index.get(&basis_part) {
            if vector[*row].len() <= target_degree {
                vector[*row].resize_with(target_degree + 1, crate::arith::rational_zero);
            }
            vector[*row][target_degree] += coefficient.clone();
            vector[*row] = trim_poly(vector[*row].clone());
        }
    }
    vector
}

fn characteristic_polynomial(variable: Variable, columns: &[Vec<Vec<Rational>>]) -> UniPolynomialQ {
    let size = columns.len();
    let matrix = (0..size)
        .map(|row| {
            (0..size)
                .map(|col| {
                    let mut entry = vec![poly_neg(&columns[col][row])];
                    if row == col {
                        entry.push(vec![crate::arith::rational_one()]);
                    }
                    trim_zpoly(entry)
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let coefficients = substitute_z_by_target(&determinant_poly(&matrix));
    let mut support = UniPolynomialQ {
        variable,
        coefficients,
    };
    support.normalize();
    support
}

fn determinant_poly(matrix: &[Vec<Vec<Vec<Rational>>>]) -> Vec<Vec<Rational>> {
    let size = matrix.len();
    if size == 0 {
        return vec![vec![crate::arith::rational_one()]];
    }
    if size == 1 {
        return matrix[0][0].clone();
    }
    let mut result = Vec::new();
    for col in 0..size {
        let minor = (1..size)
            .map(|row| {
                (0..size)
                    .filter(|minor_col| *minor_col != col)
                    .map(|minor_col| matrix[row][minor_col].clone())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let mut term = zpoly_mul(&matrix[0][col], &determinant_poly(&minor));
        if col % 2 == 1 {
            term = zpoly_neg(&term);
        }
        result = zpoly_add(&result, &term);
    }
    trim_zpoly(result)
}

fn zpoly_add(left: &[Vec<Rational>], right: &[Vec<Rational>]) -> Vec<Vec<Rational>> {
    let len = left.len().max(right.len());
    let mut result = vec![Vec::new(); len];
    for (index, coefficient) in left.iter().enumerate() {
        result[index] = poly_add(&result[index], coefficient);
    }
    for (index, coefficient) in right.iter().enumerate() {
        result[index] = poly_add(&result[index], coefficient);
    }
    trim_zpoly(result)
}

fn zpoly_mul(left: &[Vec<Rational>], right: &[Vec<Rational>]) -> Vec<Vec<Rational>> {
    if left.is_empty() || right.is_empty() {
        return Vec::new();
    }
    let mut result = vec![Vec::new(); left.len() + right.len() - 1];
    for (left_degree, left_coefficient) in left.iter().enumerate() {
        for (right_degree, right_coefficient) in right.iter().enumerate() {
            let product = poly_mul(left_coefficient, right_coefficient);
            result[left_degree + right_degree] =
                poly_add(&result[left_degree + right_degree], &product);
        }
    }
    trim_zpoly(result)
}

fn zpoly_neg(polynomial: &[Vec<Rational>]) -> Vec<Vec<Rational>> {
    polynomial
        .iter()
        .map(|coefficient| poly_neg(coefficient))
        .collect()
}

fn trim_zpoly(mut coefficients: Vec<Vec<Rational>>) -> Vec<Vec<Rational>> {
    for coefficient in &mut coefficients {
        *coefficient = trim_poly(std::mem::take(coefficient));
    }
    while coefficients.last().is_some_and(Vec::is_empty) {
        coefficients.pop();
    }
    coefficients
}

fn substitute_z_by_target(polynomial: &[Vec<Rational>]) -> Vec<Rational> {
    polynomial
        .iter()
        .enumerate()
        .fold(Vec::new(), |sum, (degree, coefficient)| {
            poly_add(&sum, &poly_shift(coefficient, degree))
        })
}

fn poly_add(left: &[Rational], right: &[Rational]) -> Vec<Rational> {
    let len = left.len().max(right.len());
    let mut result = vec![crate::arith::rational_zero(); len];
    for (index, coefficient) in left.iter().enumerate() {
        result[index] += coefficient.clone();
    }
    for (index, coefficient) in right.iter().enumerate() {
        result[index] += coefficient.clone();
    }
    trim_poly(result)
}

fn poly_mul(left: &[Rational], right: &[Rational]) -> Vec<Rational> {
    if left.is_empty() || right.is_empty() {
        return Vec::new();
    }
    let mut result = vec![crate::arith::rational_zero(); left.len() + right.len() - 1];
    for (left_degree, left_coefficient) in left.iter().enumerate() {
        for (right_degree, right_coefficient) in right.iter().enumerate() {
            result[left_degree + right_degree] +=
                left_coefficient.clone() * right_coefficient.clone();
        }
    }
    trim_poly(result)
}

fn poly_neg(polynomial: &[Rational]) -> Vec<Rational> {
    polynomial
        .iter()
        .map(|coefficient| -coefficient.clone())
        .collect()
}

fn poly_shift(polynomial: &[Rational], shift: usize) -> Vec<Rational> {
    if polynomial.is_empty() {
        return Vec::new();
    }
    let mut result = vec![crate::arith::rational_zero(); shift];
    result.extend(polynomial.iter().cloned());
    result
}

fn trim_poly(mut coefficients: Vec<Rational>) -> Vec<Rational> {
    while coefficients.last().is_some_and(Zero::is_zero) {
        coefficients.pop();
    }
    coefficients
}

fn variable_index(variables: &[Variable], variable: &Variable) -> Option<usize> {
    variables.iter().position(|candidate| candidate == variable)
}

#[cfg(test)]
mod tests {
    use num_bigint::BigInt;
    use num_rational::BigRational;

    use super::*;
    use crate::compression::CompressionReplayCertificate;

    fn variable(symbol: &str) -> Variable {
        Variable {
            symbol: symbol.to_string(),
        }
    }

    fn rational(value: i64) -> Rational {
        BigRational::from_integer(BigInt::from(value))
    }

    fn monomial(exponents: &[u32]) -> Monomial {
        Monomial {
            exponents: exponents.to_vec(),
        }
    }

    fn term(variables: &[Variable], coefficient: i64, exponents: &[u32]) -> PolynomialQ {
        PolynomialQ::from_term(
            variables.to_vec(),
            rational(coefficient),
            monomial(exponents),
        )
    }

    fn polynomial(variables: &[Variable], terms: &[(i64, Vec<u32>)]) -> PolynomialQ {
        terms
            .iter()
            .fold(PolynomialQ::zero(variables.to_vec()), |sum, entry| {
                sum.add(&term(variables, entry.0, &entry.1))
            })
    }

    #[test]
    fn tower_route_uses_monic_triangular_structure() {
        let y = variable("Y");
        let x = variable("X");
        let t = variable("T");
        let variables = vec![y.clone(), x.clone(), t.clone()];
        let system = CertifiedSystemQ {
            equations: vec![
                polynomial(&variables, &[(1, vec![2, 0, 0]), (-2, vec![0, 0, 0])]),
                polynomial(&variables, &[(1, vec![0, 2, 0]), (-1, vec![1, 0, 0])]),
                polynomial(&variables, &[(1, vec![0, 0, 1]), (-1, vec![0, 1, 0])]),
            ],
            variables,
            target: t.clone(),
            guard_certificates: Vec::new(),
            replay: CompressionReplayCertificate { steps: Vec::new() },
        };

        let candidates = norm_trace_tower_candidates(&system);

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].origin, CandidateOrigin::NormTraceTower);
        assert_eq!(
            candidates[0].reconstructed.as_ref().unwrap().coefficients,
            vec![
                rational(-2),
                rational(0),
                rational(0),
                rational(0),
                rational(1)
            ]
        );
    }
}
