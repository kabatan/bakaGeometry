use std::collections::{BTreeMap, BTreeSet};

use num_traits::Zero;

use crate::compression::CertifiedSystemQ;
use crate::linear_q::{solve_linear_system_q, LinearSolveQ};
use crate::proof::{prove_fixed_target, FixedProofInput, ProofFailure};
use crate::proof_schedule::bounded_certificate_mode_prefix;
use crate::window::ProofWindow;
use crate::{Monomial, PolynomialQ, Rational, ResourceLimits, TargetCertificate, UniPolynomialQ};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MultipleRepairBudget {
    pub max_degree: usize,
}

#[derive(Clone, Debug)]
struct MultiplierColumn {
    vector: Vec<Rational>,
}

pub(crate) fn low_degree_multiple_repair(
    system: &CertifiedSystemQ,
    candidate: &UniPolynomialQ,
    proof_window: &ProofWindow,
    limits: &ResourceLimits,
) -> Result<TargetCertificate, ProofFailure> {
    if candidate.variable != system.target
        || candidate.is_zero()
        || proof_window.multiplier_supports.len() != system.equations.len()
        || !proof_window
            .multiplier_supports
            .iter()
            .flatten()
            .all(|monomial| monomial.exponents.len() == system.variables.len())
    {
        return Err(ProofFailure::InvalidInput);
    }

    let Some(max_degree) = limits.max_window_degree else {
        return Err(ProofFailure::NoCertificateFound);
    };
    let Some(max_proof_weight) = limits.max_proof_weight else {
        return Err(ProofFailure::NoCertificateFound);
    };
    let modes = bounded_certificate_mode_prefix(max_proof_weight);
    for degree in 0..=max_degree {
        for anchor in 0..=degree {
            let Some(product) =
                solve_repaired_product(system, candidate, proof_window, degree, anchor)
            else {
                continue;
            };
            if product.is_zero() {
                continue;
            }
            for mode in &modes {
                let input = FixedProofInput {
                    system: system.clone(),
                    candidate: product.clone(),
                    proof_window: proof_window.clone(),
                    certificate_mode: mode.clone(),
                };
                if let Ok(certificate) = prove_fixed_target(input) {
                    return Ok(certificate);
                }
            }
        }
    }

    Err(ProofFailure::NoCertificateFound)
}

fn solve_repaired_product(
    system: &CertifiedSystemQ,
    candidate: &UniPolynomialQ,
    proof_window: &ProofWindow,
    degree: usize,
    anchor: usize,
) -> Option<UniPolynomialQ> {
    let basis = (0..=degree)
        .map(|degree_shift| shifted_candidate(candidate, degree_shift))
        .collect::<Vec<_>>();
    let basis_multivariate = basis
        .iter()
        .map(|polynomial| polynomial.to_multivariate(&system.variables))
        .collect::<Vec<_>>();
    let row_monomials = repair_rows(system, proof_window, &basis_multivariate);
    let multiplier_columns = multiplier_columns(system, proof_window, &row_monomials);

    let mut columns = multiplier_columns
        .iter()
        .map(|column| column.vector.clone())
        .collect::<Vec<_>>();
    let mut free_a_degrees = Vec::new();
    for (degree_shift, polynomial) in basis_multivariate.iter().enumerate() {
        if degree_shift == anchor {
            continue;
        }
        free_a_degrees.push(degree_shift);
        columns.push(vector_from_polynomial(polynomial, &row_monomials, -1));
    }

    let matrix = rows_from_columns(&columns);
    let rhs = vector_from_polynomial(&basis_multivariate[anchor], &row_monomials, 1);
    let solution = match solve_linear_system_q(&matrix, &rhs) {
        LinearSolveQ::Consistent { solution, .. } => solution,
        LinearSolveQ::Inconsistent { .. } => return None,
    };

    let mut a_coefficients = vec![crate::arith::rational_zero(); degree + 1];
    a_coefficients[anchor] = crate::arith::rational_one();
    for (offset, degree_shift) in free_a_degrees.iter().copied().enumerate() {
        a_coefficients[degree_shift] = solution[multiplier_columns.len() + offset].clone();
    }

    let product = multiply_by_univariate(candidate, &a_coefficients);
    if product.is_zero() {
        return None;
    }

    Some(product.primitive_integer_normalized())
}

