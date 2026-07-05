use serde::{Deserialize, Serialize};

use crate::types::hash::{hash_sequence, Hash};
use crate::types::rational::{int_q, rational_to_bytes, RationalQ};

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
        self.entries.len()
    }

    fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.rows.to_be_bytes().as_slice());
        bytes.extend_from_slice(self.cols.to_be_bytes().as_slice());
        for (r, c, v) in &self.entries {
            bytes.extend_from_slice(r.to_be_bytes().as_slice());
            bytes.extend_from_slice(c.to_be_bytes().as_slice());
            bytes.extend_from_slice(&rational_to_bytes(v));
        }
        bytes
    }
}

impl MatrixShape for SparseMatrixFp {
    fn shape(&self) -> (usize, usize) {
        (self.rows, self.cols)
    }

    fn nonzero_count(&self) -> usize {
        self.entries.len()
    }

    fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.rows.to_be_bytes().as_slice());
        bytes.extend_from_slice(self.cols.to_be_bytes().as_slice());
        for (r, c, v) in &self.entries {
            bytes.extend_from_slice(r.to_be_bytes().as_slice());
            bytes.extend_from_slice(c.to_be_bytes().as_slice());
            bytes.extend_from_slice(v.to_be_bytes().as_slice());
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
        (m.nonzero_count() as i64).into(),
        ((rows * cols) as i64).into(),
    )
}

pub fn hash_matrix<M: MatrixShape>(m: &M) -> Hash {
    hash_sequence("matrix", &[m.canonical_bytes()])
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
}
