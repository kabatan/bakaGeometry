use serde::{Deserialize, Serialize};

use crate::algebra::monomial_order::MonomialOrder;
use crate::algebra::polynomial_ops::reduce_by_set;
use crate::types::polynomial::{poly_add, poly_mul, poly_sub, zero_poly, SparsePolynomialQ};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MembershipCertificate {
    pub combination_terms: Vec<MembershipTerm>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MembershipTerm {
    pub relation_id: usize,
    pub multiplier: SparsePolynomialQ,
}

pub fn normal_form(
    p: &SparsePolynomialQ,
    basis: &[SparsePolynomialQ],
    order: &MonomialOrder,
) -> SparsePolynomialQ {
    reduce_by_set(p, basis, order).remainder
}

pub fn verify_membership_by_certificate(
    g: &SparsePolynomialQ,
    cert: &MembershipCertificate,
    relations: &[SparsePolynomialQ],
) -> bool {
    let mut sum = zero_poly();
    for term in &cert.combination_terms {
        let Some(relation) = relations.get(term.relation_id) else {
            return false;
        };
        sum = poly_add(&sum, &poly_mul(&term.multiplier, relation));
    }
    poly_sub(&sum, g).terms.is_empty()
}

#[cfg(test)]
mod tests {
    use crate::algebra::monomial_order::lex_order;
    use crate::types::ids::VariableId;
    use crate::types::polynomial::{constant_poly, poly_add, poly_mul, poly_sub, variable_poly};
    use crate::types::rational::int_q;

    use super::*;

    fn x() -> SparsePolynomialQ {
        variable_poly(VariableId(1))
    }

    fn y() -> SparsePolynomialQ {
        variable_poly(VariableId(2))
    }

    #[test]
    fn normal_form_returns_exact_remainder() {
        let order = lex_order(&[VariableId(1)]);
        let basis = vec![poly_sub(&x(), &constant_poly(int_q(1)))];
        let p = poly_sub(&poly_mul(&x(), &x()), &constant_poly(int_q(1)));
        assert!(normal_form(&p, &basis, &order).terms.is_empty());
    }

    #[test]
    fn correct_membership_certificate_reconstructs_identity() {
        let r1 = poly_add(&x(), &constant_poly(int_q(1)));
        let r2 = poly_sub(&y(), &x());
        let g = poly_add(&poly_mul(&x(), &x()), &y());
        let cert = MembershipCertificate {
            combination_terms: vec![
                MembershipTerm {
                    relation_id: 0,
                    multiplier: x(),
                },
                MembershipTerm {
                    relation_id: 1,
                    multiplier: constant_poly(int_q(1)),
                },
            ],
        };
        assert!(verify_membership_by_certificate(&g, &cert, &[r1, r2]));
    }

    #[test]
    fn incorrect_membership_certificate_fails_exact_identity() {
        let r1 = poly_add(&x(), &constant_poly(int_q(1)));
        let g = poly_add(&poly_mul(&x(), &x()), &constant_poly(int_q(2)));
        let cert = MembershipCertificate {
            combination_terms: vec![MembershipTerm {
                relation_id: 0,
                multiplier: x(),
            }],
        };
        assert!(!verify_membership_by_certificate(&g, &cert, &[r1]));
    }

    #[test]
    fn out_of_range_relation_reference_fails() {
        let cert = MembershipCertificate {
            combination_terms: vec![MembershipTerm {
                relation_id: 1,
                multiplier: constant_poly(int_q(1)),
            }],
        };
        assert!(!verify_membership_by_certificate(
            &constant_poly(int_q(1)),
            &cert,
            &[constant_poly(int_q(1))]
        ));
    }
}
