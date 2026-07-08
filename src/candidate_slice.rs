use crate::candidates::{
    CandidateOracle, CandidateOrigin, CandidateTrace, SliceAssignment, SliceWitnessTrace,
    TargetCandidate,
};
use crate::compression::CertifiedSystemQ;
use crate::finite_field::PrimeModulus;
use crate::window::{make_row_closed_certificate_window, CertificateWindow};
use crate::{Monomial, PolynomialQ};

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
            let assignments = deterministic_assignments(&sliced_variables, slice_index, modulus);
            let sliced_system = build_sliced_system(system, &assignments);
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
                        assignments: assignments
                            .iter()
                            .map(|(variable_index, value)| SliceAssignment {
                                variable_index: *variable_index,
                                value: *value,
                            })
                            .collect(),
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
    assignments: &[(usize, u64)],
) -> CertifiedSystemQ {
    let mut equations = system.equations.clone();
    equations.extend(
        assignments
            .iter()
            .map(|(variable_index, value)| slice_equation(system, *variable_index, *value)),
    );
    CertifiedSystemQ {
        equations,
        variables: system.variables.clone(),
        target: system.target.clone(),
        guard_certificates: system.guard_certificates.clone(),
        replay: system.replay.clone(),
    }
}

fn slice_equation(system: &CertifiedSystemQ, variable_index: usize, value: u64) -> PolynomialQ {
    let mut variable_exponents = vec![0; system.variables.len()];
    variable_exponents[variable_index] = 1;
    let variable_term = PolynomialQ::from_term(
        system.variables.clone(),
        crate::arith::rational_one(),
        Monomial {
            exponents: variable_exponents,
        },
    );
    let constant = PolynomialQ::from_term(
        system.variables.clone(),
        -num_rational::BigRational::from_integer(num_bigint::BigInt::from(value)),
        Monomial {
            exponents: vec![0; system.variables.len()],
        },
    );
    variable_term.add(&constant)
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

        let window = make_row_closed_certificate_window(
            &system,
            2,
            vec![vec![monomial(&[0, 0])]; system.equations.len()],
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
                            && trace.equation_indices.len() == 2
                            && trace.internal_origin == CandidateOrigin::ResidualCyclic
                )
        }));
    }
}