fn shifted_candidate(candidate: &UniPolynomialQ, degree_shift: usize) -> UniPolynomialQ {
    if candidate.is_zero() {
        return UniPolynomialQ::zero(candidate.variable.clone());
    }
    let mut coefficients = vec![crate::arith::rational_zero(); degree_shift];
    coefficients.extend(candidate.coefficients.iter().cloned());
    let mut result = UniPolynomialQ {
        variable: candidate.variable.clone(),
        coefficients,
    };
    result.normalize();
    result
}

fn multiply_by_univariate(candidate: &UniPolynomialQ, multiplier: &[Rational]) -> UniPolynomialQ {
    if candidate.is_zero() || multiplier.iter().all(Zero::is_zero) {
        return UniPolynomialQ::zero(candidate.variable.clone());
    }
    let mut coefficients =
        vec![crate::arith::rational_zero(); candidate.coefficients.len() + multiplier.len() - 1];
    for (left_degree, left_coefficient) in candidate.coefficients.iter().enumerate() {
        for (right_degree, right_coefficient) in multiplier.iter().enumerate() {
            coefficients[left_degree + right_degree] +=
                left_coefficient.clone() * right_coefficient.clone();
        }
    }
    let mut result = UniPolynomialQ {
        variable: candidate.variable.clone(),
        coefficients,
    };
    result.normalize();
    result
}

fn repair_rows(
    system: &CertifiedSystemQ,
    proof_window: &ProofWindow,
    basis_multivariate: &[PolynomialQ],
) -> Vec<Monomial> {
    let mut rows = BTreeSet::new();
    for polynomial in basis_multivariate {
        rows.extend(polynomial.support());
    }
    for (equation, supports) in system
        .equations
        .iter()
        .zip(&proof_window.multiplier_supports)
    {
        for multiplier_monomial in supports {
            for equation_monomial in equation.support() {
                rows.insert(multiplier_monomial.multiply(&equation_monomial));
            }
        }
    }
    let mut row_monomials = rows.into_iter().collect::<Vec<_>>();
    row_monomials.sort_by_key(canonical_monomial_key);
    row_monomials
}

fn multiplier_columns(
    system: &CertifiedSystemQ,
    proof_window: &ProofWindow,
    row_monomials: &[Monomial],
) -> Vec<MultiplierColumn> {
    let row_index = row_index_map(row_monomials);
    let mut columns = Vec::new();
    for (equation, supports) in system
        .equations
        .iter()
        .zip(&proof_window.multiplier_supports)
    {
        for multiplier_monomial in supports {
            let mut vector = vec![crate::arith::rational_zero(); row_monomials.len()];
            for (equation_monomial, coefficient) in &equation.terms {
                let product_monomial = multiplier_monomial.multiply(equation_monomial);
                if let Some(row) = row_index.get(&product_monomial) {
                    vector[*row] += coefficient.clone();
                }
            }
            columns.push(MultiplierColumn { vector });
        }
    }
    columns
}

fn vector_from_polynomial(
    polynomial: &PolynomialQ,
    row_monomials: &[Monomial],
    sign: i32,
) -> Vec<Rational> {
    let row_index = row_index_map(row_monomials);
    let factor = if sign < 0 {
        -crate::arith::rational_one()
    } else {
        crate::arith::rational_one()
    };
    let mut vector = vec![crate::arith::rational_zero(); row_monomials.len()];
    for (monomial, coefficient) in &polynomial.terms {
        if let Some(row) = row_index.get(monomial) {
            vector[*row] = coefficient.clone() * factor.clone();
        }
    }
    vector
}

