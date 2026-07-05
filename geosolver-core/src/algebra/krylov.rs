use serde::{Deserialize, Serialize};

use crate::algebra::quotient::{
    BasisHandleId, ProductionProvenancedTargetQuotientHandle, TargetQuotientHandle,
};
use crate::result::status::{FailureKind, SolverError, SolverErrorKind};
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::VariableId;
use crate::types::matrix::VectorQ;
use crate::types::rational::{
    add_q, div_q, int_q, is_zero_q, mul_q, neg_q, sub_q, zero_q, RationalQ,
};
use crate::types::univariate::{normalize_univariate, UniPolynomialQ};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KrylovPlan {
    pub start_vectors: Vec<VectorQ>,
    pub max_steps: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KrylovSequence {
    pub target: VariableId,
    pub basis_id: BasisHandleId,
    pub basis_size: usize,
    pub vectors_by_start: Vec<Vec<VectorQ>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecurrenceSource {
    MinimalFromSequence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecurrencePolynomial {
    pub polynomial: UniPolynomialQ,
    pub source: RecurrenceSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoverageKind {
    VerifiedCharacteristicSupportCoverage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageCertificate {
    pub coverage_kind: CoverageKind,
    pub quotient_handle_hash: Hash,
    pub basis_hash: Hash,
    pub quotient_rank: usize,
    pub target_action_matrix_hash: Hash,
    pub column_normal_form_certificate_hashes: Vec<Hash>,
    pub characteristic_polynomial_hash: Hash,
    pub cayley_hamilton_verification_hash: Hash,
    pub no_coordinate_roots_exported: bool,
    pub no_full_coordinate_rur_exported: bool,
    pub characteristic_polynomial: UniPolynomialQ,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnnihilatorCertificate {
    pub coverage_kind: CoverageKind,
    pub polynomial_hash: Hash,
    pub cayley_hamilton_verification_hash: Hash,
}

pub fn block_krylov_sequence(
    handle: &ProductionProvenancedTargetQuotientHandle,
    target: VariableId,
    plan: KrylovPlan,
) -> Result<KrylovSequence, SolverError> {
    if !handle.no_coordinate_solution_export() {
        return Err(implementation_bug("coordinate-exporting quotient handle"));
    }
    if plan.start_vectors.is_empty() {
        return Err(SolverError::invalid_input(
            Some(target),
            "Krylov plan requires at least one start vector",
        ));
    }
    let steps = plan.max_steps.max(handle.basis_size());
    let mut vectors_by_start = Vec::new();
    for start in plan.start_vectors {
        if start.entries.len() != handle.basis_size() {
            return Err(SolverError::invalid_input(
                Some(target),
                "Krylov start vector width must match quotient basis size",
            ));
        }
        let mut seq = vec![start];
        for _ in 0..steps {
            let next = handle.multiply_by_variable(seq.last().unwrap(), target)?;
            seq.push(next);
        }
        vectors_by_start.push(seq);
    }
    Ok(KrylovSequence {
        target,
        basis_id: handle.basis_id(),
        basis_size: handle.basis_size(),
        vectors_by_start,
    })
}

pub fn require_production_target_action_krylov_handle(
    handle: &dyn TargetQuotientHandle,
) -> Result<(), SolverError> {
    if handle.is_production_provenanced() {
        Ok(())
    } else {
        Err(certificate_design_gap(
            "TargetActionKrylov production path requires a provenance-bound quotient handle",
        ))
    }
}

pub fn recover_recurrence(seq: &KrylovSequence) -> Result<RecurrencePolynomial, SolverError> {
    for degree in 1..=seq.basis_size {
        if let Some(coeffs) = solve_monic_recurrence(seq, degree) {
            let mut coeffs_low_to_high = coeffs;
            coeffs_low_to_high.push(int_q(1));
            return Ok(RecurrencePolynomial {
                polynomial: normalize_univariate(UniPolynomialQ {
                    variable: seq.target,
                    coeffs_low_to_high,
                    hash: hash_sequence("univariate", &[]),
                }),
                source: RecurrenceSource::MinimalFromSequence,
            });
        }
    }
    Err(certificate_design_gap(
        "Krylov sequence did not yield an exact recurrence",
    ))
}

pub fn certify_krylov_coverage(
    seq: &KrylovSequence,
    recurrence: &RecurrencePolynomial,
    handle: &ProductionProvenancedTargetQuotientHandle,
) -> Result<CoverageCertificate, SolverError> {
    if seq.basis_id != handle.basis_id() || seq.basis_size != handle.basis_size() {
        return Err(certificate_design_gap(
            "Krylov sequence does not match quotient handle",
        ));
    }
    let (matrix, column_hashes) = target_action_matrix(handle, seq.target)?;
    let characteristic = characteristic_polynomial(&matrix, seq.target);
    if recurrence.polynomial != characteristic {
        return Err(certificate_design_gap(
            "Krylov recurrence is not verified characteristic support coverage",
        ));
    }
    let cayley = verify_cayley_hamilton_matrix_hash(&matrix, &characteristic)?;
    Ok(CoverageCertificate {
        coverage_kind: CoverageKind::VerifiedCharacteristicSupportCoverage,
        quotient_handle_hash: handle.quotient_handle_hash(),
        basis_hash: handle.basis_hash(),
        quotient_rank: handle.basis_size(),
        target_action_matrix_hash: matrix_hash(&matrix),
        column_normal_form_certificate_hashes: column_hashes,
        characteristic_polynomial_hash: characteristic.hash,
        cayley_hamilton_verification_hash: cayley,
        no_coordinate_roots_exported: handle.no_coordinate_solution_export(),
        no_full_coordinate_rur_exported: handle.no_coordinate_solution_export(),
        characteristic_polynomial: characteristic,
    })
}

pub fn verify_annihilator(
    handle: &ProductionProvenancedTargetQuotientHandle,
    poly: &UniPolynomialQ,
) -> Result<AnnihilatorCertificate, SolverError> {
    let (matrix, _) = target_action_matrix(handle, poly.variable)?;
    let characteristic = characteristic_polynomial(&matrix, poly.variable);
    if &characteristic != poly {
        return Err(certificate_design_gap(
            "annihilator is not the verified characteristic support polynomial",
        ));
    }
    let cayley = verify_cayley_hamilton_matrix_hash(&matrix, poly)?;
    Ok(AnnihilatorCertificate {
        coverage_kind: CoverageKind::VerifiedCharacteristicSupportCoverage,
        polynomial_hash: poly.hash,
        cayley_hamilton_verification_hash: cayley,
    })
}

fn target_action_matrix(
    handle: &ProductionProvenancedTargetQuotientHandle,
    target: VariableId,
) -> Result<(Vec<Vec<RationalQ>>, Vec<Hash>), SolverError> {
    let n = handle.basis_size();
    let mut columns = Vec::with_capacity(n);
    let mut hashes = Vec::with_capacity(n);
    for col in 0..n {
        let certificate = handle
            .action_column_certificate(target, col)
            .ok_or_else(|| certificate_design_gap("missing target action column certificate"))?;
        if certificate.source_relation_authorization_hash != handle.authorized_relation_hash() {
            return Err(certificate_design_gap(
                "target action column is not bound to the quotient authorization hash",
            ));
        }
        let action_column = certificate.normal_form_vector.clone();
        hashes.push(certificate.normal_form_certificate.certificate_hash);
        columns.push(action_column);
    }
    let mut rows = vec![vec![zero_q(); n]; n];
    for (col, column) in columns.iter().enumerate() {
        for (row, value) in column.entries.iter().enumerate() {
            rows[row][col] = value.clone();
        }
    }
    Ok((rows, hashes))
}

fn solve_monic_recurrence(seq: &KrylovSequence, degree: usize) -> Option<Vec<RationalQ>> {
    let mut rows = Vec::new();
    let mut rhs = Vec::new();
    for block in &seq.vectors_by_start {
        if block.len() <= degree {
            return None;
        }
        for k in 0..=(block.len() - degree - 1) {
            for component in 0..seq.basis_size {
                rows.push(
                    (0..degree)
                        .map(|j| block[k + j].entries[component].clone())
                        .collect::<Vec<_>>(),
                );
                rhs.push(neg_q(&block[k + degree].entries[component]));
            }
        }
    }
    solve_linear_system(&rows, &rhs)
}

fn solve_linear_system(rows: &[Vec<RationalQ>], rhs: &[RationalQ]) -> Option<Vec<RationalQ>> {
    let m = rows.len();
    let n = rows.first().map_or(0, Vec::len);
    if n == 0 {
        return None;
    }
    let mut aug = vec![vec![zero_q(); n + 1]; m];
    for r in 0..m {
        for c in 0..n {
            aug[r][c] = rows[r][c].clone();
        }
        aug[r][n] = rhs[r].clone();
    }
    let mut pivot_row = 0;
    let mut pivot_cols = Vec::new();
    for col in 0..n {
        let Some(found) = (pivot_row..m).find(|r| !is_zero_q(&aug[*r][col])) else {
            continue;
        };
        aug.swap(pivot_row, found);
        let inv = div_q(&int_q(1), &aug[pivot_row][col]).ok()?;
        for c in col..=n {
            aug[pivot_row][c] = mul_q(&aug[pivot_row][c], &inv);
        }
        for r in 0..m {
            if r == pivot_row {
                continue;
            }
            let factor = aug[r][col].clone();
            if is_zero_q(&factor) {
                continue;
            }
            for c in col..=n {
                aug[r][c] = sub_q(&aug[r][c], &mul_q(&factor, &aug[pivot_row][c]));
            }
        }
        pivot_cols.push(col);
        pivot_row += 1;
        if pivot_row == m {
            break;
        }
    }
    for row in &aug {
        if row[..n].iter().all(is_zero_q) && !is_zero_q(&row[n]) {
            return None;
        }
    }
    if pivot_cols.len() != n {
        return None;
    }
    let mut solution = vec![zero_q(); n];
    for (row, col) in pivot_cols.into_iter().enumerate() {
        solution[col] = aug[row][n].clone();
    }
    Some(solution)
}

fn characteristic_polynomial(matrix: &[Vec<RationalQ>], variable: VariableId) -> UniPolynomialQ {
    let n = matrix.len();
    let mut poly_matrix = vec![vec![zero_uni_poly(variable); n]; n];
    for r in 0..n {
        for c in 0..n {
            let entry = constant_uni_poly(variable, neg_q(&matrix[r][c]));
            poly_matrix[r][c] = if r == c {
                add_uni_poly(&variable_uni_poly(variable), &entry)
            } else {
                entry
            };
        }
    }
    det_uni_matrix(&poly_matrix)
}

fn verify_cayley_hamilton_matrix_hash(
    matrix: &[Vec<RationalQ>],
    poly: &UniPolynomialQ,
) -> Result<Hash, SolverError> {
    let evaluated = evaluate_uni_at_matrix(poly, matrix);
    if evaluated
        .iter()
        .flat_map(|row| row.iter())
        .any(|value| !is_zero_q(value))
    {
        return Err(certificate_design_gap(
            "Cayley-Hamilton verification failed exactly",
        ));
    }
    Ok(matrix_hash(&evaluated))
}

fn evaluate_uni_at_matrix(poly: &UniPolynomialQ, matrix: &[Vec<RationalQ>]) -> Vec<Vec<RationalQ>> {
    let n = matrix.len();
    let mut result = zero_matrix(n);
    let mut power = identity_matrix(n);
    for coeff in &poly.coeffs_low_to_high {
        result = matrix_add(&result, &matrix_scale(&power, coeff));
        power = matrix_mul(&power, matrix);
    }
    result
}

fn det_uni_matrix(matrix: &[Vec<UniPolynomialQ>]) -> UniPolynomialQ {
    let n = matrix.len();
    if n == 0 {
        return zero_uni_poly(VariableId(0));
    }
    if n == 1 {
        return matrix[0][0].clone();
    }
    let variable = matrix[0][0].variable;
    let mut acc = zero_uni_poly(variable);
    for col in 0..n {
        let minor = matrix
            .iter()
            .skip(1)
            .map(|row| {
                row.iter()
                    .enumerate()
                    .filter_map(|(idx, value)| {
                        if idx == col {
                            None
                        } else {
                            Some(value.clone())
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let term = mul_uni_poly(&matrix[0][col], &det_uni_matrix(&minor));
        if col % 2 == 0 {
            acc = add_uni_poly(&acc, &term);
        } else {
            acc = sub_uni_poly(&acc, &term);
        }
    }
    acc
}

fn zero_uni_poly(variable: VariableId) -> UniPolynomialQ {
    normalize_univariate(UniPolynomialQ {
        variable,
        coeffs_low_to_high: Vec::new(),
        hash: hash_sequence("univariate", &[]),
    })
}

fn constant_uni_poly(variable: VariableId, coeff: RationalQ) -> UniPolynomialQ {
    normalize_univariate(UniPolynomialQ {
        variable,
        coeffs_low_to_high: vec![coeff],
        hash: hash_sequence("univariate", &[]),
    })
}

fn variable_uni_poly(variable: VariableId) -> UniPolynomialQ {
    normalize_univariate(UniPolynomialQ {
        variable,
        coeffs_low_to_high: vec![zero_q(), int_q(1)],
        hash: hash_sequence("univariate", &[]),
    })
}

fn add_uni_poly(a: &UniPolynomialQ, b: &UniPolynomialQ) -> UniPolynomialQ {
    assert_eq!(a.variable, b.variable, "univariate variable mismatch");
    let n = a.coeffs_low_to_high.len().max(b.coeffs_low_to_high.len());
    let coeffs = (0..n)
        .map(|i| {
            add_q(
                a.coeffs_low_to_high.get(i).unwrap_or(&zero_q()),
                b.coeffs_low_to_high.get(i).unwrap_or(&zero_q()),
            )
        })
        .collect();
    normalize_univariate(UniPolynomialQ {
        variable: a.variable,
        coeffs_low_to_high: coeffs,
        hash: hash_sequence("univariate", &[]),
    })
}

fn sub_uni_poly(a: &UniPolynomialQ, b: &UniPolynomialQ) -> UniPolynomialQ {
    add_uni_poly(a, &scale_uni_poly(b, &int_q(-1)))
}

fn mul_uni_poly(a: &UniPolynomialQ, b: &UniPolynomialQ) -> UniPolynomialQ {
    assert_eq!(a.variable, b.variable, "univariate variable mismatch");
    if a.coeffs_low_to_high.is_empty() || b.coeffs_low_to_high.is_empty() {
        return zero_uni_poly(a.variable);
    }
    let mut coeffs = vec![zero_q(); a.coeffs_low_to_high.len() + b.coeffs_low_to_high.len() - 1];
    for (i, ai) in a.coeffs_low_to_high.iter().enumerate() {
        for (j, bj) in b.coeffs_low_to_high.iter().enumerate() {
            coeffs[i + j] = add_q(&coeffs[i + j], &mul_q(ai, bj));
        }
    }
    normalize_univariate(UniPolynomialQ {
        variable: a.variable,
        coeffs_low_to_high: coeffs,
        hash: hash_sequence("univariate", &[]),
    })
}

fn scale_uni_poly(a: &UniPolynomialQ, c: &RationalQ) -> UniPolynomialQ {
    normalize_univariate(UniPolynomialQ {
        variable: a.variable,
        coeffs_low_to_high: a.coeffs_low_to_high.iter().map(|x| mul_q(x, c)).collect(),
        hash: hash_sequence("univariate", &[]),
    })
}

fn zero_matrix(n: usize) -> Vec<Vec<RationalQ>> {
    vec![vec![zero_q(); n]; n]
}

fn identity_matrix(n: usize) -> Vec<Vec<RationalQ>> {
    let mut out = zero_matrix(n);
    for (idx, row) in out.iter_mut().enumerate() {
        row[idx] = int_q(1);
    }
    out
}

fn matrix_add(a: &[Vec<RationalQ>], b: &[Vec<RationalQ>]) -> Vec<Vec<RationalQ>> {
    a.iter()
        .zip(b)
        .map(|(ra, rb)| ra.iter().zip(rb).map(|(x, y)| add_q(x, y)).collect())
        .collect()
}

fn matrix_scale(a: &[Vec<RationalQ>], c: &RationalQ) -> Vec<Vec<RationalQ>> {
    a.iter()
        .map(|row| row.iter().map(|x| mul_q(x, c)).collect())
        .collect()
}

fn matrix_mul(a: &[Vec<RationalQ>], b: &[Vec<RationalQ>]) -> Vec<Vec<RationalQ>> {
    let n = a.len();
    let mut out = zero_matrix(n);
    for r in 0..n {
        for c in 0..n {
            let mut acc = zero_q();
            for (k, brow) in b.iter().enumerate() {
                acc = add_q(&acc, &mul_q(&a[r][k], &brow[c]));
            }
            out[r][c] = acc;
        }
    }
    out
}

fn matrix_hash(matrix: &[Vec<RationalQ>]) -> Hash {
    hash_sequence(
        "matrix-q",
        &matrix
            .iter()
            .flat_map(|row| row.iter().map(crate::types::rational::rational_to_bytes))
            .collect::<Vec<_>>(),
    )
}

fn certificate_design_gap(message: &str) -> SolverError {
    SolverError {
        target: None,
        kind: SolverErrorKind::Failure(FailureKind::CertificateDesignGap {
            constructed_object_hash: hash_sequence(
                "certificate-gap",
                &[message.as_bytes().to_vec()],
            ),
            missing_certificate_kind: message.to_string(),
        }),
    }
}

fn implementation_bug(message: &str) -> SolverError {
    SolverError {
        target: None,
        kind: SolverErrorKind::Failure(FailureKind::ImplementationBug {
            invariant_violated: message.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::algebra::normal_form::{MembershipCertificate, MembershipTerm};
    use crate::algebra::quotient::{
        build_debug_explicit_target_quotient_handle,
        build_production_target_relevant_quotient_handle, hash_authorized_relations,
        make_action_column_certificate, monomial_basis_polynomials, normal_form_basis_certificate,
        unit_vector, BasisHandleId, BasisScope, DebugQuotientHandleInput,
        ProductionProvenancedTargetQuotientHandle, ProductionQuotientHandleInput,
    };
    use crate::result::status::SolverStatus;
    use crate::types::polynomial::{constant_poly, poly_add, poly_mul, poly_sub, variable_poly};

    use super::*;

    fn companion_handle(target: VariableId) -> ProductionProvenancedTargetQuotientHandle {
        let basis = monomial_basis_polynomials(target, 2);
        let relation = poly_add(
            &poly_sub(
                &poly_mul(&variable_poly(target), &variable_poly(target)),
                &poly_mul(&constant_poly(int_q(3)), &variable_poly(target)),
            ),
            &constant_poly(int_q(2)),
        );
        let relations = vec![relation];
        let auth_hash = hash_authorized_relations(&relations);
        let col0 = make_action_column_certificate(
            target,
            0,
            &basis,
            &relations,
            auth_hash,
            VectorQ {
                entries: vec![int_q(0), int_q(1)],
            },
            MembershipCertificate {
                combination_terms: Vec::new(),
            },
        )
        .unwrap();
        let col1 = make_action_column_certificate(
            target,
            1,
            &basis,
            &relations,
            auth_hash,
            VectorQ {
                entries: vec![int_q(-2), int_q(3)],
            },
            MembershipCertificate {
                combination_terms: vec![MembershipTerm {
                    relation_id: 0,
                    multiplier: constant_poly(int_q(1)),
                }],
            },
        )
        .unwrap();
        build_production_target_relevant_quotient_handle(ProductionQuotientHandleInput {
            basis_id: BasisHandleId(2),
            basis_scope: BasisScope::TargetRelevant {
                variables: vec![target],
            },
            authorized_relation_hash: auth_hash,
            authorized_relations: relations,
            basis_polynomials: basis.clone(),
            normal_form_basis_certificate: normal_form_basis_certificate(&basis, auth_hash),
            action_columns: BTreeMap::from([(target, vec![col0, col1])]),
            no_coordinate_roots_exported: true,
            no_full_coordinate_rur_exported: true,
        })
        .unwrap()
    }

    #[test]
    fn verified_characteristic_support_coverage_is_accepted() {
        let target = VariableId(5);
        let handle = companion_handle(target);
        let seq = block_krylov_sequence(
            &handle,
            target,
            KrylovPlan {
                start_vectors: vec![unit_vector(2, 0), unit_vector(2, 1)],
                max_steps: 3,
            },
        )
        .unwrap();
        let recurrence = recover_recurrence(&seq).unwrap();
        let coverage = certify_krylov_coverage(&seq, &recurrence, &handle).unwrap();
        let ann = verify_annihilator(&handle, &coverage.characteristic_polynomial).unwrap();

        assert_eq!(
            coverage.coverage_kind,
            CoverageKind::VerifiedCharacteristicSupportCoverage
        );
        assert_eq!(
            coverage.characteristic_polynomial.coeffs_low_to_high,
            vec![int_q(2), int_q(-3), int_q(1)]
        );
        assert_eq!(
            ann.coverage_kind,
            CoverageKind::VerifiedCharacteristicSupportCoverage
        );
    }

    #[test]
    fn single_vector_krylov_undercoverage_is_rejected() {
        let target = VariableId(5);
        let handle = companion_handle(target);
        let missed_eigenvalue_start = VectorQ {
            entries: vec![int_q(-2), int_q(1)],
        };
        let seq = block_krylov_sequence(
            &handle,
            target,
            KrylovPlan {
                start_vectors: vec![missed_eigenvalue_start],
                max_steps: 3,
            },
        )
        .unwrap();
        let recurrence = recover_recurrence(&seq).unwrap();
        let err = certify_krylov_coverage(&seq, &recurrence, &handle).unwrap_err();

        assert_eq!(
            recurrence.polynomial.coeffs_low_to_high,
            vec![int_q(-1), int_q(1)]
        );
        assert_eq!(err.public_status(), SolverStatus::CertificateDesignGap);
    }

    #[test]
    fn debug_explicit_handle_is_rejected_by_production_krylov_boundary() {
        let target = VariableId(5);
        let debug = build_debug_explicit_target_quotient_handle(DebugQuotientHandleInput {
            basis_id: BasisHandleId(99),
            basis_scope: BasisScope::TargetRelevant {
                variables: vec![target],
            },
            basis_polynomials: monomial_basis_polynomials(target, 2),
            variable_action_columns: BTreeMap::from([(
                target,
                vec![
                    VectorQ {
                        entries: vec![int_q(0), int_q(1)],
                    },
                    VectorQ {
                        entries: vec![int_q(-2), int_q(3)],
                    },
                ],
            )]),
            no_coordinate_roots_exported: true,
            no_full_coordinate_rur_exported: true,
        })
        .unwrap();
        let err = require_production_target_action_krylov_handle(&debug).unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::CertificateDesignGap);
    }
}
