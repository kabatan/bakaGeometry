use serde::{Deserialize, Serialize};

use num_bigint::BigInt;
use num_traits::One;

use crate::algebra::crt::{crt_combine, ModInteger};
use crate::algebra::modular::{
    choose_prime_avoiding_denominators, inv_mod_u64, mul_mod, reduce_rational_coeff, sub_mod, Prime,
};
use crate::algebra::rational_reconstruction::reconstruct_rational;
use crate::algebra::sparse_matrix::{nullspace_sparse_fp, row_echelon_sparse_fp};
use crate::types::hash::hash_sequence;
use crate::types::matrix::{SparseMatrixFp, SparseMatrixQ, VectorFp, VectorQ};
use crate::types::polynomial::{SparsePolynomialQ, TermQ};
use crate::types::rational::RationalQ;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatrixBuilder {
    pub matrix: SparseMatrixQ,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModularSolvePlan {
    pub seed: u64,
    pub max_primes: usize,
    pub stable_rank_after: usize,
    pub reconstruction_height_bound: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimeSolveTrace {
    pub prime: Prime,
    pub rank: usize,
    pub nullity: usize,
    pub pivot_columns: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModularProofStatus {
    CandidateOnlyRequiresExactQCheck,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModularNullspaceResult {
    pub traces: Vec<PrimeSolveTrace>,
    pub rank: usize,
    pub basis_mod_prime: Vec<VectorFp>,
    pub reconstructed_basis_candidates: Vec<VectorQ>,
    pub proof_status: ModularProofStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModularSolveResult {
    pub traces: Vec<PrimeSolveTrace>,
    pub rank: usize,
    pub solution_mod_prime: Option<VectorFp>,
    pub reconstructed_solution_candidate: Option<VectorQ>,
    pub proof_status: ModularProofStatus,
}

pub fn solve_homogeneous_modular(
    matrix_builder: MatrixBuilder,
    plan: ModularSolvePlan,
) -> ModularNullspaceResult {
    let matrices = modular_matrices(&matrix_builder.matrix, &plan, &[]);
    let mut traces = Vec::new();
    let mut last_basis = Vec::new();
    let mut basis_samples = Vec::new();
    let mut stable_rank = 0;
    let mut stability_achieved = false;

    for (idx, (prime, matrix)) in matrices.iter().enumerate() {
        let echelon = row_echelon_sparse_fp(matrix, *prime);
        let rank = echelon.rank;
        let nullity = matrix.cols.saturating_sub(rank);
        traces.push(PrimeSolveTrace {
            prime: *prime,
            rank,
            nullity,
            pivot_columns: echelon.pivot_columns,
        });
        last_basis = nullspace_sparse_fp(matrix, *prime);
        if idx > 0 && traces[idx - 1].pivot_columns == traces[idx].pivot_columns {
            stable_rank += 1;
            basis_samples.push((*prime, last_basis.clone()));
        } else {
            stable_rank = 1;
            basis_samples.clear();
            basis_samples.push((*prime, last_basis.clone()));
        }
        if stable_rank >= plan.stable_rank_after.max(1) {
            stability_achieved = true;
            break;
        }
    }
    let rank = traces.last().map_or(0, |trace| trace.rank);
    ModularNullspaceResult {
        traces,
        rank,
        basis_mod_prime: last_basis,
        reconstructed_basis_candidates: if stability_achieved {
            reconstruct_vector_family(&basis_samples, plan.reconstruction_height_bound)
        } else {
            Vec::new()
        },
        proof_status: ModularProofStatus::CandidateOnlyRequiresExactQCheck,
    }
}

pub fn solve_inhomogeneous_modular(
    matrix_builder: MatrixBuilder,
    rhs: VectorQ,
    plan: ModularSolvePlan,
) -> ModularSolveResult {
    let matrices = modular_matrices(&matrix_builder.matrix, &plan, &rhs.entries);
    let mut traces = Vec::new();
    let mut last_solution = None;
    let mut solution_samples = Vec::new();
    let mut last_rank = 0;
    let mut stable_rank = 0;
    let mut stability_achieved = false;

    for (idx, (prime, matrix)) in matrices.iter().enumerate() {
        let rhs_fp = reduce_vector_q(&rhs, *prime);
        let (rank, pivot_columns, solution) = solve_dense_mod(matrix, &rhs_fp, *prime);
        let nullity = matrix.cols.saturating_sub(rank);
        traces.push(PrimeSolveTrace {
            prime: *prime,
            rank,
            nullity,
            pivot_columns,
        });
        last_rank = rank;
        last_solution = solution;
        if idx > 0 && traces[idx - 1].pivot_columns == traces[idx].pivot_columns {
            stable_rank += 1;
            if let Some(solution) = &last_solution {
                solution_samples.push((*prime, solution.clone()));
            }
        } else {
            stable_rank = 1;
            solution_samples.clear();
            if let Some(solution) = &last_solution {
                solution_samples.push((*prime, solution.clone()));
            }
        }
        if stable_rank >= plan.stable_rank_after.max(1) {
            stability_achieved = true;
            break;
        }
    }

    ModularSolveResult {
        traces,
        rank: last_rank,
        solution_mod_prime: last_solution,
        reconstructed_solution_candidate: if stability_achieved {
            reconstruct_vector_samples(&solution_samples, plan.reconstruction_height_bound)
        } else {
            None
        },
        proof_status: ModularProofStatus::CandidateOnlyRequiresExactQCheck,
    }
}

fn modular_matrices(
    matrix: &SparseMatrixQ,
    plan: &ModularSolvePlan,
    extra_coefficients: &[RationalQ],
) -> Vec<(Prime, SparseMatrixFp)> {
    let mut coeff_polys = matrix
        .entries
        .iter()
        .map(|(_, _, coeff)| constant_poly_for_prime_choice(coeff.clone()))
        .collect::<Vec<_>>();
    coeff_polys.extend(
        extra_coefficients
            .iter()
            .cloned()
            .map(constant_poly_for_prime_choice),
    );
    let mut search_seed = plan.seed;
    let count = plan.max_primes.max(1);
    let mut out = Vec::new();
    for _ in 0..count {
        let prime = choose_prime_avoiding_denominators(&coeff_polys, search_seed);
        out.push((prime, reduce_matrix_q_to_fp(matrix, prime)));
        search_seed = prime.saturating_add(1);
    }
    out
}

fn reduce_matrix_q_to_fp(matrix: &SparseMatrixQ, prime: Prime) -> SparseMatrixFp {
    let entries = matrix
        .entries
        .iter()
        .filter_map(|(r, c, value)| {
            let coeff = reduce_rational_coeff(&value.num, &value.den, prime);
            if coeff == 0 {
                None
            } else {
                Some((*r, *c, coeff))
            }
        })
        .collect();
    SparseMatrixFp {
        rows: matrix.rows,
        cols: matrix.cols,
        entries,
    }
}

fn reduce_vector_q(rhs: &VectorQ, prime: Prime) -> Vec<u64> {
    rhs.entries
        .iter()
        .map(|value| reduce_rational_coeff(&value.num, &value.den, prime))
        .collect()
}

fn solve_dense_mod(
    matrix: &SparseMatrixFp,
    rhs: &[u64],
    prime: Prime,
) -> (usize, Vec<usize>, Option<VectorFp>) {
    let mut augmented = crate::algebra::sparse_matrix::sparse_to_dense(matrix, prime);
    for (row, rhs_value) in augmented.iter_mut().zip(rhs.iter().copied()) {
        row.push(rhs_value % prime);
    }
    let pivot_columns = rref_augmented(&mut augmented, matrix.cols, prime);
    for row in &augmented {
        if row[..matrix.cols].iter().all(|value| *value % prime == 0)
            && row[matrix.cols] % prime != 0
        {
            return (pivot_columns.len(), pivot_columns, None);
        }
    }
    let mut solution = vec![0; matrix.cols];
    for (row, pivot_col) in pivot_columns.iter().copied().enumerate() {
        if pivot_col < matrix.cols {
            solution[pivot_col] = augmented[row][matrix.cols] % prime;
        }
    }
    (
        pivot_columns.len(),
        pivot_columns,
        Some(VectorFp { entries: solution }),
    )
}

fn rref_augmented(matrix: &mut [Vec<u64>], coefficient_cols: usize, prime: Prime) -> Vec<usize> {
    let rows = matrix.len();
    let mut pivot_row = 0;
    let mut pivot_columns = Vec::new();
    for col in 0..coefficient_cols {
        let Some(found) = (pivot_row..rows).find(|r| matrix[*r][col] % prime != 0) else {
            continue;
        };
        matrix.swap(pivot_row, found);
        let inv = inv_mod_u64(matrix[pivot_row][col], prime).unwrap();
        for c in col..=coefficient_cols {
            matrix[pivot_row][c] = mul_mod(matrix[pivot_row][c], inv, prime);
        }
        for r in 0..rows {
            if r == pivot_row {
                continue;
            }
            let factor = matrix[r][col] % prime;
            if factor == 0 {
                continue;
            }
            for c in col..=coefficient_cols {
                let scaled = mul_mod(factor, matrix[pivot_row][c], prime);
                matrix[r][c] = sub_mod(matrix[r][c], scaled, prime);
            }
        }
        pivot_columns.push(col);
        pivot_row += 1;
        if pivot_row == rows {
            break;
        }
    }
    pivot_columns
}

fn constant_poly_for_prime_choice(coeff: RationalQ) -> SparsePolynomialQ {
    SparsePolynomialQ {
        terms: vec![TermQ {
            coeff,
            monomial: crate::types::monomial::normalize_monomial(Vec::new()),
        }],
        hash: hash_sequence("poly", &[]),
    }
}

fn reconstruct_vector_family(
    samples: &[(Prime, Vec<VectorFp>)],
    height_bound: Option<usize>,
) -> Vec<VectorQ> {
    let Some((_, first_family)) = samples.first() else {
        return Vec::new();
    };
    let family_len = first_family.len();
    if samples.iter().any(|(_, family)| family.len() != family_len) {
        return Vec::new();
    }
    let mut reconstructed = Vec::new();
    for index in 0..family_len {
        let vector_samples: Vec<(Prime, VectorFp)> = samples
            .iter()
            .map(|(prime, family)| (*prime, family[index].clone()))
            .collect();
        let Some(vector) = reconstruct_vector_samples(&vector_samples, height_bound) else {
            return Vec::new();
        };
        reconstructed.push(vector);
    }
    reconstructed
}

fn reconstruct_vector_samples(
    samples: &[(Prime, VectorFp)],
    height_bound: Option<usize>,
) -> Option<VectorQ> {
    let (_, first) = samples.first()?;
    let width = first.entries.len();
    if samples
        .iter()
        .any(|(_, vector)| vector.entries.len() != width)
    {
        return None;
    }

    let mut residues = vec![BigInt::from(0); width];
    let mut modulus = BigInt::one();
    for (prime, vector) in samples {
        let prime_modulus = BigInt::from(*prime);
        for (idx, residue) in residues.iter_mut().enumerate() {
            let combined = crt_combine(
                ModInteger {
                    value: residue.clone(),
                    modulus: modulus.clone(),
                },
                ModInteger {
                    value: BigInt::from(vector.entries[idx]),
                    modulus: prime_modulus.clone(),
                },
            );
            *residue = combined.value;
        }
        modulus *= prime_modulus;
    }

    let entries = residues
        .into_iter()
        .map(|value| {
            reconstruct_rational(
                ModInteger {
                    value,
                    modulus: modulus.clone(),
                },
                modulus.clone(),
                height_bound,
            )
        })
        .collect::<Option<Vec<_>>>()?;
    Some(VectorQ { entries })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::matrix::SparseMatrixQ;
    use crate::types::rational::{add_q, int_q, mul_q, new_q};

    #[test]
    fn homogeneous_modular_solve_exposes_candidate_trace_only() {
        let matrix = SparseMatrixQ {
            rows: 2,
            cols: 3,
            entries: vec![
                (0, 0, int_q(1)),
                (0, 1, int_q(2)),
                (0, 2, int_q(3)),
                (1, 0, int_q(2)),
                (1, 1, int_q(4)),
                (1, 2, int_q(6)),
            ],
        };
        let result = solve_homogeneous_modular(
            MatrixBuilder { matrix },
            ModularSolvePlan {
                seed: 5,
                max_primes: 3,
                stable_rank_after: 2,
                reconstruction_height_bound: Some(3),
            },
        );
        assert_eq!(result.rank, 1);
        assert_eq!(result.basis_mod_prime.len(), 2);
        assert_eq!(
            result.proof_status,
            ModularProofStatus::CandidateOnlyRequiresExactQCheck
        );
        assert!(!result.traces.is_empty());
        assert_eq!(result.reconstructed_basis_candidates.len(), 2);
    }

    #[test]
    fn inhomogeneous_modular_solve_returns_candidate_solution_not_proof() {
        let matrix = SparseMatrixQ {
            rows: 2,
            cols: 2,
            entries: vec![(0, 0, int_q(1)), (1, 1, int_q(1))],
        };
        let rhs = VectorQ {
            entries: vec![int_q(2), int_q(3)],
        };
        let result = solve_inhomogeneous_modular(
            MatrixBuilder { matrix },
            rhs,
            ModularSolvePlan {
                seed: 5,
                max_primes: 2,
                stable_rank_after: 2,
                reconstruction_height_bound: Some(3),
            },
        );
        assert_eq!(result.solution_mod_prime.unwrap().entries, vec![2, 3]);
        assert_eq!(
            result.reconstructed_solution_candidate.unwrap().entries,
            vec![int_q(2), int_q(3)]
        );
        assert_eq!(
            result.proof_status,
            ModularProofStatus::CandidateOnlyRequiresExactQCheck
        );
    }

    #[test]
    fn every_prime_in_solve_sequence_avoids_matrix_denominators_and_coefficients() {
        let matrix = SparseMatrixQ {
            rows: 1,
            cols: 1,
            entries: vec![(0, 0, crate::types::rational::new_q(5.into(), 3.into()))],
        };
        let result = solve_homogeneous_modular(
            MatrixBuilder { matrix },
            ModularSolvePlan {
                seed: 2,
                max_primes: 3,
                stable_rank_after: 3,
                reconstruction_height_bound: Some(3),
            },
        );
        let primes: Vec<_> = result.traces.iter().map(|trace| trace.prime).collect();
        assert_eq!(primes, vec![2, 7, 11]);
    }

    #[test]
    fn inhomogeneous_prime_sequence_avoids_rhs_denominators() {
        let matrix = SparseMatrixQ {
            rows: 1,
            cols: 1,
            entries: vec![(0, 0, int_q(1))],
        };
        let rhs = VectorQ {
            entries: vec![crate::types::rational::new_q(1.into(), 3.into())],
        };
        let result = solve_inhomogeneous_modular(
            MatrixBuilder { matrix },
            rhs,
            ModularSolvePlan {
                seed: 3,
                max_primes: 1,
                stable_rank_after: 1,
                reconstruction_height_bound: Some(3),
            },
        );
        assert_eq!(result.traces[0].prime, 5);
    }

    #[test]
    fn modular_solve_waits_for_stable_rank_profile_not_rank_only() {
        let matrix = SparseMatrixQ {
            rows: 2,
            cols: 3,
            entries: vec![
                (0, 0, int_q(1)),
                (0, 1, int_q(1)),
                (0, 2, int_q(1)),
                (1, 0, int_q(1)),
                (1, 1, int_q(6)),
                (1, 2, int_q(2)),
            ],
        };
        let result = solve_homogeneous_modular(
            MatrixBuilder { matrix },
            ModularSolvePlan {
                seed: 2,
                max_primes: 3,
                stable_rank_after: 2,
                reconstruction_height_bound: Some(3),
            },
        );
        assert_eq!(result.traces[0].prime, 5);
        assert_eq!(result.traces[0].rank, result.traces[1].rank);
        assert_eq!(result.traces[0].rank, 2);
        assert_ne!(
            result.traces[0].pivot_columns,
            result.traces[1].pivot_columns
        );
        assert_eq!(result.traces.len(), 3);
        assert_eq!(result.traces[0].pivot_columns, vec![0, 2]);
        assert_eq!(result.traces[1].pivot_columns, vec![0, 1]);
        assert_eq!(
            result.traces[1].pivot_columns,
            result.traces[2].pivot_columns
        );
        assert_eq!(result.reconstructed_basis_candidates.len(), 1);
        let candidate = &result.reconstructed_basis_candidates[0].entries;
        assert_eq!(candidate.len(), 3);
        assert_eq!(candidate[0], new_q((-4).into(), 5.into()));
        assert_eq!(candidate[1], new_q((-1).into(), 5.into()));
        assert_eq!(candidate[2], int_q(1));
        assert_eq!(
            add_q(&add_q(&candidate[0], &candidate[1]), &candidate[2]),
            int_q(0)
        );
        assert_eq!(
            add_q(
                &add_q(&candidate[0], &mul_q(&int_q(6), &candidate[1])),
                &mul_q(&int_q(2), &candidate[2])
            ),
            int_q(0)
        );
    }

    #[test]
    fn inhomogeneous_reconstruction_uses_only_stable_rank_profile_suffix() {
        let matrix = SparseMatrixQ {
            rows: 2,
            cols: 3,
            entries: vec![
                (0, 0, int_q(1)),
                (0, 1, int_q(1)),
                (0, 2, int_q(1)),
                (1, 0, int_q(1)),
                (1, 1, int_q(6)),
                (1, 2, int_q(2)),
            ],
        };
        let rhs = VectorQ {
            entries: vec![int_q(1), int_q(2)],
        };
        let result = solve_inhomogeneous_modular(
            MatrixBuilder { matrix },
            rhs,
            ModularSolvePlan {
                seed: 5,
                max_primes: 3,
                stable_rank_after: 2,
                reconstruction_height_bound: Some(3),
            },
        );
        assert_eq!(result.traces[0].pivot_columns, vec![0, 2]);
        assert_eq!(result.traces[1].pivot_columns, vec![0, 1]);
        assert_eq!(result.traces[2].pivot_columns, vec![0, 1]);
        let candidate = result.reconstructed_solution_candidate.unwrap();
        assert_eq!(
            candidate.entries,
            vec![
                new_q(4.into(), 5.into()),
                new_q(1.into(), 5.into()),
                int_q(0)
            ]
        );
        assert_eq!(
            add_q(
                &add_q(&candidate.entries[0], &candidate.entries[1]),
                &candidate.entries[2]
            ),
            int_q(1)
        );
        assert_eq!(
            add_q(
                &add_q(
                    &candidate.entries[0],
                    &mul_q(&int_q(6), &candidate.entries[1])
                ),
                &mul_q(&int_q(2), &candidate.entries[2])
            ),
            int_q(2)
        );
    }

    #[test]
    fn homogeneous_does_not_reconstruct_without_stability() {
        let matrix = SparseMatrixQ {
            rows: 2,
            cols: 3,
            entries: vec![
                (0, 0, int_q(1)),
                (0, 1, int_q(1)),
                (0, 2, int_q(1)),
                (1, 0, int_q(1)),
                (1, 1, int_q(6)),
                (1, 2, int_q(2)),
            ],
        };
        let result = solve_homogeneous_modular(
            MatrixBuilder { matrix },
            ModularSolvePlan {
                seed: 5,
                max_primes: 2,
                stable_rank_after: 3,
                reconstruction_height_bound: Some(3),
            },
        );

        assert_eq!(result.traces.len(), 2);
        assert!(result.reconstructed_basis_candidates.is_empty());
    }

    #[test]
    fn inhomogeneous_does_not_reconstruct_without_stability() {
        let matrix = SparseMatrixQ {
            rows: 2,
            cols: 3,
            entries: vec![
                (0, 0, int_q(1)),
                (0, 1, int_q(1)),
                (0, 2, int_q(1)),
                (1, 0, int_q(1)),
                (1, 1, int_q(6)),
                (1, 2, int_q(2)),
            ],
        };
        let rhs = VectorQ {
            entries: vec![int_q(1), int_q(2)],
        };
        let result = solve_inhomogeneous_modular(
            MatrixBuilder { matrix },
            rhs,
            ModularSolvePlan {
                seed: 5,
                max_primes: 2,
                stable_rank_after: 3,
                reconstruction_height_bound: Some(3),
            },
        );

        assert_eq!(result.traces.len(), 2);
        assert!(result.reconstructed_solution_candidate.is_none());
    }
}
