use crate::candidates::{CandidateOracle, CandidateOrigin, CandidateTrace, TargetCandidate};
use crate::compression::CertifiedSystemQ;
use crate::window::CertificateWindow;

pub(crate) struct DirectTargetEquationOracle;

impl CandidateOracle for DirectTargetEquationOracle {
    fn generate(
        &self,
        system: &CertifiedSystemQ,
        _window: &CertificateWindow,
    ) -> Vec<TargetCandidate> {
        direct_target_equation_candidates(system)
    }
}

pub(crate) fn direct_target_equation_candidates(system: &CertifiedSystemQ) -> Vec<TargetCandidate> {
    system
        .equations
        .iter()
        .enumerate()
        .filter_map(|(equation_index, equation)| {
            if equation.is_zero() || !equation.depends_only_on(std::slice::from_ref(&system.target))
            {
                return None;
            }
            let reconstructed = equation
                .to_univariate_in(&system.target)?
                .primitive_integer_normalized();
            (!reconstructed.is_zero()).then_some(TargetCandidate {
                support_mod_primes: Vec::new(),
                reconstructed: Some(reconstructed),
                origin: CandidateOrigin::DirectTargetEquation,
                traces: vec![CandidateTrace::DirectEquation { equation_index }],
            })
        })
        .collect()
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
        terms.iter().fold(
            PolynomialQ::zero(variables.to_vec()),
            |accumulator, entry| accumulator.add(&term(variables, entry.0, &entry.1)),
        )
    }

    #[test]
    fn direct_route_uses_target_only_structure() {
        let t = variable("T");
        let x = variable("X");
        let variables = vec![t.clone(), x.clone()];
        let system = CertifiedSystemQ {
            equations: vec![
                polynomial(&variables, &[(2, vec![2, 0]), (-4, vec![0, 0])]),
                polynomial(&variables, &[(1, vec![0, 1]), (1, vec![1, 0])]),
            ],
            variables,
            target: t.clone(),
            guard_certificates: Vec::new(),
            replay: CompressionReplayCertificate { steps: Vec::new() },
        };

        let candidates = direct_target_equation_candidates(&system);

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].origin, CandidateOrigin::DirectTargetEquation);
        assert_eq!(
            candidates[0].reconstructed.as_ref().unwrap().coefficients,
            vec![rational(-2), rational(0), rational(1)]
        );
    }
}
