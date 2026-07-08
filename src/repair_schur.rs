#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SchurRepairScope {
    pub equation_indices: Vec<usize>,
}

use std::collections::BTreeSet;

use num_traits::Zero;

use crate::compression::CertifiedSystemQ;
use crate::linear_q::nullspace_matrix_q;
use crate::proof::{prove_fixed_target, CertificateMode, FixedProofInput};
use crate::proof_learning::LeftNullObstruction;
use crate::window::ProofWindow;
use crate::{Monomial, Rational, ResourceLimits, TargetCertificate, UniPolynomialQ, Variable};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LocalMembershipEquation {
    pub scope: SchurRepairScope,
    pub boundary_variables: Vec<Variable>,
    pub boundary_support: Vec<Monomial>,
    pub row_monomials: Vec<Monomial>,
    pub multiplier_columns: Vec<Vec<Rational>>,
    pub boundary_columns: Vec<Vec<Rational>>,
}

#[derive(Clone, Debug)]
pub(crate) enum SchurRepairOutput {
    Certified(TargetCertificate),
    SupportInformation(SchurSupportInformation),
    NoLocalScope,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SchurSupportInformation {
    pub scope: SchurRepairScope,
    pub boundary_variables: Vec<Variable>,
    pub boundary_support: Vec<Monomial>,
    pub multiplier_supports: Vec<Vec<Monomial>>,
    pub local_membership: LocalMembershipEquation,
}

pub(crate) fn localized_schur_repair(
    system: &CertifiedSystemQ,
    proof_window: &ProofWindow,
    obstructions: &[LeftNullObstruction],
    limits: &ResourceLimits,
) -> SchurRepairOutput {
    let Some(scope) = obstruction_scope(system, obstructions) else {
        return SchurRepairOutput::NoLocalScope;
    };
    if scope.equation_indices.len() >= system.equations.len() {
        return SchurRepairOutput::NoLocalScope;
    }

    let boundary_variables = boundary_variables(system, &scope);
    let Some(degree) = limits.max_window_degree else {
        return SchurRepairOutput::NoLocalScope;
    };
    let boundary_support = boundary_monomials(system, &boundary_variables, degree);
    let local_membership =
        local_membership_equation(system, proof_window, &scope, &boundary_variables, degree);
    let multiplier_supports =
        localized_predecessor_supports(system, proof_window, obstructions, &scope);

    if let Some(certificate) =
        target_certificate_from_local_relation(system, &multiplier_supports, &local_membership)
    {
        return SchurRepairOutput::Certified(certificate);
    }

    SchurRepairOutput::SupportInformation(SchurSupportInformation {
        scope,
        boundary_variables,
        boundary_support,
        multiplier_supports,
        local_membership,
    })
}

pub(crate) fn obstruction_scope(
    system: &CertifiedSystemQ,
    obstructions: &[LeftNullObstruction],
) -> Option<SchurRepairScope> {
    let mut equation_indices = BTreeSet::new();
    for obstruction in obstructions {
        for (row_monomial, coefficient) in obstruction
            .row_monomials
            .iter()
            .zip(&obstruction.coefficients)
        {
            if coefficient.is_zero() {
                continue;
            }
            for (equation_index, equation) in system.equations.iter().enumerate() {
                if equation
                    .support()
                    .iter()
                    .any(|monomial| row_monomial.is_divisible_by(monomial))
                {
                    equation_indices.insert(equation_index);
                }
            }
        }
    }

    (!equation_indices.is_empty()).then(|| SchurRepairScope {
        equation_indices: equation_indices.into_iter().collect(),
    })
}

pub(crate) fn local_membership_equation(
    system: &CertifiedSystemQ,
    proof_window: &ProofWindow,
    scope: &SchurRepairScope,
    boundary_variables: &[Variable],
    boundary_degree: usize,
) -> LocalMembershipEquation {
    let boundary_support = boundary_monomials(system, boundary_variables, boundary_degree);
    let row_monomials = local_rows(system, proof_window, scope, &boundary_support);
    let row_index = row_index_map(&row_monomials);
    let mut multiplier_columns = Vec::new();

    for equation_index in &scope.equation_indices {
        let equation = &system.equations[*equation_index];
        let supports = proof_window
            .multiplier_supports
            .get(*equation_index)
            .map_or(&[][..], Vec::as_slice);
        for multiplier_monomial in supports {
            let mut column = vec![crate::arith::rational_zero(); row_monomials.len()];
            for (equation_monomial, coefficient) in &equation.terms {
                let product_monomial = multiplier_monomial.multiply(equation_monomial);
                if let Some(row) = row_index.get(&product_monomial) {
                    column[*row] += coefficient.clone();
                }
            }
            multiplier_columns.push(column);
        }
    }

    let boundary_columns = boundary_support
        .iter()
        .map(|monomial| {
            let mut column = vec![crate::arith::rational_zero(); row_monomials.len()];
            if let Some(row) = row_index.get(monomial) {
                column[*row] = crate::arith::rational_one();
            }
            column
        })
        .collect();

    LocalMembershipEquation {
        scope: scope.clone(),
        boundary_variables: boundary_variables.to_vec(),
        boundary_support,
        row_monomials,
        multiplier_columns,
        boundary_columns,
    }
}

fn target_certificate_from_local_relation(
    system: &CertifiedSystemQ,
    multiplier_supports: &[Vec<Monomial>],
    local_membership: &LocalMembershipEquation,
) -> Option<TargetCertificate> {
    let mut columns = local_membership.multiplier_columns.clone();
    columns.extend(local_membership.boundary_columns.clone());
    let matrix = rows_from_columns(&columns);
    let boundary_start = local_membership.multiplier_columns.len();

    for relation in nullspace_matrix_q(&matrix) {
        let Some(support) = target_support_from_boundary_relation(
            system,
            &local_membership.boundary_support,
            &relation[boundary_start..],
        ) else {
            continue;
        };
        if support.degree().is_none_or(|degree| degree == 0) {
            continue;
        }
        let input = FixedProofInput {
            system: system.clone(),
            candidate: support,
            proof_window: ProofWindow {
                multiplier_supports: multiplier_supports.to_vec(),
            },
            certificate_mode: CertificateMode::Ideal,
        };
        if let Ok(certificate) = prove_fixed_target(input) {
            return Some(certificate);
        }
    }
    None
}

fn target_support_from_boundary_relation(
    system: &CertifiedSystemQ,
    boundary_support: &[Monomial],
    coefficients: &[Rational],
) -> Option<UniPolynomialQ> {
    let target_index = system
        .variables
        .iter()
        .position(|variable| variable == &system.target)?;
    let mut support_coefficients = Vec::new();
    for (monomial, coefficient) in boundary_support.iter().zip(coefficients) {
        if coefficient.is_zero() {
            continue;
        }
        if monomial
            .exponents
            .iter()
            .enumerate()
            .any(|(index, exponent)| index != target_index && *exponent != 0)
        {
            return None;
        }
        let degree = monomial.exponents[target_index] as usize;
        if support_coefficients.len() <= degree {
            support_coefficients.resize_with(degree + 1, crate::arith::rational_zero);
        }
        support_coefficients[degree] -= coefficient.clone();
    }
    let mut support = UniPolynomialQ {
        variable: system.target.clone(),
        coefficients: support_coefficients,
    }
    .primitive_integer_normalized();
    support.normalize();
    (!support.is_zero()).then_some(support)
}

fn rows_from_columns(columns: &[Vec<Rational>]) -> Vec<Vec<Rational>> {
    let rows = columns.first().map_or(0, Vec::len);
    (0..rows)
        .map(|row| columns.iter().map(|column| column[row].clone()).collect())
        .collect()
}

fn boundary_variables(system: &CertifiedSystemQ, scope: &SchurRepairScope) -> Vec<Variable> {
    let scoped = scope
        .equation_indices
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let mut variables = vec![system.target.clone()];
    for (variable_index, variable) in system.variables.iter().enumerate() {
        if variable == &system.target {
            continue;
        }
        let inside = scope.equation_indices.iter().any(|equation_index| {
            equation_uses_variable(&system.equations[*equation_index], variable_index)
        });
        let outside = system
            .equations
            .iter()
            .enumerate()
            .any(|(equation_index, equation)| {
                !scoped.contains(&equation_index)
                    && equation_uses_variable(equation, variable_index)
            });
        if inside && outside {
            variables.push(variable.clone());
        }
    }
    variables
}

fn equation_uses_variable(polynomial: &crate::PolynomialQ, variable_index: usize) -> bool {
    polynomial
        .terms
        .keys()
        .any(|monomial| monomial.exponents[variable_index] != 0)
}

fn boundary_monomials(
    system: &CertifiedSystemQ,
    boundary_variables: &[Variable],
    max_degree: usize,
) -> Vec<Monomial> {
    let boundary_indices = boundary_variables
        .iter()
        .filter_map(|variable| {
            system
                .variables
                .iter()
                .position(|candidate| candidate == variable)
        })
        .collect::<Vec<_>>();
    let mut monomials = Vec::new();
    let mut boundary_exponents = vec![0; boundary_indices.len()];
    enumerate_boundary_monomials(
        system.variables.len(),
        &boundary_indices,
        max_degree as u32,
        0,
        &mut boundary_exponents,
        &mut monomials,
    );
    monomials.sort_by_key(canonical_monomial_key);
    monomials
}

fn enumerate_boundary_monomials(
    variable_count: usize,
    boundary_indices: &[usize],
    remaining_degree: u32,
    index: usize,
    boundary_exponents: &mut [u32],
    monomials: &mut Vec<Monomial>,
) {
    if index == boundary_indices.len() {
        let mut exponents = vec![0; variable_count];
        for (boundary_index, exponent) in boundary_indices.iter().zip(boundary_exponents) {
            exponents[*boundary_index] = *exponent;
        }
        monomials.push(Monomial { exponents });
        return;
    }
    for exponent in 0..=remaining_degree {
        boundary_exponents[index] = exponent;
        enumerate_boundary_monomials(
            variable_count,
            boundary_indices,
            remaining_degree - exponent,
            index + 1,
            boundary_exponents,
            monomials,
        );
    }
    boundary_exponents[index] = 0;
}

fn local_rows(
    system: &CertifiedSystemQ,
    proof_window: &ProofWindow,
    scope: &SchurRepairScope,
    boundary_support: &[Monomial],
) -> Vec<Monomial> {
    let mut rows = boundary_support.iter().cloned().collect::<BTreeSet<_>>();
    for equation_index in &scope.equation_indices {
        let equation = &system.equations[*equation_index];
        let supports = proof_window
            .multiplier_supports
            .get(*equation_index)
            .map_or(&[][..], Vec::as_slice);
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

fn localized_predecessor_supports(
    system: &CertifiedSystemQ,
    proof_window: &ProofWindow,
    obstructions: &[LeftNullObstruction],
    scope: &SchurRepairScope,
) -> Vec<Vec<Monomial>> {
    let scoped = scope
        .equation_indices
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let mut supports = (0..system.equations.len())
        .map(|index| {
            proof_window
                .multiplier_supports
                .get(index)
                .into_iter()
                .flatten()
                .cloned()
                .collect::<BTreeSet<_>>()
        })
        .collect::<Vec<_>>();

    for obstruction in obstructions {
        for (row_monomial, coefficient) in obstruction
            .row_monomials
            .iter()
            .zip(&obstruction.coefficients)
        {
            if coefficient.is_zero() {
                continue;
            }
            for equation_index in &scope.equation_indices {
                for equation_monomial in system.equations[*equation_index].support() {
                    if let Some(predecessor) =
                        row_monomial.quotient_if_divisible_by(&equation_monomial)
                    {
                        supports[*equation_index].insert(predecessor);
                    }
                }
            }
        }
    }

    supports
        .into_iter()
        .enumerate()
        .map(|(equation_index, support)| {
            if scoped.contains(&equation_index) {
                support.into_iter().collect()
            } else {
                proof_window
                    .multiplier_supports
                    .get(equation_index)
                    .cloned()
                    .unwrap_or_default()
            }
        })
        .collect()
}

fn row_index_map(row_monomials: &[Monomial]) -> std::collections::BTreeMap<Monomial, usize> {
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
    use crate::{
        Monomial, PolynomialQ, Rational, SolverCertificate, TargetProblemQ, VerificationResult,
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

    fn local_system() -> (CertifiedSystemQ, ProofWindow, LeftNullObstruction) {
        let x = variable("X");
        let y = variable("Y");
        let t = variable("T");
        let variables = vec![x.clone(), y, t.clone()];
        let system = CertifiedSystemQ {
            equations: vec![
                polynomial(&variables, &[(1, vec![1, 0, 0]), (-1, vec![0, 0, 1])]),
                polynomial(&variables, &[(1, vec![0, 1, 0]), (-1, vec![0, 0, 1])]),
                polynomial(&variables, &[(1, vec![0, 2, 0]), (-1, vec![0, 0, 1])]),
            ],
            variables,
            target: t,
            guard_certificates: Vec::new(),
            replay: CompressionReplayCertificate { steps: Vec::new() },
        };
        let proof_window = ProofWindow {
            multiplier_supports: vec![vec![monomial(&[1, 0, 0])], Vec::new(), Vec::new()],
        };
        let obstruction = LeftNullObstruction {
            row_monomials: vec![monomial(&[2, 0, 0])],
            coefficients: vec![rational(1)],
        };
        (system, proof_window, obstruction)
    }

    #[test]
    fn obstruction_scope_uses_incidence_subset() {
        let (system, _proof_window, obstruction) = local_system();

        let scope = obstruction_scope(&system, &[obstruction]).unwrap();

        assert_eq!(scope.equation_indices, vec![0]);
    }

    #[test]
    fn schur_repair_builds_local_membership_only() {
        let (system, proof_window, obstruction) = local_system();
        let limits = ResourceLimits {
            max_window_degree: Some(1),
            max_proof_weight: None,
            max_matrix_rows: None,
            max_matrix_cols: None,
            max_candidate_count: None,
        };

        let output = localized_schur_repair(&system, &proof_window, &[obstruction], &limits);

        let SchurRepairOutput::SupportInformation(info) = output else {
            panic!("local Schur should return support information");
        };
        assert_eq!(info.scope.equation_indices, vec![0]);
        assert_eq!(info.local_membership.scope.equation_indices, vec![0]);
        assert!(info.local_membership.multiplier_columns.len() < system.equations.len());
        assert!(!info.boundary_support.is_empty());
    }

    #[test]
    fn uncertified_schur_relation_is_support_info_only() {
        let (system, proof_window, obstruction) = local_system();
        let limits = ResourceLimits {
            max_window_degree: Some(1),
            max_proof_weight: None,
            max_matrix_rows: None,
            max_matrix_cols: None,
            max_candidate_count: None,
        };

        let output = localized_schur_repair(&system, &proof_window, &[obstruction], &limits);

        assert!(matches!(output, SchurRepairOutput::SupportInformation(_)));
    }

    #[test]
    fn schur_repair_without_window_bound_does_not_use_hidden_capped_search() {
        let (system, proof_window, obstruction) = local_system();
        let limits = ResourceLimits {
            max_window_degree: None,
            max_proof_weight: None,
            max_matrix_rows: None,
            max_matrix_cols: None,
            max_candidate_count: None,
        };

        let output = localized_schur_repair(&system, &proof_window, &[obstruction], &limits);

        assert!(matches!(output, SchurRepairOutput::NoLocalScope));
    }

    #[test]
    fn schur_repair_returns_exact_certificate_for_target_only_local_relation() {
        let x = variable("X");
        let t = variable("T");
        let y = variable("Y");
        let variables = vec![x.clone(), t.clone(), y.clone()];
        let equations = vec![
            polynomial(&variables, &[(1, vec![1, 0, 0]), (-1, vec![0, 1, 0])]),
            polynomial(&variables, &[(1, vec![2, 0, 0]), (-1, vec![0, 1, 0])]),
            polynomial(&variables, &[(1, vec![0, 0, 1]), (-1, vec![0, 1, 0])]),
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
                vec![monomial(&[1, 0, 0]), monomial(&[0, 1, 0])],
                vec![monomial(&[0, 0, 0])],
                Vec::new(),
            ],
        };
        let obstruction = LeftNullObstruction {
            row_monomials: vec![monomial(&[2, 0, 0])],
            coefficients: vec![rational(1)],
        };
        let limits = ResourceLimits {
            max_window_degree: Some(2),
            max_proof_weight: Some(2),
            max_matrix_rows: None,
            max_matrix_cols: None,
            max_candidate_count: None,
        };

        let output = localized_schur_repair(&system, &proof_window, &[obstruction], &limits);

        let SchurRepairOutput::Certified(certificate) = output else {
            panic!("local target relation should produce a certificate");
        };
        assert_eq!(
            crate::verify_certificate(problem, SolverCertificate::TargetCover(certificate.clone())),
            VerificationResult::Verified
        );
        let TargetCertificate::IdealMembership { support, .. } = certificate else {
            panic!("localized Schur should replay as ideal membership");
        };
        assert_eq!(
            support.coefficients,
            vec![rational(0), rational(-1), rational(1)]
        );
    }
}
