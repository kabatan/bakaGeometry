use serde::{Deserialize, Serialize};

use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::VariableId;
use crate::types::rational::{
    add_q, div_q, int_q, is_zero_q, mul_q, rational_to_bytes, sub_q, zero_q, RationalQ,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UniPolynomialQ {
    pub variable: VariableId,
    pub coeffs_low_to_high: Vec<RationalQ>,
    pub hash: Hash,
}

pub fn normalize_univariate(mut p: UniPolynomialQ) -> UniPolynomialQ {
    while p.coeffs_low_to_high.last().map(is_zero_q).unwrap_or(false) {
        p.coeffs_low_to_high.pop();
    }
    p.hash = hash_univariate(p.variable, &p.coeffs_low_to_high);
    p
}

pub fn zero_uni(variable: VariableId) -> UniPolynomialQ {
    normalize_univariate(UniPolynomialQ {
        variable,
        coeffs_low_to_high: Vec::new(),
        hash: hash_sequence("univariate", &[]),
    })
}

pub fn constant_uni(variable: VariableId, c: RationalQ) -> UniPolynomialQ {
    normalize_univariate(UniPolynomialQ {
        variable,
        coeffs_low_to_high: vec![c],
        hash: hash_sequence("univariate", &[]),
    })
}

pub fn degree_uni(p: &UniPolynomialQ) -> Option<usize> {
    if p.coeffs_low_to_high.is_empty() {
        None
    } else {
        Some(p.coeffs_low_to_high.len() - 1)
    }
}

pub fn derivative_uni(p: &UniPolynomialQ) -> UniPolynomialQ {
    let coeffs = p
        .coeffs_low_to_high
        .iter()
        .enumerate()
        .skip(1)
        .map(|(degree, coeff)| mul_q(coeff, &int_q(degree as i64)))
        .collect();
    normalize_univariate(UniPolynomialQ {
        variable: p.variable,
        coeffs_low_to_high: coeffs,
        hash: hash_sequence("univariate", &[]),
    })
}

pub fn gcd_uni(a: &UniPolynomialQ, b: &UniPolynomialQ) -> UniPolynomialQ {
    assert_eq!(a.variable, b.variable, "univariate variable mismatch");
    let mut r0 = normalize_univariate(a.clone());
    let mut r1 = normalize_univariate(b.clone());
    while degree_uni(&r1).is_some() {
        let r = rem_uni(&r0, &r1);
        r0 = r1;
        r1 = r;
    }
    make_monic(&r0)
}

pub fn squarefree_part_uni(p: &UniPolynomialQ) -> UniPolynomialQ {
    let normalized = normalize_univariate(p.clone());
    if degree_uni(&normalized).is_none() {
        return normalized;
    }
    let derivative = derivative_uni(&normalized);
    if degree_uni(&derivative).is_none() {
        return make_monic(&normalized);
    }
    let gcd = gcd_uni(&normalized, &derivative);
    div_exact_uni(&normalized, &gcd)
}

pub fn eval_uni_q(p: &UniPolynomialQ, x: &RationalQ) -> RationalQ {
    let mut acc = zero_q();
    for coeff in p.coeffs_low_to_high.iter().rev() {
        acc = add_q(&mul_q(&acc, x), coeff);
    }
    acc
}

fn make_monic(p: &UniPolynomialQ) -> UniPolynomialQ {
    let Some(deg) = degree_uni(p) else {
        return normalize_univariate(p.clone());
    };
    let lc = &p.coeffs_low_to_high[deg];
    let coeffs = p
        .coeffs_low_to_high
        .iter()
        .map(|c| div_q(c, lc).expect("nonzero leading coefficient"))
        .collect();
    normalize_univariate(UniPolynomialQ {
        variable: p.variable,
        coeffs_low_to_high: coeffs,
        hash: hash_sequence("univariate", &[]),
    })
}

fn rem_uni(a: &UniPolynomialQ, b: &UniPolynomialQ) -> UniPolynomialQ {
    let mut r = normalize_univariate(a.clone());
    let Some(b_deg) = degree_uni(b) else {
        return r;
    };
    let b_lc = b.coeffs_low_to_high[b_deg].clone();
    while let Some(r_deg) = degree_uni(&r) {
        if r_deg < b_deg {
            break;
        }
        let shift = r_deg - b_deg;
        let scale = div_q(&r.coeffs_low_to_high[r_deg], &b_lc)
            .expect("nonzero divisor leading coefficient");
        let mut subtract = vec![zero_q(); shift];
        subtract.extend(b.coeffs_low_to_high.iter().map(|c| mul_q(c, &scale)));
        let rhs = normalize_univariate(UniPolynomialQ {
            variable: r.variable,
            coeffs_low_to_high: subtract,
            hash: hash_sequence("univariate", &[]),
        });
        r = sub_uni(&r, &rhs);
    }
    r
}

fn sub_uni(a: &UniPolynomialQ, b: &UniPolynomialQ) -> UniPolynomialQ {
    let n = a.coeffs_low_to_high.len().max(b.coeffs_low_to_high.len());
    let mut coeffs = Vec::with_capacity(n);
    for i in 0..n {
        let ai = a.coeffs_low_to_high.get(i).cloned().unwrap_or_else(zero_q);
        let bi = b.coeffs_low_to_high.get(i).cloned().unwrap_or_else(zero_q);
        coeffs.push(sub_q(&ai, &bi));
    }
    normalize_univariate(UniPolynomialQ {
        variable: a.variable,
        coeffs_low_to_high: coeffs,
        hash: hash_sequence("univariate", &[]),
    })
}

fn div_exact_uni(a: &UniPolynomialQ, b: &UniPolynomialQ) -> UniPolynomialQ {
    let Some(b_deg) = degree_uni(b) else {
        return normalize_univariate(a.clone());
    };
    let mut r = normalize_univariate(a.clone());
    let q_len = degree_uni(&r).map_or(0, |d| d.saturating_sub(b_deg) + 1);
    let mut q = vec![zero_q(); q_len];
    let b_lc = b.coeffs_low_to_high[b_deg].clone();
    while let Some(r_deg) = degree_uni(&r) {
        if r_deg < b_deg {
            break;
        }
        let shift = r_deg - b_deg;
        let scale = div_q(&r.coeffs_low_to_high[r_deg], &b_lc)
            .expect("nonzero divisor leading coefficient");
        q[shift] = add_q(&q[shift], &scale);
        let mut subtract = vec![zero_q(); shift];
        subtract.extend(b.coeffs_low_to_high.iter().map(|c| mul_q(c, &scale)));
        r = sub_uni(
            &r,
            &normalize_univariate(UniPolynomialQ {
                variable: a.variable,
                coeffs_low_to_high: subtract,
                hash: hash_sequence("univariate", &[]),
            }),
        );
    }
    normalize_univariate(UniPolynomialQ {
        variable: a.variable,
        coeffs_low_to_high: q,
        hash: hash_sequence("univariate", &[]),
    })
}

fn hash_univariate(variable: VariableId, coeffs: &[RationalQ]) -> Hash {
    let mut chunks = vec![variable.0.to_be_bytes().to_vec()];
    chunks.extend(coeffs.iter().map(rational_to_bytes));
    hash_sequence("univariate", &chunks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;

    fn q(n: i64) -> RationalQ {
        int_q(n)
    }

    #[test]
    fn derivative_and_eval_are_exact() {
        let p = normalize_univariate(UniPolynomialQ {
            variable: VariableId(1),
            coeffs_low_to_high: vec![q(1), q(0), q(3)],
            hash: hash_sequence("univariate", &[]),
        });
        assert_eq!(eval_uni_q(&p, &q(2)), q(13));
        assert_eq!(derivative_uni(&p).coeffs_low_to_high, vec![q(0), q(6)]);
    }

    #[test]
    fn gcd_and_squarefree_are_monic() {
        let x_minus_1_squared = normalize_univariate(UniPolynomialQ {
            variable: VariableId(1),
            coeffs_low_to_high: vec![q(1), q(-2), q(1)],
            hash: hash_sequence("univariate", &[]),
        });
        let sqfree = squarefree_part_uni(&x_minus_1_squared);
        assert_eq!(sqfree.coeffs_low_to_high, vec![q(-1), q(1)]);
        assert_eq!(
            gcd_uni(&x_minus_1_squared, &derivative_uni(&x_minus_1_squared)).coeffs_low_to_high,
            vec![q(-1), q(1)]
        );
    }

    #[test]
    fn hash_is_stable_for_equal_polynomials() {
        let a = constant_uni(
            VariableId(9),
            RationalQ {
                num: BigInt::from(4),
                den: BigInt::from(1),
            },
        );
        let b = constant_uni(VariableId(9), int_q(4));
        assert_eq!(a.hash, b.hash);
    }
}
