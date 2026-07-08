use std::collections::{BTreeSet, VecDeque};

use crate::compression::CertifiedSystemQ;
use crate::window::{make_row_closed_certificate_window, CertificateWindow};
use crate::{Monomial, ResourceLimits};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TargetDependencyDag {
    pub nodes: Vec<DependencyNode>,
    pub relation_variables: Vec<Vec<usize>>,
    pub target_cone: BTreeSet<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DependencyNode {
    pub variable_index: usize,
    pub target_level: Option<usize>,
    pub incident_relations: Vec<usize>,
}

pub(crate) fn build_target_dependency_dag(system: &CertifiedSystemQ) -> TargetDependencyDag {
    let relation_variables = relation_variable_incidence(system);
    let variable_relations =
        variable_relation_incidence(system.variables.len(), &relation_variables);
    let target_index = system
        .variables
        .iter()
        .position(|variable| variable == &system.target)
        .unwrap();
    let target_levels = target_levels(target_index, &relation_variables, &variable_relations);
    let target_cone = target_levels
        .iter()
        .enumerate()
        .filter_map(|(index, level)| level.map(|_| index))
        .collect::<BTreeSet<_>>();
    let nodes = (0..system.variables.len())
        .map(|variable_index| DependencyNode {
            variable_index,
            target_level: target_levels[variable_index],
            incident_relations: variable_relations[variable_index].clone(),
        })
        .collect();

    TargetDependencyDag {
        nodes,
        relation_variables,
        target_cone,
    }
}

pub(crate) fn plan_certificate_windows(
    system: &CertifiedSystemQ,
    dag: &TargetDependencyDag,
    limits: &ResourceLimits,
) -> Vec<CertificateWindow> {
    let Some(max_degree) = limits.max_window_degree else {
        return Vec::new();
    };
    CertificateWindowSchedule::new(system, dag, Some(max_degree)).collect()
}

pub(crate) fn certificate_window_schedule<'a>(
    system: &'a CertifiedSystemQ,
    dag: &'a TargetDependencyDag,
    limits: &ResourceLimits,
) -> CertificateWindowSchedule<'a> {
    CertificateWindowSchedule::new(system, dag, limits.max_window_degree)
}

pub(crate) struct CertificateWindowSchedule<'a> {
    system: &'a CertifiedSystemQ,
    dag: &'a TargetDependencyDag,
    max_degree: Option<usize>,
    next_degree: usize,
}

impl<'a> CertificateWindowSchedule<'a> {
    pub(crate) fn new(
        system: &'a CertifiedSystemQ,
        dag: &'a TargetDependencyDag,
        max_degree: Option<usize>,
    ) -> Self {
        Self {
            system,
            dag,
            max_degree,
            next_degree: 0,
        }
    }
}

impl Iterator for CertificateWindowSchedule<'_> {
    type Item = CertificateWindow;

    fn next(&mut self) -> Option<Self::Item> {
        if self
            .max_degree
            .is_some_and(|max_degree| self.next_degree > max_degree)
        {
            return None;
        }

        let degree = self.next_degree;
        self.next_degree += 1;
        Some(certificate_window_for_degree(self.system, self.dag, degree))
    }
}

fn relation_variable_incidence(system: &CertifiedSystemQ) -> Vec<Vec<usize>> {
    system
        .equations
        .iter()
        .map(|equation| {
            let mut variables = BTreeSet::new();
            for monomial in equation.support() {
                for (variable_index, exponent) in monomial.exponents.iter().enumerate() {
                    if *exponent != 0 {
                        variables.insert(variable_index);
                    }
                }
            }
            variables.into_iter().collect()
        })
        .collect()
}

fn variable_relation_incidence(
    variable_count: usize,
    relation_variables: &[Vec<usize>],
) -> Vec<Vec<usize>> {
    let mut variable_relations = vec![Vec::new(); variable_count];
    for (relation_index, variables) in relation_variables.iter().enumerate() {
        for variable_index in variables {
            variable_relations[*variable_index].push(relation_index);
        }
    }
    variable_relations
}

fn target_levels(
    target_index: usize,
    relation_variables: &[Vec<usize>],
    variable_relations: &[Vec<usize>],
) -> Vec<Option<usize>> {
    let mut levels = vec![None; variable_relations.len()];
    let mut queue = VecDeque::new();
    levels[target_index] = Some(0);
    queue.push_back(target_index);

    while let Some(variable_index) = queue.pop_front() {
        let level = levels[variable_index].unwrap();
        for relation_index in &variable_relations[variable_index] {
            for neighbor in &relation_variables[*relation_index] {
                if levels[*neighbor].is_none() {
                    levels[*neighbor] = Some(level + 1);
                    queue.push_back(*neighbor);
                }
            }
        }
    }

    levels
}

