use crate::finite_field::PrimeModulus;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ModularLinearSystemShape {
    pub rows: usize,
    pub cols: usize,
    pub modulus: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RowReductionFp {
    pub rref: Vec<Vec<u64>>,
    pub pivot_columns: Vec<usize>,
}

pub(crate) fn row_reduce_matrix_fp(matrix: &[Vec<u64>], modulus: PrimeModulus) -> RowReductionFp {
    let rows = matrix.len();
    let cols = matrix.first().map_or(0, Vec::len);
    assert!(matrix.iter().all(|row| row.len() == cols));

    let mut rref = matrix
        .iter()
        .map(|row| {
            row.iter()
                .map(|entry| modulus.normalize(*entry))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let mut rank = 0;
    let mut pivot_columns = Vec::new();

    for col in 0..cols {
        let pivot_row = (rank..rows).find(|row| rref[*row][col] != 0);
        let Some(pivot_row) = pivot_row else {
            continue;
        };
        rref.swap(rank, pivot_row);
        let pivot_inverse = modulus
            .inv(rref[rank][col])
            .expect("nonzero finite-field element has inverse");
        for entry_col in col..cols {
            rref[rank][entry_col] = modulus.mul(rref[rank][entry_col], pivot_inverse);
        }
        for row in 0..rows {
            if row == rank || rref[row][col] == 0 {
                continue;
            }
            let factor = rref[row][col];
            for entry_col in col..cols {
                let subtractor = modulus.mul(factor, rref[rank][entry_col]);
                rref[row][entry_col] = modulus.sub(rref[row][entry_col], subtractor);
            }
        }
        pivot_columns.push(col);
        rank += 1;
    }

    RowReductionFp {
        rref,
        pivot_columns,
    }
}

pub(crate) fn nullspace_matrix_fp(matrix: &[Vec<u64>], modulus: PrimeModulus) -> Vec<Vec<u64>> {
    let reduction = row_reduce_matrix_fp(matrix, modulus);
    let cols = matrix.first().map_or(0, Vec::len);
    let free_columns = (0..cols)
        .filter(|col| !reduction.pivot_columns.contains(col))
        .collect::<Vec<_>>();

    free_columns
        .into_iter()
        .map(|free_col| {
            let mut vector = vec![0; cols];
            vector[free_col] = 1;
            for (row, pivot_col) in reduction.pivot_columns.iter().copied().enumerate() {
                vector[pivot_col] = modulus.neg(reduction.rref[row][free_col]);
            }
            vector
        })
        .collect()
}

pub(crate) fn find_column_relation_fp(
    vectors: &[Vec<u64>],
    modulus: PrimeModulus,
) -> Option<Vec<u64>> {
    let matrix = columns_to_matrix(vectors);
    nullspace_matrix_fp(&matrix, modulus)
        .into_iter()
        .find(|vector| vector.iter().any(|entry| *entry != 0))
}

pub(crate) fn columns_to_matrix(vectors: &[Vec<u64>]) -> Vec<Vec<u64>> {
    let rows = vectors.first().map_or(0, Vec::len);
    assert!(vectors.iter().all(|vector| vector.len() == rows));
    (0..rows)
        .map(|row| vectors.iter().map(|vector| vector[row]).collect())
        .collect()
}

pub(crate) fn matrix_vector_mul_fp(
    matrix: &[Vec<u64>],
    vector: &[u64],
    modulus: PrimeModulus,
) -> Vec<u64> {
    matrix
        .iter()
        .map(|row| {
            assert_eq!(row.len(), vector.len());
            row.iter()
                .zip(vector)
                .fold(0, |accumulator, (left, right)| {
                    modulus.add(accumulator, modulus.mul(*left, *right))
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finite_field_nullspace_vectors_are_exact_relations() {
        let modulus = PrimeModulus::new(5).unwrap();
        let matrix = vec![vec![1, 2, 3], vec![2, 4, 1]];

        let basis = nullspace_matrix_fp(&matrix, modulus);

        assert_eq!(basis.len(), 2);
        assert!(basis[0].iter().any(|entry| *entry != 0));
        assert_eq!(
            matrix_vector_mul_fp(&matrix, &basis[0], modulus),
            vec![0, 0]
        );
    }

    #[test]
    fn residual_relation_finds_nonzero_column_relation() {
        let modulus = PrimeModulus::new(5).unwrap();
        let vectors = vec![vec![1, 0], vec![0, 1], vec![1, 1]];

        let relation = find_column_relation_fp(&vectors, modulus).unwrap();

        assert!(relation.iter().any(|entry| *entry != 0));
        let matrix = columns_to_matrix(&vectors);
        assert_eq!(
            matrix_vector_mul_fp(&matrix, &relation, modulus),
            vec![0, 0]
        );
    }
}
