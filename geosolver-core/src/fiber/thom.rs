use serde::{Deserialize, Serialize};

use crate::algebra::sign::{
    sign_at_algebraic_root, thom_encoding, SignDetermination, ThomEncoding,
};
use crate::result::status::{FailureKind, SolverError, SolverErrorKind};
use crate::roots::isolate::RealRootRecord;
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::VariableId;
use crate::types::univariate::UniPolynomialQ;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThomSignInput {
    pub target: VariableId,
    pub polynomial: UniPolynomialQ,
    pub root: RealRootRecord,
    pub source_hash: Option<Hash>,
    pub semantic_hash: Option<Hash>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignClassificationCertificate {
    pub target: VariableId,
    pub polynomial_hash: Hash,
    pub support_hash: Hash,
    pub root_index: usize,
    pub sign: SignDetermination,
    pub thom_encoding: ThomEncoding,
    pub source_hash: Option<Hash>,
    pub semantic_hash: Option<Hash>,
    pub certificate_hash: Hash,
}

pub fn thom_sign_classify(
    input: ThomSignInput,
) -> Result<SignClassificationCertificate, SolverError> {
    if input.polynomial.variable != input.target || input.root.support_hash.0 == [0; 32] {
        return Err(certificate_gap(
            input.target,
            input.polynomial.hash,
            "ThomSignInput target/root binding",
        ));
    }
    let sign = sign_at_algebraic_root(&input.polynomial, &input.root);
    let encoding = thom_encoding(&input.polynomial, &input.root);
    let mut cert = SignClassificationCertificate {
        target: input.target,
        polynomial_hash: input.polynomial.hash,
        support_hash: input.root.support_hash,
        root_index: input.root.root_index,
        sign,
        thom_encoding: encoding,
        source_hash: input.source_hash,
        semantic_hash: input.semantic_hash,
        certificate_hash: hash_sequence("thom-sign-certificate", &[]),
    };
    cert.certificate_hash = hash_sign_classification_certificate(&cert);
    Ok(cert)
}

pub fn hash_sign_classification_certificate(cert: &SignClassificationCertificate) -> Hash {
    let mut chunks = vec![
        cert.target.0.to_be_bytes().to_vec(),
        cert.polynomial_hash.0.to_vec(),
        cert.support_hash.0.to_vec(),
        cert.root_index.to_be_bytes().to_vec(),
        format!("{:?}", cert.sign).into_bytes(),
        cert.thom_encoding.polynomial_hash.0.to_vec(),
        cert.thom_encoding.root_index.to_be_bytes().to_vec(),
        vec![cert.thom_encoding.all_signs_determined as u8],
        optional_hash_bytes(cert.source_hash),
        optional_hash_bytes(cert.semantic_hash),
    ];
    chunks.extend(
        cert.thom_encoding
            .signs_by_derivative_order
            .iter()
            .map(|sign| format!("{sign:?}").into_bytes()),
    );
    hash_sequence("thom-sign-certificate", &chunks)
}

fn optional_hash_bytes(hash: Option<Hash>) -> Vec<u8> {
    hash.map(|hash| hash.0.to_vec())
        .unwrap_or_else(|| vec![0xff])
}

fn certificate_gap(target: VariableId, object_hash: Hash, missing: &str) -> SolverError {
    SolverError {
        target: Some(target),
        kind: SolverErrorKind::Failure(FailureKind::CertificateDesignGap {
            constructed_object_hash: object_hash,
            missing_certificate_kind: missing.to_owned(),
        }),
    }
}
