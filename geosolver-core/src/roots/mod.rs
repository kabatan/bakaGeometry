pub mod algebraic_number;
pub mod decode;
pub mod isolate;
pub mod squarefree;

pub use algebraic_number::*;
pub use decode::*;
pub use isolate::*;
pub use squarefree::*;

#[cfg(test)]
mod tests {
    use num_bigint::BigInt;

    use super::algebraic_number::{
        algebraic_roots_from_support, compare_algebraic_roots, refine_algebraic_root_to_width,
        AlgebraicRootOrdering,
    };
    use super::decode::{decode_candidates, hash_target_candidate};
    use super::isolate::{isolate_real_roots, RootIsolationOptions};
    use super::squarefree::squarefree_support;
    use crate::solver::options::RootIsolationMethod;
    use crate::types::hash::hash_sequence;
    use crate::types::ids::VariableId;
    use crate::types::interval::interval_contains_q;
    use crate::types::rational::{int_q, new_q, sub_q};
    use crate::types::univariate::{eval_uni_q, normalize_univariate, UniPolynomialQ};

    fn poly(variable: VariableId, coeffs: Vec<i64>) -> UniPolynomialQ {
        normalize_univariate(UniPolynomialQ {
            variable,
            coeffs_low_to_high: coeffs.into_iter().map(int_q).collect(),
            hash: hash_sequence("univariate", &[]),
        })
    }

    fn root_options() -> RootIsolationOptions {
        RootIsolationOptions {
            method: RootIsolationMethod::Sturm,
        }
    }

    #[test]
    fn p12_squarefree_rejects_zero_and_removes_repeated_factors() {
        let t = VariableId(0);
        let zero = normalize_univariate(UniPolynomialQ {
            variable: t,
            coeffs_low_to_high: Vec::new(),
            hash: hash_sequence("univariate", &[]),
        });
        assert!(squarefree_support(&zero).is_err());

        let repeated = poly(t, vec![-1, 3, -3, 1]);
        let squarefree = squarefree_support(&repeated).unwrap();
        assert_eq!(squarefree.coeffs_low_to_high, vec![int_q(-1), int_q(1)]);
    }

    #[test]
    fn p12_exact_isolation_handles_no_real_one_rational_and_multiple_rational_roots() {
        let t = VariableId(0);
        let no_real = poly(t, vec![1, 0, 1]);
        assert!(isolate_real_roots(&no_real, root_options())
            .unwrap()
            .is_empty());

        let one = poly(t, vec![-2, 1]);
        let roots = isolate_real_roots(&one, root_options()).unwrap();
        assert_eq!(roots.len(), 1);
        assert_eq!(
            roots[0].support_hash,
            squarefree_support(&one).unwrap().hash
        );
        assert!(interval_contains_q(&roots[0].isolating_interval, &int_q(2)));

        let two = poly(t, vec![3, -4, 1]);
        let roots = isolate_real_roots(&two, root_options()).unwrap();
        assert_eq!(roots.len(), 2);
        assert_eq!(roots[0].root_index, 0);
        assert_eq!(roots[1].root_index, 1);
        assert!(interval_contains_q(&roots[0].isolating_interval, &int_q(1)));
        assert!(interval_contains_q(&roots[1].isolating_interval, &int_q(3)));
    }

    #[test]
    fn p12_exact_isolation_handles_irrational_repeated_and_high_coefficient_roots() {
        let t = VariableId(0);
        let irrational = poly(t, vec![-2, 0, 1]);
        let roots = isolate_real_roots(&irrational, root_options()).unwrap();
        assert_eq!(roots.len(), 2);
        for root in &roots {
            let lo = eval_uni_q(&irrational, &root.isolating_interval.lo);
            let hi = eval_uni_q(&irrational, &root.isolating_interval.hi);
            assert!(lo.num.sign() != hi.num.sign() || lo.num == 0.into() || hi.num == 0.into());
        }

        let repeated = poly(t, vec![1, -2, 1]);
        let squarefree = squarefree_support(&repeated).unwrap();
        let roots = isolate_real_roots(&repeated, root_options()).unwrap();
        assert_eq!(squarefree.coeffs_low_to_high, vec![int_q(-1), int_q(1)]);
        assert_eq!(roots.len(), 1);
        assert!(interval_contains_q(&roots[0].isolating_interval, &int_q(1)));

        let high_coefficient = normalize_univariate(UniPolynomialQ {
            variable: t,
            coeffs_low_to_high: vec![int_q(-1), int_q(1_000_000)],
            hash: hash_sequence("univariate", &[]),
        });
        let roots = isolate_real_roots(&high_coefficient, root_options()).unwrap();
        assert_eq!(roots.len(), 1);
        assert!(interval_contains_q(
            &roots[0].isolating_interval,
            &new_q(BigInt::from(1), BigInt::from(1_000_000))
        ));
    }

    #[test]
    fn p12_decode_candidates_bind_target_support_index_interval_and_hash() {
        let t = VariableId(0);
        let support = squarefree_support(&poly(t, vec![3, -4, 1])).unwrap();
        let roots = isolate_real_roots(&support, root_options()).unwrap();
        let candidates = decode_candidates(t, &support, &roots);
        assert_eq!(candidates.len(), roots.len());
        for (candidate, root) in candidates.iter().zip(roots.iter()) {
            assert_eq!(candidate.target, t);
            assert_eq!(candidate.support_hash, support.hash);
            assert_eq!(candidate.root_index, root.root_index);
            assert_eq!(candidate.isolating_interval, root.isolating_interval);
            assert_eq!(
                candidate.candidate_hash,
                hash_target_candidate(t, support.hash, root.root_index, &root.isolating_interval)
            );
        }
    }

    #[test]
    fn p12_algebraic_records_bind_interval_hash_and_compare_disjoint_roots() {
        let t = VariableId(0);
        let support = squarefree_support(&poly(t, vec![3, -4, 1])).unwrap();
        let records = algebraic_roots_from_support(&support, root_options()).unwrap();
        assert_eq!(records.len(), 2);
        assert_ne!(records[0].root_hash, records[1].root_hash);
        assert_eq!(
            compare_algebraic_roots(&records[0], &records[1]),
            Some(AlgebraicRootOrdering::Less)
        );

        let refined =
            refine_algebraic_root_to_width(&records[0], new_q(BigInt::from(1), BigInt::from(2)))
                .unwrap();
        let width = sub_q(
            &refined.isolating_interval.hi,
            &refined.isolating_interval.lo,
        );
        assert!(
            crate::algebra::real_root::cmp_q(&width, &new_q(BigInt::from(1), BigInt::from(2)))
                != std::cmp::Ordering::Greater
        );
    }
}
