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
}
