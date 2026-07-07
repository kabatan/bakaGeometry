use std::collections::{BTreeMap, VecDeque};

use serde::{Deserialize, Serialize};

use crate::algebra::groebner::{
    certified_s_pair, implementation_bug, scale_membership_certificate,
    subtract_membership_certificate, CertifiedPolynomialQ,
};
use crate::algebra::linear_solve::{
    solve_homogeneous_modular, MatrixBuilder, ModularSolvePlan, PrimeSolveTrace,
};
use crate::algebra::monomial_order::{elimination_order, MonomialOrder};
use crate::algebra::normal_form::verify_membership_by_certificate;
use crate::algebra::polynomial_ops::leading_term;
use crate::result::status::{FailureKind, SolverError, SolverErrorKind, StageId};
use crate::types::hash::hash_sequence;
use crate::types::ids::VariableId;
use crate::types::matrix::SparseMatrixQ;
use crate::types::monomial::{monomial_div, normalize_monomial, Monomial};
use crate::types::polynomial::{
    constant_poly, normalize_poly, poly_mul, poly_scale, poly_sub, poly_variables,
    SparsePolynomialQ, TermQ,
};
use crate::types::rational::{div_q, int_q, is_zero_q, RationalQ};

use crate::algebra::elimination::{
    local_result_from_certified_basis, LocalEliminationResult, LocalEliminationStrategyName,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct F4Options {
    pub max_pairs: usize,
    pub max_basis_size: usize,
    pub batch_size: usize,
    pub modular_seed: u64,
    pub modular_prime_count: usize,
}

impl Default for F4Options {
    fn default() -> Self {
        Self {
            max_pairs: 256,
            max_basis_size: 64,
            batch_size: 8,
            modular_seed: 2,
            modular_prime_count: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct F4BatchMatrixTrace {
    pub matrix_rows: usize,
    pub matrix_cols: usize,
    pub reducer_count: usize,
    pub target_count: usize,
    pub modular_traces: Vec<PrimeSolveTrace>,
    pub trace_hash: crate::types::hash::Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct F4BatchReductionResult {
    pub reductions: Vec<CertifiedPolynomialQ>,
    pub matrix_trace: F4BatchMatrixTrace,
    pub exact_certificates_verified: bool,
}

pub fn f4_reduce_batch(
    reducers: &[CertifiedPolynomialQ],
    targets: &[CertifiedPolynomialQ],
    authorized_relations: &[SparsePolynomialQ],
    order: &MonomialOrder,
    options: &F4Options,
) -> Result<F4BatchReductionResult, SolverError> {
    let preprocessing_rows = symbolic_preprocessing_rows(reducers, targets, order)?;
    let matrix = symbolic_preprocessing_matrix(&preprocessing_rows);
    let modular = solve_homogeneous_modular(
        MatrixBuilder {
            matrix: matrix.clone(),
        },
        ModularSolvePlan {
            seed: options.modular_seed,
            max_primes: options.modular_prime_count.max(1),
            stable_rank_after: 1,
            reconstruction_height_bound: Some(16),
        },
    );

    let reductions = exact_batch_row_reduce(
        preprocessing_rows,
        targets.len(),
        order,
        authorized_relations,
    )?;

    Ok(F4BatchReductionResult {
        reductions,
        matrix_trace: F4BatchMatrixTrace {
            matrix_rows: matrix.rows,
            matrix_cols: matrix.cols,
            reducer_count: reducers.len(),
            target_count: targets.len(),
            modular_traces: modular.traces,
            trace_hash: hash_f4_matrix_trace(
                matrix.rows,
                matrix.cols,
                reducers.len(),
                targets.len(),
            ),
        },
        exact_certificates_verified: true,
    })
}

pub fn f4_elimination_local(
    relations: &[SparsePolynomialQ],
    eliminate: &[VariableId],
    keep: &[VariableId],
    options: F4Options,
) -> Result<LocalEliminationResult, SolverError> {
    let order = elimination_order(eliminate, keep);
    let mut basis = initial_certified_basis(relations);
    let mut pairs = initial_pairs(basis.len());
    let mut pairs_processed = 0usize;
    let mut matrix_rows = 0usize;
    let mut matrix_cols = 0usize;

    while !pairs.is_empty() {
        if pairs_processed >= options.max_pairs || basis.len() > options.max_basis_size {
            return Err(f4_resource_failure(
                pairs_processed.saturating_add(1),
                basis.len(),
                matrix_rows,
                matrix_cols,
            ));
        }
        let batch_pairs = drain_batch(&mut pairs, options.batch_size.max(1));
        pairs_processed = pairs_processed.saturating_add(batch_pairs.len());
        let mut targets = Vec::new();
        for (i, j) in &batch_pairs {
            let s_pair = certified_s_pair(&basis[*i], &basis[*j], &order)?;
            if !s_pair.polynomial.terms.is_empty() {
                targets.push(s_pair);
            }
        }
        if targets.is_empty() {
            continue;
        }
        let batch = f4_reduce_batch(&basis, &targets, relations, &order, &options)?;
        matrix_rows = matrix_rows.saturating_add(batch.matrix_trace.matrix_rows);
        matrix_cols = matrix_cols.max(batch.matrix_trace.matrix_cols);
        for reduction in batch.reductions {
            let polynomial = normalize_poly(reduction.polynomial);
            if polynomial.terms.is_empty() {
                continue;
            }
            if basis
                .iter()
                .any(|entry| entry.polynomial.hash == polynomial.hash)
            {
                continue;
            }
            if !verify_membership_by_certificate(&polynomial, &reduction.certificate, relations) {
                return Err(implementation_bug(
                    "F4 produced a basis element without exact source membership",
                ));
            }
            let new_index = basis.len();
            basis.push(CertifiedPolynomialQ {
                polynomial,
                certificate: reduction.certificate,
            });
            for old in 0..new_index {
                pairs.push_back((old, new_index));
            }
        }
    }

    local_result_from_certified_basis(
        relations,
        keep,
        basis,
        LocalEliminationStrategyName::F4EliminationLocal,
        matrix_rows.max(pairs_processed),
        matrix_cols,
    )
}

fn initial_certified_basis(relations: &[SparsePolynomialQ]) -> Vec<CertifiedPolynomialQ> {
    relations
        .iter()
        .enumerate()
        .filter_map(|(idx, relation)| {
            let polynomial = normalize_poly(relation.clone());
            if polynomial.terms.is_empty() {
                return None;
            }
            Some(CertifiedPolynomialQ {
                polynomial,
                certificate: crate::algebra::normal_form::MembershipCertificate {
                    combination_terms: vec![crate::algebra::normal_form::MembershipTerm {
                        relation_id: idx,
                        multiplier: constant_poly(int_q(1)),
                    }],
                },
            })
        })
        .collect()
}

fn initial_pairs(len: usize) -> VecDeque<(usize, usize)> {
    let mut pairs = VecDeque::new();
    for i in 0..len {
        for j in (i + 1)..len {
            pairs.push_back((i, j));
        }
    }
    pairs
}

fn drain_batch(pairs: &mut VecDeque<(usize, usize)>, batch_size: usize) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    while out.len() < batch_size {
        let Some(pair) = pairs.pop_front() else {
            break;
        };
        out.push(pair);
    }
    out
}

#[derive(Debug, Clone)]
struct F4MatrixRow {
    entry: CertifiedPolynomialQ,
    target_index: Option<usize>,
}

fn symbolic_preprocessing_rows(
    reducers: &[CertifiedPolynomialQ],
    targets: &[CertifiedPolynomialQ],
    order: &MonomialOrder,
) -> Result<Vec<F4MatrixRow>, SolverError> {
    let mut rows = Vec::new();
    for target in targets {
        for term in &target.polynomial.terms {
            for reducer in reducers {
                let Some(lt) = leading_term(&reducer.polynomial, order) else {
                    continue;
                };
                let Some(multiplier_monomial) = monomial_div(&term.monomial, &lt.monomial) else {
                    continue;
                };
                let multiplier = multiplier_poly(int_q(1), multiplier_monomial);
                rows.push(F4MatrixRow {
                    entry: CertifiedPolynomialQ {
                        polynomial: poly_mul(&multiplier, &reducer.polynomial),
                        certificate: scale_membership_certificate(
                            &reducer.certificate,
                            &multiplier,
                        ),
                    },
                    target_index: None,
                });
            }
        }
    }
    for (idx, target) in targets.iter().cloned().enumerate() {
        rows.push(F4MatrixRow {
            entry: target,
            target_index: Some(idx),
        });
    }
    Ok(rows)
}

fn exact_batch_row_reduce(
    rows: Vec<F4MatrixRow>,
    target_count: usize,
    order: &MonomialOrder,
    authorized_relations: &[SparsePolynomialQ],
) -> Result<Vec<CertifiedPolynomialQ>, SolverError> {
    let mut pivots = Vec::<CertifiedPolynomialQ>::new();
    let mut reductions = vec![None; target_count];
    for row in rows {
        let reduced = reduce_row_by_pivots(row.entry, &pivots, order)?;
        let polynomial = normalize_poly(reduced.polynomial);
        if let Some(target_index) = row.target_index {
            if polynomial.terms.is_empty() {
                reductions[target_index] = Some(CertifiedPolynomialQ {
                    polynomial,
                    certificate: crate::algebra::normal_form::MembershipCertificate {
                        combination_terms: Vec::new(),
                    },
                });
            } else {
                if !verify_membership_by_certificate(
                    &polynomial,
                    &reduced.certificate,
                    authorized_relations,
                ) {
                    return Err(implementation_bug(
                        "F4 matrix row reduction failed exact target certificate verification",
                    ));
                }
                reductions[target_index] = Some(CertifiedPolynomialQ {
                    polynomial: polynomial.clone(),
                    certificate: reduced.certificate.clone(),
                });
                pivots.push(CertifiedPolynomialQ {
                    polynomial,
                    certificate: reduced.certificate,
                });
            }
        } else if !polynomial.terms.is_empty() {
            if !verify_membership_by_certificate(
                &polynomial,
                &reduced.certificate,
                authorized_relations,
            ) {
                return Err(implementation_bug(
                    "F4 matrix row reduction failed exact reducer certificate verification",
                ));
            }
            pivots.push(CertifiedPolynomialQ {
                polynomial,
                certificate: reduced.certificate,
            });
        }
    }
    Ok(reductions
        .into_iter()
        .map(|entry| {
            entry.unwrap_or_else(|| CertifiedPolynomialQ {
                polynomial: crate::types::polynomial::zero_poly(),
                certificate: crate::algebra::normal_form::MembershipCertificate {
                    combination_terms: Vec::new(),
                },
            })
        })
        .collect())
}

fn reduce_row_by_pivots(
    mut row: CertifiedPolynomialQ,
    pivots: &[CertifiedPolynomialQ],
    order: &MonomialOrder,
) -> Result<CertifiedPolynomialQ, SolverError> {
    while let Some(lt_row) = leading_term(&row.polynomial, order) {
        let Some(pivot) = pivots.iter().find(|pivot| {
            leading_term(&pivot.polynomial, order)
                .is_some_and(|lt_pivot| lt_pivot.monomial == lt_row.monomial)
        }) else {
            break;
        };
        let lt_pivot = leading_term(&pivot.polynomial, order)
            .ok_or_else(|| implementation_bug("F4 pivot unexpectedly has no leading term"))?;
        let scale = div_q(&lt_row.coeff, &lt_pivot.coeff)
            .map_err(|_| implementation_bug("F4 pivot has zero leading coefficient"))?;
        if is_zero_q(&scale) {
            break;
        }
        let multiplier = multiplier_poly(scale, normalize_monomial(Vec::new()));
        let scaled_pivot = poly_scale(&pivot.polynomial, &multiplier.terms[0].coeff);
        row.polynomial = poly_sub(&row.polynomial, &scaled_pivot);
        let scaled_cert = scale_membership_certificate(&pivot.certificate, &multiplier);
        row.certificate = subtract_membership_certificate(&row.certificate, &scaled_cert);
    }
    row.polynomial = normalize_poly(row.polynomial);
    Ok(row)
}

fn multiplier_poly(coeff: RationalQ, monomial: Monomial) -> SparsePolynomialQ {
    if is_zero_q(&coeff) {
        return crate::types::polynomial::zero_poly();
    }
    normalize_poly(SparsePolynomialQ {
        terms: vec![TermQ { coeff, monomial }],
        hash: hash_sequence("poly", &[]),
    })
}

fn symbolic_preprocessing_matrix(rows: &[F4MatrixRow]) -> SparseMatrixQ {
    let mut monomial_to_col = BTreeMap::<Monomial, usize>::new();
    for row in rows {
        collect_monomials(&row.entry.polynomial, &mut monomial_to_col);
    }
    let row_count = rows.len();
    let col_count = monomial_to_col.len();
    let mut entries = Vec::new();
    for (row_idx, row) in rows.iter().enumerate() {
        append_matrix_row(
            row_idx,
            &row.entry.polynomial,
            &monomial_to_col,
            &mut entries,
        );
    }
    SparseMatrixQ {
        rows: row_count,
        cols: col_count,
        entries,
    }
}

fn collect_monomials(poly: &SparsePolynomialQ, columns: &mut BTreeMap<Monomial, usize>) {
    for term in &poly.terms {
        let next = columns.len();
        columns.entry(term.monomial.clone()).or_insert(next);
    }
}

fn append_matrix_row(
    row: usize,
    poly: &SparsePolynomialQ,
    columns: &BTreeMap<Monomial, usize>,
    entries: &mut Vec<(usize, usize, crate::types::rational::RationalQ)>,
) {
    for term in &poly.terms {
        if let Some(col) = columns.get(&term.monomial) {
            entries.push((row, *col, term.coeff.clone()));
        }
    }
}

fn hash_f4_matrix_trace(
    matrix_rows: usize,
    matrix_cols: usize,
    reducer_count: usize,
    target_count: usize,
) -> crate::types::hash::Hash {
    hash_sequence(
        "f4-batch-matrix-trace",
        &[
            matrix_rows.to_be_bytes().to_vec(),
            matrix_cols.to_be_bytes().to_vec(),
            reducer_count.to_be_bytes().to_vec(),
            target_count.to_be_bytes().to_vec(),
        ],
    )
}

fn f4_resource_failure(
    pairs_processed: usize,
    basis_len: usize,
    matrix_rows: usize,
    matrix_cols: usize,
) -> SolverError {
    SolverError {
        target: None,
        kind: SolverErrorKind::Failure(FailureKind::FiniteResourceFailure {
            stage: StageId("F4EliminationLocalBatchLimit".to_string()),
            block_id: None,
            matrix_rows: Some(matrix_rows),
            matrix_cols: Some(matrix_cols),
            matrix_density: None,
            quotient_rank_estimate: Some(basis_len),
            coefficient_height_bits: None,
            memory_bytes: Some(
                pairs_processed
                    .saturating_mul(matrix_rows.max(1))
                    .saturating_mul(matrix_cols.max(1))
                    .min(u64::MAX as usize) as u64,
            ),
        }),
    }
}

#[allow(dead_code)]
fn generator_variables(result: &LocalEliminationResult) -> Vec<VariableId> {
    let mut vars = result
        .generators
        .iter()
        .flat_map(|generator| poly_variables(&generator.generator))
        .collect::<Vec<_>>();
    vars.sort();
    vars.dedup();
    vars
}

#[cfg(test)]
mod tests {
    use crate::algebra::elimination::{
        validate_local_elimination_result, LocalEliminationStrategyName,
    };
    use crate::algebra::monomial_order::{elimination_order, lex_order};
    use crate::types::polynomial::{constant_poly, poly_sub, variable_poly};

    use super::*;

    #[test]
    fn f4_reduce_batch_builds_matrix_and_exact_remainder_certificates() {
        let x = VariableId(1);
        let reducer = CertifiedPolynomialQ {
            polynomial: poly_sub(&variable_poly(x), &constant_poly(int_q(1))),
            certificate: crate::algebra::normal_form::MembershipCertificate {
                combination_terms: vec![crate::algebra::normal_form::MembershipTerm {
                    relation_id: 0,
                    multiplier: constant_poly(int_q(1)),
                }],
            },
        };
        let target = CertifiedPolynomialQ {
            polynomial: poly_sub(
                &crate::types::polynomial::poly_mul(&variable_poly(x), &variable_poly(x)),
                &constant_poly(int_q(1)),
            ),
            certificate: crate::algebra::normal_form::MembershipCertificate {
                combination_terms: vec![crate::algebra::normal_form::MembershipTerm {
                    relation_id: 0,
                    multiplier: crate::types::polynomial::poly_add(
                        &variable_poly(x),
                        &constant_poly(int_q(1)),
                    ),
                }],
            },
        };
        let result = f4_reduce_batch(
            &[reducer.clone()],
            &[target],
            &[reducer.polynomial.clone()],
            &lex_order(&[x]),
            &F4Options::default(),
        )
        .unwrap();

        assert!(result.exact_certificates_verified);
        assert_eq!(result.matrix_trace.reducer_count, 1);
        assert_eq!(result.matrix_trace.target_count, 1);
        assert!(!result.matrix_trace.modular_traces.is_empty());
    }

    #[test]
    fn f4_elimination_exports_keep_only_generators_with_certificates() {
        let x = VariableId(1);
        let y = VariableId(2);
        let relations = vec![
            poly_sub(&variable_poly(y), &variable_poly(x)),
            poly_sub(&variable_poly(y), &constant_poly(int_q(1))),
        ];

        let result = f4_elimination_local(&relations, &[y], &[x], F4Options::default()).unwrap();

        assert_eq!(
            result.strategy,
            LocalEliminationStrategyName::F4EliminationLocal
        );
        assert!(!result.generators.is_empty());
        assert!(result.matrix_rows > 0);
        assert!(result.matrix_cols > 0);
        validate_local_elimination_result(&result, &[x], &relations).unwrap();
    }

    #[test]
    fn f4_and_groebner_agree_on_small_eliminant() {
        let x = VariableId(1);
        let y = VariableId(2);
        let relations = vec![
            poly_sub(&variable_poly(y), &variable_poly(x)),
            poly_sub(&variable_poly(y), &constant_poly(int_q(1))),
        ];
        let f4 = f4_elimination_local(&relations, &[y], &[x], F4Options::default()).unwrap();
        let order = elimination_order(&[y], &[x]);
        let groebner = crate::algebra::groebner::groebner_elimination_basis(
            &relations,
            &order,
            crate::algebra::groebner::GroebnerOptions::default(),
        )
        .unwrap();
        let groebner = crate::algebra::elimination::local_result_from_groebner(
            &relations,
            &[x],
            groebner,
            LocalEliminationStrategyName::EliminationGroebnerLocal,
        )
        .unwrap();
        let f4_hashes = f4
            .generators
            .iter()
            .map(|generator| generator.generator.hash)
            .collect::<Vec<_>>();
        let groebner_hashes = groebner
            .generators
            .iter()
            .map(|generator| generator.generator.hash)
            .collect::<Vec<_>>();
        assert_eq!(f4_hashes, groebner_hashes);
    }
}
