use std::cmp::Ordering;

use num_bigint::BigInt;
use num_traits::One;

use crate::result::status::{AlgebraicReason, FailureKind, SolverError, SolverErrorKind, StageId};
use crate::roots::isolate::RealRootRecord;
use crate::types::hash::hash_sequence;
use crate::types::ids::VariableId;
use crate::types::interval::RationalInterval;
use crate::types::rational::{
    add_q, div_q, int_q, is_zero_q, mul_q, neg_q, new_q, sign_q, sub_q, zero_q, RationalQ,
};
use crate::types::univariate::{
    degree_uni, derivative_uni, eval_uni_q, normalize_univariate, squarefree_part_uni,
    UniPolynomialQ,
};

pub fn sturm_sequence(p: &UniPolynomialQ) -> Vec<UniPolynomialQ> {
    let p0 = normalize_univariate(p.clone());
    if degree_uni(&p0).is_none() {
        return vec![p0];
    }
    let p1 = derivative_uni(&p0);
    if degree_uni(&p1).is_none() {
        return vec![p0, p1];
    }

    let mut seq = vec![p0, p1];
    loop {
        let len = seq.len();
        let rem = rem_uni(&seq[len - 2], &seq[len - 1]);
        if degree_uni(&rem).is_none() {
            break;
        }
        seq.push(neg_uni(&rem));
    }
    seq
}

pub fn isolate_real_roots_sturm(p: &UniPolynomialQ) -> Result<Vec<RealRootRecord>, SolverError> {
    isolate_real_roots_sturm_with_max_width(p, int_q(1))
}

pub fn isolate_real_roots_sturm_with_max_width(
    p: &UniPolynomialQ,
    max_width: RationalQ,
) -> Result<Vec<RealRootRecord>, SolverError> {
    if cmp_q(&max_width, &zero_q()) != Ordering::Greater {
        return Err(algorithmic_hard_case(
            p.variable,
            "SturmIsolation",
            "root isolation max interval width must be positive",
        ));
    }
    let squarefree = squarefree_part_uni(&normalize_univariate(p.clone()));
    if degree_uni(&squarefree).is_none() {
        if squarefree.coeffs_low_to_high.is_empty() {
            return Err(algorithmic_hard_case(
                squarefree.variable,
                "SturmIsolation",
                "zero support polynomial has no finite isolating root set",
            ));
        }
        return Ok(Vec::new());
    }

    let seq = sturm_sequence(&squarefree);
    let bound = cauchy_bound(&squarefree)?;
    let lo = neg_q(&bound);
    let hi = bound;
    let total = root_count_between(&seq, &lo, &hi)?;
    let mut intervals = Vec::new();
    isolate_interval(&seq, &squarefree, lo, hi, total, &max_width, &mut intervals)?;
    intervals.sort_by(|a, b| cmp_q(&a.lo, &b.lo));

    Ok(intervals
        .into_iter()
        .enumerate()
        .map(|(root_index, isolating_interval)| RealRootRecord {
            support_hash: squarefree.hash,
            root_index,
            isolating_interval,
        })
        .collect())
}

pub fn isolate_real_roots_descartes(
    p: &UniPolynomialQ,
) -> Result<Vec<RealRootRecord>, SolverError> {
    isolate_real_roots_sturm(p)
}

fn isolate_interval(
    seq: &[UniPolynomialQ],
    support: &UniPolynomialQ,
    lo: RationalQ,
    hi: RationalQ,
    root_count: usize,
    max_width: &RationalQ,
    intervals: &mut Vec<RationalInterval>,
) -> Result<(), SolverError> {
    if root_count == 0 {
        return Ok(());
    }
    if root_count == 1 && !interval_width_gt(&lo, &hi, max_width) {
        intervals.push(RationalInterval { lo, hi });
        return Ok(());
    }

    let split = choose_nonroot_split(support, &lo, &hi)?;
    let left_count = root_count_between(seq, &lo, &split)?;
    if left_count > root_count {
        return Err(algorithmic_hard_case(
            support.variable,
            "SturmIsolation",
            "Sturm count increased after interval subdivision",
        ));
    }
    let right_count = root_count - left_count;
    isolate_interval(
        seq,
        support,
        lo,
        split.clone(),
        left_count,
        max_width,
        intervals,
    )?;
    isolate_interval(seq, support, split, hi, right_count, max_width, intervals)
}

fn choose_nonroot_split(
    support: &UniPolynomialQ,
    lo: &RationalQ,
    hi: &RationalQ,
) -> Result<RationalQ, SolverError> {
    for denom in 2..=128 {
        for numer in 1..denom {
            let point = affine_interval_point(lo, hi, numer, denom);
            if !is_zero_q(&eval_uni_q(support, &point)) {
                return Ok(point);
            }
        }
    }
    Err(algorithmic_hard_case(
        support.variable,
        "SturmIsolation",
        "could not find a rational subdivision point avoiding support roots",
    ))
}

