use serde::{Deserialize, Serialize};

use crate::algebra::modular::{inv_mod_u64, mul_mod, sub_mod, Prime};
use crate::types::matrix::{SparseMatrixFp, VectorFp};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SparseRowFp {
    pub entries: Vec<(usize, u64)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EchelonResultFp {
    pub matrix: SparseMatrixFp,
    pub pivot_columns: Vec<usize>,
    pub rank: usize,
    pub prime: Prime,
}

pub fn build_sparse_matrix_fp(rows: Vec<SparseRowFp>, ncols: usize) -> SparseMatrixFp {
    let row_count = rows.len();
    let mut entries = Vec::new();
    for (r, row) in rows.into_iter().enumerate() {
        for (c, value) in row.entries {
            if value != 0 {
                entries.push((r, c, value));
            }
        }
    }
    entries.sort_by_key(|(r, c, _)| (*r, *c));
    SparseMatrixFp {
        rows: row_count,
        cols: ncols,
        entries,
    }
}

pub fn row_echelon_sparse_fp(m: &SparseMatrixFp, prime: Prime) -> EchelonResultFp {
    let mut dense = sparse_to_dense(m, prime);
    let pivot_columns = rref_in_place(&mut dense, prime);
    EchelonResultFp {
        matrix: dense_to_sparse(&dense, prime),
        rank: pivot_columns.len(),
        pivot_columns,
        prime,
    }
}

pub fn nullspace_sparse_fp(m: &SparseMatrixFp, prime: Prime) -> Vec<VectorFp> {
    let echelon = row_echelon_sparse_fp(m, prime);
    let dense = sparse_to_dense(&echelon.matrix, prime);
    let mut pivot_for_col = vec![None; m.cols];
    for (row, col) in echelon.pivot_columns.iter().copied().enumerate() {
        pivot_for_col[col] = Some(row);
    }
    let mut basis = Vec::new();
    for free_col in 0..m.cols {
        if pivot_for_col[free_col].is_some() {
            continue;
        }
        let mut vector = vec![0; m.cols];
        vector[free_col] = 1 % prime;
        for (row, pivot_col) in echelon.pivot_columns.iter().copied().enumerate() {
            let coeff = dense[row][free_col] % prime;
            vector[pivot_col] = if coeff == 0 { 0 } else { prime - coeff };
        }
        basis.push(VectorFp { entries: vector });
    }
    basis
}

pub fn rank_sparse_fp(m: &SparseMatrixFp, prime: Prime) -> usize {
    row_echelon_sparse_fp(m, prime).rank
}

pub(crate) fn sparse_to_dense(m: &SparseMatrixFp, prime: Prime) -> Vec<Vec<u64>> {
    let mut dense = vec![vec![0; m.cols]; m.rows];
    for (r, c, value) in &m.entries {
        dense[*r][*c] = (dense[*r][*c] + value % prime) % prime;
    }
    dense
}

pub(crate) fn dense_to_sparse(dense: &[Vec<u64>], prime: Prime) -> SparseMatrixFp {
    let rows = dense.len();
    let cols = dense.first().map_or(0, Vec::len);
    let mut entries = Vec::new();
    for (r, row) in dense.iter().enumerate() {
        for (c, value) in row.iter().copied().enumerate() {
            let v = value % prime;
            if v != 0 {
                entries.push((r, c, v));
            }
        }
    }
    SparseMatrixFp {
        rows,
        cols,
        entries,
    }
}

pub(crate) fn rref_in_place(matrix: &mut [Vec<u64>], prime: Prime) -> Vec<usize> {
    let rows = matrix.len();
    let cols = matrix.first().map_or(0, Vec::len);
    let mut pivot_row = 0;
    let mut pivot_columns = Vec::new();

    for col in 0..cols {
        let Some(found) = (pivot_row..rows).find(|r| matrix[*r][col] % prime != 0) else {
            continue;
        };
        matrix.swap(pivot_row, found);
        let inv = inv_mod_u64(matrix[pivot_row][col], prime).unwrap();
        for c in col..cols {
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
            for c in col..cols {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sparse_rank_and_nullspace_are_exact_mod_prime() {
        let m = build_sparse_matrix_fp(
            vec![
                SparseRowFp {
                    entries: vec![(0, 1), (1, 2), (2, 3)],
                },
                SparseRowFp {
                    entries: vec![(0, 2), (1, 4), (2, 6)],
                },
            ],
            3,
        );
        assert_eq!(rank_sparse_fp(&m, 5), 1);
        let ns = nullspace_sparse_fp(&m, 5);
        assert_eq!(ns.len(), 2);
        for v in ns {
            let dot = (v.entries[0] + 2 * v.entries[1] + 3 * v.entries[2]) % 5;
            assert_eq!(dot, 0);
        }
    }
}
