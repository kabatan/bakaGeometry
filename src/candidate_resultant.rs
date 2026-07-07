use std::collections::{BTreeMap, BTreeSet};

use crate::candidates::{
    CandidateOracle, CandidateOrigin, CandidateTrace, ModularWitnessTrace, RouteWitnessTrace,
    TargetCandidate,
};
use crate::compression::CertifiedSystemQ;
use crate::finite_field::{rational_to_mod_prime, PrimeModulus};
use crate::linear_fp::{columns_to_matrix, nullspace_matrix_fp};
use crate::univariate::UniPolynomialFp;
use crate::window::CertificateWindow;
use crate::{Monomial, PolynomialQ};

pub(crate) struct HiddenVariableSparseResultantOracle {
    pub primes: Vec<u64>,
}

impl CandidateOracle for HiddenVariableSparseResultantOracle {
    fn generate(
        &self,
        system: &CertifiedSystemQ,
        window: &CertificateWindow,
    ) -> Vec<TargetCandidate> {
        hidden_variable_sparse_resultant_candidates(system, window, &self.primes)
    }
}

pub(crate) fn hidden_variable_sparse_resultant_candidates(
    system: &CertifiedSystemQ,
    window: &CertificateWindow,
    primes: &[u64],
) -> Vec<TargetCandidate> {
    let target_degree = window.target_degree.max(1);
    let multiplier_supports = support_array_from_newton_data(system, target_degree - 1);
    let row_monomials = resultant_rows(system, target_degree, &multiplier_supports);
    let exact_columns = resultant_columns(system, &row_monomials, &multiplier_supports);
    let target_columns = target_power_columns(system, &row_monomials, target_degree);
    let mut candidates = Vec::new();

    for prime in primes {
        let Some(modulus) = PrimeModulus::new(*prime) else {
            continue;
        };
        let Some(mut modular_columns) = exact_columns
            .iter()
            .map(|column| rational_column_to_mod(column, modulus))
            .collect::<Option<Vec<_>>>()
        else {
            continue;
        };
        let Some(modular_target_columns) = target_columns
            .iter()
            .map(|column| rational_column_to_mod(column, modulus))
            .collect::<Option<Vec<_>>>()
        else {
            continue;
        };
        modular_columns.extend(modular_target_columns.clone());

        let relation_matrix = columns_to_matrix(&modular_columns);
        for relation in nullspace_matrix_fp(&relation_matrix, modulus) {
            let target_relation = relation[exact_columns.len()..].to_vec();
            let Some(coefficients) = normalized_coefficients(target_relation, modulus) else {
                continue;
            };
            let mut support = UniPolynomialFp {
                variable: system.target.clone(),
                modulus: *prime,
                coefficients: coefficients.clone(),
            };
            support.normalize();
            if support.is_zero() {
                continue;
            }
            candidates.push(TargetCandidate {
                support_mod_primes: vec![support],
                reconstructed: None,
                origin: CandidateOrigin::HiddenVariableSparseResultant,
                traces: vec![
                    CandidateTrace::RouteWitness(RouteWitnessTrace {
                        origin: CandidateOrigin::HiddenVariableSparseResultant,
                        equation_indices: (0..system.equations.len()).collect(),
                        support_size: row_monomials.len(),
                    }),
                    CandidateTrace::ModularWitness(ModularWitnessTrace {
                        prime: *prime,
                        active_multiplier_supports: multiplier_supports.clone(),
                        relation_coefficients: coefficients,
                        residual_vectors: Vec::new(),
                    }),
                ],
            });
        }
    }

    candidates
}

fn support_array_from_newton_data(
    system: &CertifiedSystemQ,
    max_degree: usize,
) -> Vec<Vec<Monomial>> {
    let base = monomials_up_to_degree(system.variables.len(), max_degree);
    system
        .equations
        .iter()
        .map(|equation| {
            let mut support = base.clone();
            if equation.support().iter().any(|monomial| {
                monomial
                    .exponents
                    .iter()
                    .enumerate()
                    .any(|(index, exponent)| {
                        *exponent != 0 && system.variables[index] != system.target
                    })
            }) {
                support.extend(equation.support());
            }
            support.sort_by_key(canonical_monomial_key);
            support.dedup();
            support
        })
        .collect()
}

