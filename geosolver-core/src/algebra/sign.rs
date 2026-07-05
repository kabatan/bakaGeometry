use serde::{Deserialize, Serialize};

use crate::algebra::real_root::{cmp_q, sturm_sequence};
use crate::roots::isolate::RealRootRecord;
use crate::types::hash::Hash;
use crate::types::rational::{is_zero_q, sign_q};
use crate::types::univariate::{degree_uni, derivative_uni, eval_uni_q, UniPolynomialQ};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignDetermination {
    Negative,
    Zero,
    Positive,
    RefinementRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThomEncoding {
    pub polynomial_hash: Hash,
    pub root_index: usize,
    pub signs_by_derivative_order: Vec<SignDetermination>,
    pub all_signs_determined: bool,
}

pub fn sign_at_algebraic_root(poly: &UniPolynomialQ, root: &RealRootRecord) -> SignDetermination {
    if poly.hash == root.support_hash {
        return SignDetermination::Zero;
    }
    if degree_uni(poly).is_none() {
        return sign_from_i8(poly.coeffs_low_to_high.first().map_or(0, sign_q));
    }

    let interval = &root.isolating_interval;
    if interval.lo == interval.hi {
        return sign_from_i8(sign_q(&eval_uni_q(poly, &interval.lo)));
    }
    if cmp_q(&interval.lo, &interval.hi).is_gt() {
        return SignDetermination::RefinementRequired;
    }

    let lo_value = eval_uni_q(poly, &interval.lo);
    let hi_value = eval_uni_q(poly, &interval.hi);
    if is_zero_q(&lo_value) || is_zero_q(&hi_value) {
        return SignDetermination::RefinementRequired;
    }
    let lo_sign = sign_q(&lo_value);
    let hi_sign = sign_q(&hi_value);
    if lo_sign != hi_sign {
        return SignDetermination::RefinementRequired;
    }
    if roots_in_interval(poly, interval) == 0 {
        sign_from_i8(lo_sign)
    } else {
        SignDetermination::RefinementRequired
    }
}

pub fn thom_encoding(poly: &UniPolynomialQ, root: &RealRootRecord) -> ThomEncoding {
    let mut current = poly.clone();
    let mut signs = Vec::new();
    loop {
        let sign = sign_at_algebraic_root(&current, root);
        signs.push(sign);
        if degree_uni(&current).is_none() {
            break;
        }
        current = derivative_uni(&current);
    }
    let all_signs_determined = signs
        .iter()
        .all(|sign| *sign != SignDetermination::RefinementRequired);
    ThomEncoding {
        polynomial_hash: poly.hash,
        root_index: root.root_index,
        signs_by_derivative_order: signs,
        all_signs_determined,
    }
}

fn roots_in_interval(
    poly: &UniPolynomialQ,
    interval: &crate::types::interval::RationalInterval,
) -> usize {
    let seq = sturm_sequence(poly);
    let v_lo = sign_variations_at(&seq, &interval.lo);
    let v_hi = sign_variations_at(&seq, &interval.hi);
    v_lo.saturating_sub(v_hi)
}

fn sign_variations_at(seq: &[UniPolynomialQ], x: &crate::types::rational::RationalQ) -> usize {
    let mut last = 0_i8;
    let mut variations = 0;
    for poly in seq {
        let sign = sign_q(&eval_uni_q(poly, x));
        if sign == 0 {
            continue;
        }
        if last != 0 && sign != last {
            variations += 1;
        }
        last = sign;
    }
    variations
}

fn sign_from_i8(sign: i8) -> SignDetermination {
    match sign {
        -1 => SignDetermination::Negative,
        0 => SignDetermination::Zero,
        1 => SignDetermination::Positive,
        _ => SignDetermination::RefinementRequired,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algebra::real_root::isolate_real_roots_sturm;
    use crate::types::hash::hash_sequence;
    use crate::types::ids::VariableId;
    use crate::types::rational::int_q;
    use crate::types::univariate::normalize_univariate;

    fn poly(variable: VariableId, coeffs: Vec<i64>) -> UniPolynomialQ {
        normalize_univariate(UniPolynomialQ {
            variable,
            coeffs_low_to_high: coeffs.into_iter().map(int_q).collect(),
            hash: hash_sequence("univariate", &[]),
        })
    }

    #[test]
    fn sign_at_isolated_root_is_exact_when_interval_has_no_guard_root() {
        let x = VariableId(1);
        let support = poly(x, vec![-2, 0, 1]);
        let roots = isolate_real_roots_sturm(&support).unwrap();
        let positive_root = &roots[1];
        let guard = poly(x, vec![0, 1]);
        assert_eq!(
            sign_at_algebraic_root(&guard, positive_root),
            SignDetermination::Positive
        );
        assert_eq!(
            sign_at_algebraic_root(&support, positive_root),
            SignDetermination::Zero
        );
    }

    #[test]
    fn thom_encoding_records_derivative_signs() {
        let x = VariableId(1);
        let support = poly(x, vec![-2, 0, 1]);
        let roots = isolate_real_roots_sturm(&support).unwrap();
        let encoding = thom_encoding(&support, &roots[1]);
        assert!(encoding.all_signs_determined);
        assert_eq!(
            encoding.signs_by_derivative_order[0],
            SignDetermination::Zero
        );
        assert_eq!(
            encoding.signs_by_derivative_order[1],
            SignDetermination::Positive
        );
    }
}
