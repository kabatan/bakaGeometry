use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::algebra::monomial_order::MonomialOrder;
use crate::types::ids::VariableId;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Monomial {
    pub exponents: Vec<(VariableId, u32)>,
}

pub fn normalize_monomial(entries: Vec<(VariableId, u32)>) -> Monomial {
    let mut map: BTreeMap<VariableId, u32> = BTreeMap::new();
    for (var, exp) in entries {
        if exp == 0 {
            continue;
        }
        let next = map.get(&var).copied().unwrap_or(0).saturating_add(exp);
        if next != 0 {
            map.insert(var, next);
        }
    }
    Monomial {
        exponents: map.into_iter().collect(),
    }
}

pub fn monomial_mul(a: &Monomial, b: &Monomial) -> Monomial {
    let mut entries = a.exponents.clone();
    entries.extend_from_slice(&b.exponents);
    normalize_monomial(entries)
}

pub fn monomial_div(a: &Monomial, b: &Monomial) -> Option<Monomial> {
    let mut map: BTreeMap<VariableId, u32> = a.exponents.iter().copied().collect();
    for (var, b_exp) in &b.exponents {
        let a_exp = map.get(var).copied().unwrap_or(0);
        if a_exp < *b_exp {
            return None;
        }
        let remaining = a_exp - *b_exp;
        if remaining == 0 {
            map.remove(var);
        } else {
            map.insert(*var, remaining);
        }
    }
    Some(Monomial {
        exponents: map.into_iter().collect(),
    })
}

pub fn monomial_degree(m: &Monomial) -> u32 {
    m.exponents.iter().map(|(_, exp)| *exp).sum()
}

pub fn monomial_variables(m: &Monomial) -> BTreeSet<VariableId> {
    m.exponents.iter().map(|(var, _)| *var).collect()
}

pub fn compare_monomial(a: &Monomial, b: &Monomial, order: &MonomialOrder) -> Ordering {
    order.compare(a, b)
}

pub fn monomial_to_bytes(m: &Monomial) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice((m.exponents.len() as u64).to_be_bytes().as_slice());
    for (var, exp) in &m.exponents {
        out.extend_from_slice(var.0.to_be_bytes().as_slice());
        out.extend_from_slice(exp.to_be_bytes().as_slice());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monomial_normalization_sorts_and_combines() {
        let m = normalize_monomial(vec![
            (VariableId(3), 1),
            (VariableId(1), 2),
            (VariableId(3), 4),
        ]);
        assert_eq!(m.exponents, vec![(VariableId(1), 2), (VariableId(3), 5)]);
    }

    #[test]
    fn monomial_division_is_exact_when_possible() {
        let a = normalize_monomial(vec![(VariableId(1), 3), (VariableId(2), 1)]);
        let b = normalize_monomial(vec![(VariableId(1), 2)]);
        assert_eq!(
            monomial_div(&a, &b).unwrap(),
            normalize_monomial(vec![(VariableId(1), 1), (VariableId(2), 1)])
        );
        assert!(monomial_div(&b, &a).is_none());
    }
}
