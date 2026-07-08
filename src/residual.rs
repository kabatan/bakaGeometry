use crate::finite_field::PrimeModulus;

pub trait ResidualOracleFp {
    fn modulus(&self) -> u64;
    fn reduce(&self, vector: &[u64]) -> Vec<u64>;

    fn is_in_column_space(&self, vector: &[u64]) -> bool {
        self.reduce(vector).iter().all(|entry| *entry == 0)
    }
}

#[derive(Clone, Debug)]
pub struct DenseEchelonResidualOracleFp {
    modulus: PrimeModulus,
    row_count: usize,
    basis: Vec<EchelonVector>,
}

#[derive(Clone, Debug)]
struct EchelonVector {
    pivot: usize,
    entries: Vec<u64>,
}

impl DenseEchelonResidualOracleFp {
    pub fn from_columns(modulus: u64, columns: Vec<Vec<u64>>) -> Option<Self> {
        let row_count = columns.first().map_or(0, Vec::len);
        Self::from_columns_with_row_count(modulus, row_count, columns)
    }

    pub(crate) fn from_columns_with_row_count(
        modulus: u64,
        row_count: usize,
        columns: Vec<Vec<u64>>,
    ) -> Option<Self> {
        let modulus = PrimeModulus::new(modulus)?;
        if columns.iter().any(|column| column.len() != row_count) {
            return None;
        }

        let mut oracle = Self {
            modulus,
            row_count,
            basis: Vec::new(),
        };

        for column in columns {
            oracle.add_basis_candidate(column);
        }

        Some(oracle)
    }

    fn add_basis_candidate(&mut self, vector: Vec<u64>) {
        let mut residual = self.reduce(&vector);
        let Some(pivot) = first_nonzero_index(&residual) else {
            return;
        };

        let inverse = self
            .modulus
            .inv(residual[pivot])
            .expect("pivot must have inverse");
        for entry in &mut residual {
            *entry = self.modulus.mul(*entry, inverse);
        }

        for basis_vector in &mut self.basis {
            let factor = basis_vector.entries[pivot];
            if factor == 0 {
                continue;
            }
            eliminate_with(self.modulus, &mut basis_vector.entries, factor, &residual);
        }

        self.basis.push(EchelonVector {
            pivot,
            entries: residual,
        });
        self.basis.sort_by_key(|vector| vector.pivot);
    }
}

impl ResidualOracleFp for DenseEchelonResidualOracleFp {
    fn modulus(&self) -> u64 {
        self.modulus.value()
    }

    fn reduce(&self, vector: &[u64]) -> Vec<u64> {
        assert_eq!(vector.len(), self.row_count);
        let mut residual = vector
            .iter()
            .map(|entry| self.modulus.normalize(*entry))
            .collect::<Vec<_>>();

        for basis_vector in &self.basis {
            let factor = residual[basis_vector.pivot];
            if factor == 0 {
                continue;
            }
            eliminate_with(self.modulus, &mut residual, factor, &basis_vector.entries);
        }

        residual
    }
}

fn eliminate_with(modulus: PrimeModulus, target: &mut [u64], factor: u64, basis_entries: &[u64]) {
    for (target_entry, basis_entry) in target.iter_mut().zip(basis_entries) {
        let subtractor = modulus.mul(factor, *basis_entry);
        *target_entry = modulus.sub(*target_entry, subtractor);
    }
}

fn first_nonzero_index(vector: &[u64]) -> Option<usize> {
    vector.iter().position(|entry| *entry != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dense_residual_oracle_reduces_zero_exactly_for_column_space_vectors() {
        let oracle =
            DenseEchelonResidualOracleFp::from_columns(5, vec![vec![1, 2, 0], vec![0, 1, 1]])
                .unwrap();

        assert_eq!(oracle.modulus(), 5);
        assert!(oracle.is_in_column_space(&[3, 4, 3]));
        assert_eq!(oracle.reduce(&[3, 4, 3]), vec![0, 0, 0]);

        let outside = oracle.reduce(&[1, 0, 1]);
        assert_ne!(outside, vec![0, 0, 0]);
        assert_eq!(oracle.reduce(&outside), outside);
    }

    #[test]
    fn residual_oracle_matches_bruteforce_column_space_over_multiple_primes() {
        for prime in [2, 3, 5, 7] {
            for seed in 0..8 {
                let columns = pseudo_random_columns(prime, 3, 2, seed);
                let oracle = DenseEchelonResidualOracleFp::from_columns(prime, columns.clone())
                    .expect("prime modulus");
                for vector in all_vectors(prime, 3) {
                    let residual = oracle.reduce(&vector);
                    assert_eq!(oracle.reduce(&residual), residual);
                    assert_eq!(
                        oracle.is_in_column_space(&vector),
                        brute_force_column_space_contains(prime, &columns, &vector),
                        "prime={prime} seed={seed} vector={vector:?} columns={columns:?}"
                    );
                }
            }
        }
    }

    fn pseudo_random_columns(
        prime: u64,
        row_count: usize,
        column_count: usize,
        seed: u64,
    ) -> Vec<Vec<u64>> {
        let mut state = seed + 1;
        (0..column_count)
            .map(|_| {
                (0..row_count)
                    .map(|_| {
                        state = state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
                        state % prime
                    })
                    .collect()
            })
            .collect()
    }

    fn all_vectors(prime: u64, row_count: usize) -> Vec<Vec<u64>> {
        let mut vectors = Vec::new();
        let mut current = vec![0; row_count];
        enumerate_vectors(prime, 0, &mut current, &mut vectors);
        vectors
    }

    fn enumerate_vectors(
        prime: u64,
        index: usize,
        current: &mut [u64],
        vectors: &mut Vec<Vec<u64>>,
    ) {
        if index == current.len() {
            vectors.push(current.to_vec());
            return;
        }
        for value in 0..prime {
            current[index] = value;
            enumerate_vectors(prime, index + 1, current, vectors);
        }
        current[index] = 0;
    }

    fn brute_force_column_space_contains(prime: u64, columns: &[Vec<u64>], target: &[u64]) -> bool {
        let mut coefficients = vec![0; columns.len()];
        brute_force_coefficients(prime, columns, target, 0, &mut coefficients)
    }

    fn brute_force_coefficients(
        prime: u64,
        columns: &[Vec<u64>],
        target: &[u64],
        index: usize,
        coefficients: &mut [u64],
    ) -> bool {
        if index == coefficients.len() {
            let mut sum = vec![0; target.len()];
            for (coefficient, column) in coefficients.iter().zip(columns) {
                for (entry, column_entry) in sum.iter_mut().zip(column) {
                    *entry = (*entry + coefficient * column_entry) % prime;
                }
            }
            return sum == target;
        }
        for value in 0..prime {
            coefficients[index] = value;
            if brute_force_coefficients(prime, columns, target, index + 1, coefficients) {
                return true;
            }
        }
        coefficients[index] = 0;
        false
    }
}
