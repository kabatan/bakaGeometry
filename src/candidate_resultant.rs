use std::collections::{BTreeMap, BTreeSet};

use crate::candidates::{
    CandidateOracle, CandidateOrigin, CandidateTrace, ModularWitnessTrace, RouteWitnessTrace,
    SparseResultantTemplateKind, SparseResultantWitnessTrace, TargetCandidate,
};
use crate::compression::CertifiedSystemQ;
use crate::finite_field::{rational_to_mod_prime, PrimeModulus};
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
    let template = SparseEliminantTemplate::from_system(system, target_degree);
    let row_monomials = template.row_monomials.clone();
    let exact_columns = resultant_columns(system, &row_monomials, &template.multiplier_supports);
    let target_columns = target_power_columns(system, &row_monomials, target_degree);
    let exact_vectors = exact_columns
        .iter()
        .map(|column| column.vector.clone())
        .collect::<Vec<_>>();
    let mut candidates = Vec::new();

    for prime in primes {
        let Some(modulus) = PrimeModulus::new(*prime) else {
            continue;
        };
        let Some(mut modular_columns) = exact_vectors
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

        for relation in sparse_eliminant_minor_relations(
            &modular_columns,
            exact_columns.len(),
            target_columns.len(),
            modulus,
        ) {
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
            let eliminant_degree = support
                .coefficients
                .iter()
                .rposition(|coefficient| *coefficient != 0)
                .unwrap_or(0);
            let active_multiplier_supports = active_multiplier_supports_from_relation(
                &relation[..exact_columns.len()],
                &exact_columns,
                system.equations.len(),
            );
            candidates.push(TargetCandidate::from_origin(
                vec![support],
                None,
                CandidateOrigin::HiddenVariableSparseResultant,
                vec![
                    CandidateTrace::RouteWitness(RouteWitnessTrace {
                        origin: CandidateOrigin::HiddenVariableSparseResultant,
                        equation_indices: (0..system.equations.len()).collect(),
                        support_size: row_monomials.len(),
                    }),
                    CandidateTrace::SparseResultantWitness(SparseResultantWitnessTrace {
                        prime: *prime,
                        template_kind: template.kind,
                        eliminated_variable_count: template.eliminated_variable_count,
                        equation_indices: (0..system.equations.len()).collect(),
                        support_sums: template.support_sums.clone(),
                        row_support_size: row_monomials.len(),
                        multiplier_support_sizes: template
                            .multiplier_supports
                            .iter()
                            .map(Vec::len)
                            .collect(),
                        minor_or_determinant_degree: eliminant_degree,
                        active_multiplier_supports: active_multiplier_supports.clone(),
                    }),
                    CandidateTrace::ModularWitness(ModularWitnessTrace {
                        prime: *prime,
                        active_multiplier_supports,
                        relation_coefficients: coefficients,
                        residual_vectors: Vec::new(),
                    }),
                ],
            ));
        }
    }

    candidates
}

#[derive(Clone, Debug)]
struct SparseEliminantTemplate {
    kind: SparseResultantTemplateKind,
    eliminated_variable_count: usize,
    support_sums: Vec<Vec<Monomial>>,
    multiplier_supports: Vec<Vec<Monomial>>,
    row_monomials: Vec<Monomial>,
}

impl SparseEliminantTemplate {
    fn from_system(system: &CertifiedSystemQ, target_degree: usize) -> Self {
        let target_index = target_index(system);
        let equation_supports = system
            .equations
            .iter()
            .map(|equation| hidden_support(system, equation, target_index))
            .collect::<Vec<_>>();
        let support_sums = sparse_multiplier_support_sums(system, &equation_supports);
        let multiplier_supports =
            target_power_lifted_supports(system, &support_sums, target_degree);
        let row_monomials = resultant_rows(system, target_degree, &multiplier_supports);
        let eliminated_variable_count = system.variables.len().saturating_sub(1);
        let kind = if system.equations.len() == eliminated_variable_count + 1 {
            SparseResultantTemplateKind::SquareSparseResultant
        } else {
            SparseResultantTemplateKind::OverdeterminedSparseEliminantMinor
        };

        Self {
            kind,
            eliminated_variable_count,
            support_sums,
            multiplier_supports,
            row_monomials,
        }
    }
}

