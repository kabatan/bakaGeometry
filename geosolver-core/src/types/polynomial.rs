use std::collections::{BTreeMap, BTreeSet};

use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{One, Signed, Zero};
use serde::{Deserialize, Serialize};

use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::VariableId;
use crate::types::monomial::{monomial_mul, monomial_to_bytes, normalize_monomial, Monomial};
use crate::types::rational::{
    add_q, int_q, is_zero_q, lcm_denominators, mul_q, neg_q, rational_to_bytes, RationalQ,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TermQ {
    pub coeff: RationalQ,
    pub monomial: Monomial,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SparsePolynomialQ {
    pub terms: Vec<TermQ>,
    pub hash: Hash,
}

pub type SubstitutionMap = BTreeMap<VariableId, SparsePolynomialQ>;

pub fn normalize_poly(p: SparsePolynomialQ) -> SparsePolynomialQ {
    let mut map: BTreeMap<Monomial, RationalQ> = BTreeMap::new();
    for term in p.terms {
        if is_zero_q(&term.coeff) {
            continue;
        }
        let monomial = normalize_monomial(term.monomial.exponents);
        let coeff = map
            .remove(&monomial)
            .map_or(term.coeff.clone(), |old| add_q(&old, &term.coeff));
        if !is_zero_q(&coeff) {
            map.insert(monomial, coeff);
        }
    }
    let terms: Vec<TermQ> = map
        .into_iter()
        .map(|(monomial, coeff)| TermQ { coeff, monomial })
        .collect();
    let hash = hash_terms(&terms);
    SparsePolynomialQ { terms, hash }
}

pub fn zero_poly() -> SparsePolynomialQ {
    normalize_poly(SparsePolynomialQ {
        terms: Vec::new(),
        hash: hash_sequence("poly", &[]),
    })
}

pub fn constant_poly(c: RationalQ) -> SparsePolynomialQ {
    if is_zero_q(&c) {
        return zero_poly();
    }
    normalize_poly(SparsePolynomialQ {
        terms: vec![TermQ {
            coeff: c,
            monomial: normalize_monomial(Vec::new()),
        }],
        hash: hash_sequence("poly", &[]),
    })
}

pub fn variable_poly(v: VariableId) -> SparsePolynomialQ {
    normalize_poly(SparsePolynomialQ {
        terms: vec![TermQ {
            coeff: int_q(1),
            monomial: normalize_monomial(vec![(v, 1)]),
        }],
        hash: hash_sequence("poly", &[]),
    })
}

pub fn poly_add(a: &SparsePolynomialQ, b: &SparsePolynomialQ) -> SparsePolynomialQ {
    let mut terms = a.terms.clone();
    terms.extend_from_slice(&b.terms);
    normalize_poly(SparsePolynomialQ {
        terms,
        hash: hash_sequence("poly", &[]),
    })
}

pub fn poly_sub(a: &SparsePolynomialQ, b: &SparsePolynomialQ) -> SparsePolynomialQ {
    let neg_terms: Vec<TermQ> = b
        .terms
        .iter()
        .map(|term| TermQ {
            coeff: neg_q(&term.coeff),
            monomial: term.monomial.clone(),
        })
        .collect();
    let mut terms = a.terms.clone();
    terms.extend(neg_terms);
    normalize_poly(SparsePolynomialQ {
        terms,
        hash: hash_sequence("poly", &[]),
    })
}

pub fn poly_mul(a: &SparsePolynomialQ, b: &SparsePolynomialQ) -> SparsePolynomialQ {
    let mut terms = Vec::new();
    for ta in &a.terms {
        for tb in &b.terms {
            terms.push(TermQ {
                coeff: mul_q(&ta.coeff, &tb.coeff),
                monomial: monomial_mul(&ta.monomial, &tb.monomial),
            });
        }
    }
    normalize_poly(SparsePolynomialQ {
        terms,
        hash: hash_sequence("poly", &[]),
    })
}

pub fn poly_scale(a: &SparsePolynomialQ, c: &RationalQ) -> SparsePolynomialQ {
    if is_zero_q(c) {
        return zero_poly();
    }
    normalize_poly(SparsePolynomialQ {
        terms: a
            .terms
            .iter()
            .map(|term| TermQ {
                coeff: mul_q(&term.coeff, c),
                monomial: term.monomial.clone(),
            })
            .collect(),
        hash: hash_sequence("poly", &[]),
    })
}

pub fn poly_derivative(a: &SparsePolynomialQ, v: VariableId) -> SparsePolynomialQ {
    let mut terms = Vec::new();
    for term in &a.terms {
        if let Some((_, exp)) = term.monomial.exponents.iter().find(|(var, _)| *var == v) {
            let mut entries = term.monomial.exponents.clone();
            for (var, e) in &mut entries {
                if *var == v {
                    *e -= 1;
                    break;
                }
            }
            terms.push(TermQ {
                coeff: mul_q(&term.coeff, &int_q(*exp as i64)),
                monomial: normalize_monomial(entries),
            });
        }
    }
    normalize_poly(SparsePolynomialQ {
        terms,
        hash: hash_sequence("poly", &[]),
    })
}

pub fn poly_variables(a: &SparsePolynomialQ) -> BTreeSet<VariableId> {
    let mut vars = BTreeSet::new();
    for term in &a.terms {
        for (var, _) in &term.monomial.exponents {
            vars.insert(*var);
        }
    }
    vars
}

pub fn poly_total_degree(a: &SparsePolynomialQ) -> u32 {
    a.terms
        .iter()
        .map(|term| term.monomial.exponents.iter().map(|(_, exp)| *exp).sum())
        .max()
        .unwrap_or(0)
}

pub fn poly_monomial_count(a: &SparsePolynomialQ) -> usize {
    a.terms.len()
}

pub fn clear_denominators_primitive(a: &SparsePolynomialQ) -> SparsePolynomialQ {
    if a.terms.is_empty() {
        return zero_poly();
    }
    let lcm = lcm_denominators(a.terms.iter().map(|term| &term.coeff));
    let mut integer_coeffs: Vec<BigInt> = a
        .terms
        .iter()
        .map(|term| &term.coeff.num * (&lcm / &term.coeff.den))
        .collect();
    let content = integer_coeffs
        .iter()
        .fold(BigInt::zero(), |acc, c| acc.gcd(&c.abs()));
    if !content.is_zero() && content != BigInt::one() {
        for coeff in &mut integer_coeffs {
            *coeff /= &content;
        }
    }
    if integer_coeffs
        .iter()
        .find(|c| !c.is_zero())
        .map(|c| c.sign() == num_bigint::Sign::Minus)
        .unwrap_or(false)
    {
        for coeff in &mut integer_coeffs {
            *coeff = -coeff.clone();
        }
    }
    normalize_poly(SparsePolynomialQ {
        terms: a
            .terms
            .iter()
            .zip(integer_coeffs)
            .map(|(term, coeff)| TermQ {
                coeff: RationalQ {
                    num: coeff,
                    den: BigInt::one(),
                },
                monomial: term.monomial.clone(),
            })
            .collect(),
        hash: hash_sequence("poly", &[]),
    })
}

pub fn substitute_poly(a: &SparsePolynomialQ, subst: &SubstitutionMap) -> SparsePolynomialQ {
    let mut acc = zero_poly();
    for term in &a.terms {
        let mut product = constant_poly(term.coeff.clone());
        for (var, exp) in &term.monomial.exponents {
            let base = subst
                .get(var)
                .cloned()
                .unwrap_or_else(|| variable_poly(*var));
            product = poly_mul(&product, &poly_pow(&base, *exp));
        }
        acc = poly_add(&acc, &product);
    }
    acc
}

fn poly_pow(base: &SparsePolynomialQ, exp: u32) -> SparsePolynomialQ {
    let mut result = constant_poly(int_q(1));
    for _ in 0..exp {
        result = poly_mul(&result, base);
    }
    result
}

pub fn hash_terms(terms: &[TermQ]) -> Hash {
    let chunks: Vec<Vec<u8>> = terms
        .iter()
        .map(|term| {
            let mut bytes = Vec::new();
            bytes.extend_from_slice(&rational_to_bytes(&term.coeff));
            bytes.extend_from_slice(&monomial_to_bytes(&term.monomial));
            bytes
        })
        .collect();
    hash_sequence("poly-terms", &chunks)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn x() -> SparsePolynomialQ {
        variable_poly(VariableId(1))
    }

    #[test]
    fn normalization_merges_terms_and_hashes_deterministically() {
        let p = normalize_poly(SparsePolynomialQ {
            terms: vec![
                TermQ {
                    coeff: int_q(2),
                    monomial: normalize_monomial(vec![(VariableId(1), 1)]),
                },
                TermQ {
                    coeff: int_q(3),
                    monomial: normalize_monomial(vec![(VariableId(1), 1)]),
                },
            ],
            hash: hash_sequence("poly", &[]),
        });
        let q = poly_scale(&x(), &int_q(5));
        assert_eq!(p, q);
        assert_eq!(p.hash, q.hash);
    }

    #[test]
    fn derivative_and_substitution_are_exact() {
        let p = poly_mul(&x(), &x());
        let dp = poly_derivative(&p, VariableId(1));
        assert_eq!(dp, poly_scale(&x(), &int_q(2)));

        let mut subst = SubstitutionMap::new();
        subst.insert(VariableId(1), constant_poly(int_q(3)));
        assert_eq!(substitute_poly(&p, &subst), constant_poly(int_q(9)));
    }

    #[test]
    fn clear_denominators_makes_primitive_integer_poly() {
        let p = normalize_poly(SparsePolynomialQ {
            terms: vec![TermQ {
                coeff: crate::types::rational::new_q(BigInt::from(2), BigInt::from(6)),
                monomial: normalize_monomial(vec![(VariableId(1), 1)]),
            }],
            hash: hash_sequence("poly", &[]),
        });
        assert_eq!(clear_denominators_primitive(&p), x());
    }
}
