use num_traits::Zero;

use crate::Rational;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LinearSystemShape {
    pub rows: usize,
    pub cols: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum LinearSolveQ {
    Consistent {
        solution: Vec<Rational>,
        free_columns: Vec<usize>,
    },
    Inconsistent {
        obstruction: LeftNullObstructionQ,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LeftNullObstructionQ {
    pub coefficients: Vec<Rational>,
}

pub(crate) fn solve_linear_system_q(matrix: &[Vec<Rational>], rhs: &[Rational]) -> LinearSolveQ {
    assert_eq!(matrix.len(), rhs.len());
    let rows = matrix.len();
    let cols = matrix.first().map_or(0, Vec::len);
    assert!(matrix.iter().all(|row| row.len() == cols));

    let mut augmented = matrix
        .iter()
        .zip(rhs)
        .map(|(row, value)| {
            let mut augmented_row = row.clone();
            augmented_row.push(value.clone());
            augmented_row
        })
        .collect::<Vec<_>>();
    let mut transform = identity_q(rows);
    let mut rank = 0;
    let mut pivot_columns = Vec::new();

    for col in 0..cols {
        let pivot_row = (rank..rows).find(|row| !augmented[*row][col].is_zero());
        let Some(pivot_row) = pivot_row else {
            continue;
        };
        augmented.swap(rank, pivot_row);
        transform.swap(rank, pivot_row);

        let pivot = augmented[rank][col].clone();
        for entry in &mut augmented[rank] {
            *entry /= pivot.clone();
        }
        for entry in &mut transform[rank] {
            *entry /= pivot.clone();
        }

        for row in 0..rows {
            if row == rank || augmented[row][col].is_zero() {
                continue;
            }
            let factor = augmented[row][col].clone();
            for entry_col in col..=cols {
                let pivot_entry = augmented[rank][entry_col].clone();
                augmented[row][entry_col] -= factor.clone() * pivot_entry;
            }
            for entry_col in 0..rows {
                let pivot_entry = transform[rank][entry_col].clone();
                transform[row][entry_col] -= factor.clone() * pivot_entry;
            }
        }

        pivot_columns.push(col);
        rank += 1;
    }

    for row in 0..rows {
        let zero_left = augmented[row][..cols].iter().all(Zero::is_zero);
        if zero_left && !augmented[row][cols].is_zero() {
            return LinearSolveQ::Inconsistent {
                obstruction: LeftNullObstructionQ {
                    coefficients: transform[row].clone(),
                },
            };
        }
    }

    let mut solution = vec![crate::arith::rational_zero(); cols];
    for (row, col) in pivot_columns.iter().copied().enumerate() {
        solution[col] = augmented[row][cols].clone();
    }
    let free_columns = (0..cols)
        .filter(|col| !pivot_columns.contains(col))
        .collect();

    LinearSolveQ::Consistent {
        solution,
        free_columns,
    }
}

pub(crate) fn nullspace_matrix_q(matrix: &[Vec<Rational>]) -> Vec<Vec<Rational>> {
    let rows = matrix.len();
    let cols = matrix.first().map_or(0, Vec::len);
    assert!(matrix.iter().all(|row| row.len() == cols));

    let mut rref = matrix.to_vec();
    let mut rank = 0;
    let mut pivot_columns = Vec::new();

    for col in 0..cols {
        let pivot_row = (rank..rows).find(|row| !rref[*row][col].is_zero());
        let Some(pivot_row) = pivot_row else {
            continue;
        };
        rref.swap(rank, pivot_row);
        let pivot = rref[rank][col].clone();
        for entry in &mut rref[rank] {
            *entry /= pivot.clone();
        }
        for row in 0..rows {
            if row == rank || rref[row][col].is_zero() {
                continue;
            }
            let factor = rref[row][col].clone();
            for entry_col in col..cols {
                let pivot_entry = rref[rank][entry_col].clone();
                rref[row][entry_col] -= factor.clone() * pivot_entry;
            }
        }
        pivot_columns.push(col);
        rank += 1;
    }

    (0..cols)
        .filter(|col| !pivot_columns.contains(col))
        .map(|free_col| {
            let mut vector = vec![crate::arith::rational_zero(); cols];
            vector[free_col] = crate::arith::rational_one();
            for (row, pivot_col) in pivot_columns.iter().copied().enumerate() {
                vector[pivot_col] = -rref[row][free_col].clone();
            }
            vector
        })
        .collect()
}

pub(crate) fn matrix_vector_mul_q(matrix: &[Vec<Rational>], vector: &[Rational]) -> Vec<Rational> {
    matrix
        .iter()
        .map(|row| dot_q(row, vector))
        .collect::<Vec<_>>()
}

pub(crate) fn left_multiply_q(left: &[Rational], matrix: &[Vec<Rational>]) -> Vec<Rational> {
    assert_eq!(left.len(), matrix.len());
    let cols = matrix.first().map_or(0, Vec::len);
    let mut result = vec![crate::arith::rational_zero(); cols];
    for (row_index, coefficient) in left.iter().enumerate() {
        for (col, value) in matrix[row_index].iter().enumerate() {
            result[col] += coefficient.clone() * value.clone();
        }
    }
    result
}

pub(crate) fn dot_q(left: &[Rational], right: &[Rational]) -> Rational {
    assert_eq!(left.len(), right.len());
    left.iter()
        .zip(right)
        .fold(crate::arith::rational_zero(), |accumulator, (a, b)| {
            accumulator + a.clone() * b.clone()
        })
}

fn identity_q(size: usize) -> Vec<Vec<Rational>> {
    (0..size)
        .map(|row| {
            (0..size)
                .map(|col| {
                    if row == col {
                        crate::arith::rational_one()
                    } else {
                        crate::arith::rational_zero()
                    }
                })
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;
    use num_rational::BigRational;
    use num_traits::Zero;

    fn rat(value: i64) -> Rational {
        BigRational::from_integer(BigInt::from(value))
    }

    #[test]
    fn rational_solve_handles_unique_solution() {
        let matrix = vec![vec![rat(2), rat(1)], vec![rat(1), rat(-1)]];
        let rhs = vec![rat(5), rat(1)];

        let result = solve_linear_system_q(&matrix, &rhs);

        match result {
            LinearSolveQ::Consistent {
                solution,
                free_columns,
            } => {
                assert!(free_columns.is_empty());
                assert_eq!(solution, vec![rat(2), rat(1)]);
            }
            LinearSolveQ::Inconsistent { .. } => panic!("system should be consistent"),
        }
    }

    #[test]
    fn rational_solve_handles_multiple_solutions() {
        let matrix = vec![vec![rat(1), rat(1)]];
        let rhs = vec![rat(3)];

        let result = solve_linear_system_q(&matrix, &rhs);

        match result {
            LinearSolveQ::Consistent {
                solution,
                free_columns,
            } => {
                assert_eq!(free_columns, vec![1]);
                assert_eq!(matrix_vector_mul_q(&matrix, &solution), rhs);
            }
            LinearSolveQ::Inconsistent { .. } => panic!("system should be consistent"),
        }
    }

    #[test]
    fn inconsistent_system_returns_left_null_obstruction() {
        let matrix = vec![vec![rat(1), rat(1)], vec![rat(2), rat(2)]];
        let rhs = vec![rat(1), rat(3)];

        let result = solve_linear_system_q(&matrix, &rhs);

        match result {
            LinearSolveQ::Consistent { .. } => panic!("system should be inconsistent"),
            LinearSolveQ::Inconsistent { obstruction } => {
                assert_eq!(
                    left_multiply_q(&obstruction.coefficients, &matrix),
                    vec![Rational::zero(), Rational::zero()]
                );
                assert_ne!(dot_q(&obstruction.coefficients, &rhs), Rational::zero());
            }
        }
    }

    #[test]
    fn rational_nullspace_vectors_are_exact_relations() {
        let matrix = vec![vec![rat(1), rat(2), rat(3)], vec![rat(2), rat(4), rat(6)]];

        let basis = nullspace_matrix_q(&matrix);

        assert_eq!(basis.len(), 2);
        for vector in basis {
            assert_eq!(
                matrix_vector_mul_q(&matrix, &vector),
                vec![Rational::zero(), Rational::zero()]
            );
        }
    }
}