#[derive(Clone, Debug)]
struct ResultantColumn {
    equation_index: usize,
    multiplier_monomial: Monomial,
    vector: Vec<crate::Rational>,
}

fn target_index(system: &CertifiedSystemQ) -> usize {
    system
        .variables
        .iter()
        .position(|variable| variable == &system.target)
        .unwrap()
}

fn hidden_support(
    system: &CertifiedSystemQ,
    equation: &PolynomialQ,
    target_index: usize,
) -> Vec<Monomial> {
    let mut support = equation
        .support()
        .into_iter()
        .map(|mut monomial| {
            monomial.exponents[target_index] = 0;
            monomial
        })
        .collect::<Vec<_>>();
    support.push(monomial_one(system.variables.len()));
    support.sort_by_key(canonical_monomial_key);
    support.dedup();
    if support.is_empty() {
        vec![monomial_one(system.variables.len())]
    } else {
        support
    }
}

fn sparse_multiplier_support_sums(
    system: &CertifiedSystemQ,
    equation_supports: &[Vec<Monomial>],
) -> Vec<Vec<Monomial>> {
    let global_subset_sums = all_subset_minkowski_sums(system, equation_supports);
    (0..equation_supports.len())
        .map(|equation_index| {
            let mut sums = BTreeSet::from([monomial_one(system.variables.len())]);
            for (other_index, other_support) in equation_supports.iter().enumerate() {
                if other_index == equation_index {
                    continue;
                }
                sums = minkowski_sum_supports(&sums, other_support);
            }
            sums.extend(global_subset_sums.iter().cloned());
            let mut support = sums.into_iter().collect::<Vec<_>>();
            support.sort_by_key(canonical_monomial_key);
            support
        })
        .collect()
}

fn all_subset_minkowski_sums(
    system: &CertifiedSystemQ,
    equation_supports: &[Vec<Monomial>],
) -> BTreeSet<Monomial> {
    let mut sums = BTreeSet::from([monomial_one(system.variables.len())]);
    for support in equation_supports {
        let existing = sums.iter().cloned().collect::<Vec<_>>();
        for left in existing {
            for right in support {
                sums.insert(left.multiply(right));
            }
        }
    }
    sums
}

fn target_power_lifted_supports(
    system: &CertifiedSystemQ,
    support_sums: &[Vec<Monomial>],
    target_degree: usize,
) -> Vec<Vec<Monomial>> {
    let target_index = target_index(system);
    support_sums
        .iter()
        .map(|support| {
            let mut lifted = Vec::new();
            for monomial in support {
                for target_power in 0..=target_degree {
                    let mut lifted_monomial = monomial.clone();
                    lifted_monomial.exponents[target_index] = target_power as u32;
                    lifted.push(lifted_monomial);
                }
            }
            lifted.sort_by_key(canonical_monomial_key);
            lifted.dedup();
            lifted
        })
        .collect()
}

fn minkowski_sum_supports(left: &BTreeSet<Monomial>, right: &[Monomial]) -> BTreeSet<Monomial> {
    let mut sums = BTreeSet::new();
    for left_monomial in left {
        for right_monomial in right {
            sums.insert(left_monomial.multiply(right_monomial));
        }
    }
    sums
}

fn monomial_one(variable_count: usize) -> Monomial {
    Monomial {
        exponents: vec![0; variable_count],
    }
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
) -> Vec<ResultantColumn> {
    let row_index = row_index_map(row_monomials);
    let mut columns = Vec::new();
    for (equation_index, (equation, supports)) in
        system.equations.iter().zip(multiplier_supports).enumerate()
    {
        for multiplier_monomial in supports {
            let product = monomial_times_polynomial(system, multiplier_monomial, equation);
            columns.push(ResultantColumn {
                equation_index,
                multiplier_monomial: multiplier_monomial.clone(),
                vector: vector_from_polynomial(&product, &row_index, row_monomials.len()),
            });
        }
    }
    columns
}

