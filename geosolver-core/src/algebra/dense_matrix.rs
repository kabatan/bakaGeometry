use serde::{Deserialize, Serialize};

use crate::algebra::modular::Prime;
use crate::algebra::sparse_matrix::{dense_to_sparse, nullspace_sparse_fp, rref_in_place};
use crate::types::matrix::{DenseMatrixFp, VectorFp};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DenseEchelonResultFp {
    pub matrix: DenseMatrixFp,
    pub pivot_columns: Vec<usize>,
    pub rank: usize,
    pub prime: Prime,
}

pub fn build_dense_matrix_fp(
    rows: usize,
    cols: usize,
    entries_row_major: Vec<u64>,
) -> DenseMatrixFp {
    assert_eq!(
        entries_row_major.len(),
        rows * cols,
        "dense matrix entry count must match shape"
    );
    DenseMatrixFp {
        rows,
        cols,
        entries_row_major,
    }
}

pub fn row_echelon_dense_fp(m: &DenseMatrixFp, prime: Prime) -> DenseEchelonResultFp {
    let mut dense = dense_matrix_to_rows(m, prime);
    let pivot_columns = rref_in_place(&mut dense, prime);
    DenseEchelonResultFp {
        matrix: rows_to_dense_matrix(&dense, prime),
        rank: pivot_columns.len(),
        pivot_columns,
        prime,
    }
}

pub fn nullspace_dense_fp(m: &DenseMatrixFp, prime: Prime) -> Vec<VectorFp> {
    nullspace_sparse_fp(
        &dense_to_sparse(&dense_matrix_to_rows(m, prime), prime),
        prime,
    )
}

pub fn rank_dense_fp(m: &DenseMatrixFp, prime: Prime) -> usize {
    row_echelon_dense_fp(m, prime).rank
}

fn dense_matrix_to_rows(m: &DenseMatrixFp, prime: Prime) -> Vec<Vec<u64>> {
    let mut rows = vec![vec![0; m.cols]; m.rows];
    for r in 0..m.rows {
        for c in 0..m.cols {
            rows[r][c] = m.entries_row_major[r * m.cols + c] % prime;
        }
    }
    rows
}

fn rows_to_dense_matrix(rows: &[Vec<u64>], prime: Prime) -> DenseMatrixFp {
    let row_count = rows.len();
    let col_count = rows.first().map_or(0, Vec::len);
    let mut entries = Vec::with_capacity(row_count * col_count);
    for row in rows {
        for value in row {
            entries.push(value % prime);
        }
    }
    DenseMatrixFp {
        rows: row_count,
        cols: col_count,
        entries_row_major: entries,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dense_rank_matches_sparse_algorithm() {
        let m = build_dense_matrix_fp(2, 3, vec![1, 2, 3, 2, 4, 6]);
        assert_eq!(rank_dense_fp(&m, 5), 1);
        assert_eq!(nullspace_dense_fp(&m, 5).len(), 2);
    }
}
