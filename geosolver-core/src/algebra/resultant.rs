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
    normalize_poly, poly_add, poly_mul, poly_sub, poly_variables, zero_poly, SparsePolynomialQ,
    TermQ,
};

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
        return Err(finite_resource_failure(
            "SparseResultantTemplateMatrixCap",
            dim,
            dim,
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
    let relation = sylvester_resultant(
        &template.input.polynomials[0],
        &template.input.polynomials[1],
        template.input.eliminate,
    )?;
    let modular_traces = modular_relation_traces(&template.input.polynomials, &relation, options);
    let certificate = SparseResultantCertificate {
        input: template.input.clone(),
        template_hash: template.template_hash,
        relation_hash: relation.hash,
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
    let Ok(recomputed) = sylvester_resultant(
        &cert.input.polynomials[0],
        &cert.input.polynomials[1],
        cert.input.eliminate,
    ) else {
        return false;
    };
    if recomputed.hash != cert.relation_hash {
        return false;
    }
    cert.modular_traces.iter().all(|trace| {
        trace_prime_is_valid_for_poly(&recomputed, trace.prime) && {
            let reduced = reduce_q_to_fp(&recomputed, trace.prime);
            reduced.hash == trace.relation_mod_hash
        }
    })
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

fn finite_resource_failure(stage: &str, rows: usize, cols: usize) -> SolverError {
    SolverError {
        target: None,
        kind: SolverErrorKind::Failure(FailureKind::FiniteResourceFailure {
            stage: StageId(stage.to_string()),
            block_id: None,
            matrix_rows: Some(rows),
            matrix_cols: Some(cols),
            matrix_density: None,
            quotient_rank_estimate: None,
            coefficient_height_bits: None,
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
    use crate::types::polynomial::{constant_poly, poly_scale, poly_sub, variable_poly};
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
}
