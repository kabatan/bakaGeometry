use std::collections::{BTreeMap, BTreeSet};

use num_traits::Zero;

use crate::candidates::{
    CandidateOracle, CandidateOrigin, CandidateTrace, RouteWitnessTrace, TargetCandidate,
};
use crate::compression::CertifiedSystemQ;
use crate::matrix_q::{solve_linear_system_q, LinearSolveQ};
use crate::verifier::verify_guard_certificate;
use crate::window::CertificateWindow;
use crate::{
    GuardCertificate, Monomial, PolynomialQ, Rational, TargetProblemQ, UniPolynomialQ, Variable,
    VerificationResult,
};

pub(crate) struct NormTraceTowerOracle;

#[derive(Clone, Debug)]
struct TowerLevel {
    variable_index: usize,
    equation_index: usize,
    degree: u32,
    leading: PolynomialQ,
    lower: PolynomialQ,
    leading_guard: Option<GuardCertificate>,
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
    let Some(levels) = detect_guarded_tower(system, target_equation) else {
        return Vec::new();
    };
    if levels.is_empty() || !target_expression_depends_on_tower(system, &target_expression, &levels)
    {
        return Vec::new();
    }

    let basis = tower_basis(system.variables.len(), &levels);
    let _verified_guard_count = levels
        .iter()
        .filter(|level| {
            level.leading_guard.as_ref().is_some_and(|certificate| {
                guard_certificate_has_polynomial(certificate, &level.leading)
            })
        })
        .count();
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

    vec![TargetCandidate::from_origin(
        Vec::new(),
        Some(support),
        CandidateOrigin::NormTraceTower,
        vec![CandidateTrace::RouteWitness(RouteWitnessTrace {
            origin: CandidateOrigin::NormTraceTower,
            equation_indices,
            support_size: basis.len(),
        })],
    )]
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
        if valid && !target_coefficient.is_zero() {
            let scale = -crate::arith::rational_one() / target_coefficient;
            return Some((equation_index, rest.scale(&scale)));
        }
    }
    None
}

fn detect_guarded_tower(
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
                if let Some(level) = guarded_level(
                    system,
                    equation,
                    equation_index,
                    variable_index,
                    &available,
                    &levels,
                ) {
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

fn guarded_level(
    system: &CertifiedSystemQ,
    equation: &PolynomialQ,
    equation_index: usize,
    variable_index: usize,
    available: &BTreeSet<usize>,
    previous_levels: &[TowerLevel],
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

    let mut leading = PolynomialQ::zero(system.variables.clone());
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
            let mut leading_exponents = monomial.exponents.clone();
            leading_exponents[variable_index] = 0;
            leading = leading.add(&PolynomialQ::from_term(
                system.variables.clone(),
                coefficient.clone(),
                Monomial {
                    exponents: leading_exponents,
                },
            ));
        } else {
            lower = lower.add(&PolynomialQ::from_term(
                system.variables.clone(),
                coefficient.clone(),
                monomial.clone(),
            ));
        }
    }
    if leading.is_zero() || polynomial_depends_on_variable(&leading, variable_index) {
        return None;
    }
    let leading_guard = if leading == PolynomialQ::one(system.variables.clone()) {
        None
    } else {
        Some(verified_guard_for_polynomial(system, &leading)?)
    };
    let leading_inverse = inverse_mod_previous_tower(system, &leading, previous_levels)?;
    let replacement = lower
        .scale(&-crate::arith::rational_one())
        .mul(&leading_inverse);
    let replacement = reduce_by_tower(&replacement, previous_levels);

    Some(TowerLevel {
        variable_index,
        equation_index,
        degree,
        leading,
        lower: replacement,
        leading_guard,
    })
}

fn inverse_mod_previous_tower(
    system: &CertifiedSystemQ,
    leading: &PolynomialQ,
    previous_levels: &[TowerLevel],
) -> Option<PolynomialQ> {
    if let Some(constant) = constant_value(leading) {
        if constant.is_zero() {
            return None;
        }
        return Some(constant_polynomial(
            system,
            crate::arith::rational_one() / constant,
        ));
    }
    if polynomial_depends_on_variable_index(
        leading,
        variable_index(&system.variables, &system.target)?,
    ) {
        return None;
    }
    let basis = tower_basis(system.variables.len(), previous_levels);
    let mut matrix = vec![vec![crate::arith::rational_zero(); basis.len()]; basis.len()];
    for (column_index, basis_monomial) in basis.iter().enumerate() {
        let basis_polynomial = PolynomialQ::from_term(
            system.variables.clone(),
            crate::arith::rational_one(),
            basis_monomial.clone(),
        );
        let reduced = reduce_by_tower(&basis_polynomial.mul(leading), previous_levels);
        let vector = vector_over_tower_basis(&reduced, &basis, system)?;
        for (row_index, coefficient) in vector.into_iter().enumerate() {
            matrix[row_index][column_index] = coefficient;
        }
    }
    let mut rhs = vec![crate::arith::rational_zero(); basis.len()];
    rhs[0] = crate::arith::rational_one();
    let LinearSolveQ::Consistent { solution, .. } = solve_linear_system_q(&matrix, &rhs) else {
        return None;
    };
    let mut inverse = PolynomialQ::zero(system.variables.clone());
    for (coefficient, monomial) in solution.into_iter().zip(basis) {
        if coefficient.is_zero() {
            continue;
        }
        inverse = inverse.add(&PolynomialQ::from_term(
            system.variables.clone(),
            coefficient,
            monomial,
        ));
    }
    Some(inverse)
}

