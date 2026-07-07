use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::types::rational::RationalQ;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RationalInterval {
    pub lo: RationalQ,
    pub hi: RationalQ,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum IntervalError {
    #[error("interval lower bound must be strictly less than upper bound")]
    Empty,
}

pub fn interval_new(lo: RationalQ, hi: RationalQ) -> Result<RationalInterval, IntervalError> {
    if lo >= hi {
        return Err(IntervalError::Empty);
    }
    Ok(RationalInterval { lo, hi })
}

pub fn interval_contains_q(i: &RationalInterval, x: &RationalQ) -> bool {
    &i.lo <= x && x <= &i.hi
}

pub fn interval_disjoint(a: &RationalInterval, b: &RationalInterval) -> bool {
    a.hi < b.lo || b.hi < a.lo
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::rational::int_q;

    #[test]
    fn interval_invariants_hold() {
        let i = interval_new(int_q(1), int_q(3)).unwrap();
        assert!(interval_contains_q(&i, &int_q(2)));
        assert!(!interval_contains_q(&i, &int_q(4)));
        let j = interval_new(int_q(4), int_q(5)).unwrap();
        assert!(interval_disjoint(&i, &j));
        assert!(interval_new(int_q(5), int_q(4)).is_err());
        assert!(interval_new(int_q(5), int_q(5)).is_err());
    }
}