fn resultant_rows(
    system: &CertifiedSystemQ,
    target_degree: usize,
    multiplier_supports: &[Vec<Monomial>],
) -> Vec<Monomial> {
    let mut rows = BTreeSet::new();
    let target_index = system
        .variables
        .iter()
        .position(|variable| variable == &system.target)
        .unwrap();
    for degree in 0..=target_degree {
        let mut exponents = vec![0; system.variables.len()];
        exponents[target_index] = degree as u32;
        rows.insert(Monomial { exponents });
    }
    for (equation, supports) in system.equations.iter().zip(multiplier_supports) {
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

fn resultant_columns(
    system: &CertifiedSystemQ,
    row_monomials: &[Monomial],
    multiplier_supports: &[Vec<Monomial>],
) -> Vec<Vec<crate::Rational>> {
    let row_index = row_index_map(row_monomials);
    let mut columns = Vec::new();
    for (equation, supports) in system.equations.iter().zip(multiplier_supports) {
        for multiplier_monomial in supports {
            let product = monomial_times_polynomial(system, multiplier_monomial, equation);
            columns.push(vector_from_polynomial(
                &product,
                &row_index,
                row_monomials.len(),
            ));
        }
    }
    columns
}

fn target_power_columns(
    system: &CertifiedSystemQ,
    row_monomials: &[Monomial],
    target_degree: usize,
) -> Vec<Vec<crate::Rational>> {
    let row_index = row_index_map(row_monomials);
    let target_index = system
        .variables
        .iter()
        .position(|variable| variable == &system.target)
        .unwrap();
    (0..=target_degree)
        .map(|degree| {
            let mut exponents = vec![0; system.variables.len()];
            exponents[target_index] = degree as u32;
            let polynomial = PolynomialQ::from_term(
                system.variables.clone(),
                crate::arith::rational_one(),
                Monomial { exponents },
            );
            vector_from_polynomial(&polynomial, &row_index, row_monomials.len())
        })
        .collect()
}

fn monomial_times_polynomial(
    system: &CertifiedSystemQ,
    monomial: &Monomial,
    polynomial: &PolynomialQ,
) -> PolynomialQ {
    let multiplier = PolynomialQ::from_term(
        system.variables.clone(),
        crate::arith::rational_one(),
        monomial.clone(),
    );
    multiplier.mul(polynomial)
}

fn vector_from_polynomial(
    polynomial: &PolynomialQ,
    row_index: &BTreeMap<Monomial, usize>,
    row_count: usize,
) -> Vec<crate::Rational> {
    let mut vector = vec![crate::arith::rational_zero(); row_count];
    for (monomial, coefficient) in &polynomial.terms {
        if let Some(row) = row_index.get(monomial) {
            vector[*row] += coefficient.clone();
        }
    }
    vector
}

fn rational_column_to_mod(column: &[crate::Rational], modulus: PrimeModulus) -> Option<Vec<u64>> {
    column
        .iter()
        .map(|coefficient| rational_to_mod_prime(coefficient, modulus))
        .collect()
}

fn normalized_coefficients(mut coefficients: Vec<u64>, modulus: PrimeModulus) -> Option<Vec<u64>> {
    while coefficients
        .last()
        .is_some_and(|coefficient| *coefficient == 0)
    {
        coefficients.pop();
    }
    let leading = *coefficients.last()?;
    let inverse = modulus.inv(leading)?;
    for coefficient in &mut coefficients {
        *coefficient = modulus.mul(*coefficient, inverse);
    }
    Some(coefficients)
}

fn monomials_up_to_degree(variable_count: usize, max_degree: usize) -> Vec<Monomial> {
    let mut monomials = Vec::new();
    let mut current = vec![0; variable_count];
    enumerate_monomials(
        variable_count,
        max_degree as u32,
        0,
        &mut current,
        &mut monomials,
    );
    monomials
}

fn enumerate_monomials(
    variable_count: usize,
    remaining_degree: u32,
    index: usize,
    current: &mut [u32],
    monomials: &mut Vec<Monomial>,
) {
    if index == variable_count {
        monomials.push(Monomial {
            exponents: current.to_vec(),
        });
        return;
    }
    for exponent in 0..=remaining_degree {
        current[index] = exponent;
        enumerate_monomials(
            variable_count,
            remaining_degree - exponent,
            index + 1,
            current,
            monomials,
        );
    }
    current[index] = 0;
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
    use crate::window::make_row_closed_certificate_window;
    use crate::{Rational, Variable};

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
    fn resultant_route_uses_three_polynomial_expansion() {
        let x = variable("X");
        let y = variable("Y");
        let t = variable("T");
        let variables = vec![x.clone(), y.clone(), t.clone()];
        let system = CertifiedSystemQ {
            equations: vec![
                polynomial(&variables, &[(1, vec![1, 0, 0]), (-1, vec![0, 1, 0])]),
                polynomial(&variables, &[(1, vec![0, 1, 0]), (-1, vec![0, 0, 1])]),
                polynomial(&variables, &[(1, vec![1, 0, 0]), (-2, vec![0, 0, 0])]),
            ],
            variables,
            target: t.clone(),
            guard_certificates: Vec::new(),
            replay: CompressionReplayCertificate { steps: Vec::new() },
        };
        let window = make_row_closed_certificate_window(
            &system,
            1,
            vec![Vec::new(); system.equations.len()],
        );

        let candidates = hidden_variable_sparse_resultant_candidates(&system, &window, &[5]);

        assert!(candidates.iter().any(|candidate| {
            candidate.origin == CandidateOrigin::HiddenVariableSparseResultant
                && candidate
                    .support_mod_primes
                    .iter()
                    .any(|support| support.variable == t && support.coefficients == vec![3, 1])
        }));
    }
}
