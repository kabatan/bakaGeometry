use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::algebra::linear_solve::{solve_inhomogeneous_modular, MatrixBuilder, ModularSolvePlan};
use crate::algebra::modular::Prime;
use crate::result::status::SolverError;
use crate::types::hash::Hash;
use crate::types::ids::VariableId;
use crate::types::matrix::{SparseMatrixQ, VectorQ};
use crate::types::monomial::{monomial_mul, normalize_monomial, Monomial};
use crate::types::polynomial::{
    constant_poly, normalize_poly, poly_add, poly_mul, poly_scale, poly_sub, substitute_poly,
    variable_poly, SparsePolynomialQ, SubstitutionMap, TermQ,
};
use crate::types::rational::{add_q, div_q, int_q, is_zero_q, mul_q, sub_q, zero_q, RationalQ};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpecializationPoint {
    pub prime: Prime,
    pub assignments: BTreeMap<VariableId, RationalQ>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpecializedRelation {
    pub point: SpecializationPoint,
    pub relation: SparsePolynomialQ,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterpolationProofStatus {
    CandidateOnlyRequiresExactMembership,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterpolationCertificate {
    pub relation_hash: Hash,
    pub samples: Vec<SpecializedRelation>,
    pub specialized_variables: Vec<VariableId>,
    pub proof_status: InterpolationProofStatus,
}

pub fn choose_specialization_points(
    vars: &[VariableId],
    count: usize,
    prime: Prime,
) -> Vec<SpecializationPoint> {
    (0..count)
        .map(|idx| {
            let assignments = vars
                .iter()
                .enumerate()
                .map(|(offset, var)| {
                    let value = ((idx + offset + 1) as u64 % prime.max(2)) as i64;
                    (*var, int_q(value))
                })
                .collect();
            SpecializationPoint { prime, assignments }
        })
        .collect()
}

pub fn specialize_polynomials(
    polys: &[SparsePolynomialQ],
    point: &SpecializationPoint,
) -> Vec<SparsePolynomialQ> {
    let subst: SubstitutionMap = point
        .assignments
        .iter()
        .map(|(var, value)| (*var, constant_poly(value.clone())))
        .collect();
    polys
        .iter()
        .map(|poly| substitute_poly(poly, &subst))
        .collect()
}

pub fn interpolate_sparse_coefficients(
    samples: &[SpecializedRelation],
) -> Result<SparsePolynomialQ, SolverError> {
    let (specialized_var, xs) = validate_univariate_sample_points(samples)?;
    let specialized_set = BTreeSet::from([specialized_var]);
    let mut support_values: BTreeMap<Monomial, Vec<RationalQ>> = BTreeMap::new();

    for (sample_idx, sample) in samples.iter().enumerate() {
        for term in &sample.relation.terms {
            if term
                .monomial
                .exponents
                .iter()
                .any(|(var, _)| specialized_set.contains(var))
            {
                return Err(SolverError::invalid_input(
                    Some(specialized_var),
                    "specialized sample relation still contains the specialized variable",
                ));
            }
            support_values
                .entry(term.monomial.clone())
                .or_insert_with(|| vec![zero_q(); samples.len()])[sample_idx] = term.coeff.clone();
        }
    }

    let mut reconstructed = constant_poly(zero_q());
    for (monomial, values) in support_values {
        let coeff_poly = interpolate_values_for_variable(specialized_var, &xs, &values)?;
        let monomial_poly = normalize_poly(SparsePolynomialQ {
            terms: vec![TermQ {
                coeff: int_q(1),
                monomial,
            }],
            hash: crate::types::hash::hash_sequence("poly", &[]),
        });
        reconstructed = poly_add(&reconstructed, &poly_mul(&coeff_poly, &monomial_poly));
    }
    Ok(reconstructed)
}

pub fn build_multiseparator_coefficient_support(
    vars: &[VariableId],
    max_total_degree: usize,
) -> Vec<Monomial> {
    let mut sorted = vars.to_vec();
    sorted.sort();
    sorted.dedup();
    let mut out = Vec::new();
    build_support_recursive(&sorted, 0, max_total_degree as u32, Vec::new(), &mut out);
    out.sort();
    out.dedup();
    out
}

pub fn choose_multiseparator_specialization_points(
    vars: &[VariableId],
    coefficient_support: &[Monomial],
    prime: Prime,
) -> Vec<SpecializationPoint> {
    let mut sorted = vars.to_vec();
    sorted.sort();
    sorted.dedup();
    if sorted.is_empty() {
        return vec![SpecializationPoint {
            prime,
            assignments: BTreeMap::new(),
        }];
    }
    let mut max_by_var = BTreeMap::new();
    for monomial in coefficient_support {
        for (var, exp) in &monomial.exponents {
            if sorted.contains(var) {
                max_by_var
                    .entry(*var)
                    .and_modify(|current: &mut u32| *current = (*current).max(*exp))
                    .or_insert(*exp);
            }
        }
    }
    let value_lists = sorted
        .iter()
        .map(|var| {
            let count = max_by_var.get(var).copied().unwrap_or(0) as usize + 1;
            (1..=count.max(1))
                .map(|value| int_q(value as i64))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let mut assignments = Vec::new();
    build_assignment_grid(
        &sorted,
        &value_lists,
        0,
        BTreeMap::new(),
        prime,
        &mut assignments,
    );
    assignments
}

pub fn interpolate_sparse_coefficients_with_support(
    samples: &[SpecializedRelation],
    specialized_variables: &[VariableId],
    coefficient_support: &[Monomial],
    reconstruction_height_bound: Option<usize>,
) -> Result<SparsePolynomialQ, SolverError> {
    validate_multiseparator_samples(samples, specialized_variables)?;
    if coefficient_support.is_empty() {
        return Err(SolverError::invalid_input(
            None,
            "multiseparator interpolation requires a nonempty coefficient support",
        ));
    }
    let specialized_set = specialized_variables
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    for monomial in coefficient_support {
        if monomial
            .exponents
            .iter()
            .any(|(var, _)| !specialized_set.contains(var))
        {
            return Err(SolverError::invalid_input(
                None,
                "coefficient support contains a variable that is not specialized",
            ));
        }
    }

    let interpolation_matrix = build_interpolation_matrix(samples, coefficient_support);
    let base_monomials = sample_base_monomials(samples, &specialized_set)?;
    let mut reconstructed = constant_poly(zero_q());
    for base in base_monomials {
        let rhs = VectorQ {
            entries: samples
                .iter()
                .map(|sample| coefficient_for_monomial(&sample.relation, &base))
                .collect(),
        };
        let solve = solve_inhomogeneous_modular(
            MatrixBuilder {
                matrix: interpolation_matrix.clone(),
            },
            rhs.clone(),
            ModularSolvePlan {
                seed: 109,
                max_primes: 4,
                stable_rank_after: 2,
                reconstruction_height_bound,
            },
        );
        let Some(coeffs) = solve.reconstructed_solution_candidate else {
            return Err(SolverError::invalid_input(
                None,
                "multiseparator interpolation did not reconstruct a rational candidate",
            ));
        };
        if !verify_interpolation_solution(&interpolation_matrix, &coeffs, &rhs) {
            return Err(SolverError::invalid_input(
                None,
                "multiseparator interpolation candidate failed exact Q sample verification",
            ));
        }
        for (support_monomial, coeff) in coefficient_support.iter().zip(coeffs.entries.iter()) {
            if is_zero_q(coeff) {
                continue;
            }
            let monomial = monomial_mul(&base, support_monomial);
            reconstructed = poly_add(
                &reconstructed,
                &normalize_poly(SparsePolynomialQ {
                    terms: vec![TermQ {
                        coeff: coeff.clone(),
                        monomial,
                    }],
                    hash: crate::types::hash::hash_sequence("poly", &[]),
                }),
            );
        }
    }
    Ok(reconstructed)
}

pub fn verify_interpolated_relation(
    relation: &SparsePolynomialQ,
    certificate: &InterpolationCertificate,
) -> bool {
    if relation.hash != certificate.relation_hash {
        return false;
    }
    if certificate.proof_status != InterpolationProofStatus::CandidateOnlyRequiresExactMembership {
        return false;
    }
    certificate.samples.iter().all(|sample| {
        let specialized = specialize_polynomials(&[relation.clone()], &sample.point);
        specialized.first() == Some(&sample.relation)
    })
}

pub fn build_interpolation_certificate(
    relation: &SparsePolynomialQ,
    samples: Vec<SpecializedRelation>,
    specialized_variables: Vec<VariableId>,
) -> InterpolationCertificate {
    InterpolationCertificate {
        relation_hash: relation.hash,
        samples,
        specialized_variables,
        proof_status: InterpolationProofStatus::CandidateOnlyRequiresExactMembership,
    }
}

fn validate_univariate_sample_points(
    samples: &[SpecializedRelation],
) -> Result<(VariableId, Vec<RationalQ>), SolverError> {
    let Some(first) = samples.first() else {
        return Err(SolverError::invalid_input(
            None,
            "interpolation requires at least one sample",
        ));
    };
    if first.point.assignments.len() != 1 {
        return Err(SolverError::invalid_input(
            None,
            "P3d sparse interpolation primitive expects one specialized variable",
        ));
    }
    let specialized_var = *first.point.assignments.keys().next().unwrap();
    let mut xs = Vec::new();
    let mut seen = BTreeSet::new();
    for sample in samples {
        if sample.point.assignments.len() != 1
            || !sample.point.assignments.contains_key(&specialized_var)
        {
            return Err(SolverError::invalid_input(
                Some(specialized_var),
                "all interpolation samples must specialize the same variable",
            ));
        }
        let x = sample.point.assignments[&specialized_var].clone();
        if !seen.insert(x.clone()) {
            return Err(SolverError::invalid_input(
                Some(specialized_var),
                "interpolation sample points must be distinct",
            ));
        }
        xs.push(x);
    }
    Ok((specialized_var, xs))
}

fn interpolate_values_for_variable(
    var: VariableId,
    xs: &[RationalQ],
    values: &[RationalQ],
) -> Result<SparsePolynomialQ, SolverError> {
    let mut acc = constant_poly(zero_q());
    for (i, yi) in values.iter().enumerate() {
        if is_zero_q(yi) {
            continue;
        }
        let mut basis = constant_poly(int_q(1));
        for (j, xj) in xs.iter().enumerate() {
            if i == j {
                continue;
            }
            let numerator = poly_sub(&variable_poly(var), &constant_poly(xj.clone()));
            let denominator = sub_q(&xs[i], xj);
            let scale = div_q(&int_q(1), &denominator).map_err(|_| {
                SolverError::invalid_input(Some(var), "duplicate interpolation point")
            })?;
            basis = poly_mul(&basis, &poly_scale(&numerator, &scale));
        }
        acc = poly_add(&acc, &poly_scale(&basis, yi));
    }
    Ok(acc)
}

fn build_support_recursive(
    vars: &[VariableId],
    idx: usize,
    remaining_degree: u32,
    current: Vec<(VariableId, u32)>,
    out: &mut Vec<Monomial>,
) {
    if idx == vars.len() {
        out.push(normalize_monomial(current));
        return;
    }
    for exp in 0..=remaining_degree {
        let mut next = current.clone();
        if exp != 0 {
            next.push((vars[idx], exp));
        }
        build_support_recursive(vars, idx + 1, remaining_degree - exp, next, out);
    }
}

fn build_assignment_grid(
    vars: &[VariableId],
    value_lists: &[Vec<RationalQ>],
    idx: usize,
    current: BTreeMap<VariableId, RationalQ>,
    prime: Prime,
    out: &mut Vec<SpecializationPoint>,
) {
    if idx == vars.len() {
        out.push(SpecializationPoint {
            prime,
            assignments: current,
        });
        return;
    }
    for value in &value_lists[idx] {
        let mut next = current.clone();
        next.insert(vars[idx], value.clone());
        build_assignment_grid(vars, value_lists, idx + 1, next, prime, out);
    }
}

fn validate_multiseparator_samples(
    samples: &[SpecializedRelation],
    specialized_variables: &[VariableId],
) -> Result<(), SolverError> {
    if samples.is_empty() {
        return Err(SolverError::invalid_input(
            None,
            "multiseparator interpolation requires at least one sample",
        ));
    }
    let expected = specialized_variables
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    for sample in samples {
        let keys = sample
            .point
            .assignments
            .keys()
            .copied()
            .collect::<BTreeSet<_>>();
        if keys != expected {
            return Err(SolverError::invalid_input(
                None,
                "all interpolation samples must specialize exactly the declared variables",
            ));
        }
        let fingerprint = sample
            .point
            .assignments
            .iter()
            .flat_map(|(var, value)| {
                let mut bytes = var.0.to_be_bytes().to_vec();
                bytes.extend(crate::types::rational::rational_to_bytes(value));
                bytes
            })
            .collect::<Vec<_>>();
        if !seen.insert(fingerprint) {
            return Err(SolverError::invalid_input(
                None,
                "interpolation sample points must be distinct",
            ));
        }
    }
    Ok(())
}

fn build_interpolation_matrix(
    samples: &[SpecializedRelation],
    coefficient_support: &[Monomial],
) -> SparseMatrixQ {
    let mut entries = Vec::new();
    for (row, sample) in samples.iter().enumerate() {
        for (col, monomial) in coefficient_support.iter().enumerate() {
            let value = evaluate_monomial(monomial, &sample.point.assignments);
            if !is_zero_q(&value) {
                entries.push((row, col, value));
            }
        }
    }
    SparseMatrixQ {
        rows: samples.len(),
        cols: coefficient_support.len(),
        entries,
    }
}

fn sample_base_monomials(
    samples: &[SpecializedRelation],
    specialized_set: &BTreeSet<VariableId>,
) -> Result<Vec<Monomial>, SolverError> {
    let mut out = BTreeSet::new();
    for sample in samples {
        for term in &sample.relation.terms {
            if term
                .monomial
                .exponents
                .iter()
                .any(|(var, _)| specialized_set.contains(var))
            {
                return Err(SolverError::invalid_input(
                    None,
                    "specialized sample relation still contains a specialized variable",
                ));
            }
            out.insert(term.monomial.clone());
        }
    }
    Ok(out.into_iter().collect())
}

fn coefficient_for_monomial(poly: &SparsePolynomialQ, monomial: &Monomial) -> RationalQ {
    poly.terms
        .iter()
        .find(|term| &term.monomial == monomial)
        .map(|term| term.coeff.clone())
        .unwrap_or_else(zero_q)
}

fn evaluate_monomial(
    monomial: &Monomial,
    assignments: &BTreeMap<VariableId, RationalQ>,
) -> RationalQ {
    let mut acc = int_q(1);
    for (var, exp) in &monomial.exponents {
        let Some(value) = assignments.get(var) else {
            return zero_q();
        };
        acc = mul_q(&acc, &pow_q(value, *exp));
    }
    acc
}

fn pow_q(value: &RationalQ, exp: u32) -> RationalQ {
    let mut acc = int_q(1);
    for _ in 0..exp {
        acc = mul_q(&acc, value);
    }
    acc
}

fn verify_interpolation_solution(matrix: &SparseMatrixQ, coeffs: &VectorQ, rhs: &VectorQ) -> bool {
    if coeffs.entries.len() != matrix.cols || rhs.entries.len() != matrix.rows {
        return false;
    }
    let mut rows = vec![zero_q(); matrix.rows];
    for (row, col, value) in &matrix.entries {
        rows[*row] = add_q(&rows[*row], &mul_q(value, &coeffs.entries[*col]));
    }
    rows == rhs.entries
}

#[cfg(test)]
mod tests {
    use crate::types::polynomial::{poly_add, variable_poly};

    use super::*;

    #[test]
    fn specialization_points_are_deterministic_and_specialization_is_exact() {
        let a = VariableId(1);
        let x = VariableId(2);
        let point = choose_specialization_points(&[a], 1, 7).remove(0);
        let polys =
            specialize_polynomials(&[poly_mul(&variable_poly(a), &variable_poly(x))], &point);

        assert_eq!(point.assignments[&a], int_q(1));
        assert_eq!(polys[0], variable_poly(x));
    }

    #[test]
    fn sparse_coefficient_interpolation_reconstructs_relation_candidate() {
        let a = VariableId(1);
        let x = VariableId(2);
        let relation = poly_add(
            &poly_mul(&variable_poly(a), &variable_poly(x)),
            &variable_poly(x),
        );
        let points = choose_specialization_points(&[a], 2, 11);
        let samples = points
            .iter()
            .map(|point| SpecializedRelation {
                point: point.clone(),
                relation: specialize_polynomials(&[relation.clone()], point).remove(0),
            })
            .collect::<Vec<_>>();

        let reconstructed = interpolate_sparse_coefficients(&samples).unwrap();
        let certificate = build_interpolation_certificate(&reconstructed, samples, vec![a]);

        assert_eq!(reconstructed, relation);
        assert_eq!(
            certificate.proof_status,
            InterpolationProofStatus::CandidateOnlyRequiresExactMembership
        );
        assert!(verify_interpolated_relation(&reconstructed, &certificate));
    }

    #[test]
    fn bad_interpolation_sample_fails_final_verification() {
        let a = VariableId(1);
        let x = VariableId(2);
        let relation = poly_add(
            &poly_mul(&variable_poly(a), &variable_poly(x)),
            &variable_poly(x),
        );
        let points = choose_specialization_points(&[a], 2, 11);
        let mut samples = points
            .iter()
            .map(|point| SpecializedRelation {
                point: point.clone(),
                relation: specialize_polynomials(&[relation.clone()], point).remove(0),
            })
            .collect::<Vec<_>>();
        samples[1].relation = variable_poly(x);

        let certificate = build_interpolation_certificate(&relation, samples, vec![a]);

        assert!(!verify_interpolated_relation(&relation, &certificate));
    }

    #[test]
    fn multiseparator_sparse_interpolation_reconstructs_candidate_then_exact_checks_samples() {
        let t = VariableId(1);
        let u = VariableId(2);
        let v = VariableId(3);
        let relation = poly_add(
            &poly_add(&variable_poly(t), &variable_poly(u)),
            &variable_poly(v),
        );
        let support = build_multiseparator_coefficient_support(&[u, v], 1);
        let points = choose_multiseparator_specialization_points(&[u, v], &support, 101);
        let samples = points
            .iter()
            .map(|point| SpecializedRelation {
                point: point.clone(),
                relation: specialize_polynomials(&[relation.clone()], point).remove(0),
            })
            .collect::<Vec<_>>();

        let reconstructed =
            interpolate_sparse_coefficients_with_support(&samples, &[u, v], &support, None)
                .unwrap();

        assert_eq!(reconstructed, relation);
    }

    #[test]
    fn bad_multiseparator_sample_fails_exact_interpolation_verification() {
        let t = VariableId(1);
        let u = VariableId(2);
        let v = VariableId(3);
        let relation = poly_add(
            &poly_add(&variable_poly(t), &variable_poly(u)),
            &variable_poly(v),
        );
        let support = build_multiseparator_coefficient_support(&[u, v], 1);
        let points = choose_multiseparator_specialization_points(&[u, v], &support, 101);
        let mut samples = points
            .iter()
            .map(|point| SpecializedRelation {
                point: point.clone(),
                relation: specialize_polynomials(&[relation.clone()], point).remove(0),
            })
            .collect::<Vec<_>>();
        samples[0].relation = variable_poly(t);

        let reconstructed =
            interpolate_sparse_coefficients_with_support(&samples, &[u, v], &support, None);
        if let Ok(reconstructed) = reconstructed {
            let certificate = build_interpolation_certificate(&reconstructed, samples, vec![u, v]);
            assert!(!verify_interpolated_relation(&relation, &certificate));
        }
    }
}