fn verified_guard_for_polynomial(
    system: &CertifiedSystemQ,
    guard: &PolynomialQ,
) -> Option<GuardCertificate> {
    let problem = guard_problem_from_system(system);
    system
        .guard_certificates
        .iter()
        .find(|certificate| {
            guard_certificate_has_polynomial(certificate, guard)
                && verify_guard_certificate(&problem, certificate) == VerificationResult::Verified
        })
        .cloned()
}

fn guard_problem_from_system(system: &CertifiedSystemQ) -> TargetProblemQ {
    TargetProblemQ {
        equations: system.equations.clone(),
        variables: system.variables.clone(),
        target: system.target.clone(),
        semantic_guards: system.semantic_guards.clone(),
    }
}

fn guard_certificate_has_polynomial(certificate: &GuardCertificate, guard: &PolynomialQ) -> bool {
    match certificate {
        GuardCertificate::InputSemanticNonzero {
            guard: certificate_guard,
            record,
        } => certificate_guard == guard && &record.polynomial == guard,
        GuardCertificate::AlgebraicNonvanishing {
            guard: certificate_guard,
            ..
        }
        | GuardCertificate::RealAdmissibleNonvanishing {
            guard: certificate_guard,
            ..
        } => certificate_guard == guard,
        GuardCertificate::DerivedProduct {
            product, factors, ..
        } => {
            product == guard
                || factors
                    .iter()
                    .any(|factor| guard_certificate_has_polynomial(factor, guard))
        }
    }
}

fn constant_value(polynomial: &PolynomialQ) -> Option<Rational> {
    let mut value = None;
    for (monomial, coefficient) in &polynomial.terms {
        if monomial.exponents.iter().any(|exponent| *exponent != 0) {
            return None;
        }
        value = Some(coefficient.clone());
    }
    value
}

fn constant_polynomial(system: &CertifiedSystemQ, coefficient: Rational) -> PolynomialQ {
    PolynomialQ::from_term(
        system.variables.clone(),
        coefficient,
        Monomial {
            exponents: vec![0; system.variables.len()],
        },
    )
}

fn polynomial_depends_on_variable(polynomial: &PolynomialQ, variable_index: usize) -> bool {
    polynomial_depends_on_variable_index(polynomial, variable_index)
}

fn polynomial_depends_on_variable_index(polynomial: &PolynomialQ, variable_index: usize) -> bool {
    polynomial
        .terms
        .keys()
        .any(|monomial| monomial.exponents[variable_index] != 0)
}