fn active_multiplier_supports_from_relation(
    relation: &[u64],
    columns: &[ResultantColumn],
    equation_count: usize,
) -> Vec<Vec<Monomial>> {
    let mut active = vec![BTreeSet::new(); equation_count];
    for (coefficient, column) in relation.iter().zip(columns) {
        if *coefficient != 0 {
            active[column.equation_index].insert(column.multiplier_monomial.clone());
        }
    }
    active
        .into_iter()
        .map(|support| {
            let mut support = support.into_iter().collect::<Vec<_>>();
            support.sort_by_key(canonical_monomial_key);
            support
        })
        .collect()
}

fn sparse_eliminant_minor_relations(
    columns: &[Vec<u64>],
    exact_column_count: usize,
    target_column_count: usize,
    modulus: PrimeModulus,
) -> Vec<Vec<u64>> {
    let target_indices =
        (exact_column_count..exact_column_count + target_column_count).collect::<Vec<_>>();
    let mut selected = target_indices.clone();
    let mut relations = Vec::new();

    for exact_index in 0..exact_column_count {
        selected.push(exact_index);
        if column_rank(columns, &selected, modulus) == selected.len() {
            continue;
        }

        let dependent_subset =
            minimize_dependent_exact_columns(columns, &selected, &target_indices, modulus);
        let Some(subset_relation) = determinant_minor_relation(columns, &dependent_subset, modulus)
        else {
            selected.pop();
            continue;
        };

        let mut relation = vec![0; columns.len()];
        for (column_index, coefficient) in dependent_subset.iter().zip(subset_relation) {
            relation[*column_index] = coefficient;
        }
        if relation[exact_column_count..exact_column_count + target_column_count]
            .iter()
            .any(|coefficient| *coefficient != 0)
            && !relations.contains(&relation)
        {
            relations.push(relation);
        } else {
            selected.pop();
        }
    }

    relations
}

fn minimize_dependent_exact_columns(
    columns: &[Vec<u64>],
    selected: &[usize],
    target_indices: &[usize],
    modulus: PrimeModulus,
) -> Vec<usize> {
    let target_set = target_indices.iter().copied().collect::<BTreeSet<_>>();
    let mut minimized = selected.to_vec();
    for column_index in selected {
        if target_set.contains(column_index) {
            continue;
        }
        let trial = minimized
            .iter()
            .copied()
            .filter(|candidate| candidate != column_index)
            .collect::<Vec<_>>();
        if column_rank(columns, &trial, modulus) < trial.len() {
            minimized = trial;
        }
    }
    minimized
}

