use crate::candidates::{
    CandidateOracle, CandidateOrigin, CandidateTrace, SliceAffineCoefficient, SliceAffineEquation,
    SliceAssignment, SliceWitnessTrace, TargetCandidate,
};
use crate::compression::CertifiedSystemQ;
use crate::finite_field::{rational_to_mod_prime, PrimeModulus};
use crate::window::{make_row_closed_certificate_window, CertificateWindow};
use crate::{GuardCertificate, Monomial, PolynomialQ};

pub(crate) struct SliceSpecializationOracle {
    pub primes: Vec<u64>,
    pub slice_count: usize,
}

impl CandidateOracle for SliceSpecializationOracle {
    fn generate(
        &self,
        system: &CertifiedSystemQ,
        window: &CertificateWindow,
    ) -> Vec<TargetCandidate> {
        slice_specialization_candidates(system, window, &self.primes, self.slice_count)
    }
}

pub(crate) fn slice_specialization_candidates(
    system: &CertifiedSystemQ,
    window: &CertificateWindow,
    primes: &[u64],
    slice_count: usize,
) -> Vec<TargetCandidate> {
    let Some(target_index) = system
        .variables
        .iter()
        .position(|variable| variable == &system.target)
    else {
        return Vec::new();
    };
    let sliced_variables = (0..system.variables.len())
        .filter(|index| *index != target_index)
        .collect::<Vec<_>>();
    if sliced_variables.is_empty() {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    for prime in primes {
        let Some(modulus) = PrimeModulus::new(*prime) else {
            continue;
        };
        for slice_index in 0..slice_count.max(1) {
            let affine_equations =
                deterministic_affine_slice(&sliced_variables, slice_index, modulus);
            if affine_equations.is_empty()
                || !slice_input_is_prime_admissible(system, &affine_equations, modulus)
            {
                continue;
            }
            let sliced_system = build_sliced_system(system, &affine_equations);
            let sliced_window = make_row_closed_certificate_window(
                &sliced_system,
                window.target_degree,
                scheduled_slice_supports(&sliced_system, window.target_degree),
            );
            let mut inner_candidates = crate::candidate_residual::residual_cyclic_candidates(
                &sliced_system,
                &sliced_window,
                &[*prime],
            );
            if inner_candidates.is_empty() {
                inner_candidates =
                    crate::candidate_resultant::hidden_variable_sparse_resultant_candidates(
                        &sliced_system,
                        &sliced_window,
                        &[*prime],
                    );
            }

            for inner in inner_candidates {
                let Some(coefficients) = modular_coefficients_for_prime(&inner, *prime) else {
                    continue;
                };
                candidates.push(TargetCandidate::from_origin(
                    inner.support_mod_primes,
                    inner.reconstructed,
                    CandidateOrigin::SliceSpecialization,
                    vec![CandidateTrace::SliceWitness(SliceWitnessTrace {
                        prime: *prime,
                        assignments: coordinate_assignments_for_compat_key(&affine_equations),
                        affine_equations: affine_equations.clone(),
                        equation_index: 0,
                        equation_indices: (0..sliced_system.equations.len()).collect(),
                        internal_origin: inner.origin,
                        relation_coefficients: coefficients,
                    })],
                ));
            }
        }
    }

    candidates
}

fn build_sliced_system(
    system: &CertifiedSystemQ,
    affine_equations: &[SliceAffineEquation],
) -> CertifiedSystemQ {
    let mut equations = system.equations.clone();
    equations.extend(
        affine_equations
            .iter()
            .map(|equation| affine_slice_equation(system, equation)),
    );
    CertifiedSystemQ {
        equations,
        variables: system.variables.clone(),
        target: system.target.clone(),
        semantic_guards: system.semantic_guards.clone(),
        guard_certificates: system.guard_certificates.clone(),
        replay: system.replay.clone(),
    }
}

fn affine_slice_equation(system: &CertifiedSystemQ, equation: &SliceAffineEquation) -> PolynomialQ {
    let mut polynomial = PolynomialQ::zero(system.variables.clone());
    for coefficient in &equation.coefficients {
        let mut variable_exponents = vec![0; system.variables.len()];
        variable_exponents[coefficient.variable_index] = 1;
        polynomial = polynomial.add(&PolynomialQ::from_term(
            system.variables.clone(),
            num_rational::BigRational::from_integer(num_bigint::BigInt::from(
                coefficient.coefficient,
            )),
            Monomial {
                exponents: variable_exponents,
            },
        ));
    }
    let constant = PolynomialQ::from_term(
        system.variables.clone(),
        -num_rational::BigRational::from_integer(num_bigint::BigInt::from(equation.constant)),
        Monomial {
            exponents: vec![0; system.variables.len()],
        },
    );
    polynomial.add(&constant)
}

fn scheduled_slice_supports(system: &CertifiedSystemQ, target_degree: usize) -> Vec<Vec<Monomial>> {
    let support = monomials_up_to_degree(system.variables.len(), target_degree);
    vec![support; system.equations.len()]
}

fn modular_coefficients_for_prime(candidate: &TargetCandidate, prime: u64) -> Option<Vec<u64>> {
    candidate
        .support_mod_primes
        .iter()
        .find(|support| support.modulus == prime)
        .map(|support| support.coefficients.clone())
}

fn slice_input_is_prime_admissible(
    system: &CertifiedSystemQ,
    affine_equations: &[SliceAffineEquation],
    modulus: PrimeModulus,
) -> bool {
    !affine_equations.is_empty()
        && system
            .equations
            .iter()
            .all(|polynomial| polynomial_coefficients_admissible(polynomial, modulus))
        && system
            .semantic_guards
            .iter()
            .all(|record| polynomial_coefficients_admissible(&record.polynomial, modulus))
        && system
            .guard_certificates
            .iter()
            .all(|certificate| guard_certificate_admissible(certificate, modulus))
        && affine_equations
            .iter()
            .all(|equation| equation.denominator_admissible)
}

fn polynomial_coefficients_admissible(polynomial: &PolynomialQ, modulus: PrimeModulus) -> bool {
    polynomial
        .terms
        .values()
        .all(|coefficient| rational_to_mod_prime(coefficient, modulus).is_some())
}

fn guard_certificate_admissible(certificate: &GuardCertificate, modulus: PrimeModulus) -> bool {
    match certificate {
        GuardCertificate::InputSemanticNonzero { guard, record } => {
            polynomial_coefficients_admissible(guard, modulus)
                && polynomial_coefficients_admissible(&record.polynomial, modulus)
        }
        GuardCertificate::AlgebraicNonvanishing { guard, certificate } => {
            polynomial_coefficients_admissible(guard, modulus)
                && certificate
                    .multipliers
                    .iter()
                    .all(|polynomial| polynomial_coefficients_admissible(polynomial, modulus))
                && polynomial_coefficients_admissible(&certificate.guard_multiplier, modulus)
        }
        GuardCertificate::RealAdmissibleNonvanishing { guard, .. } => {
            polynomial_coefficients_admissible(guard, modulus)
        }
        GuardCertificate::DerivedProduct {
            product, factors, ..
        } => {
            polynomial_coefficients_admissible(product, modulus)
                && factors
                    .iter()
                    .all(|factor| guard_certificate_admissible(factor, modulus))
        }
    }
}

fn deterministic_affine_slice(
    variable_indices: &[usize],
    slice_index: usize,
    modulus: PrimeModulus,
) -> Vec<SliceAffineEquation> {
    if variable_indices.is_empty() {
        return Vec::new();
    }
    for seed in 0..modulus.value() {
        let matrix = affine_coefficient_matrix(variable_indices.len(), slice_index, seed, modulus);
        let determinant = determinant_mod(&matrix, modulus);
        if determinant == 0 {
            continue;
        }
        let equations = matrix
            .into_iter()
            .enumerate()
            .map(|(row_index, row)| {
                let denominator_admissible = affine_row_denominator_admissible(&row, modulus);
                SliceAffineEquation {
                    coefficients: row
                        .into_iter()
                        .zip(variable_indices)
                        .filter_map(|(coefficient, variable_index)| {
                            (coefficient != 0).then_some(SliceAffineCoefficient {
                                variable_index: *variable_index,
                                coefficient,
                            })
                        })
                        .collect(),
                    constant: ((slice_index + row_index + seed as usize + 1) as u64)
                        % modulus.value(),
                    denominator_admissible,
                }
            })
            .collect::<Vec<_>>();
        if affine_slice_is_prime_admissible(&equations, determinant, modulus) {
            return equations;
        }
    }
    Vec::new()
}

fn affine_row_denominator_admissible(row: &[u64], modulus: PrimeModulus) -> bool {
    row.iter()
        .all(|coefficient| modulus.normalize(*coefficient) == *coefficient % modulus.value())
}

fn affine_slice_is_prime_admissible(
    equations: &[SliceAffineEquation],
    determinant: u64,
    modulus: PrimeModulus,
) -> bool {
    determinant % modulus.value() != 0
        && !equations.is_empty()
        && equations.iter().all(|equation| {
            equation.denominator_admissible
                && equation.constant < modulus.value()
                && !equation.coefficients.is_empty()
                && equation
                    .coefficients
                    .iter()
                    .all(|coefficient| coefficient.coefficient < modulus.value())
        })
}

fn affine_coefficient_matrix(
    variable_count: usize,
    slice_index: usize,
    seed: u64,
    modulus: PrimeModulus,
) -> Vec<Vec<u64>> {
    (0..variable_count)
        .map(|row| {
            (0..variable_count)
                .map(|column| {
                    let raw =
                        seed + slice_index as u64 + ((row + 1) as u64 * (column + 2) as u64) + 1;
                    let value = raw % modulus.value();
                    if value == 0 {
                        1
                    } else {
                        value
                    }
                })
                .collect()
        })
        .collect()
}

fn determinant_mod(matrix: &[Vec<u64>], modulus: PrimeModulus) -> u64 {
    let size = matrix.len();
    if size == 0 {
        return 1;
    }
    let mut work = matrix.to_vec();
    let mut determinant = 1;
    for pivot_col in 0..size {
        let Some(pivot_row) = (pivot_col..size).find(|row| work[*row][pivot_col] != 0) else {
            return 0;
        };
        if pivot_row != pivot_col {
            work.swap(pivot_row, pivot_col);
            determinant = modulus.neg(determinant);
        }
        let pivot = work[pivot_col][pivot_col];
        determinant = modulus.mul(determinant, pivot);
        let Some(inverse) = modulus.inv(pivot) else {
            return 0;
        };
        for row in pivot_col + 1..size {
            if work[row][pivot_col] == 0 {
                continue;
            }
            let factor = modulus.mul(work[row][pivot_col], inverse);
            for column in pivot_col..size {
                work[row][column] = modulus.sub(
                    work[row][column],
                    modulus.mul(factor, work[pivot_col][column]),
                );
            }
        }
    }
    determinant
}

fn coordinate_assignments_for_compat_key(
    affine_equations: &[SliceAffineEquation],
) -> Vec<SliceAssignment> {
    affine_equations
        .iter()
        .filter_map(|equation| {
            (equation.coefficients.len() == 1).then_some(SliceAssignment {
                variable_index: equation.coefficients[0].variable_index,
                value: equation.constant,
            })
        })
        .collect()
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

#[cfg(test)]
mod tests {
    use num_bigint::BigInt;
    use num_rational::BigRational;

    use super::*;
    use crate::compression::CompressionReplayCertificate;
    use crate::{Monomial, PolynomialQ, Rational, Variable};

    fn variable(symbol: &str) -> Variable {
        Variable {
            symbol: symbol.to_string(),
        }
    }

    fn rational(value: i64) -> Rational {
        BigRational::from_integer(BigInt::from(value))
    }

    fn rational_fraction(numerator: i64, denominator: i64) -> Rational {
        BigRational::new(BigInt::from(numerator), BigInt::from(denominator))
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

    fn polynomial_q(variables: &[Variable], terms: &[(Rational, Vec<u32>)]) -> PolynomialQ {
        terms
            .iter()
            .fold(PolynomialQ::zero(variables.to_vec()), |sum, entry| {
                sum.add(&PolynomialQ::from_term(
                    variables.to_vec(),
                    entry.0.clone(),
                    monomial(&entry.1),
                ))
            })
    }

    #[test]
    fn slice_route_records_affine_slice_candidate_only() {
        let x = variable("X");
        let y = variable("Y");
        let t = variable("T");
        let variables = vec![x.clone(), y.clone(), t.clone()];
        let system = CertifiedSystemQ {
            equations: vec![polynomial(
                &variables,
                &[(1, vec![0, 0, 2]), (-2, vec![0, 0, 0])],
            )],
            variables,
            target: t.clone(),
            semantic_guards: Vec::new(),
            guard_certificates: Vec::new(),
            replay: CompressionReplayCertificate { steps: Vec::new() },
        };

        let window = make_row_closed_certificate_window(
            &system,
            2,
            vec![vec![monomial(&[0, 0, 0])]; system.equations.len()],
        );

        let candidates = slice_specialization_candidates(&system, &window, &[5], 1);

        assert!(candidates.iter().any(|candidate| {
            candidate.origin == CandidateOrigin::SliceSpecialization
                && candidate
                    .support_mod_primes
                    .iter()
                    .any(|support| support.variable == t && support.coefficients.len() > 1)
                && matches!(
                    candidate.traces.first(),
                    Some(CandidateTrace::SliceWitness(trace))
                        if trace.prime == 5
                            && trace.equation_indices.len() == 3
                            && trace.internal_origin == CandidateOrigin::ResidualCyclic
                            && trace.assignments.is_empty()
                            && trace.affine_equations.len() == 2
                            && trace.affine_equations.iter().all(|equation| equation.denominator_admissible)
                            && trace.affine_equations.iter().any(|equation| equation.coefficients.len() > 1)
                )
        }));
    }

    #[test]
    fn affine_slice_admissibility_rejects_singular_or_bad_denominator_trace() {
        let modulus = PrimeModulus::new(5).unwrap();
        let singular_equations = vec![
            SliceAffineEquation {
                coefficients: vec![
                    SliceAffineCoefficient {
                        variable_index: 0,
                        coefficient: 1,
                    },
                    SliceAffineCoefficient {
                        variable_index: 1,
                        coefficient: 1,
                    },
                ],
                constant: 1,
                denominator_admissible: true,
            },
            SliceAffineEquation {
                coefficients: vec![
                    SliceAffineCoefficient {
                        variable_index: 0,
                        coefficient: 2,
                    },
                    SliceAffineCoefficient {
                        variable_index: 1,
                        coefficient: 2,
                    },
                ],
                constant: 2,
                denominator_admissible: true,
            },
        ];
        assert!(!affine_slice_is_prime_admissible(
            &singular_equations,
            0,
            modulus
        ));

        let bad_denominator = vec![SliceAffineEquation {
            coefficients: vec![SliceAffineCoefficient {
                variable_index: 0,
                coefficient: 1,
            }],
            constant: 1,
            denominator_admissible: false,
        }];
        assert!(!affine_slice_is_prime_admissible(
            &bad_denominator,
            1,
            modulus
        ));
    }

    #[test]
    fn slice_route_rejects_prime_with_input_denominator_obstruction() {
        let x = variable("X");
        let t = variable("T");
        let variables = vec![x.clone(), t.clone()];
        let system = CertifiedSystemQ {
            equations: vec![polynomial_q(
                &variables,
                &[
                    (rational_fraction(1, 5), vec![1, 0]),
                    (rational(1), vec![0, 2]),
                ],
            )],
            variables,
            target: t,
            semantic_guards: Vec::new(),
            guard_certificates: Vec::new(),
            replay: CompressionReplayCertificate { steps: Vec::new() },
        };
        let window = make_row_closed_certificate_window(
            &system,
            2,
            vec![vec![monomial(&[0, 0])]; system.equations.len()],
        );

        let candidates = slice_specialization_candidates(&system, &window, &[5], 1);

        assert!(candidates.is_empty());
    }
}
