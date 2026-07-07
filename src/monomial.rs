#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Monomial {
    pub exponents: Vec<u32>,
}

impl Monomial {
    pub fn multiply(&self, rhs: &Self) -> Self {
        assert_eq!(self.exponents.len(), rhs.exponents.len());
        Self {
            exponents: self
                .exponents
                .iter()
                .zip(&rhs.exponents)
                .map(|(left, right)| left + right)
                .collect(),
        }
    }

    pub fn is_divisible_by(&self, rhs: &Self) -> bool {
        assert_eq!(self.exponents.len(), rhs.exponents.len());
        self.exponents
            .iter()
            .zip(&rhs.exponents)
            .all(|(left, right)| left >= right)
    }

    pub fn quotient_if_divisible_by(&self, rhs: &Self) -> Option<Self> {
        if !self.is_divisible_by(rhs) {
            return None;
        }
        Some(Self {
            exponents: self
                .exponents
                .iter()
                .zip(&rhs.exponents)
                .map(|(left, right)| left - right)
                .collect(),
        })
    }

    pub fn total_degree(&self) -> u32 {
        self.exponents.iter().copied().sum()
    }
}
