use std::collections::{BTreeMap, BTreeSet};

use crate::compression::CertifiedSystemQ;
use crate::finite_field::{rational_to_mod_prime, PrimeModulus};
use crate::{Monomial, Rational};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CertificateWindow {
    pub target_degree: usize,
    pub multiplier_supports: Vec<Vec<Monomial>>,
    pub row_monomials: Vec<Monomial>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProofWindow {
    pub multiplier_supports: Vec<Vec<Monomial>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MembershipMatrixQ {
    pub row_monomials: Vec<Monomial>,
    columns: Vec<Vec<Rational>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetPowerMatrixQ {
    pub row_monomials: Vec<Monomial>,
    columns: Vec<Vec<Rational>>,
}

impl CertificateWindow {
    pub fn row_index(&self, monomial: &Monomial) -> Option<usize> {
        self.row_monomials
            .iter()
            .position(|candidate| candidate == monomial)
    }
}

impl MembershipMatrixQ {
    pub fn row_count(&self) -> usize {
        self.row_monomials.len()
    }

    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    pub fn column(&self, index: usize) -> &[Rational] {
        &self.columns[index]
    }

    pub(crate) fn to_modular_columns(&self, modulus: PrimeModulus) -> Option<Vec<Vec<u64>>> {
        self.columns
            .iter()
            .map(|column| {
                column
                    .iter()
                    .map(|coefficient| rational_to_mod_prime(coefficient, modulus))
                    .collect()
            })
            .collect()
    }
}

impl TargetPowerMatrixQ {
    pub fn row_count(&self) -> usize {
        self.row_monomials.len()
    }

    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    pub fn column(&self, index: usize) -> &[Rational] {
        &self.columns[index]
    }

    pub(crate) fn to_modular_columns(&self, modulus: PrimeModulus) -> Option<Vec<Vec<u64>>> {
        self.columns
            .iter()
            .map(|column| {
                column
                    .iter()
                    .map(|coefficient| rational_to_mod_prime(coefficient, modulus))
                    .collect()
            })
            .collect()
    }
}

pub fn make_row_closed_certificate_window(
    system: &CertifiedSystemQ,
    target_degree: usize,
    multiplier_supports: Vec<Vec<Monomial>>,
) -> CertificateWindow {
    assert_eq!(multiplier_supports.len(), system.equations.len());
    assert!(multiplier_supports
        .iter()
        .flatten()
        .all(|monomial| monomial.exponents.len() == system.variables.len()));

    let target_index = target_index(system);
    let mut rows = BTreeSet::new();
    for degree in 0..=target_degree {
        rows.insert(target_power_monomial(
            system.variables.len(),
            target_index,
            degree,
        ));
    }

    for (equation, supports) in system.equations.iter().zip(&multiplier_supports) {
        assert_eq!(equation.variables, system.variables);
        for multiplier_monomial in supports {
            for equation_monomial in equation.support() {
                rows.insert(multiplier_monomial.multiply(&equation_monomial));
            }
        }
    }

    let mut row_monomials = rows.into_iter().collect::<Vec<_>>();
    row_monomials.sort_by_key(canonical_monomial_key);

    CertificateWindow {
        target_degree,
        multiplier_supports,
        row_monomials,
    }
}

pub fn build_membership_matrix_q(
    system: &CertifiedSystemQ,
    window: &CertificateWindow,
) -> MembershipMatrixQ {
    let canonical_window = make_row_closed_certificate_window(
        system,
        window.target_degree,
        window.multiplier_supports.clone(),
    );
    let row_index = row_index_map(&canonical_window.row_monomials);
    let mut columns = Vec::new();

    for (equation, supports) in system
        .equations
        .iter()
        .zip(&canonical_window.multiplier_supports)
    {
        assert_eq!(equation.variables, system.variables);
        for multiplier_monomial in supports {
            let mut column =
                vec![crate::arith::rational_zero(); canonical_window.row_monomials.len()];
            for (equation_monomial, coefficient) in &equation.terms {
                let product_monomial = multiplier_monomial.multiply(equation_monomial);
                if let Some(row) = row_index.get(&product_monomial) {
                    column[*row] += coefficient.clone();
                }
            }
            columns.push(column);
        }
    }

    MembershipMatrixQ {
        row_monomials: canonical_window.row_monomials,
        columns,
    }
}

pub fn build_target_power_matrix_q(
    system: &CertifiedSystemQ,
    window: &CertificateWindow,
) -> TargetPowerMatrixQ {
    let canonical_window = make_row_closed_certificate_window(
        system,
        window.target_degree,
        window.multiplier_supports.clone(),
    );
    let target_index = target_index(system);
    let row_index = row_index_map(&canonical_window.row_monomials);
    let mut columns = Vec::new();

    for degree in 0..=canonical_window.target_degree {
        let mut column = vec![crate::arith::rational_zero(); canonical_window.row_monomials.len()];
        let monomial = target_power_monomial(system.variables.len(), target_index, degree);
        if let Some(row) = row_index.get(&monomial) {
            column[*row] = crate::arith::rational_one();
        }
        columns.push(column);
    }

    TargetPowerMatrixQ {
        row_monomials: canonical_window.row_monomials,
        columns,
    }
}

fn target_index(system: &CertifiedSystemQ) -> usize {
    system
        .variables
        .iter()
        .position(|variable| variable == &system.target)
        .expect("target variable must be present")
}

fn target_power_monomial(variable_count: usize, target_index: usize, degree: usize) -> Monomial {
    let mut exponents = vec![0; variable_count];
    exponents[target_index] = degree as u32;
    Monomial { exponents }
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
    use crate::{PolynomialQ, Rational, Variable};

    fn variable(symbol: &str) -> Variable {
        Variable {
            symbol: symbol.to_string(),
        }
    }

    fn rational(value: i64) -> Rational {
        BigRational::from_integer(BigInt::from(value))
    }

    fn fraction(numerator: i64, denominator: i64) -> Rational {
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
        terms.iter().fold(
            PolynomialQ::zero(variables.to_vec()),
            |accumulator, entry| accumulator.add(&term(variables, entry.0, &entry.1)),
        )
    }

    fn system(
        equations: Vec<PolynomialQ>,
        variables: Vec<Variable>,
        target: Variable,
    ) -> CertifiedSystemQ {
        CertifiedSystemQ {
            equations,
            variables,
            target,
            guard_certificates: Vec::new(),
            replay: CompressionReplayCertificate { steps: Vec::new() },
        }
    }

    #[test]
    fn row_closed_window_recomputes_target_powers_and_multiplier_products() {
        let x = variable("X");
        let t = variable("T");
        let variables = vec![x.clone(), t.clone()];
        let equation = polynomial(&variables, &[(1, vec![2, 0]), (-2, vec![0, 0])]);
        let one = monomial(&[0, 0]);
        let x_support = monomial(&[1, 0]);
        let sys = system(vec![equation], variables, t);

        let window =
            make_row_closed_certificate_window(&sys, 2, vec![vec![one.clone(), x_support]]);

        assert!(window.row_monomials.contains(&monomial(&[0, 0])));
        assert!(window.row_monomials.contains(&monomial(&[0, 1])));
        assert!(window.row_monomials.contains(&monomial(&[0, 2])));
        assert!(window.row_monomials.contains(&monomial(&[2, 0])));
        assert!(window.row_monomials.contains(&monomial(&[3, 0])));

        let membership = build_membership_matrix_q(&sys, &window);
        assert_eq!(membership.row_monomials, window.row_monomials);
        assert_eq!(membership.column_count(), 2);

        let powers = build_target_power_matrix_q(&sys, &window);
        assert_eq!(powers.column_count(), 3);
        assert_eq!(
            powers.column(2)[window.row_index(&monomial(&[0, 2])).unwrap()],
            rational(1)
        );

        let mut forged = window.clone();
        forged.row_monomials.clear();
        assert_eq!(
            build_membership_matrix_q(&sys, &forged).row_monomials,
            window.row_monomials
        );
        assert_eq!(
            build_target_power_matrix_q(&sys, &forged).row_monomials,
            window.row_monomials
        );
    }

    #[test]
    fn modular_matrix_reduction_filters_denominator_bad_primes() {
        let t = variable("T");
        let variables = vec![t.clone()];
        let equation = PolynomialQ::from_term(variables.clone(), fraction(1, 2), monomial(&[1]));
        let sys = system(vec![equation], variables, t);
        let window = make_row_closed_certificate_window(&sys, 1, vec![vec![monomial(&[0])]]);
        let membership = build_membership_matrix_q(&sys, &window);

        assert!(membership
            .to_modular_columns(crate::finite_field::PrimeModulus::new(2).unwrap())
            .is_none());
        assert!(membership
            .to_modular_columns(crate::finite_field::PrimeModulus::new(3).unwrap())
            .is_some());
    }
}