fn rows_from_columns(columns: &[Vec<Rational>]) -> Vec<Vec<Rational>> {
    let rows = columns.first().map_or(0, Vec::len);
    (0..rows)
        .map(|row| columns.iter().map(|column| column[row].clone()).collect())
        .collect()
}

fn row_index_map(row_monomials: &[Monomial]) -> BTreeMap<Monomial, usize> {
    row_monomials
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, monomial)| (monomial, index))
        .collect()
}

fn canonical_monomial_key(monomial: &Monomial) -> (u32, Vec<u32>) {
    (monomial.total_degree(), monomial.exponents.clone())
}

#[cfg(test)]
mod tests {
    use num_bigint::BigInt;
    use num_rational::BigRational;

    use super::*;
    use crate::compression::CompressionReplayCertificate;
    use crate::window::ProofWindow;
    use crate::{
        verify_certificate, Monomial, PolynomialQ, SolverCertificate, TargetProblemQ, Variable,
        VerificationResult,
    };

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

    fn uni(variable: &Variable, coefficients: &[i64]) -> UniPolynomialQ {
        let mut polynomial = UniPolynomialQ {
            variable: variable.clone(),
            coefficients: coefficients.iter().map(|value| rational(*value)).collect(),
        };
        polynomial.normalize();
        polynomial
    }

    #[test]
    fn low_degree_multiple_returns_repaired_product() {
        let x = variable("X");
        let t = variable("T");
        let variables = vec![x.clone(), t.clone()];
        let equations = vec![
            polynomial(&variables, &[(1, vec![2, 0]), (-1, vec![1, 0])]),
            polynomial(&variables, &[(-1, vec![1, 0]), (1, vec![0, 1])]),
        ];
        let problem = TargetProblemQ {
            equations: equations.clone(),
            variables: variables.clone(),
            target: t.clone(),
            semantic_guards: Vec::new(),
        };
        let system = CertifiedSystemQ {
            equations,
            variables,
            target: t.clone(),
            guard_certificates: Vec::new(),
            replay: CompressionReplayCertificate { steps: Vec::new() },
        };
        let proof_window = ProofWindow {
            multiplier_supports: vec![
                vec![monomial(&[0, 0])],
                vec![monomial(&[0, 0]), monomial(&[1, 0]), monomial(&[0, 1])],
            ],
        };
        let limits = ResourceLimits {
            max_window_degree: Some(1),
            max_proof_weight: Some(1),
            max_matrix_rows: None,
            max_matrix_cols: None,
            max_candidate_count: None,
        };

        let certificate =
            low_degree_multiple_repair(&system, &uni(&t, &[0, 1]), &proof_window, &limits).unwrap();

        let TargetCertificate::IdealMembership { support, .. } = &certificate else {
            panic!("repair should return an ideal target certificate");
        };
        assert_eq!(support.primitive_integer_normalized(), uni(&t, &[0, -1, 1]));
        assert_ne!(support.primitive_integer_normalized(), uni(&t, &[0, 1]));
        assert_eq!(
            verify_certificate(problem, SolverCertificate::TargetCover(certificate)),
            VerificationResult::Verified
        );
    }

    #[test]
    fn low_degree_multiple_without_window_bound_does_not_use_hidden_capped_search() {
        let t = variable("T");
        let variables = vec![t.clone()];
        let system = CertifiedSystemQ {
            equations: vec![polynomial(&variables, &[(1, vec![1])])],
            variables,
            target: t.clone(),
            guard_certificates: Vec::new(),
            replay: CompressionReplayCertificate { steps: Vec::new() },
        };
        let proof_window = ProofWindow {
            multiplier_supports: vec![vec![monomial(&[0])]],
        };
        let limits = ResourceLimits {
            max_window_degree: None,
            max_proof_weight: Some(1),
            max_matrix_rows: None,
            max_matrix_cols: None,
            max_candidate_count: None,
        };

        let result = low_degree_multiple_repair(&system, &uni(&t, &[0, 1]), &proof_window, &limits);

        assert!(matches!(result, Err(ProofFailure::NoCertificateFound)));
    }
}