pub(crate) fn certificate_window_for_degree(
    system: &CertifiedSystemQ,
    dag: &TargetDependencyDag,
    degree: usize,
) -> CertificateWindow {
    let support = prioritized_monomials(system.variables.len(), degree, dag);
    let multiplier_supports = system
        .equations
        .iter()
        .map(|_| support.clone())
        .collect::<Vec<_>>();
    make_row_closed_certificate_window(system, degree.max(1), multiplier_supports)
}

fn prioritized_monomials(
    variable_count: usize,
    max_degree: usize,
    dag: &TargetDependencyDag,
) -> Vec<Monomial> {
    let mut monomials = Vec::new();
    let mut current = vec![0; variable_count];
    enumerate_monomials(
        variable_count,
        max_degree as u32,
        0,
        &mut current,
        &mut monomials,
    );
    monomials.sort_by_key(|monomial| monomial_priority(monomial, dag));
    monomials
}

fn monomial_priority(
    monomial: &Monomial,
    dag: &TargetDependencyDag,
) -> (usize, usize, u32, Vec<u32>) {
    let outside_cone_count = monomial
        .exponents
        .iter()
        .enumerate()
        .filter(|(variable_index, exponent)| {
            **exponent != 0 && !dag.target_cone.contains(variable_index)
        })
        .count();
    let level_weight = monomial
        .exponents
        .iter()
        .enumerate()
        .map(|(variable_index, exponent)| {
            let level = dag.nodes[variable_index]
                .target_level
                .unwrap_or(usize::MAX / 4);
            level.saturating_mul(*exponent as usize)
        })
        .sum::<usize>();
    (
        outside_cone_count,
        level_weight,
        monomial.total_degree(),
        monomial.exponents.clone(),
    )
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
    use crate::window::build_membership_matrix_q;
    use crate::{PolynomialQ, Rational, Variable};

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

    fn system() -> CertifiedSystemQ {
        let x = variable("X");
        let y = variable("Y");
        let t = variable("T");
        let variables = vec![x.clone(), y.clone(), t.clone()];
        CertifiedSystemQ {
            equations: vec![
                polynomial(&variables, &[(1, vec![1, 0, 0]), (-1, vec![0, 0, 1])]),
                polynomial(&variables, &[(1, vec![0, 1, 0]), (-1, vec![1, 0, 0])]),
            ],
            variables,
            target: t,
            semantic_guards: Vec::new(),
            guard_certificates: Vec::new(),
            replay: CompressionReplayCertificate { steps: Vec::new() },
        }
    }

    #[test]
    fn dag_uses_algebraic_incidence() {
        let system = system();

        let dag = build_target_dependency_dag(&system);

        assert_eq!(dag.relation_variables[0], vec![0, 2]);
        assert_eq!(dag.relation_variables[1], vec![0, 1]);
        assert_eq!(dag.nodes[2].target_level, Some(0));
        assert_eq!(dag.nodes[0].target_level, Some(1));
        assert_eq!(dag.nodes[1].target_level, Some(2));
    }

    #[test]
    fn planner_outputs_row_closed_windows_by_degree() {
        let system = system();
        let dag = build_target_dependency_dag(&system);
        let limits = ResourceLimits {
            max_window_degree: Some(2),
            max_proof_weight: None,
            max_matrix_rows: None,
            max_matrix_cols: None,
            max_candidate_count: None,
        };

        let windows = plan_certificate_windows(&system, &dag, &limits);

        assert_eq!(windows.len(), 3);
        assert!(windows[0].multiplier_supports[0].contains(&monomial(&[0, 0, 0])));
        assert!(windows[2].multiplier_supports[0].contains(&monomial(&[0, 0, 2])));
        for window in windows {
            let membership = build_membership_matrix_q(&system, &window);
            assert_eq!(membership.row_monomials, window.row_monomials);
        }
    }

    #[test]
    fn unbounded_window_schedule_advances_degree() {
        let system = system();
        let dag = build_target_dependency_dag(&system);

        let windows = CertificateWindowSchedule::new(&system, &dag, None)
            .take(4)
            .collect::<Vec<_>>();

        assert_eq!(windows.len(), 4);
        assert!(windows[3].multiplier_supports[0].contains(&monomial(&[0, 0, 3])));
    }
}
