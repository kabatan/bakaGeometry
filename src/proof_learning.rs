use std::collections::BTreeSet;

use num_traits::Zero;

use crate::candidates::CandidateTrace;
use crate::compression::CertifiedSystemQ;
use crate::window::{CertificateWindow, ProofWindow};
use crate::{Monomial, Rational};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LeftNullObstruction {
    pub row_monomials: Vec<Monomial>,
    pub coefficients: Vec<Rational>,
}

pub(crate) fn learn_initial_proof_window(
    window: &CertificateWindow,
    traces: &[CandidateTrace],
) -> ProofWindow {
    let mut supports = window
        .multiplier_supports
        .iter()
        .map(|support| support.iter().cloned().collect::<BTreeSet<_>>())
        .collect::<Vec<_>>();

    for trace in traces {
        let CandidateTrace::ModularWitness(witness) = trace else {
            continue;
        };
        if supports.len() < witness.active_multiplier_supports.len() {
            supports.resize_with(witness.active_multiplier_supports.len(), BTreeSet::new);
        }
        for (target_support, active_support) in
            supports.iter_mut().zip(&witness.active_multiplier_supports)
        {
            target_support.extend(active_support.iter().cloned());
        }
    }

    ProofWindow {
        multiplier_supports: supports
            .into_iter()
            .map(|support| support.into_iter().collect())
            .collect(),
    }
}

pub(crate) fn expand_by_obstruction_predecessors(
    system: &CertifiedSystemQ,
    proof_window: &ProofWindow,
    obstruction: &LeftNullObstruction,
) -> ProofWindow {
    let mut supports = support_sets_for_system(system, proof_window);

    for (row_monomial, coefficient) in obstruction
        .row_monomials
        .iter()
        .zip(&obstruction.coefficients)
    {
        if coefficient.is_zero() {
            continue;
        }
        for (equation_index, equation) in system.equations.iter().enumerate() {
            for equation_monomial in equation.support() {
                if let Some(predecessor) = row_monomial.quotient_if_divisible_by(&equation_monomial)
                {
                    supports[equation_index].insert(predecessor);
                }
            }
        }
    }

    ProofWindow {
        multiplier_supports: supports
            .into_iter()
            .map(|support| support.into_iter().collect())
            .collect(),
    }
}

pub(crate) fn expand_by_total_degree(
    system: &CertifiedSystemQ,
    proof_window: &ProofWindow,
    total_degree: usize,
) -> ProofWindow {
    let mut supports = support_sets_for_system(system, proof_window);
    let degree_support = monomials_up_to_total_degree(system.variables.len(), total_degree);
    for support in &mut supports {
        support.extend(degree_support.iter().cloned());
    }
    proof_window_from_sets(supports)
}

pub(crate) struct ProofWindowDegreeSchedule<'a> {
    system: &'a CertifiedSystemQ,
    seed: ProofWindow,
    next_degree: usize,
}

impl<'a> ProofWindowDegreeSchedule<'a> {
    pub(crate) fn new(system: &'a CertifiedSystemQ, seed: ProofWindow) -> Self {
        Self {
            system,
            seed,
            next_degree: 0,
        }
    }
}

impl Iterator for ProofWindowDegreeSchedule<'_> {
    type Item = ProofWindow;

    fn next(&mut self) -> Option<Self::Item> {
        let window = expand_by_total_degree(self.system, &self.seed, self.next_degree);
        self.next_degree += 1;
        Some(window)
    }
}

fn support_sets_for_system(
    system: &CertifiedSystemQ,
    proof_window: &ProofWindow,
) -> Vec<BTreeSet<Monomial>> {
    (0..system.equations.len())
        .map(|index| {
            proof_window
                .multiplier_supports
                .get(index)
                .into_iter()
                .flatten()
                .cloned()
                .collect::<BTreeSet<_>>()
        })
        .collect()
}

fn proof_window_from_sets(supports: Vec<BTreeSet<Monomial>>) -> ProofWindow {
    ProofWindow {
        multiplier_supports: supports
            .into_iter()
            .map(|support| support.into_iter().collect())
            .collect(),
    }
}

fn monomials_up_to_total_degree(variable_count: usize, max_degree: usize) -> Vec<Monomial> {
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
    use crate::candidates::ModularWitnessTrace;
    use crate::compression::CompressionReplayCertificate;
    use crate::{PolynomialQ, Variable};

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
    fn obstruction_expansion_adds_predecessor_support() {
        let x = variable("X");
        let t = variable("T");
        let variables = vec![x.clone(), t.clone()];
        let system = CertifiedSystemQ {
            equations: vec![polynomial(&variables, &[(1, vec![2, 0]), (-2, vec![0, 0])])],
            variables,
            target: t,
            guard_certificates: Vec::new(),
            replay: CompressionReplayCertificate { steps: Vec::new() },
        };
        let proof_window = ProofWindow {
            multiplier_supports: vec![Vec::new()],
        };
        let obstruction = LeftNullObstruction {
            row_monomials: vec![monomial(&[3, 0])],
            coefficients: vec![rational(1)],
        };

        let expanded = expand_by_obstruction_predecessors(&system, &proof_window, &obstruction);

        assert!(expanded.multiplier_supports[0].contains(&monomial(&[1, 0])));
    }

    #[test]
    fn modular_witness_support_seeds_initial_proof_window() {
        let window = CertificateWindow {
            target_degree: 2,
            multiplier_supports: vec![vec![monomial(&[0, 0])]],
            row_monomials: Vec::new(),
        };
        let traces = vec![CandidateTrace::ModularWitness(ModularWitnessTrace {
            prime: 5,
            active_multiplier_supports: vec![vec![monomial(&[1, 0])]],
            relation_coefficients: vec![3, 0, 1],
            residual_vectors: Vec::new(),
        })];

        let learned = learn_initial_proof_window(&window, &traces);

        assert!(learned.multiplier_supports[0].contains(&monomial(&[0, 0])));
        assert!(learned.multiplier_supports[0].contains(&monomial(&[1, 0])));
    }

    #[test]
    fn degree_schedule_reaches_larger_multiplier_support() {
        let x = variable("X");
        let t = variable("T");
        let variables = vec![x.clone(), t.clone()];
        let system = CertifiedSystemQ {
            equations: vec![
                polynomial(&variables, &[(1, vec![1, 0])]),
                polynomial(&variables, &[(1, vec![0, 1])]),
            ],
            variables,
            target: t,
            guard_certificates: Vec::new(),
            replay: CompressionReplayCertificate { steps: Vec::new() },
        };
        let seed = ProofWindow {
            multiplier_supports: vec![Vec::new(), Vec::new()],
        };

        let windows = ProofWindowDegreeSchedule::new(&system, seed)
            .take(3)
            .collect::<Vec<_>>();

        for supports in &windows[2].multiplier_supports {
            assert!(supports.contains(&monomial(&[0, 0])));
            assert!(supports.contains(&monomial(&[2, 0])));
            assert!(supports.contains(&monomial(&[1, 1])));
            assert!(supports.contains(&monomial(&[0, 2])));
        }
    }
}
