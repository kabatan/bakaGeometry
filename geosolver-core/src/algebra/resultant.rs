use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::algebra::modular::{choose_prime_avoiding_denominators, reduce_q_to_fp, Prime};
use num_integer::Integer;
use num_traits::Zero;

use crate::result::status::{AlgebraicReason, FailureKind, SolverError, SolverErrorKind, StageId};
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::VariableId;
use crate::types::monomial::{monomial_to_bytes, normalize_monomial, Monomial};
use crate::types::polynomial::{
    constant_poly, max_poly_coefficient_height_bits, normalize_poly, poly_add, poly_mul,
    poly_scale, poly_sub, poly_variables, zero_poly, SparsePolynomialQ, TermQ,
};
use crate::types::rational::int_q;

const SYMBOLIC_DETERMINANT_MAX_DIM: usize = 6;
const SYMBOLIC_DETERMINANT_MAX_ENTRY_TERMS: usize = 32;
const SYMBOLIC_DETERMINANT_MAX_TOTAL_ENTRY_TERMS: usize = 512;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonomialSupport {
    pub monomials: Vec<Monomial>,
    pub variables: Vec<VariableId>,
    pub hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResultantInput {
    pub polynomials: Vec<SparsePolynomialQ>,
    pub eliminate: VariableId,
    pub keep_variables: Vec<VariableId>,
    pub max_matrix_dim: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResultantTemplate {
    pub input: ResultantInput,
    pub supports: Vec<MonomialSupport>,
    pub matrix_rows: usize,
    pub matrix_cols: usize,
    pub template_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModularOptions {
    pub prime_seed: u64,
    pub max_primes: usize,
}

impl Default for ModularOptions {
    fn default() -> Self {
        Self {
            prime_seed: 5,
            max_primes: 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResultantModularTrace {
    pub prime: Prime,
    pub relation_mod_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResultantProofStatus {
    CandidateOnlyRequiresExactMembership,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResultantBackendKind {
    LinearSubresultant,
    SmallEntrySymbolicDeterminant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResultantRelation {
    pub relation: SparsePolynomialQ,
    pub certificate: SparseResultantCertificate,
    pub proof_status: ResultantProofStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SparseResultantCertificate {
    pub input: ResultantInput,
    pub template_hash: Hash,
    pub relation_hash: Hash,
    pub backend: ResultantBackendKind,
    pub exact_verification_hash: Hash,
    pub modular_traces: Vec<ResultantModularTrace>,
}

pub fn support_sets(polys: &[SparsePolynomialQ]) -> Vec<MonomialSupport> {
    polys.iter().map(monomial_support).collect()
}

pub fn build_sparse_resultant_template(
    input: ResultantInput,
) -> Result<ResultantTemplate, SolverError> {
    if input.polynomials.len() != 2 {
        return Err(algorithmic_hard_case(
            "SparseResultantTemplate",
            "P3d local resultant template currently requires exactly two polynomials",
        ));
    }
    let keep_set = validate_keep_variables(input.eliminate, &input.keep_variables)?;
    for poly in &input.polynomials {
        let vars = poly_variables(poly);
        if vars
            .iter()
            .any(|var| *var != input.eliminate && !keep_set.contains(var))
        {
            return Err(SolverError::invalid_input(
                None,
                "resultant input contains a variable outside eliminate/keep sets",
            ));
        }
    }

    let deg_a = degree_in_variable(&input.polynomials[0], input.eliminate);
    let deg_b = degree_in_variable(&input.polynomials[1], input.eliminate);
    if deg_a == 0 || deg_b == 0 {
        return Err(algorithmic_hard_case(
            "SparseResultantTemplate",
            "resultant requires positive degree in the eliminated variable",
        ));
    }
    let dim = deg_a as usize + deg_b as usize;
    if dim > input.max_matrix_dim {
        let coefficient_height_bits = max_poly_coefficient_height_bits(&input.polynomials);
        return Err(finite_resource_failure(
            "SparseResultantTemplateMatrixCap",
            dim,
            dim,
            coefficient_height_bits,
        ));
    }

    let supports = support_sets(&input.polynomials);
    let template_hash = hash_template(&input, &supports, dim);
    Ok(ResultantTemplate {
        input,
        supports,
        matrix_rows: dim,
        matrix_cols: dim,
        template_hash,
    })
}

fn validate_keep_variables(
    eliminate: VariableId,
    keep_variables: &[VariableId],
) -> Result<BTreeSet<VariableId>, SolverError> {
    let mut keep_set = BTreeSet::new();
    let mut previous = None;
    for var in keep_variables {
        if *var == eliminate {
            return Err(SolverError::invalid_input(
                Some(eliminate),
                "eliminate variable must not also be a keep variable",
            ));
        }
        if previous.is_some_and(|last| last > *var) {
            return Err(SolverError::invalid_input(
                Some(*var),
                "keep variables must be sorted in canonical ascending order",
            ));
        }
        if !keep_set.insert(*var) {
            return Err(SolverError::invalid_input(
                Some(*var),
                "keep variables must be duplicate-free and canonical",
            ));
        }
        previous = Some(*var);
    }
    Ok(keep_set)
}

pub fn compute_resultant_relation(
    template: &ResultantTemplate,
    options: ModularOptions,
) -> Result<ResultantRelation, SolverError> {
    let (relation, backend) = compute_exact_resultant_relation(template)?;
    let modular_traces = modular_relation_traces(&template.input.polynomials, &relation, options);
    let certificate = SparseResultantCertificate {
        input: template.input.clone(),
        template_hash: template.template_hash,
        relation_hash: relation.hash,
        backend,
        exact_verification_hash: resultant_exact_verification_hash(template, &relation, backend),
        modular_traces,
    };
    Ok(ResultantRelation {
        relation,
        certificate,
        proof_status: ResultantProofStatus::CandidateOnlyRequiresExactMembership,
    })
}

pub fn verify_resultant_certificate(cert: &SparseResultantCertificate) -> bool {
    let Ok(template) = build_sparse_resultant_template(cert.input.clone()) else {
        return false;
    };
    if template.template_hash != cert.template_hash {
        return false;
    }
    let Ok((recomputed, backend)) = compute_exact_resultant_relation(&template) else {
        return false;
    };
    if recomputed.hash != cert.relation_hash {
        return false;
    }
    if backend != cert.backend {
        return false;
    }
    if cert.exact_verification_hash
        != resultant_exact_verification_hash(&template, &recomputed, backend)
    {
        return false;
    }
    cert.modular_traces.iter().all(|trace| {
        trace_prime_is_valid_for_poly(&recomputed, trace.prime) && {
            let reduced = reduce_q_to_fp(&recomputed, trace.prime);
            reduced.hash == trace.relation_mod_hash
        }
    })
}

fn compute_exact_resultant_relation(
    template: &ResultantTemplate,
) -> Result<(SparsePolynomialQ, ResultantBackendKind), SolverError> {
    let left = &template.input.polynomials[0];
    let right = &template.input.polynomials[1];
    let eliminate = template.input.eliminate;
    if let Some(relation) = linear_subresultant_relation(left, right, eliminate) {
        return Ok((relation, ResultantBackendKind::LinearSubresultant));
    }
    Ok((
        sylvester_resultant(left, right, eliminate)?,
        ResultantBackendKind::SmallEntrySymbolicDeterminant,
    ))
}

fn trace_prime_is_valid_for_poly(poly: &SparsePolynomialQ, prime: Prime) -> bool {
    if !is_prime_u64(prime) {
        return false;
    }
    let p = num_bigint::BigInt::from(prime);
    poly.terms
        .iter()
        .all(|term| !term.coeff.den.mod_floor(&p).is_zero())
}

fn is_prime_u64(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 {
        return true;
    }
    if n % 2 == 0 {
        return false;
    }
    let mut d = 3;
    while d <= n / d {
        if n % d == 0 {
            return false;
        }
        d += 2;
    }
    true
}

fn monomial_support(poly: &SparsePolynomialQ) -> MonomialSupport {
    let mut monomials: Vec<_> = poly
        .terms
        .iter()
        .map(|term| term.monomial.clone())
        .collect();
    monomials.sort();
    monomials.dedup();
    let mut variables = BTreeSet::new();
    for monomial in &monomials {
        for (var, _) in &monomial.exponents {
            variables.insert(*var);
        }
    }
    let chunks = monomials.iter().map(monomial_to_bytes).collect::<Vec<_>>();
    MonomialSupport {
        monomials,
        variables: variables.into_iter().collect(),
        hash: hash_sequence("monomial-support", &chunks),
    }
}

fn degree_in_variable(poly: &SparsePolynomialQ, var: VariableId) -> u32 {
    poly.terms
        .iter()
        .map(|term| exponent_of(&term.monomial, var))
        .max()
        .unwrap_or(0)
}

fn exponent_of(monomial: &Monomial, var: VariableId) -> u32 {
    monomial
        .exponents
        .iter()
        .find(|(v, _)| *v == var)
        .map_or(0, |(_, exp)| *exp)
}

fn remove_variable_power(monomial: &Monomial, var: VariableId) -> Monomial {
    normalize_monomial(
        monomial
            .exponents
            .iter()
            .filter_map(|(v, exp)| if *v == var { None } else { Some((*v, *exp)) })
            .collect(),
    )
}

fn coefficients_by_eliminate_degree(
    poly: &SparsePolynomialQ,
    var: VariableId,
) -> BTreeMap<u32, SparsePolynomialQ> {
    let mut by_degree: BTreeMap<u32, Vec<TermQ>> = BTreeMap::new();
    for term in &poly.terms {
        by_degree
            .entry(exponent_of(&term.monomial, var))
            .or_default()
            .push(TermQ {
                coeff: term.coeff.clone(),
                monomial: remove_variable_power(&term.monomial, var),
            });
    }
    by_degree
        .into_iter()
        .map(|(degree, terms)| {
            (
                degree,
                normalize_poly(SparsePolynomialQ {
                    terms,
                    hash: hash_sequence("poly", &[]),
                }),
            )
        })
        .collect()
}

fn linear_subresultant_relation(
    left: &SparsePolynomialQ,
    right: &SparsePolynomialQ,
    eliminate: VariableId,
) -> Option<SparsePolynomialQ> {
    let left_degree = degree_in_variable(left, eliminate);
    let right_degree = degree_in_variable(right, eliminate);
    if left_degree == 1 {
        return linear_first_resultant(left, right, eliminate, right_degree);
    }
    if right_degree == 1 {
        let mut relation = linear_first_resultant(right, left, eliminate, left_degree)?;
        if left_degree % 2 == 1 {
            relation = poly_scale(&relation, &int_q(-1));
        }
        return Some(relation);
    }
    None
}

fn linear_first_resultant(
    linear: &SparsePolynomialQ,
    other: &SparsePolynomialQ,
    eliminate: VariableId,
    other_degree: u32,
) -> Option<SparsePolynomialQ> {
    let linear_coeffs = coefficients_by_eliminate_degree(linear, eliminate);
    let leading = linear_coeffs.get(&1)?;
    let constant = linear_coeffs.get(&0).cloned().unwrap_or_else(zero_poly);
    let negative_constant = poly_scale(&constant, &int_q(-1));
    let other_coeffs = coefficients_by_eliminate_degree(other, eliminate);
    let mut acc = zero_poly();
    for (exp, coeff) in other_coeffs {
        if exp > other_degree {
            return None;
        }
        let numerator_power = poly_pow_local(&negative_constant, exp);
        let leading_power = poly_pow_local(leading, other_degree - exp);
        let term = poly_mul(&poly_mul(&coeff, &numerator_power), &leading_power);
        acc = poly_add(&acc, &term);
    }
    Some(clear_eliminate_variable(&acc, eliminate))
}

fn poly_pow_local(base: &SparsePolynomialQ, exp: u32) -> SparsePolynomialQ {
    let mut result = constant_poly(int_q(1));
    for _ in 0..exp {
        result = poly_mul(&result, base);
    }
    result
}

fn sylvester_resultant(
    a: &SparsePolynomialQ,
    b: &SparsePolynomialQ,
    eliminate: VariableId,
) -> Result<SparsePolynomialQ, SolverError> {
    let deg_a = degree_in_variable(a, eliminate);
    let deg_b = degree_in_variable(b, eliminate);
    if deg_a == 0 || deg_b == 0 {
        return Err(algorithmic_hard_case(
            "SparseResultantComputation",
            "resultant requires positive degree in the eliminated variable",
        ));
    }
    let coeff_a = coefficients_by_eliminate_degree(a, eliminate);
    let coeff_b = coefficients_by_eliminate_degree(b, eliminate);
    let dim = (deg_a + deg_b) as usize;
    let mut matrix = vec![vec![zero_poly(); dim]; dim];

    for row in 0..deg_b as usize {
        fill_sylvester_row(&mut matrix[row], row, deg_a, &coeff_a);
    }
    for row in 0..deg_a as usize {
        fill_sylvester_row(&mut matrix[deg_b as usize + row], row, deg_b, &coeff_b);
    }

    guard_symbolic_determinant(&matrix)?;
    Ok(clear_eliminate_variable(
        &det_poly_matrix(&matrix),
        eliminate,
    ))
}

fn fill_sylvester_row(
    row: &mut [SparsePolynomialQ],
    shift: usize,
    degree: u32,
    coeffs: &BTreeMap<u32, SparsePolynomialQ>,
) {
    for exp in (0..=degree).rev() {
        let col = shift + (degree - exp) as usize;
        row[col] = coeffs.get(&exp).cloned().unwrap_or_else(zero_poly);
    }
}

fn det_poly_matrix(matrix: &[Vec<SparsePolynomialQ>]) -> SparsePolynomialQ {
    let n = matrix.len();
    if n == 0 {
        return zero_poly();
    }
    if n == 1 {
        return matrix[0][0].clone();
    }
    let mut acc = zero_poly();
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
        let term = poly_mul(&matrix[0][col], &det_poly_matrix(&minor));
        if col % 2 == 0 {
            acc = poly_add(&acc, &term);
        } else {
            acc = poly_sub(&acc, &term);
        }
    }
    acc
}

fn guard_symbolic_determinant(matrix: &[Vec<SparsePolynomialQ>]) -> Result<(), SolverError> {
    let dim = matrix.len();
    let max_entry_terms = matrix
        .iter()
        .flat_map(|row| row.iter())
        .map(|poly| poly.terms.len())
        .max()
        .unwrap_or(0);
    let total_entry_terms = matrix
        .iter()
        .flat_map(|row| row.iter())
        .map(|poly| poly.terms.len())
        .sum::<usize>();
    if dim > SYMBOLIC_DETERMINANT_MAX_DIM
        || max_entry_terms > SYMBOLIC_DETERMINANT_MAX_ENTRY_TERMS
        || total_entry_terms > SYMBOLIC_DETERMINANT_MAX_TOTAL_ENTRY_TERMS
    {
        let coefficient_height_bits = matrix
            .iter()
            .flat_map(|row| row.iter())
            .map(|poly| max_poly_coefficient_height_bits(std::slice::from_ref(poly)))
            .max()
            .unwrap_or(0);
        return Err(finite_resource_failure(
            "SparseResultantSymbolicDeterminantCap",
            dim,
            dim,
            coefficient_height_bits,
        ));
    }
    Ok(())
}

fn clear_eliminate_variable(poly: &SparsePolynomialQ, eliminate: VariableId) -> SparsePolynomialQ {
    normalize_poly(SparsePolynomialQ {
        terms: poly
            .terms
            .iter()
            .map(|term| TermQ {
                coeff: term.coeff.clone(),
                monomial: remove_variable_power(&term.monomial, eliminate),
            })
            .collect(),
        hash: hash_sequence("poly", &[]),
    })
}

fn modular_relation_traces(
    inputs: &[SparsePolynomialQ],
    relation: &SparsePolynomialQ,
    options: ModularOptions,
) -> Vec<ResultantModularTrace> {
    let mut polys = inputs.to_vec();
    polys.push(relation.clone());
    let mut seed = options.prime_seed;
    let mut traces = Vec::new();
    for _ in 0..options.max_primes.max(1) {
        let prime = choose_prime_avoiding_denominators(&polys, seed);
        let reduced = reduce_q_to_fp(relation, prime);
        traces.push(ResultantModularTrace {
            prime,
            relation_mod_hash: reduced.hash,
        });
        seed = prime.saturating_add(1);
    }
    traces
}

fn resultant_exact_verification_hash(
    template: &ResultantTemplate,
    relation: &SparsePolynomialQ,
    backend: ResultantBackendKind,
) -> Hash {
    let mut chunks = vec![
        template.template_hash.0.to_vec(),
        relation.hash.0.to_vec(),
        format!("{backend:?}").into_bytes(),
        template.input.eliminate.0.to_be_bytes().to_vec(),
    ];
    for input in &template.input.polynomials {
        chunks.push(input.hash.0.to_vec());
    }
    hash_sequence("resultant-exact-verification", &chunks)
}

fn hash_template(input: &ResultantInput, supports: &[MonomialSupport], dim: usize) -> Hash {
    let mut chunks = Vec::new();
    chunks.push(input.eliminate.0.to_be_bytes().to_vec());
    chunks.push((input.keep_variables.len() as u64).to_be_bytes().to_vec());
    for var in &input.keep_variables {
        chunks.push(var.0.to_be_bytes().to_vec());
    }
    chunks.push((dim as u64).to_be_bytes().to_vec());
    for support in supports {
        chunks.push(support.hash.0.to_vec());
    }
    hash_sequence("resultant-template", &chunks)
}

fn finite_resource_failure(
    stage: &str,
    rows: usize,
    cols: usize,
    coefficient_height_bits: usize,
) -> SolverError {
    SolverError {
        target: None,
        kind: SolverErrorKind::Failure(FailureKind::FiniteResourceFailure {
            stage: StageId(stage.to_string()),
            block_id: None,
            matrix_rows: Some(rows),
            matrix_cols: Some(cols),
            matrix_density: None,
            quotient_rank_estimate: None,
            coefficient_height_bits: Some(coefficient_height_bits),
            memory_bytes: None,
        }),
    }
}

fn algorithmic_hard_case(stage: &str, reason: &str) -> SolverError {
    SolverError {
        target: None,
        kind: SolverErrorKind::Failure(FailureKind::AlgorithmicHardCase {
            stage: StageId(stage.to_string()),
            reason: AlgebraicReason(reason.to_string()),
            minimal_block_hash: hash_sequence("resultant-hard-case", &[reason.as_bytes().to_vec()]),
        }),
    }
}

#[cfg(test)]
mod tests {
    use crate::result::status::SolverStatus;
    use crate::types::monomial::normalize_monomial;
    use crate::types::polynomial::{
        constant_poly, normalize_poly, poly_scale, poly_sub, variable_poly, SparsePolynomialQ,
        TermQ,
    };
    use crate::types::rational::{int_q, new_q};

    use super::*;

    #[test]
    fn resultant_support_template_and_certificate_are_exact() {
        let x = VariableId(1);
        let y = VariableId(2);
        let f = poly_sub(&variable_poly(y), &variable_poly(x));
        let g = poly_sub(&variable_poly(y), &constant_poly(int_q(1)));
        let input = ResultantInput {
            polynomials: vec![f, g],
            eliminate: y,
            keep_variables: vec![x],
            max_matrix_dim: 4,
        };

        let template = build_sparse_resultant_template(input).unwrap();
        let relation = compute_resultant_relation(&template, ModularOptions::default()).unwrap();

        assert_eq!(
            relation.proof_status,
            ResultantProofStatus::CandidateOnlyRequiresExactMembership
        );
        assert_eq!(
            relation.relation,
            poly_sub(&variable_poly(x), &constant_poly(int_q(1)))
        );
        assert!(verify_resultant_certificate(&relation.certificate));
    }

    #[test]
    fn acr_p4_linear_subresultant_backend_handles_large_entries_exactly() {
        let x = VariableId(1);
        let z = VariableId(2);
        let y = VariableId(3);
        let left_coeff = dense_keep_polynomial(&[x, z], 80, 0);
        let right_coeff = dense_keep_polynomial(&[x, z], 80, 97);
        let f = poly_sub(&variable_poly(y), &left_coeff);
        let g = poly_sub(&variable_poly(y), &right_coeff);
        let input = ResultantInput {
            polynomials: vec![f, g],
            eliminate: y,
            keep_variables: vec![x, z],
            max_matrix_dim: 4,
        };

        let template = build_sparse_resultant_template(input).unwrap();
        let relation = compute_resultant_relation(&template, ModularOptions::default()).unwrap();

        assert_eq!(
            relation.certificate.backend,
            ResultantBackendKind::LinearSubresultant
        );
        assert!(relation.relation.terms.len() > 32);
        assert!(verify_resultant_certificate(&relation.certificate));
    }

    #[test]
    fn acr_p4_recursive_determinant_rejects_large_polynomial_entries() {
        let x = VariableId(1);
        let z = VariableId(2);
        let y = VariableId(3);
        let y_squared = monomial_polynomial(&[(y, 2)]);
        let left = poly_sub(&y_squared, &dense_keep_polynomial(&[x, z], 40, 0));
        let right = poly_sub(&y_squared, &dense_keep_polynomial(&[x, z], 40, 113));
        let input = ResultantInput {
            polynomials: vec![left, right],
            eliminate: y,
            keep_variables: vec![x, z],
            max_matrix_dim: 4,
        };
        let template = build_sparse_resultant_template(input).unwrap();
        let err = compute_resultant_relation(&template, ModularOptions::default()).unwrap_err();

        assert_eq!(err.public_status(), SolverStatus::FiniteResourceFailure);
    }

    #[test]
    fn resultant_certificate_rejects_tampered_relation_hash() {
        let x = VariableId(1);
        let y = VariableId(2);
        let f = poly_sub(&variable_poly(y), &variable_poly(x));
        let g = poly_sub(&variable_poly(y), &constant_poly(int_q(1)));
        let input = ResultantInput {
            polynomials: vec![f, g],
            eliminate: y,
            keep_variables: vec![x],
            max_matrix_dim: 4,
        };
        let template = build_sparse_resultant_template(input).unwrap();
        let mut relation =
            compute_resultant_relation(&template, ModularOptions::default()).unwrap();
        relation.certificate.relation_hash = zero_poly().hash;

        assert!(!verify_resultant_certificate(&relation.certificate));
    }

    #[test]
    fn resultant_template_rejects_non_keep_variables() {
        let x = VariableId(1);
        let y = VariableId(2);
        let z = VariableId(3);
        let input = ResultantInput {
            polynomials: vec![
                poly_sub(&variable_poly(y), &variable_poly(z)),
                variable_poly(y),
            ],
            eliminate: y,
            keep_variables: vec![x],
            max_matrix_dim: 4,
        };
        let err = build_sparse_resultant_template(input).unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::InvalidInput);
    }

    #[test]
    fn resultant_template_rejects_overlapping_or_duplicate_keep_variables() {
        let x = VariableId(1);
        let y = VariableId(2);
        let f = poly_sub(&variable_poly(y), &variable_poly(x));
        let g = poly_sub(&variable_poly(y), &constant_poly(int_q(1)));
        let overlap = ResultantInput {
            polynomials: vec![f.clone(), g.clone()],
            eliminate: y,
            keep_variables: vec![x, y],
            max_matrix_dim: 4,
        };
        let duplicate = ResultantInput {
            polynomials: vec![f, g],
            eliminate: y,
            keep_variables: vec![x, x],
            max_matrix_dim: 4,
        };

        assert_eq!(
            build_sparse_resultant_template(overlap)
                .unwrap_err()
                .public_status(),
            SolverStatus::InvalidInput
        );
        assert_eq!(
            build_sparse_resultant_template(duplicate)
                .unwrap_err()
                .public_status(),
            SolverStatus::InvalidInput
        );
    }

    #[test]
    fn resultant_template_rejects_noncanonical_keep_order() {
        let x = VariableId(1);
        let z = VariableId(3);
        let y = VariableId(2);
        let f = poly_sub(&variable_poly(y), &variable_poly(x));
        let g = poly_sub(&variable_poly(y), &constant_poly(int_q(1)));
        let noncanonical = ResultantInput {
            polynomials: vec![f, g],
            eliminate: y,
            keep_variables: vec![z, x],
            max_matrix_dim: 4,
        };

        assert_eq!(
            build_sparse_resultant_template(noncanonical)
                .unwrap_err()
                .public_status(),
            SolverStatus::InvalidInput
        );
    }

    #[test]
    fn resultant_certificate_rejects_tampered_trace_prime_without_panic() {
        let x = VariableId(1);
        let y = VariableId(2);
        let f = poly_sub(
            &variable_poly(y),
            &poly_scale(&variable_poly(x), &new_q(1.into(), 2.into())),
        );
        let g = poly_sub(&variable_poly(y), &constant_poly(int_q(1)));
        let input = ResultantInput {
            polynomials: vec![f, g],
            eliminate: y,
            keep_variables: vec![x],
            max_matrix_dim: 4,
        };
        let template = build_sparse_resultant_template(input).unwrap();
        let mut relation =
            compute_resultant_relation(&template, ModularOptions::default()).unwrap();
        relation.certificate.modular_traces[0].prime = 2;

        assert!(!verify_resultant_certificate(&relation.certificate));
    }

    #[test]
    fn resultant_certificate_rejects_nonprime_trace_modulus() {
        let x = VariableId(1);
        let y = VariableId(2);
        let f = poly_sub(&variable_poly(y), &variable_poly(x));
        let g = poly_sub(&variable_poly(y), &constant_poly(int_q(1)));
        let input = ResultantInput {
            polynomials: vec![f, g],
            eliminate: y,
            keep_variables: vec![x],
            max_matrix_dim: 4,
        };
        let template = build_sparse_resultant_template(input).unwrap();
        let mut relation =
            compute_resultant_relation(&template, ModularOptions::default()).unwrap();
        relation.certificate.modular_traces[0].prime = 9;

        assert!(!verify_resultant_certificate(&relation.certificate));
    }

    fn monomial_polynomial(exponents: &[(VariableId, u32)]) -> SparsePolynomialQ {
        normalize_poly(SparsePolynomialQ {
            terms: vec![TermQ {
                coeff: int_q(1),
                monomial: normalize_monomial(exponents.to_vec()),
            }],
            hash: hash_sequence("poly", &[]),
        })
    }

    fn dense_keep_polynomial(
        keep_variables: &[VariableId],
        term_count: usize,
        offset: usize,
    ) -> SparsePolynomialQ {
        let terms = (0..term_count)
            .map(|idx| {
                let mut exponents = Vec::new();
                if let Some(first) = keep_variables.first() {
                    exponents.push((*first, (idx + offset + 1) as u32));
                }
                if let Some(second) = keep_variables.get(1) {
                    let exponent = ((idx + offset) % 5 + 1) as u32;
                    exponents.push((*second, exponent));
                }
                TermQ {
                    coeff: int_q((idx + 1) as i64),
                    monomial: normalize_monomial(exponents),
                }
            })
            .collect::<Vec<_>>();
        normalize_poly(SparsePolynomialQ {
            terms,
            hash: hash_sequence("poly", &[]),
        })
    }
}
