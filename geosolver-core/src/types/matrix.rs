use std::collections::BTreeMap;

use num_bigint::BigInt;
use serde::{Deserialize, Serialize};

use crate::types::hash::{hash_sequence, Hash};
use crate::types::rational::{add_q, int_q, is_zero_q, rational_to_bytes, RationalQ};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SparseMatrixQ {
    pub rows: usize,
    pub cols: usize,
    pub entries: Vec<(usize, usize, RationalQ)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SparseMatrixFp {
    pub rows: usize,
    pub cols: usize,
    pub entries: Vec<(usize, usize, u64)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DenseMatrixFp {
    pub rows: usize,
    pub cols: usize,
    pub entries_row_major: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VectorQ {
    pub entries: Vec<RationalQ>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VectorFp {
    pub entries: Vec<u64>,
}

pub trait MatrixShape {
    fn shape(&self) -> (usize, usize);
    fn nonzero_count(&self) -> usize;
    fn canonical_bytes(&self) -> Vec<u8>;
}

impl MatrixShape for SparseMatrixQ {
    fn shape(&self) -> (usize, usize) {
        (self.rows, self.cols)
    }

    fn nonzero_count(&self) -> usize {
        canonical_sparse_q_entries(self).len()
    }

    fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.rows.to_be_bytes().as_slice());
        bytes.extend_from_slice(self.cols.to_be_bytes().as_slice());
        for (r, c, v) in canonical_sparse_q_entries(self) {
            bytes.extend_from_slice(r.to_be_bytes().as_slice());
            bytes.extend_from_slice(c.to_be_bytes().as_slice());
            bytes.extend_from_slice(&rational_to_bytes(&v));
        }
        bytes
    }
}

impl MatrixShape for SparseMatrixFp {
    fn shape(&self) -> (usize, usize) {
        (self.rows, self.cols)
    }

    fn nonzero_count(&self) -> usize {
        canonical_sparse_fp_entries(self).len()
    }

    fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.rows.to_be_bytes().as_slice());
        bytes.extend_from_slice(self.cols.to_be_bytes().as_slice());
        for (r, c, values) in canonical_sparse_fp_entries(self) {
            bytes.extend_from_slice(r.to_be_bytes().as_slice());
            bytes.extend_from_slice(c.to_be_bytes().as_slice());
            bytes.extend_from_slice(values.len().to_be_bytes().as_slice());
            for value in values {
                bytes.extend_from_slice(value.to_be_bytes().as_slice());
            }
        }
        bytes
    }
}

impl MatrixShape for DenseMatrixFp {
    fn shape(&self) -> (usize, usize) {
        (self.rows, self.cols)
    }

    fn nonzero_count(&self) -> usize {
        self.entries_row_major.iter().filter(|v| **v != 0).count()
    }

    fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.rows.to_be_bytes().as_slice());
        bytes.extend_from_slice(self.cols.to_be_bytes().as_slice());
        for v in &self.entries_row_major {
            bytes.extend_from_slice(v.to_be_bytes().as_slice());
        }
        bytes
    }
}

pub fn matrix_shape<M: MatrixShape>(m: &M) -> (usize, usize) {
    m.shape()
}

pub fn matrix_density<M: MatrixShape>(m: &M) -> RationalQ {
    let (rows, cols) = m.shape();
    if rows == 0 || cols == 0 {
        return int_q(0);
    }
    crate::types::rational::new_q(
        BigInt::from(m.nonzero_count()),
        BigInt::from(rows) * BigInt::from(cols),
    )
}

pub fn hash_matrix<M: MatrixShape>(m: &M) -> Hash {
    hash_sequence("matrix", &[m.canonical_bytes()])
}

fn canonical_sparse_q_entries(matrix: &SparseMatrixQ) -> Vec<(usize, usize, RationalQ)> {
    let mut entries: BTreeMap<(usize, usize), RationalQ> = BTreeMap::new();
    for (row, col, value) in &matrix.entries {
        if *row >= matrix.rows || *col >= matrix.cols || is_zero_q(value) {
            continue;
        }
        let next = entries
            .remove(&(*row, *col))
            .map_or_else(|| value.clone(), |old| add_q(&old, value));
        if !is_zero_q(&next) {
            entries.insert((*row, *col), next);
        }
    }
    entries
        .into_iter()
        .map(|((row, col), value)| (row, col, value))
        .collect()
}

fn canonical_sparse_fp_entries(matrix: &SparseMatrixFp) -> Vec<(usize, usize, Vec<u64>)> {
    // SparseMatrixFp does not carry its modulus, so duplicate coordinates are
    // canonicalized as an order-independent residue multiset instead of summed.
    let mut entries: BTreeMap<(usize, usize), Vec<u64>> = BTreeMap::new();
    for (row, col, value) in &matrix.entries {
        if *row >= matrix.rows || *col >= matrix.cols || *value == 0 {
            continue;
        }
        entries.entry((*row, *col)).or_default().push(*value);
    }
    for values in entries.values_mut() {
        values.sort_unstable();
    }
    entries
        .into_iter()
        .map(|((row, col), values)| (row, col, values))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_shape_density_and_hash_are_deterministic() {
        let m = SparseMatrixQ {
            rows: 2,
            cols: 3,
            entries: vec![(0, 0, int_q(1)), (1, 2, int_q(2))],
        };
        assert_eq!(matrix_shape(&m), (2, 3));
        assert_eq!(
            matrix_density(&m),
            crate::types::rational::new_q(2.into(), 6.into())
        );
        assert_eq!(hash_matrix(&m), hash_matrix(&m.clone()));
    }

    #[test]
    fn sparse_matrix_density_and_hash_canonicalize_entries() {
        let canonical = SparseMatrixQ {
            rows: 2,
            cols: 2,
            entries: vec![(0, 0, int_q(3))],
        };
        let noncanonical = SparseMatrixQ {
            rows: 2,
            cols: 2,
            entries: vec![
                (0, 0, int_q(1)),
                (1, 1, int_q(0)),
                (0, 0, int_q(2)),
                (3, 3, int_q(9)),
            ],
        };
        assert_eq!(
            matrix_density(&noncanonical),
            crate::types::rational::new_q(1.into(), 4.into())
        );
        assert_eq!(hash_matrix(&canonical), hash_matrix(&noncanonical));

        let fp_duplicates = SparseMatrixFp {
            rows: 2,
            cols: 2,
            entries: vec![(0, 0, 2), (1, 1, 0), (0, 0, 1), (3, 3, 7)],
        };
        let fp_duplicates_reordered = SparseMatrixFp {
            rows: 2,
            cols: 2,
            entries: vec![(3, 3, 7), (0, 0, 1), (0, 0, 2), (1, 1, 0)],
        };
        assert_eq!(
            matrix_density(&fp_duplicates),
            crate::types::rational::new_q(1.into(), 4.into())
        );
        assert_eq!(
            hash_matrix(&fp_duplicates),
            hash_matrix(&fp_duplicates_reordered)
        );
    }
}