fn determinant_minor_relation(
    columns: &[Vec<u64>],
    selected: &[usize],
    modulus: PrimeModulus,
) -> Option<Vec<u64>> {
    if selected.is_empty() {
        return None;
    }
    let rank = column_rank(columns, selected, modulus);
    if rank + 1 != selected.len() {
        return None;
    }
    let rows = independent_row_subset(columns, selected, rank, modulus)?;
    let mut relation = Vec::with_capacity(selected.len());
    for omitted_column_position in 0..selected.len() {
        let square = rows
            .iter()
            .map(|row_index| {
                selected
                    .iter()
                    .enumerate()
                    .filter_map(|(column_position, column_index)| {
                        (column_position != omitted_column_position)
                            .then_some(columns[*column_index][*row_index])
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let mut coefficient = determinant_mod_matrix(&square, modulus);
        if omitted_column_position % 2 == 1 {
            coefficient = modulus.neg(coefficient);
        }
        relation.push(coefficient);
    }
    relation
        .iter()
        .any(|coefficient| *coefficient != 0)
        .then_some(relation)
}

fn independent_row_subset(
    columns: &[Vec<u64>],
    selected: &[usize],
    rank: usize,
    modulus: PrimeModulus,
) -> Option<Vec<usize>> {
    if rank == 0 {
        return Some(Vec::new());
    }
    let row_count = columns.first().map_or(0, Vec::len);
    let mut rows = Vec::new();
    let mut current_rank = 0;
    for row_index in 0..row_count {
        let mut trial = rows.clone();
        trial.push(row_index);
        let trial_rank = row_rank_for_rows(columns, selected, &trial, modulus);
        if trial_rank > current_rank {
            rows = trial;
            current_rank = trial_rank;
            if current_rank == rank {
                return Some(rows);
            }
        }
    }
    None
}

fn column_rank(columns: &[Vec<u64>], selected: &[usize], modulus: PrimeModulus) -> usize {
    if selected.is_empty() {
        return 0;
    }
    let row_count = columns.first().map_or(0, Vec::len);
    let matrix = (0..row_count)
        .map(|row_index| {
            selected
                .iter()
                .map(|column_index| columns[*column_index][row_index])
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    matrix_rank_mod(matrix, modulus)
}

fn row_rank_for_rows(
    columns: &[Vec<u64>],
    selected: &[usize],
    rows: &[usize],
    modulus: PrimeModulus,
) -> usize {
    let matrix = rows
        .iter()
        .map(|row_index| {
            selected
                .iter()
                .map(|column_index| columns[*column_index][*row_index])
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    matrix_rank_mod(matrix, modulus)
}

fn matrix_rank_mod(mut matrix: Vec<Vec<u64>>, modulus: PrimeModulus) -> usize {
    let Some(column_count) = matrix.first().map(Vec::len) else {
        return 0;
    };
    let mut rank = 0;
    for column in 0..column_count {
        let Some(pivot_row) = (rank..matrix.len()).find(|row| matrix[*row][column] != 0) else {
            continue;
        };
        matrix.swap(rank, pivot_row);
        let pivot_inverse = modulus
            .inv(matrix[rank][column])
            .expect("nonzero finite-field pivot must be invertible");
        for entry in &mut matrix[rank] {
            *entry = modulus.mul(*entry, pivot_inverse);
        }
        for row in 0..matrix.len() {
            if row == rank || matrix[row][column] == 0 {
                continue;
            }
            let factor = matrix[row][column];
            for col in column..column_count {
                matrix[row][col] =
                    modulus.sub(matrix[row][col], modulus.mul(factor, matrix[rank][col]));
            }
        }
        rank += 1;
        if rank == matrix.len() {
            break;
        }
    }
    rank
}

fn determinant_mod_matrix(matrix: &[Vec<u64>], modulus: PrimeModulus) -> u64 {
    if matrix.is_empty() {
        return 1;
    }
    let size = matrix.len();
    if matrix.iter().any(|row| row.len() != size) {
        return 0;
    }
    let mut matrix = matrix.to_vec();
    let mut determinant = 1;
    for column in 0..size {
        let Some(pivot_row) = (column..size).find(|row| matrix[*row][column] != 0) else {
            return 0;
        };
        if pivot_row != column {
            matrix.swap(column, pivot_row);
            determinant = modulus.neg(determinant);
        }
        let pivot = matrix[column][column];
        determinant = modulus.mul(determinant, pivot);
        let pivot_inverse = modulus
            .inv(pivot)
            .expect("nonzero finite-field pivot must be invertible");
        for row in column + 1..size {
            let factor = modulus.mul(matrix[row][column], pivot_inverse);
            for col in column..size {
                matrix[row][col] =
                    modulus.sub(matrix[row][col], modulus.mul(factor, matrix[column][col]));
            }
        }
    }
    determinant
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
    fn resultant_route_handles_two_polynomial_hidden_resultant() {
        let x = variable("X");
        let t = variable("T");
        let variables = vec![x.clone(), t.clone()];
        let system = CertifiedSystemQ {
            equations: vec![
                polynomial(&variables, &[(1, vec![2, 0]), (-2, vec![0, 0])]),
                polynomial(&variables, &[(1, vec![0, 1]), (-1, vec![1, 0])]),
            ],
            variables,
            target: t.clone(),
            semantic_guards: Vec::new(),
            guard_certificates: Vec::new(),
            replay: CompressionReplayCertificate { steps: Vec::new() },
        };
        let window = make_row_closed_certificate_window(
            &system,
            2,
            vec![Vec::new(); system.equations.len()],
        );

        let candidates = hidden_variable_sparse_resultant_candidates(&system, &window, &[5]);

        assert!(candidates.iter().any(|candidate| {
            candidate.origin == CandidateOrigin::HiddenVariableSparseResultant
                && candidate
                    .support_mod_primes
                    .iter()
                    .any(|support| support.variable == t && support.coefficients == vec![3, 0, 1])
                && candidate.traces.iter().any(|trace| {
                    matches!(
                        trace,
                        CandidateTrace::SparseResultantWitness(witness)
                            if witness.template_kind == SparseResultantTemplateKind::SquareSparseResultant
                                && witness.eliminated_variable_count == 1
                                && witness.minor_or_determinant_degree == 2
                    )
                })
        }));
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
            semantic_guards: Vec::new(),
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
        assert!(candidates.iter().any(|candidate| {
            candidate.traces.iter().any(|trace| {
                matches!(
                    trace,
                    CandidateTrace::SparseResultantWitness(witness)
                        if witness.template_kind == SparseResultantTemplateKind::SquareSparseResultant
                            && witness.eliminated_variable_count == 2
                            && witness.row_support_size > 0
                            && witness.minor_or_determinant_degree > 0
                            && witness.active_multiplier_supports.iter().any(|support| !support.is_empty())
                )
            })
        }));
    }

    #[test]
    fn resultant_route_handles_non_chain_three_equation_eliminant() {
        let x = variable("X");
        let y = variable("Y");
        let t = variable("T");
        let variables = vec![x.clone(), y.clone(), t.clone()];
        let system = CertifiedSystemQ {
            equations: vec![
                polynomial(&variables, &[(1, vec![2, 0, 0]), (-2, vec![0, 0, 0])]),
                polynomial(&variables, &[(1, vec![1, 1, 0]), (-1, vec![0, 0, 0])]),
                polynomial(
                    &variables,
                    &[(1, vec![0, 0, 1]), (-1, vec![1, 0, 0]), (-1, vec![0, 1, 0])],
                ),
            ],
            variables,
            target: t.clone(),
            semantic_guards: Vec::new(),
            guard_certificates: Vec::new(),
            replay: CompressionReplayCertificate { steps: Vec::new() },
        };
        let window = make_row_closed_certificate_window(
            &system,
            2,
            vec![Vec::new(); system.equations.len()],
        );

        let candidates = hidden_variable_sparse_resultant_candidates(&system, &window, &[5]);

        assert!(candidates.iter().any(|candidate| {
            candidate.origin == CandidateOrigin::HiddenVariableSparseResultant
                && candidate
                    .support_mod_primes
                    .iter()
                    .any(|support| support.variable == t && support.coefficients == vec![3, 0, 1])
                && candidate.traces.iter().any(|trace| {
                    matches!(
                        trace,
                        CandidateTrace::SparseResultantWitness(witness)
                            if witness.template_kind == SparseResultantTemplateKind::SquareSparseResultant
                    )
                })
        }));
    }

    #[test]
    fn sparse_template_support_sums_do_not_fill_total_degree_macaulay_shape() {
        let x = variable("X");
        let y = variable("Y");
        let t = variable("T");
        let variables = vec![x.clone(), y.clone(), t.clone()];
        let system = CertifiedSystemQ {
            equations: vec![
                polynomial(&variables, &[(1, vec![4, 0, 0]), (1, vec![0, 0, 0])]),
                polynomial(&variables, &[(1, vec![0, 4, 0]), (1, vec![0, 0, 0])]),
                polynomial(
                    &variables,
                    &[(1, vec![0, 0, 1]), (-1, vec![4, 0, 0]), (-1, vec![0, 4, 0])],
                ),
            ],
            variables,
            target: t,
            semantic_guards: Vec::new(),
            guard_certificates: Vec::new(),
            replay: CompressionReplayCertificate { steps: Vec::new() },
        };

        let template = SparseEliminantTemplate::from_system(&system, 2);

        assert!(template.support_sums[2].contains(&monomial(&[4, 0, 0])));
        assert!(template.support_sums[2].contains(&monomial(&[0, 4, 0])));
        assert!(!template.support_sums[2].contains(&monomial(&[1, 0, 0])));
        assert!(!template.support_sums[2].contains(&monomial(&[0, 1, 0])));
    }
}