fn root_count_between(
    seq: &[UniPolynomialQ],
    lo: &RationalQ,
    hi: &RationalQ,
) -> Result<usize, SolverError> {
    let v_lo = sign_variations_at(seq, lo);
    let v_hi = sign_variations_at(seq, hi);
    if v_lo < v_hi {
        return Err(algorithmic_hard_case(
            seq.first().map_or(VariableId(0), |p| p.variable),
            "SturmIsolation",
            "Sturm variation count is negative",
        ));
    }
    Ok(v_lo - v_hi)
}

fn sign_variations_at(seq: &[UniPolynomialQ], x: &RationalQ) -> usize {
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

fn cauchy_bound(p: &UniPolynomialQ) -> Result<RationalQ, SolverError> {
    let Some(deg) = degree_uni(p) else {
        return Ok(int_q(1));
    };
    let lc = &p.coeffs_low_to_high[deg];
    if is_zero_q(lc) {
        return Err(algorithmic_hard_case(
            p.variable,
            "SturmIsolation",
            "normalized polynomial has zero leading coefficient",
        ));
    }
    let mut max_ratio = zero_q();
    for coeff in p.coeffs_low_to_high.iter().take(deg) {
        let ratio = div_q(&abs_q(coeff), &abs_q(lc)).map_err(|_| {
            algorithmic_hard_case(
                p.variable,
                "SturmIsolation",
                "leading coefficient vanished during Cauchy bound",
            )
        })?;
        if cmp_q(&ratio, &max_ratio) == Ordering::Greater {
            max_ratio = ratio;
        }
    }
    Ok(add_q(&int_q(1), &max_ratio))
}

fn affine_interval_point(lo: &RationalQ, hi: &RationalQ, numer: i64, denom: i64) -> RationalQ {
    let left_weight = int_q(denom - numer);
    let right_weight = int_q(numer);
    let weighted = add_q(&mul_q(lo, &left_weight), &mul_q(hi, &right_weight));
    div_q(&weighted, &int_q(denom)).expect("positive denominator")
}

fn interval_width_gt(lo: &RationalQ, hi: &RationalQ, max_width: &RationalQ) -> bool {
    cmp_q(&sub_q(hi, lo), max_width) == Ordering::Greater
}

fn rem_uni(a: &UniPolynomialQ, b: &UniPolynomialQ) -> UniPolynomialQ {
    assert_eq!(a.variable, b.variable, "univariate variable mismatch");
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
    assert_eq!(a.variable, b.variable, "univariate variable mismatch");
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

fn neg_uni(p: &UniPolynomialQ) -> UniPolynomialQ {
    normalize_univariate(UniPolynomialQ {
        variable: p.variable,
        coeffs_low_to_high: p.coeffs_low_to_high.iter().map(neg_q).collect(),
        hash: hash_sequence("univariate", &[]),
    })
}

fn abs_q(q: &RationalQ) -> RationalQ {
    if q.num.sign() == num_bigint::Sign::Minus {
        neg_q(q)
    } else {
        q.clone()
    }
}

pub(crate) fn cmp_q(a: &RationalQ, b: &RationalQ) -> Ordering {
    (&a.num * &b.den).cmp(&(&b.num * &a.den))
}

fn algorithmic_hard_case(target: VariableId, stage: &str, reason: &str) -> SolverError {
    SolverError {
        target: Some(target),
        kind: SolverErrorKind::Failure(FailureKind::AlgorithmicHardCase {
            stage: StageId(stage.to_owned()),
            reason: AlgebraicReason(reason.to_owned()),
            minimal_block_hash: hash_sequence(
                "p3f-real-root-hard-case",
                &[reason.as_bytes().to_vec()],
            ),
        }),
    }
}

#[allow(dead_code)]
fn rational_from_bigint(num: BigInt) -> RationalQ {
    new_q(num, BigInt::one())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::hash::hash_sequence;

    fn poly(variable: VariableId, coeffs: Vec<i64>) -> UniPolynomialQ {
        normalize_univariate(UniPolynomialQ {
            variable,
            coeffs_low_to_high: coeffs.into_iter().map(int_q).collect(),
            hash: hash_sequence("univariate", &[]),
        })
    }

    #[test]
    fn sturm_sequence_counts_two_real_roots_exactly() {
        let x = VariableId(1);
        let p = poly(x, vec![-2, 0, 1]);
        let roots = isolate_real_roots_sturm(&p).unwrap();
        assert_eq!(roots.len(), 2);
        for root in &roots {
            let lo_val = eval_uni_q(&p, &root.isolating_interval.lo);
            let hi_val = eval_uni_q(&p, &root.isolating_interval.hi);
            assert!(sign_q(&lo_val) * sign_q(&hi_val) <= 0);
        }
        assert!(
            cmp_q(
                &roots[0].isolating_interval.hi,
                &roots[1].isolating_interval.lo
            ) != Ordering::Greater
        );
    }

    #[test]
    fn rational_midpoint_roots_do_not_break_isolation() {
        let x = VariableId(1);
        let p = poly(x, vec![-1, 0, 1]);
        let roots = isolate_real_roots_sturm(&p).unwrap();
        assert_eq!(roots.len(), 2);
        assert_eq!(roots[0].support_hash, squarefree_part_uni(&p).hash);
    }
}
