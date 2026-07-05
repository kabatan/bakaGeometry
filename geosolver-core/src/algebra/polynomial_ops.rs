use std::collections::BTreeMap;

use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{One, Signed, Zero};
use serde::{Deserialize, Serialize};

use crate::algebra::monomial_order::MonomialOrder;
use crate::types::hash::hash_sequence;
use crate::types::monomial::{monomial_div, normalize_monomial, Monomial};
use crate::types::polynomial::{
    normalize_poly, poly_add, poly_mul, poly_sub, zero_poly, SparsePolynomialQ, TermQ,
};
use crate::types::rational::{div_q, int_q, is_zero_q, new_q, RationalQ};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReductionResult {
    pub quotients: Vec<SparsePolynomialQ>,
    pub remainder: SparsePolynomialQ,
}

pub fn leading_term(p: &SparsePolynomialQ, order: &MonomialOrder) -> Option<TermQ> {
    p.terms.iter().cloned().max_by(|a, b| {
        order
            .compare(&a.monomial, &b.monomial)
            .then(a.monomial.cmp(&b.monomial))
    })
}

pub fn s_polynomial(
    f: &SparsePolynomialQ,
    g: &SparsePolynomialQ,
    order: &MonomialOrder,
) -> SparsePolynomialQ {
    let Some(lt_f) = leading_term(f, order) else {
        return zero_poly();
    };
    let Some(lt_g) = leading_term(g, order) else {
        return zero_poly();
    };
    let lcm = monomial_lcm(&lt_f.monomial, &lt_g.monomial);
    let Some(m_f) = monomial_div(&lcm, &lt_f.monomial) else {
        return zero_poly();
    };
    let Some(m_g) = monomial_div(&lcm, &lt_g.monomial) else {
        return zero_poly();
    };
    let Ok(c_f) = div_q(&int_q(1), &lt_f.coeff) else {
        return zero_poly();
    };
    let Ok(c_g) = div_q(&int_q(1), &lt_g.coeff) else {
        return zero_poly();
    };
    let left = poly_mul(&single_term_poly(c_f, m_f), f);
    let right = poly_mul(&single_term_poly(c_g, m_g), g);
    poly_sub(&left, &right)
}

pub fn reduce_by_set(
    f: &SparsePolynomialQ,
    gs: &[SparsePolynomialQ],
    order: &MonomialOrder,
) -> ReductionResult {
    let mut current = normalize_poly(f.clone());
    let mut remainder = zero_poly();
    let mut quotients = vec![zero_poly(); gs.len()];
    let leading_terms: Vec<Option<TermQ>> = gs.iter().map(|g| leading_term(g, order)).collect();

    while let Some(lt_current) = leading_term(&current, order) {
        let mut reduced = false;
        for (idx, lt_g_opt) in leading_terms.iter().enumerate() {
            let Some(lt_g) = lt_g_opt else {
                continue;
            };
            let Some(multiplier_monomial) = monomial_div(&lt_current.monomial, &lt_g.monomial)
            else {
                continue;
            };
            let Ok(multiplier_coeff) = div_q(&lt_current.coeff, &lt_g.coeff) else {
                continue;
            };
            if is_zero_q(&multiplier_coeff) {
                continue;
            }
            let multiplier = single_term_poly(multiplier_coeff, multiplier_monomial);
            let subtractor = poly_mul(&multiplier, &gs[idx]);
            current = poly_sub(&current, &subtractor);
            quotients[idx] = poly_add(&quotients[idx], &multiplier);
            reduced = true;
            break;
        }

        if !reduced {
            let lt_poly = single_term_poly(lt_current.coeff, lt_current.monomial);
            remainder = poly_add(&remainder, &lt_poly);
            current = poly_sub(&current, &lt_poly);
        }
    }

    ReductionResult {
        quotients,
        remainder,
    }
}

pub fn content_primitive_part(f: &SparsePolynomialQ) -> (RationalQ, SparsePolynomialQ) {
    let normalized = normalize_poly(f.clone());
    if normalized.terms.is_empty() {
        return (int_q(0), zero_poly());
    }

    let primitive = primitive_integer_part(&normalized);
    let content = normalized
        .terms
        .iter()
        .find_map(|term| {
            primitive
                .terms
                .iter()
                .find(|pterm| pterm.monomial == term.monomial)
                .and_then(|pterm| div_q(&term.coeff, &pterm.coeff).ok())
        })
        .unwrap_or_else(|| int_q(0));
    (content, primitive)
}

fn single_term_poly(coeff: RationalQ, monomial: Monomial) -> SparsePolynomialQ {
    if is_zero_q(&coeff) {
        return zero_poly();
    }
    normalize_poly(SparsePolynomialQ {
        terms: vec![TermQ { coeff, monomial }],
        hash: hash_sequence("poly", &[]),
    })
}

