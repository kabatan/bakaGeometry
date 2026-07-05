use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::algebra::modular::Prime;
use crate::result::status::SolverError;
use crate::types::hash::Hash;
use crate::types::ids::VariableId;
use crate::types::monomial::Monomial;
use crate::types::polynomial::{
    constant_poly, normalize_poly, poly_add, poly_mul, poly_scale, poly_sub, substitute_poly,
    variable_poly, SparsePolynomialQ, SubstitutionMap, TermQ,
};
use crate::types::rational::{div_q, int_q, is_zero_q, sub_q, zero_q, RationalQ};

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
}
