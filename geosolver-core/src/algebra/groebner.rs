use std::collections::{BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::algebra::monomial_order::MonomialOrder;
use crate::algebra::normal_form::{
    verify_membership_by_certificate, MembershipCertificate, MembershipTerm,
};
use crate::algebra::polynomial_ops::{leading_term, reduce_by_set, s_polynomial};
use crate::result::status::{FailureKind, SolverError, SolverErrorKind, StageId};
use crate::types::hash::hash_sequence;
use crate::types::ids::VariableId;
use crate::types::monomial::{monomial_div, normalize_monomial, Monomial};
use crate::types::polynomial::{
    max_poly_coefficient_height_bits, normalize_poly, poly_mul, poly_sub, poly_variables,
    zero_poly, SparsePolynomialQ, TermQ,
};
use crate::types::rational::{div_q, int_q, is_zero_q, RationalQ};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroebnerOptions {
    pub max_pairs: usize,
    pub max_basis_size: usize,
}

impl Default for GroebnerOptions {
    fn default() -> Self {
        Self {
            max_pairs: 256,
            max_basis_size: 64,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertifiedPolynomialQ {
    pub polynomial: SparsePolynomialQ,
    pub certificate: MembershipCertificate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroebnerBasisResult {
    pub basis: Vec<CertifiedPolynomialQ>,
    pub pairs_processed: usize,
    pub order: MonomialOrder,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertifiedReduction {
    pub remainder: SparsePolynomialQ,
    pub membership_certificate: MembershipCertificate,
}

pub fn groebner_elimination_basis(
    relations: &[SparsePolynomialQ],
    order: &MonomialOrder,
    options: GroebnerOptions,
) -> Result<GroebnerBasisResult, SolverError> {
    let mut basis: Vec<CertifiedPolynomialQ> = relations
        .iter()
        .enumerate()
        .filter_map(|(idx, relation)| {
            let p = normalize_poly(relation.clone());
            if p.terms.is_empty() {
                None
            } else {
                Some(CertifiedPolynomialQ {
                    polynomial: p,
                    certificate: MembershipCertificate {
                        combination_terms: vec![MembershipTerm {
                            relation_id: idx,
                            multiplier: constant_poly(int_q(1)),
                        }],
                    },
                })
            }
        })
        .collect();

    let mut pairs = VecDeque::new();
    for i in 0..basis.len() {
        for j in (i + 1)..basis.len() {
            pairs.push_back((i, j));
        }
    }

    let mut pairs_processed = 0;
    while let Some((i, j)) = pairs.pop_front() {
        pairs_processed += 1;
        if pairs_processed > options.max_pairs || basis.len() > options.max_basis_size {
            let observed_height = max_poly_coefficient_height_bits(&basis_polys(&basis))
                .max(max_poly_coefficient_height_bits(relations));
            return Err(finite_resource_failure(
                "GroebnerLocalPairLimit",
                pairs_processed,
                basis.len(),
                observed_height,
            ));
        }

        let s = s_polynomial(&basis[i].polynomial, &basis[j].polynomial, order);
        if s.terms.is_empty() {
            continue;
        }
        let reduction = reduce_by_set(&s, &basis_polys(&basis), order);
        if reduction.remainder.terms.is_empty() {
            continue;
        }
        let mut cert = s_pair_certificate(&basis[i], &basis[j], order)?;
        for (quotient, reducer) in reduction.quotients.iter().zip(basis.iter()) {
            if quotient.terms.is_empty() {
                continue;
            }
            cert = subtract_cert(&cert, &scale_cert(&reducer.certificate, quotient));
        }
        if !verify_membership_by_certificate(&reduction.remainder, &cert, relations) {
            return Err(implementation_bug(
                "Groebner remainder certificate failed exact membership reconstruction",
            ));
        }
        let new_index = basis.len();
        basis.push(CertifiedPolynomialQ {
            polynomial: reduction.remainder,
            certificate: cert,
        });
        for k in 0..new_index {
            pairs.push_back((k, new_index));
        }
    }

    Ok(GroebnerBasisResult {
        basis,
        pairs_processed,
        order: order.clone(),
    })
}

pub fn reduce_with_certified_basis(
    polynomial: &SparsePolynomialQ,
    basis_result: &GroebnerBasisResult,
    authorized_relations: &[SparsePolynomialQ],
) -> Result<CertifiedReduction, SolverError> {
    let basis_polynomials = basis_polys(&basis_result.basis);
    let reduction = reduce_by_set(polynomial, &basis_polynomials, &basis_result.order);
    let mut certificate = MembershipCertificate {
        combination_terms: Vec::new(),
    };
    for (quotient, basis_entry) in reduction.quotients.iter().zip(&basis_result.basis) {
        if quotient.terms.is_empty() {
            continue;
        }
        certificate = add_cert(
            &certificate,
            &scale_cert(&basis_entry.certificate, quotient),
        );
    }
    let represented_difference = poly_sub(polynomial, &reduction.remainder);
    if !verify_membership_by_certificate(
        &represented_difference,
        &certificate,
        authorized_relations,
    ) {
        return Err(implementation_bug(
            "certified Groebner reduction failed exact membership reconstruction",
        ));
    }
    Ok(CertifiedReduction {
        remainder: reduction.remainder,
        membership_certificate: certificate,
    })
}

pub fn extract_elimination_generators(
    basis: &[SparsePolynomialQ],
    keep_variables: &BTreeSet<VariableId>,
) -> Vec<SparsePolynomialQ> {
    basis
        .iter()
        .filter(|p| polynomial_in_keep_variables(p, keep_variables))
        .cloned()
        .collect()
}

pub fn extract_certified_elimination_generators(
    basis: &[CertifiedPolynomialQ],
    keep_variables: &BTreeSet<VariableId>,
) -> Vec<CertifiedPolynomialQ> {
    basis
        .iter()
        .filter(|p| polynomial_in_keep_variables(&p.polynomial, keep_variables))
        .cloned()
        .collect()
}

pub fn polynomial_in_keep_variables(
    p: &SparsePolynomialQ,
    keep_variables: &BTreeSet<VariableId>,
) -> bool {
    poly_variables(p)
        .iter()
        .all(|var| keep_variables.contains(var))
}

fn basis_polys(basis: &[CertifiedPolynomialQ]) -> Vec<SparsePolynomialQ> {
    basis.iter().map(|entry| entry.polynomial.clone()).collect()
}

pub(crate) fn certified_s_pair(
    f: &CertifiedPolynomialQ,
    g: &CertifiedPolynomialQ,
    order: &MonomialOrder,
) -> Result<CertifiedPolynomialQ, SolverError> {
    Ok(CertifiedPolynomialQ {
        polynomial: s_polynomial(&f.polynomial, &g.polynomial, order),
        certificate: s_pair_certificate(f, g, order)?,
    })
}

fn s_pair_certificate(
    f: &CertifiedPolynomialQ,
    g: &CertifiedPolynomialQ,
    order: &MonomialOrder,
) -> Result<MembershipCertificate, SolverError> {
    let lt_f = leading_term(&f.polynomial, order)
        .ok_or_else(|| implementation_bug("zero polynomial in S-pair certificate"))?;
    let lt_g = leading_term(&g.polynomial, order)
        .ok_or_else(|| implementation_bug("zero polynomial in S-pair certificate"))?;
    let lcm = monomial_lcm(&lt_f.monomial, &lt_g.monomial);
    let m_f = monomial_div(&lcm, &lt_f.monomial)
        .ok_or_else(|| implementation_bug("LCM not divisible by leading monomial"))?;
    let m_g = monomial_div(&lcm, &lt_g.monomial)
        .ok_or_else(|| implementation_bug("LCM not divisible by leading monomial"))?;
    let c_f = div_q(&int_q(1), &lt_f.coeff)
        .map_err(|_| implementation_bug("zero leading coefficient in S-pair"))?;
    let c_g = div_q(&int_q(1), &lt_g.coeff)
        .map_err(|_| implementation_bug("zero leading coefficient in S-pair"))?;
    let left = scale_cert(&f.certificate, &single_term_poly(c_f, m_f));
    let right = scale_cert(&g.certificate, &single_term_poly(c_g, m_g));
    Ok(subtract_cert(&left, &right))
}

pub(crate) fn scale_membership_certificate(
    cert: &MembershipCertificate,
    multiplier: &SparsePolynomialQ,
) -> MembershipCertificate {
    MembershipCertificate {
        combination_terms: cert
            .combination_terms
            .iter()
            .map(|term| MembershipTerm {
                relation_id: term.relation_id,
                multiplier: poly_mul(multiplier, &term.multiplier),
            })
            .collect(),
    }
}

pub(crate) fn subtract_membership_certificate(
    a: &MembershipCertificate,
    b: &MembershipCertificate,
) -> MembershipCertificate {
    let mut terms = a.combination_terms.clone();
    terms.extend(b.combination_terms.iter().map(|term| MembershipTerm {
        relation_id: term.relation_id,
        multiplier: poly_sub(&zero_poly(), &term.multiplier),
    }));
    MembershipCertificate {
        combination_terms: terms,
    }
}

fn scale_cert(
    cert: &MembershipCertificate,
    multiplier: &SparsePolynomialQ,
) -> MembershipCertificate {
    scale_membership_certificate(cert, multiplier)
}

fn subtract_cert(a: &MembershipCertificate, b: &MembershipCertificate) -> MembershipCertificate {
    subtract_membership_certificate(a, b)
}

fn add_cert(a: &MembershipCertificate, b: &MembershipCertificate) -> MembershipCertificate {
    let mut terms = a.combination_terms.clone();
    terms.extend(b.combination_terms.iter().cloned());
    MembershipCertificate {
        combination_terms: terms,
    }
}

fn constant_poly(coeff: RationalQ) -> SparsePolynomialQ {
    single_term_poly(coeff, normalize_monomial(Vec::new()))
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
    let mut entries = a.exponents.clone();
    for (var, exp_b) in &b.exponents {
        let exp_a = a
            .exponents
            .iter()
            .find(|(v, _)| v == var)
            .map_or(0, |(_, e)| *e);
        if *exp_b > exp_a {
            entries.push((*var, *exp_b - exp_a));
        }
    }
    normalize_monomial(entries)
}

fn finite_resource_failure(
    stage: &str,
    rows: usize,
    cols: usize,
    coefficient_height_bits: usize,
) -> SolverError {
    SolverError {
        target: None,
        kind: SolverErrorKind::Failure(FailureKind::FiniteResourceFailure {
            stage: StageId(stage.to_string()),
            block_id: None,
            matrix_rows: Some(rows),
            matrix_cols: Some(cols),
            matrix_density: None,
            quotient_rank_estimate: None,
            coefficient_height_bits: Some(coefficient_height_bits),
            memory_bytes: None,
        }),
    }
}

pub(crate) fn implementation_bug(message: impl Into<String>) -> SolverError {
    SolverError {
        target: None,
        kind: SolverErrorKind::Failure(FailureKind::ImplementationBug {
            invariant_violated: message.into(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use crate::algebra::monomial_order::elimination_order;
    use crate::types::polynomial::{constant_poly as type_constant_poly, poly_sub, variable_poly};

    use super::*;

    #[test]
    fn local_groebner_eliminates_to_keep_variable_with_certificate() {
        let x = VariableId(1);
        let y = VariableId(2);
        let relations = vec![
            poly_sub(&variable_poly(y), &variable_poly(x)),
            poly_sub(&variable_poly(y), &type_constant_poly(int_q(1))),
        ];
        let order = elimination_order(&[y], &[x]);
        let result =
            groebner_elimination_basis(&relations, &order, GroebnerOptions::default()).unwrap();
        let keep = BTreeSet::from([x]);
        let generators = extract_certified_elimination_generators(&result.basis, &keep);
        assert!(generators.iter().any(|g| {
            polynomial_in_keep_variables(&g.polynomial, &keep)
                && verify_membership_by_certificate(&g.polynomial, &g.certificate, &relations)
        }));
    }
}