fn vector_over_tower_basis(
    polynomial: &PolynomialQ,
    basis: &[Monomial],
    system: &CertifiedSystemQ,
) -> Option<Vec<Rational>> {
    let target_index = variable_index(&system.variables, &system.target)?;
    let index = basis
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, monomial)| (monomial, index))
        .collect::<BTreeMap<_, _>>();
    let mut vector = vec![crate::arith::rational_zero(); basis.len()];
    for (monomial, coefficient) in &polynomial.terms {
        if monomial.exponents[target_index] != 0 {
            return None;
        }
        let row = *index.get(monomial)?;
        vector[row] += coefficient.clone();
    }
    Some(vector)
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
    use crate::{ExactIdentity, ExactIdentityKind, GuardKind, GuardProvenance, GuardRecord};

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

    fn nonzero_guard(variables: &[Variable], coefficient: i64) -> GuardCertificate {
        let guard = PolynomialQ::from_term(
            variables.to_vec(),
            rational(coefficient),
            monomial(&vec![0; variables.len()]),
        );
        let record = GuardRecord {
            polynomial: guard.clone(),
            kind: GuardKind::NonZero,
            provenance: GuardProvenance {
                description: "nonzero tower leading coefficient".to_string(),
            },
        };
        GuardCertificate::InputSemanticNonzero { guard, record }
    }

    fn nonzero_polynomial_guard(guard: PolynomialQ) -> GuardCertificate {
        let record = nonzero_polynomial_record(&guard);
        GuardCertificate::InputSemanticNonzero { guard, record }
    }

    fn nonzero_polynomial_record(guard: &PolynomialQ) -> GuardRecord {
        GuardRecord {
            polynomial: guard.clone(),
            kind: GuardKind::NonZero,
            provenance: GuardProvenance {
                description: "semantic nonzero tower leading polynomial".to_string(),
            },
        }
    }

    fn derived_product_guard(guard: PolynomialQ) -> GuardCertificate {
        GuardCertificate::DerivedProduct {
            product: guard.clone(),
            factors: vec![nonzero_polynomial_guard(guard)],
            identity: ExactIdentity {
                kind: ExactIdentityKind::GuardProduct,
            },
        }
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
            semantic_guards: Vec::new(),
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

    #[test]
    fn tower_route_requires_guard_for_nonmonic_leading_coefficient() {
        let y = variable("Y");
        let t = variable("T");
        let variables = vec![y.clone(), t.clone()];
        let mut system = CertifiedSystemQ {
            equations: vec![
                polynomial(&variables, &[(2, vec![2, 0]), (-4, vec![0, 0])]),
                polynomial(&variables, &[(1, vec![0, 1]), (-1, vec![1, 0])]),
            ],
            variables: variables.clone(),
            target: t.clone(),
            semantic_guards: Vec::new(),
            guard_certificates: Vec::new(),
            replay: CompressionReplayCertificate { steps: Vec::new() },
        };

        assert!(norm_trace_tower_candidates(&system).is_empty());

        let leading_guard = nonzero_guard(&variables, 2);
        if let GuardCertificate::InputSemanticNonzero { record, .. } = &leading_guard {
            system.semantic_guards.push(record.clone());
        }
        system.guard_certificates.push(leading_guard);
        let candidates = norm_trace_tower_candidates(&system);

        assert_eq!(candidates.len(), 1);
        assert_eq!(
            candidates[0].reconstructed.as_ref().unwrap().coefficients,
            vec![rational(-2), rational(0), rational(1)]
        );
    }

    #[test]
    fn tower_route_allows_non_unit_target_coefficient() {
        let y = variable("Y");
        let x = variable("X");
        let t = variable("T");
        let variables = vec![y.clone(), x.clone(), t.clone()];
        let system = CertifiedSystemQ {
            equations: vec![
                polynomial(&variables, &[(1, vec![2, 0, 0]), (-2, vec![0, 0, 0])]),
                polynomial(&variables, &[(1, vec![0, 2, 0]), (-1, vec![1, 0, 0])]),
                polynomial(&variables, &[(2, vec![0, 0, 1]), (-1, vec![0, 1, 0])]),
            ],
            variables,
            target: t.clone(),
            semantic_guards: Vec::new(),
            guard_certificates: Vec::new(),
            replay: CompressionReplayCertificate { steps: Vec::new() },
        };

        let candidates = norm_trace_tower_candidates(&system);

        assert_eq!(candidates.len(), 1);
        assert_eq!(
            candidates[0].reconstructed.as_ref().unwrap().coefficients,
            vec![
                rational(-1),
                rational(0),
                rational(0),
                rational(0),
                rational(8)
            ]
        );
    }

    #[test]
    fn tower_route_uses_verified_guarded_nonconstant_leading_coefficient() {
        let x = variable("X");
        let y = variable("Y");
        let t = variable("T");
        let variables = vec![x.clone(), y.clone(), t.clone()];
        let x_guard = polynomial(&variables, &[(1, vec![1, 0, 0])]);
        let mut system = CertifiedSystemQ {
            equations: vec![
                polynomial(&variables, &[(1, vec![2, 0, 0]), (-2, vec![0, 0, 0])]),
                polynomial(&variables, &[(1, vec![1, 2, 0]), (-1, vec![0, 0, 0])]),
                polynomial(&variables, &[(1, vec![0, 0, 1]), (-1, vec![0, 1, 0])]),
            ],
            variables,
            target: t.clone(),
            semantic_guards: Vec::new(),
            guard_certificates: Vec::new(),
            replay: CompressionReplayCertificate { steps: Vec::new() },
        };

        assert!(norm_trace_tower_candidates(&system).is_empty());

        system
            .guard_certificates
            .push(derived_product_guard(x_guard.clone()));
        assert!(norm_trace_tower_candidates(&system).is_empty());

        system
            .semantic_guards
            .push(nonzero_polynomial_record(&x_guard));
        let candidates = norm_trace_tower_candidates(&system);

        assert_eq!(candidates.len(), 1);
        assert_eq!(
            candidates[0].reconstructed.as_ref().unwrap().coefficients,
            vec![
                rational(-1),
                rational(0),
                rational(0),
                rational(0),
                rational(2)
            ]
        );
        let levels = detect_guarded_tower(&system, 2).unwrap();
        let guarded_level = levels
            .iter()
            .find(|level| level.variable_index == 1)
            .expect("y level should be detected");
        assert_eq!(
            guarded_level.leading,
            polynomial(&system.variables, &[(1, vec![1, 0, 0])])
        );
        assert!(guarded_level.leading_guard.is_some());
    }
}
