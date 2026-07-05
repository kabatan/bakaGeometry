use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::compose::compose::{hash_composed_projection, ComposedProjection};
use crate::result::status::{FailureKind, SolverError, SolverErrorKind};
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::VariableId;
use crate::types::polynomial::{poly_variables, SparsePolynomialQ};
use crate::types::rational::{add_q, int_q, is_zero_q, mul_q, zero_q, RationalQ};
use crate::types::univariate::{
    degree_uni, normalize_univariate, squarefree_part_uni, UniPolynomialQ,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalSupportCertificate {
    pub target: VariableId,
    pub composed_hash: Hash,
    pub support_hash: Hash,
    pub root_relation_hashes: Vec<Hash>,
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
    let expected = support_from_target_only_relations(composed).ok_or_else(|| {
        certificate_gap(
            composed.target,
            composed.composed_hash,
            "global support requires a nonconstant target-only relation product",
        )
    })?;
    if normalized != expected {
        return Err(implementation_bug(
            composed.target,
            "global support polynomial does not match verified target-only relation product",
        ));
    }
    let mut cert = GlobalSupportCertificate {
        target: composed.target,
        composed_hash: composed.composed_hash,
        support_hash: normalized.hash,
        root_relation_hashes: composed
            .root_relations
            .iter()
            .map(|relation| relation.hash)
            .collect(),
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
    Ok(cert)
}

pub fn hash_global_support_certificate(cert: &GlobalSupportCertificate) -> Hash {
    let mut chunks = vec![
        cert.target.0.to_be_bytes().to_vec(),
        cert.composed_hash.0.to_vec(),
        cert.support_hash.0.to_vec(),
        cert.proof_hash.0.to_vec(),
    ];
    for hash in &cert.root_relation_hashes {
        chunks.push(hash.0.to_vec());
    }
    hash_sequence("global-support-certificate", &chunks)
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
