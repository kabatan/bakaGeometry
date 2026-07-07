use crate::candidates::{
    CandidateOracle, CandidateOrigin, CandidateTrace, SliceAssignment, SliceWitnessTrace,
    TargetCandidate,
};
use crate::compression::CertifiedSystemQ;
use crate::finite_field::{rational_to_mod_prime, PrimeModulus};
use crate::univariate::UniPolynomialFp;
use crate::window::CertificateWindow;

pub(crate) struct SliceSpecializationOracle {
    pub primes: Vec<u64>,
    pub slice_count: usize,
}

impl CandidateOracle for SliceSpecializationOracle {
    fn generate(
        &self,
        system: &CertifiedSystemQ,
        _window: &CertificateWindow,
    ) -> Vec<TargetCandidate> {
        slice_specialization_candidates(system, &self.primes, self.slice_count)
    }
}

pub(crate) fn slice_specialization_candidates(
    system: &CertifiedSystemQ,
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
            let assignments = deterministic_assignments(&sliced_variables, slice_index, modulus);
            for (equation_index, equation) in system.equations.iter().enumerate() {
                let Some(coefficients) =
                    sliced_target_coefficients(equation, target_index, &assignments, modulus)
                else {
                    continue;
                };
                if coefficients.len() <= 1 {
                    continue;
                }
                let Some(normalized) = normalized_coefficients(coefficients, modulus) else {
                    continue;
                };
                let mut support = UniPolynomialFp {
                    variable: system.target.clone(),
                    modulus: *prime,
                    coefficients: normalized.clone(),
                };
                support.normalize();
                if support.is_zero() {
                    continue;
                }
                candidates.push(TargetCandidate {
                    support_mod_primes: vec![support],
                    reconstructed: None,
                    origin: CandidateOrigin::SliceSpecialization,
                    traces: vec![CandidateTrace::SliceWitness(SliceWitnessTrace {
                        prime: *prime,
                        assignments: assignments
                            .iter()
                            .map(|(variable_index, value)| SliceAssignment {
                                variable_index: *variable_index,
                                value: *value,
                            })
                            .collect(),
                        equation_index,
                        relation_coefficients: normalized,
                    })],
                });
            }
        }
    }

    candidates
}

fn deterministic_assignments(
    variable_indices: &[usize],
    slice_index: usize,
    modulus: PrimeModulus,
) -> Vec<(usize, u64)> {
    variable_indices
        .iter()
        .enumerate()
        .map(|(offset, variable_index)| {
            let value = ((slice_index + offset + 1) as u64) % modulus.value();
            (*variable_index, value)
        })
        .collect()
}

fn sliced_target_coefficients(
    polynomial: &crate::PolynomialQ,
    target_index: usize,
    assignments: &[(usize, u64)],
    modulus: PrimeModulus,
) -> Option<Vec<u64>> {
    let mut coefficients = Vec::new();
    for (monomial, coefficient) in &polynomial.terms {
        let mut scale = rational_to_mod_prime(coefficient, modulus)?;
        for (variable_index, value) in assignments {
            scale = modulus.mul(
                scale,
                pow_mod(*value, monomial.exponents[*variable_index], modulus),
            );
        }
        let degree = monomial.exponents[target_index] as usize;
        if coefficients.len() <= degree {
            coefficients.resize(degree + 1, 0);
        }
        coefficients[degree] = modulus.add(coefficients[degree], scale);
    }
    while coefficients
        .last()
        .is_some_and(|coefficient| *coefficient == 0)
    {
        coefficients.pop();
    }
    Some(coefficients)
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

fn pow_mod(mut base: u64, mut exponent: u32, modulus: PrimeModulus) -> u64 {
    let mut result = 1;
    while exponent > 0 {
        if exponent % 2 == 1 {
            result = modulus.mul(result, base);
        }
        base = modulus.mul(base, base);
        exponent /= 2;
    }
    result
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
    fn slice_route_records_affine_slice_candidate_only() {
        let x = variable("X");
        let t = variable("T");
        let variables = vec![x.clone(), t.clone()];
        let system = CertifiedSystemQ {
            equations: vec![polynomial(
                &variables,
                &[(1, vec![0, 2]), (1, vec![1, 0]), (-2, vec![0, 0])],
            )],
            variables,
            target: t.clone(),
            guard_certificates: Vec::new(),
            replay: CompressionReplayCertificate { steps: Vec::new() },
        };

        let candidates = slice_specialization_candidates(&system, &[5], 1);

        assert!(candidates.iter().any(|candidate| {
            candidate.origin == CandidateOrigin::SliceSpecialization
                && candidate
                    .support_mod_primes
                    .iter()
                    .any(|support| support.variable == t && support.coefficients == vec![4, 0, 1])
                && matches!(
                    candidate.traces.first(),
                    Some(CandidateTrace::SliceWitness(trace)) if trace.prime == 5
                )
        }));
    }
}