fn monomial_lcm(a: &Monomial, b: &Monomial) -> Monomial {
    let mut exponents: BTreeMap<_, _> = a.exponents.iter().copied().collect();
    for (var, exp_b) in &b.exponents {
        let exp = exponents.get(var).copied().unwrap_or(0).max(*exp_b);
        if exp == 0 {
            exponents.remove(var);
        } else {
            exponents.insert(*var, exp);
        }
    }
    normalize_monomial(exponents.into_iter().collect())
}

fn primitive_integer_part(f: &SparsePolynomialQ) -> SparsePolynomialQ {
    let lcm = f
        .terms
        .iter()
        .fold(BigInt::one(), |acc, term| acc.lcm(&term.coeff.den));
    let mut integer_coeffs: Vec<BigInt> = f
        .terms
        .iter()
        .map(|term| &term.coeff.num * (&lcm / &term.coeff.den))
        .collect();
    let content = integer_coeffs
        .iter()
        .fold(BigInt::zero(), |acc, coeff| acc.gcd(&coeff.abs()));
    if !content.is_zero() && content != BigInt::one() {
        for coeff in &mut integer_coeffs {
            *coeff /= &content;
        }
    }
    if integer_coeffs
        .iter()
        .find(|coeff| !coeff.is_zero())
        .map(|coeff| coeff.sign() == num_bigint::Sign::Minus)
        .unwrap_or(false)
    {
        for coeff in &mut integer_coeffs {
            *coeff = -coeff.clone();
        }
    }
    normalize_poly(SparsePolynomialQ {
        terms: f
            .terms
            .iter()
            .zip(integer_coeffs)
            .map(|(term, coeff)| TermQ {
                coeff: new_q(coeff, BigInt::one()),
                monomial: term.monomial.clone(),
            })
            .collect(),
        hash: hash_sequence("poly", &[]),
    })
}

#[cfg(test)]
mod tests {
    use num_bigint::BigInt;

    use super::*;
    use crate::algebra::monomial_order::lex_order;
    use crate::types::ids::VariableId;
    use crate::types::monomial::normalize_monomial;
    use crate::types::polynomial::{constant_poly, poly_add, poly_scale, variable_poly};
    use crate::types::rational::new_q;

    fn x() -> SparsePolynomialQ {
        variable_poly(VariableId(1))
    }

    fn y() -> SparsePolynomialQ {
        variable_poly(VariableId(2))
    }

    fn term(coeff: RationalQ, entries: Vec<(VariableId, u32)>) -> SparsePolynomialQ {
        single_term_poly(coeff, normalize_monomial(entries))
    }

    #[test]
    fn leading_term_uses_declared_order() {
        let order = lex_order(&[VariableId(1), VariableId(2)]);
        let p = poly_add(
            &term(int_q(1), vec![(VariableId(1), 1), (VariableId(2), 5)]),
            &term(int_q(1), vec![(VariableId(1), 2)]),
        );
        assert_eq!(
            leading_term(&p, &order).unwrap().monomial,
            normalize_monomial(vec![(VariableId(1), 2)])
        );
    }

    #[test]
    fn s_polynomial_cancels_leading_terms() {
        let order = lex_order(&[VariableId(1), VariableId(2)]);
        let f = poly_sub(&poly_mul(&x(), &x()), &y());
        let g = poly_sub(&poly_mul(&x(), &y()), &constant_poly(int_q(1)));
        let wanted = poly_sub(&x(), &poly_mul(&y(), &y()));
        assert_eq!(s_polynomial(&f, &g, &order), wanted);
    }

    #[test]
    fn reduction_respects_coefficients_and_records_quotients() {
        let order = lex_order(&[VariableId(1)]);
        let basis = vec![poly_add(
            &poly_scale(&x(), &int_q(2)),
            &constant_poly(int_q(2)),
        )];
        let f = poly_add(&poly_scale(&x(), &int_q(4)), &constant_poly(int_q(4)));
        let reduction = reduce_by_set(&f, &basis, &order);
        assert!(reduction.remainder.terms.is_empty());
        assert_eq!(reduction.quotients[0], constant_poly(int_q(2)));
        let reconstructed = poly_add(
            &poly_mul(&reduction.quotients[0], &basis[0]),
            &reduction.remainder,
        );
        assert_eq!(reconstructed, f);
    }

    #[test]
    fn content_and_primitive_part_reconstruct_input() {
        let p = poly_add(
            &term(
                new_q(BigInt::from(2), BigInt::from(3)),
                vec![(VariableId(1), 1)],
            ),
            &constant_poly(new_q(BigInt::from(4), BigInt::from(3))),
        );
        let (content, primitive) = content_primitive_part(&p);
        assert_eq!(content, new_q(BigInt::from(2), BigInt::from(3)));
        assert_eq!(primitive, poly_add(&x(), &constant_poly(int_q(2))));
        assert_eq!(poly_scale(&primitive, &content), p);
    }
}
