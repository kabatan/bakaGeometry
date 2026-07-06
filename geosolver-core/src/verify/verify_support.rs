use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::algebra::groebner::{
    groebner_elimination_basis, reduce_with_certified_basis, GroebnerOptions,
};
use crate::algebra::monomial_order::elimination_order;
use crate::algebra::normal_form::MembershipCertificate;
use crate::compose::compose::{hash_composed_projection, ComposedProjection};
use crate::compose::separator_elimination::verify_separator_elimination_message;
use crate::result::status::{FailureKind, SolverError, SolverErrorKind};
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::VariableId;
use crate::types::monomial::normalize_monomial;
use crate::types::polynomial::{
    normalize_poly, poly_add, poly_mul, poly_sub, poly_variables, zero_poly, SparsePolynomialQ,
    TermQ,
};
use crate::types::rational::{add_q, int_q, is_zero_q, mul_q, zero_q, RationalQ};
use crate::types::univariate::{
    degree_uni, normalize_univariate, squarefree_part_uni, UniPolynomialQ,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GlobalSupportProofRoute {
    TargetOnlyRootRelationProduct,
    ComposedIdealMembership,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComposedIdealMembershipSupportCertificate {
    pub target: VariableId,
    pub support_hash: Hash,
    pub composed_hash: Hash,
    pub relation_hashes: Vec<Hash>,
    pub multipliers: Vec<SparsePolynomialQ>,
    pub exact_identity_hash: Hash,
    pub certificate_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalSupportCertificate {
    pub target: VariableId,
    pub composed_hash: Hash,
    pub support_hash: Hash,
    pub root_relation_hashes: Vec<Hash>,
    pub proof_route: GlobalSupportProofRoute,
    pub composed_membership: Option<ComposedIdealMembershipSupportCertificate>,
    pub proof_hash: Hash,
    pub certificate_hash: Hash,
}

pub fn verify_global_support(
    support: &UniPolynomialQ,
    composed: &ComposedProjection,
) -> Result<GlobalSupportCertificate, SolverError> {
    if hash_composed_projection(composed) != composed.composed_hash {
        return Err(implementation_bug(
            composed.target,
            "composed projection hash mismatch during support verification",
        ));
    }
    let normalized = normalize_univariate(support.clone());
    if normalized.variable != composed.target {
        return Err(implementation_bug(
            composed.target,
            "support variable does not match composed target",
        ));
    }
    verify_separator_evidence(composed)?;
    let target_only_support = support_from_target_only_relations(composed);
    if let Some(expected) = &target_only_support {
        if normalized == *expected {
            return Ok(target_only_relation_product_certificate(
                &normalized,
                composed,
            ));
        }
    }
    match build_composed_ideal_membership_support_certificate(&normalized, composed) {
        Ok(membership) => Ok(composed_ideal_membership_certificate(
            &normalized,
            composed,
            membership,
        )),
        Err(err) if target_only_support.is_some() => {
            if matches!(
                err.kind,
                SolverErrorKind::Failure(FailureKind::FiniteResourceFailure { .. })
            ) {
                Err(err)
            } else {
                Err(implementation_bug(
                    composed.target,
                    "global support polynomial is neither the verified target-only product nor a certified composed-ideal member",
                ))
            }
        }
        Err(err) => Err(err),
    }
}

pub fn verify_composed_ideal_membership_support_certificate(
    cert: &ComposedIdealMembershipSupportCertificate,
    composed: &ComposedProjection,
) -> Result<(), SolverError> {
    if cert.target != composed.target
        || cert.composed_hash != composed.composed_hash
        || hash_composed_projection(composed) != composed.composed_hash
    {
        return Err(implementation_bug(
            composed.target,
            "composed ideal membership certificate target or composed hash mismatch",
        ));
    }
    let relations = composed_membership_relations(composed);
    if cert.relation_hashes
        != relations
            .iter()
            .map(|relation| relation.hash)
            .collect::<Vec<_>>()
        || cert.multipliers.len() != relations.len()
    {
        return Err(implementation_bug(
            composed.target,
            "composed ideal membership certificate relation binding mismatch",
        ));
    }
    let identity = support_identity_from_multipliers(&cert.multipliers, &relations);
    if !poly_variables(&identity).is_subset(&BTreeSet::from([cert.target]))
        || cert.exact_identity_hash
            != hash_composed_ideal_membership_identity(
                cert.target,
                cert.support_hash,
                &identity,
                &cert.relation_hashes,
                &cert.multipliers,
            )
        || cert.certificate_hash != hash_composed_ideal_membership_support_certificate(cert)
    {
        return Err(implementation_bug(
            composed.target,
            "composed ideal membership certificate exact identity mismatch",
        ));
    }
    Ok(())
}

fn target_only_relation_product_certificate(
    normalized: &UniPolynomialQ,
    composed: &ComposedProjection,
) -> GlobalSupportCertificate {
    let mut cert = GlobalSupportCertificate {
        target: composed.target,
        composed_hash: composed.composed_hash,
        support_hash: normalized.hash,
        root_relation_hashes: composed
            .root_relations
            .iter()
            .map(|relation| relation.hash)
            .collect(),
        proof_route: GlobalSupportProofRoute::TargetOnlyRootRelationProduct,
        composed_membership: None,
        proof_hash: hash_sequence("global-support-proof", &[]),
        certificate_hash: hash_sequence("global-support-certificate", &[]),
    };
    cert.proof_hash = hash_sequence(
        "global-support-proof",
        &cert
            .root_relation_hashes
            .iter()
            .map(|hash| hash.0.to_vec())
            .collect::<Vec<_>>(),
    );
    cert.certificate_hash = hash_global_support_certificate(&cert);
    cert
}

fn composed_ideal_membership_certificate(
    normalized: &UniPolynomialQ,
    composed: &ComposedProjection,
    membership: ComposedIdealMembershipSupportCertificate,
) -> GlobalSupportCertificate {
    let mut chunks = Vec::new();
    chunks.push(membership.certificate_hash.0.to_vec());
    chunks.extend(
        membership
            .relation_hashes
            .iter()
            .map(|hash| hash.0.to_vec()),
    );
    let mut cert = GlobalSupportCertificate {
        target: composed.target,
        composed_hash: composed.composed_hash,
        support_hash: normalized.hash,
        root_relation_hashes: composed
            .root_relations
            .iter()
            .map(|relation| relation.hash)
            .collect(),
        proof_route: GlobalSupportProofRoute::ComposedIdealMembership,
        composed_membership: Some(membership),
        proof_hash: hash_sequence("global-support-proof", &[]),
        certificate_hash: hash_sequence("global-support-certificate", &[]),
    };
    cert.proof_hash = hash_sequence("global-support-proof", &chunks);
    cert.certificate_hash = hash_global_support_certificate(&cert);
    cert
}

fn verify_separator_evidence(composed: &ComposedProjection) -> Result<(), SolverError> {
    if composed.separator_elimination_hashes.len() != composed.separator_elimination_messages.len()
    {
        return Err(implementation_bug(
            composed.target,
            "separator elimination evidence hash/message count mismatch",
        ));
    }
    for (expected_hash, message) in composed
        .separator_elimination_hashes
        .iter()
        .zip(&composed.separator_elimination_messages)
    {
        if *expected_hash != message.package_hash {
            return Err(implementation_bug(
                composed.target,
                "separator elimination message package hash mismatch",
            ));
        }
        verify_separator_elimination_message(
            &composed.message_relations,
            composed.target,
            message,
        )?;
        for relation in &message.relation_generators {
            if !composed.root_relations.iter().any(|root| root == relation) {
                return Err(implementation_bug(
                    composed.target,
                    "separator elimination output is absent from composed root relations",
                ));
            }
        }
    }
    Ok(())
}

pub fn hash_global_support_certificate(cert: &GlobalSupportCertificate) -> Hash {
    let mut chunks = vec![
        cert.target.0.to_be_bytes().to_vec(),
        cert.composed_hash.0.to_vec(),
        cert.support_hash.0.to_vec(),
        format!("{:?}", cert.proof_route).into_bytes(),
        cert.proof_hash.0.to_vec(),
    ];
    for hash in &cert.root_relation_hashes {
        chunks.push(hash.0.to_vec());
    }
    if let Some(membership) = &cert.composed_membership {
        chunks.push(membership.certificate_hash.0.to_vec());
    }
    hash_sequence("global-support-certificate", &chunks)
}

pub fn hash_composed_ideal_membership_support_certificate(
    cert: &ComposedIdealMembershipSupportCertificate,
) -> Hash {
    let mut chunks = vec![
        cert.target.0.to_be_bytes().to_vec(),
        cert.support_hash.0.to_vec(),
        cert.composed_hash.0.to_vec(),
        cert.exact_identity_hash.0.to_vec(),
    ];
    for hash in &cert.relation_hashes {
        chunks.push(hash.0.to_vec());
    }
    chunks.push(Vec::new());
    for multiplier in &cert.multipliers {
        chunks.push(multiplier.hash.0.to_vec());
    }
    hash_sequence("composed-ideal-membership-support-certificate", &chunks)
}

fn build_composed_ideal_membership_support_certificate(
    support: &UniPolynomialQ,
    composed: &ComposedProjection,
) -> Result<ComposedIdealMembershipSupportCertificate, SolverError> {
    let support_sparse = univariate_to_sparse(support);
    let relations = composed_membership_relations(composed);
    if relations.is_empty() || support_sparse.terms.is_empty() {
        return Err(certificate_gap(
            composed.target,
            composed.composed_hash,
            "ComposedIdealMembershipSupportCertificate requires nonzero composed relations",
        ));
    }
    let all_vars = relations
        .iter()
        .flat_map(poly_variables)
        .collect::<BTreeSet<_>>();
    let eliminate = all_vars
        .iter()
        .copied()
        .filter(|var| *var != composed.target)
        .collect::<Vec<_>>();
    let order = elimination_order(&eliminate, &[composed.target]);
    let basis = groebner_elimination_basis(&relations, &order, GroebnerOptions::default())?;
    let reduction = reduce_with_certified_basis(&support_sparse, &basis, &relations)?;
    if !reduction.remainder.terms.is_empty() {
        return Err(certificate_gap(
            composed.target,
            composed.composed_hash,
            "ComposedIdealMembershipSupportCertificate could not reduce S(T) to zero",
        ));
    }
    let multipliers = aggregate_membership_multipliers(
        reduction.membership_certificate,
        relations.len(),
        composed.target,
    )?;
    if poly_sub(
        &support_identity_from_multipliers(&multipliers, &relations),
        &support_sparse,
    )
    .terms
    .is_empty()
    {
        let relation_hashes = relations
            .iter()
            .map(|relation| relation.hash)
            .collect::<Vec<_>>();
        let exact_identity_hash = hash_composed_ideal_membership_identity(
            composed.target,
            support.hash,
            &support_sparse,
            &relation_hashes,
            &multipliers,
        );
        let mut cert = ComposedIdealMembershipSupportCertificate {
            target: composed.target,
            support_hash: support.hash,
            composed_hash: composed.composed_hash,
            relation_hashes,
            multipliers,
            exact_identity_hash,
            certificate_hash: hash_sequence("composed-ideal-membership-support-certificate", &[]),
        };
        cert.certificate_hash = hash_composed_ideal_membership_support_certificate(&cert);
        verify_composed_ideal_membership_support_certificate(&cert, composed)?;
        Ok(cert)
    } else {
        Err(implementation_bug(
            composed.target,
            "composed ideal membership multipliers do not reconstruct support",
        ))
    }
}

fn composed_membership_relations(composed: &ComposedProjection) -> Vec<SparsePolynomialQ> {
    composed
        .message_relations
        .iter()
        .filter(|relation| !relation.terms.is_empty())
        .cloned()
        .collect()
}

fn aggregate_membership_multipliers(
    cert: MembershipCertificate,
    relation_count: usize,
    target: VariableId,
) -> Result<Vec<SparsePolynomialQ>, SolverError> {
    let mut multipliers = vec![zero_poly(); relation_count];
    for term in cert.combination_terms {
        let Some(slot) = multipliers.get_mut(term.relation_id) else {
            return Err(implementation_bug(
                target,
                "membership certificate references a relation outside composed ideal",
            ));
        };
        *slot = poly_add(slot, &term.multiplier);
    }
    Ok(multipliers)
}

fn support_identity_from_multipliers(
    multipliers: &[SparsePolynomialQ],
    relations: &[SparsePolynomialQ],
) -> SparsePolynomialQ {
    multipliers
        .iter()
        .zip(relations)
        .fold(zero_poly(), |acc, (multiplier, relation)| {
            poly_add(&acc, &poly_mul(multiplier, relation))
        })
}

fn hash_composed_ideal_membership_identity(
    target: VariableId,
    support_hash: Hash,
    identity: &SparsePolynomialQ,
    relation_hashes: &[Hash],
    multipliers: &[SparsePolynomialQ],
) -> Hash {
    let mut chunks = vec![
        target.0.to_be_bytes().to_vec(),
        support_hash.0.to_vec(),
        identity.hash.0.to_vec(),
    ];
    for hash in relation_hashes {
        chunks.push(hash.0.to_vec());
    }
    chunks.push(Vec::new());
    for multiplier in multipliers {
        chunks.push(multiplier.hash.0.to_vec());
    }
    hash_sequence("composed-ideal-membership-identity", &chunks)
}

fn univariate_to_sparse(poly: &UniPolynomialQ) -> SparsePolynomialQ {
    let terms = poly
        .coeffs_low_to_high
        .iter()
        .enumerate()
        .filter_map(|(degree, coeff)| {
            if is_zero_q(coeff) {
                return None;
            }
            let monomial = if degree == 0 {
                normalize_monomial(Vec::new())
            } else {
                normalize_monomial(vec![(poly.variable, degree as u32)])
            };
            Some(TermQ {
                coeff: coeff.clone(),
                monomial,
            })
        })
        .collect();
    normalize_poly(SparsePolynomialQ {
        terms,
        hash: hash_sequence("poly", &[]),
    })
}

fn support_from_target_only_relations(composed: &ComposedProjection) -> Option<UniPolynomialQ> {
    let target_set = BTreeSet::from([composed.target]);
    let mut support = normalize_univariate(UniPolynomialQ {
        variable: composed.target,
        coeffs_low_to_high: vec![int_q(1)],
        hash: hash_sequence("univariate", &[]),
    });
    let mut found = false;
    for relation in &composed.root_relations {
        if relation.terms.is_empty() || !poly_variables(relation).is_subset(&target_set) {
            continue;
        }
        let uni = polynomial_to_univariate(relation, composed.target)?;
        if degree_uni(&uni).is_none() || degree_uni(&uni) == Some(0) {
            continue;
        }
        let sq = squarefree_part_uni(&uni);
        support = squarefree_part_uni(&univariate_mul(&support, &sq));
        found = true;
    }
    found.then_some(support)
}

fn polynomial_to_univariate(
    poly: &SparsePolynomialQ,
    target: VariableId,
) -> Option<UniPolynomialQ> {
    let mut coeffs = BTreeMap::<usize, RationalQ>::new();
    for term in &poly.terms {
        let mut degree = 0_usize;
        for (var, exp) in &term.monomial.exponents {
            if *var != target {
                return None;
            }
            degree = *exp as usize;
        }
        let next = coeffs
            .remove(&degree)
            .map_or(term.coeff.clone(), |old| add_q(&old, &term.coeff));
        if !is_zero_q(&next) {
            coeffs.insert(degree, next);
        }
    }
    let max_degree = coeffs.keys().copied().max().unwrap_or(0);
    let mut out = vec![zero_q(); max_degree + 1];
    for (degree, coeff) in coeffs {
        out[degree] = coeff;
    }
    Some(normalize_univariate(UniPolynomialQ {
        variable: target,
        coeffs_low_to_high: out,
        hash: hash_sequence("univariate", &[]),
    }))
}

fn univariate_mul(a: &UniPolynomialQ, b: &UniPolynomialQ) -> UniPolynomialQ {
    if a.variable != b.variable
        || a.coeffs_low_to_high.is_empty()
        || b.coeffs_low_to_high.is_empty()
    {
        return normalize_univariate(UniPolynomialQ {
            variable: a.variable,
            coeffs_low_to_high: Vec::new(),
            hash: hash_sequence("univariate", &[]),
        });
    }
    let mut coeffs = vec![zero_q(); a.coeffs_low_to_high.len() + b.coeffs_low_to_high.len() - 1];
    for (i, ca) in a.coeffs_low_to_high.iter().enumerate() {
        for (j, cb) in b.coeffs_low_to_high.iter().enumerate() {
            coeffs[i + j] = add_q(&coeffs[i + j], &mul_q(ca, cb));
        }
    }
    normalize_univariate(UniPolynomialQ {
        variable: a.variable,
        coeffs_low_to_high: coeffs,
        hash: hash_sequence("univariate", &[]),
    })
}

fn certificate_gap(target: VariableId, hash: Hash, missing: &str) -> SolverError {
    SolverError {
        target: Some(target),
        kind: SolverErrorKind::Failure(FailureKind::CertificateDesignGap {
            constructed_object_hash: hash,
            missing_certificate_kind: missing.to_owned(),
        }),
    }
}

fn implementation_bug(target: VariableId, message: &str) -> SolverError {
    SolverError {
        target: Some(target),
        kind: SolverErrorKind::Failure(FailureKind::ImplementationBug {
            invariant_violated: message.to_owned(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compose::compose::ComposedProjection;
    use crate::result::cost_trace::CompositionCostTrace;
    use crate::types::ids::BlockId;
    use crate::types::polynomial::{constant_poly, poly_sub, variable_poly};

    fn v(id: u32) -> SparsePolynomialQ {
        variable_poly(VariableId(id))
    }

    fn c(value: i64) -> SparsePolynomialQ {
        constant_poly(int_q(value))
    }

    fn support_t_minus_one(target: VariableId) -> UniPolynomialQ {
        normalize_univariate(UniPolynomialQ {
            variable: target,
            coeffs_low_to_high: vec![int_q(-1), int_q(1)],
            hash: hash_sequence("univariate", &[]),
        })
    }

    fn route_b_composed() -> ComposedProjection {
        let target = VariableId(0);
        let x = VariableId(1);
        let mut composed = ComposedProjection {
            target,
            root_block_id: BlockId(0),
            message_relations: vec![poly_sub(&v(x.0), &c(1)), poly_sub(&v(target.0), &v(x.0))],
            root_relations: Vec::new(),
            source_message_hashes: vec![hash_sequence("route-b-message", &[b"0".to_vec()])],
            separator_elimination_hashes: Vec::new(),
            separator_elimination_messages: Vec::new(),
            composition_cost: CompositionCostTrace {
                relation_count_before: 2,
                relation_count_after: 0,
            },
            composed_hash: hash_sequence("composed-projection", &[]),
        };
        composed.composed_hash = hash_composed_projection(&composed);
        composed
    }

    #[test]
    fn composed_ideal_membership_route_verifies_support_without_target_only_root_relation() {
        let composed = route_b_composed();
        let support = support_t_minus_one(composed.target);

        let cert = verify_global_support(&support, &composed).unwrap();

        assert_eq!(
            cert.proof_route,
            GlobalSupportProofRoute::ComposedIdealMembership
        );
        assert!(cert.root_relation_hashes.is_empty());
        let membership = cert.composed_membership.as_ref().unwrap();
        verify_composed_ideal_membership_support_certificate(membership, &composed).unwrap();
    }

    #[test]
    fn composed_ideal_membership_route_rejects_multiplier_tamper() {
        let composed = route_b_composed();
        let support = support_t_minus_one(composed.target);
        let mut membership = verify_global_support(&support, &composed)
            .unwrap()
            .composed_membership
            .unwrap();
        membership.multipliers[0] = zero_poly();

        assert!(
            verify_composed_ideal_membership_support_certificate(&membership, &composed).is_err()
        );
    }

    #[test]
    fn composed_ideal_membership_route_rejects_removed_relation() {
        let composed = route_b_composed();
        let support = support_t_minus_one(composed.target);
        let membership = verify_global_support(&support, &composed)
            .unwrap()
            .composed_membership
            .unwrap();
        let mut removed = composed.clone();
        removed.message_relations.pop();
        removed.composed_hash = hash_composed_projection(&removed);

        assert!(
            verify_composed_ideal_membership_support_certificate(&membership, &removed).is_err()
        );
    }
}
