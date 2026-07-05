use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::compose::compose::{hash_composed_projection, ComposedProjection};
use crate::compose::message::ProjectionMessage;
use crate::result::cost_trace::GlobalCostTrace;
use crate::result::diagnostics::DiagnosticRecord;
use crate::result::output::TargetSolveResult;
use crate::result::status::{
    AlgebraicReason, FailureKind, SolverError, SolverErrorKind, SolverStatus, StageId,
};
use crate::roots::decode::{decode_candidates, TargetCandidate};
use crate::roots::isolate::{isolate_real_roots, RealRootRecord, RootIsolationOptions};
use crate::roots::squarefree::squarefree_support;
use crate::solver::options::RootIsolationMethod;
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::VariableId;
use crate::types::polynomial::{poly_variables, SparsePolynomialQ};
use crate::types::rational::{
    add_q, int_q, is_zero_q, mul_q, rational_to_bytes, zero_q, RationalQ,
};
use crate::types::univariate::{
    degree_uni, normalize_univariate, squarefree_part_uni, UniPolynomialQ,
};
use crate::verify::run_certificate::CoreRunCertificate;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FinalSupportComputation {
    Support(UniPolynomialQ),
    CertifiedNonFinite(NonFiniteCertificate),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetEliminationZeroCertificate {
    pub target: VariableId,
    pub composed_hash: Hash,
    pub root_relation_hashes: Vec<Hash>,
    pub rational_consistency_witness: Vec<(VariableId, RationalQ)>,
    pub elimination_basis_hash: Hash,
    pub certificate_hash: Hash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NonFiniteProofKind {
    ZeroTargetEliminationOverQ,
    ZeroTargetEliminationWithRealWitness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NonFiniteCertificate {
    pub target: VariableId,
    pub composed_hash: Hash,
    pub proof_kind: NonFiniteProofKind,
    pub zero_target_elimination: TargetEliminationZeroCertificate,
    pub real_certificate: Option<RealNonFiniteTargetCertificate>,
    pub certificate_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RealNonFiniteInput {
    pub composed: ComposedProjection,
    pub zero_certificate: TargetEliminationZeroCertificate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RealNonFiniteTargetCertificate {
    pub target: VariableId,
    pub composed_hash: Hash,
    pub rational_real_witness: Vec<(VariableId, RationalQ)>,
    pub certificate_hash: Hash,
}

pub fn build_global_support_polynomial(
    composed: ComposedProjection,
    target: VariableId,
    _ctx: &mut crate::problem::context::SolverContext,
) -> Result<UniPolynomialQ, SolverError> {
    support_from_target_only_relations(&composed, target).ok_or_else(|| {
        algorithmic_hard_case(
            target,
            composed.composed_hash,
            "no target-only support after composition",
        )
    })
}

pub fn build_final_support_or_nonfinite(
    composed: ComposedProjection,
    target: VariableId,
    ctx: &mut crate::problem::context::SolverContext,
) -> Result<FinalSupportComputation, SolverError> {
    if let Some(support) = support_from_target_only_relations(&composed, target) {
        return Ok(FinalSupportComputation::Support(support));
    }
    match certify_nonfinite_target_image(&composed, ctx) {
        Ok(cert) => Ok(FinalSupportComputation::CertifiedNonFinite(cert)),
        Err(_) => Err(algorithmic_hard_case(
            target,
            composed.composed_hash,
            "no target-only relation and non-finiteness not certified",
        )),
    }
}

pub fn certify_nonfinite_target_image(
    composed: &ComposedProjection,
    ctx: &mut crate::problem::context::SolverContext,
) -> Result<NonFiniteCertificate, SolverError> {
    let zero_target_elimination = certify_zero_target_elimination_ideal(composed, ctx)?;
    let real_certificate = if ctx.options.exact_image_mode {
        Some(certify_real_nonfinite_target_image(
            RealNonFiniteInput {
                composed: composed.clone(),
                zero_certificate: zero_target_elimination.clone(),
            },
            ctx,
        )?)
    } else {
        None
    };
    let mut cert = NonFiniteCertificate {
        target: composed.target,
        composed_hash: composed.composed_hash,
        proof_kind: if real_certificate.is_some() {
            NonFiniteProofKind::ZeroTargetEliminationWithRealWitness
        } else {
            NonFiniteProofKind::ZeroTargetEliminationOverQ
        },
        zero_target_elimination,
        real_certificate,
        certificate_hash: hash_sequence("nonfinite-target-certificate", &[]),
    };
    cert.certificate_hash = hash_nonfinite_certificate(&cert);
    verify_nonfinite_certificate(&cert, composed)?;
    Ok(cert)
}

pub fn certify_zero_target_elimination_ideal(
    composed: &ComposedProjection,
    _ctx: &mut crate::problem::context::SolverContext,
) -> Result<TargetEliminationZeroCertificate, SolverError> {
    validate_composed_hash(composed)?;
    if composed
        .root_relations
        .iter()
        .any(|relation| poly_variables(relation).contains(&composed.target))
    {
        return Err(algorithmic_hard_case(
            composed.target,
            composed.composed_hash,
            "target occurs in composed root ideal generators",
        ));
    }
    let Some(witness) =
        find_rational_consistency_witness(&composed.root_relations, composed.target)
    else {
        return Err(certificate_gap(
            composed.target,
            composed.composed_hash,
            "TargetEliminationZeroCertificate requires an exact consistency witness",
        ));
    };
    let mut cert = TargetEliminationZeroCertificate {
        target: composed.target,
        composed_hash: composed.composed_hash,
        root_relation_hashes: composed
            .root_relations
            .iter()
            .map(|relation| relation.hash)
            .collect(),
        rational_consistency_witness: witness,
        elimination_basis_hash: hash_sequence("target-elimination-zero-basis", &[]),
        certificate_hash: hash_sequence("target-elimination-zero-certificate", &[]),
    };
    cert.elimination_basis_hash = hash_sequence(
        "target-elimination-zero-basis",
        &cert
            .root_relation_hashes
            .iter()
            .map(|hash| hash.0.to_vec())
            .collect::<Vec<_>>(),
    );
    cert.certificate_hash = hash_zero_certificate(&cert);
    verify_zero_target_elimination_certificate(&cert, composed)?;
    Ok(cert)
}

pub fn verify_nonfinite_certificate(
    cert: &NonFiniteCertificate,
    composed: &ComposedProjection,
) -> Result<(), SolverError> {
    validate_composed_hash(composed)?;
    if cert.target != composed.target
        || cert.composed_hash != composed.composed_hash
        || hash_nonfinite_certificate(cert) != cert.certificate_hash
    {
        return Err(implementation_bug(
            composed.target,
            "nonfinite certificate hash or target binding mismatch",
        ));
    }
    match (cert.proof_kind, cert.real_certificate.is_some()) {
        (NonFiniteProofKind::ZeroTargetEliminationOverQ, false)
        | (NonFiniteProofKind::ZeroTargetEliminationWithRealWitness, true) => {}
        _ => {
            return Err(implementation_bug(
                composed.target,
                "nonfinite proof kind does not match supplied proof evidence",
            ));
        }
    }
    verify_zero_target_elimination_certificate(&cert.zero_target_elimination, composed)?;
    if let Some(real_cert) = &cert.real_certificate {
        verify_real_nonfinite_certificate(real_cert, composed)?;
    }
    Ok(())
}

pub fn certify_real_nonfinite_target_image(
    input: RealNonFiniteInput,
    _ctx: &mut crate::problem::context::SolverContext,
) -> Result<RealNonFiniteTargetCertificate, SolverError> {
    verify_zero_target_elimination_certificate(&input.zero_certificate, &input.composed)?;
    let mut cert = RealNonFiniteTargetCertificate {
        target: input.composed.target,
        composed_hash: input.composed.composed_hash,
        rational_real_witness: input.zero_certificate.rational_consistency_witness,
        certificate_hash: hash_sequence("real-nonfinite-target-certificate", &[]),
    };
    cert.certificate_hash = hash_real_nonfinite_certificate(&cert);
    verify_real_nonfinite_certificate(&cert, &input.composed)?;
    Ok(cert)
}

pub fn finalize_nonfinite_result(
    target: VariableId,
    cert: NonFiniteCertificate,
    composed: &ComposedProjection,
    projection_messages: Vec<ProjectionMessage>,
    cost_trace: GlobalCostTrace,
) -> Result<TargetSolveResult, SolverError> {
    verify_nonfinite_certificate(&cert, composed)?;
    Ok(TargetSolveResult {
        status: SolverStatus::CertifiedNonFiniteTargetImage,
        target,
        support_polynomial: None,
        squarefree_support_polynomial: None,
        root_isolation: Vec::<RealRootRecord>::new(),
        decoded_candidates: Vec::<TargetCandidate>::new(),
        projection_messages,
        certificate: None::<CoreRunCertificate>,
        diagnostics: vec![DiagnosticRecord::new(
            "CertifiedNonFiniteTargetImage",
            format!("nonfinite certificate hash {:?}", cert.certificate_hash),
            Some(StageId("P10FinalSupport".to_owned())),
        )],
        cost_trace,
    })
}

pub fn finalize_candidate_cover_result(
    target: VariableId,
    support: UniPolynomialQ,
    projection_messages: Vec<ProjectionMessage>,
    cost_trace: GlobalCostTrace,
    root_isolation_method: RootIsolationMethod,
) -> Result<TargetSolveResult, SolverError> {
    let squarefree_support = squarefree_support(&support)?;
    let root_isolation = isolate_real_roots(
        &squarefree_support,
        RootIsolationOptions {
            method: root_isolation_method,
        },
    )?;
    let decoded_candidates = decode_candidates(target, &squarefree_support, &root_isolation);
    if decoded_candidates.len() != root_isolation.len() {
        return Err(implementation_bug(
            target,
            "decoded candidate count does not match isolated root count",
        ));
    }
    let diagnostics = if root_isolation.is_empty() {
        vec![DiagnosticRecord::new(
            "EmptyRealCandidateCover",
            "support has no real roots; certified candidate cover is empty".to_owned(),
            Some(StageId("P12RootDecode".to_owned())),
        )]
    } else {
        Vec::new()
    };
    Ok(TargetSolveResult {
        status: SolverStatus::CertifiedCandidateCover,
        target,
        support_polynomial: Some(support),
        squarefree_support_polynomial: Some(squarefree_support),
        root_isolation,
        decoded_candidates,
        projection_messages,
        certificate: None::<CoreRunCertificate>,
        diagnostics,
        cost_trace,
    })
}

fn support_from_target_only_relations(
    composed: &ComposedProjection,
    target: VariableId,
) -> Option<UniPolynomialQ> {
    if composed.target != target || hash_composed_projection(composed) != composed.composed_hash {
        return None;
    }
    let target_set = BTreeSet::from([target]);
    let mut support = normalize_univariate(UniPolynomialQ {
        variable: target,
        coeffs_low_to_high: vec![int_q(1)],
        hash: hash_sequence("univariate", &[]),
    });
    let mut found = false;
    for relation in &composed.root_relations {
        if relation.terms.is_empty() || !poly_variables(relation).is_subset(&target_set) {
            continue;
        }
        let uni = polynomial_to_univariate(relation, target)?;
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
    if a.variable != b.variable {
        return normalize_univariate(UniPolynomialQ {
            variable: a.variable,
            coeffs_low_to_high: Vec::new(),
            hash: hash_sequence("univariate", &[]),
        });
    }
    if a.coeffs_low_to_high.is_empty() || b.coeffs_low_to_high.is_empty() {
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

fn verify_zero_target_elimination_certificate(
    cert: &TargetEliminationZeroCertificate,
    composed: &ComposedProjection,
) -> Result<(), SolverError> {
    if cert.target != composed.target
        || cert.composed_hash != composed.composed_hash
        || cert.root_relation_hashes
            != composed
                .root_relations
                .iter()
                .map(|relation| relation.hash)
                .collect::<Vec<_>>()
        || hash_zero_certificate(cert) != cert.certificate_hash
    {
        return Err(implementation_bug(
            composed.target,
            "zero target elimination certificate binding mismatch",
        ));
    }
    if composed
        .root_relations
        .iter()
        .any(|relation| poly_variables(relation).contains(&composed.target))
    {
        return Err(implementation_bug(
            composed.target,
            "zero target elimination certificate has target-bearing generator",
        ));
    }
    let assignment = cert
        .rational_consistency_witness
        .iter()
        .cloned()
        .collect::<BTreeMap<_, _>>();
    if composed
        .root_relations
        .iter()
        .any(|relation| !is_zero_q(&eval_poly_at_assignment(relation, &assignment)))
    {
        return Err(implementation_bug(
            composed.target,
            "zero target elimination consistency witness does not satisfy all relations",
        ));
    }
    Ok(())
}

fn verify_real_nonfinite_certificate(
    cert: &RealNonFiniteTargetCertificate,
    composed: &ComposedProjection,
) -> Result<(), SolverError> {
    if cert.target != composed.target
        || cert.composed_hash != composed.composed_hash
        || hash_real_nonfinite_certificate(cert) != cert.certificate_hash
    {
        return Err(implementation_bug(
            composed.target,
            "real nonfinite certificate binding mismatch",
        ));
    }
    let assignment = cert
        .rational_real_witness
        .iter()
        .cloned()
        .collect::<BTreeMap<_, _>>();
    if composed
        .root_relations
        .iter()
        .any(|relation| !is_zero_q(&eval_poly_at_assignment(relation, &assignment)))
    {
        return Err(implementation_bug(
            composed.target,
            "real nonfinite witness does not satisfy all relations",
        ));
    }
    Ok(())
}

fn find_rational_consistency_witness(
    relations: &[SparsePolynomialQ],
    target: VariableId,
) -> Option<Vec<(VariableId, RationalQ)>> {
    let variables = relations
        .iter()
        .flat_map(poly_variables)
        .filter(|var| *var != target)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let candidates = [-2, -1, 0, 1, 2].into_iter().map(int_q).collect::<Vec<_>>();
    let mut assignment = BTreeMap::new();
    search_witness(0, &variables, &candidates, relations, &mut assignment)
        .then(|| assignment.into_iter().collect())
}

fn search_witness(
    idx: usize,
    variables: &[VariableId],
    candidates: &[RationalQ],
    relations: &[SparsePolynomialQ],
    assignment: &mut BTreeMap<VariableId, RationalQ>,
) -> bool {
    if idx == variables.len() {
        return relations
            .iter()
            .all(|relation| is_zero_q(&eval_poly_at_assignment(relation, assignment)));
    }
    let variable = variables[idx];
    for value in candidates {
        assignment.insert(variable, value.clone());
        if search_witness(idx + 1, variables, candidates, relations, assignment) {
            return true;
        }
    }
    assignment.remove(&variable);
    false
}

fn eval_poly_at_assignment(
    relation: &SparsePolynomialQ,
    assignment: &BTreeMap<VariableId, RationalQ>,
) -> RationalQ {
    let mut acc = zero_q();
    for term in &relation.terms {
        let mut value = term.coeff.clone();
        for (var, exp) in &term.monomial.exponents {
            let Some(base) = assignment.get(var) else {
                return int_q(1);
            };
            value = mul_q(&value, &pow_q(base, *exp));
        }
        acc = add_q(&acc, &value);
    }
    acc
}

fn pow_q(base: &RationalQ, exp: u32) -> RationalQ {
    let mut out = int_q(1);
    for _ in 0..exp {
        out = mul_q(&out, base);
    }
    out
}

fn hash_zero_certificate(cert: &TargetEliminationZeroCertificate) -> Hash {
    let mut chunks = vec![
        cert.target.0.to_be_bytes().to_vec(),
        cert.composed_hash.0.to_vec(),
        cert.elimination_basis_hash.0.to_vec(),
    ];
    for hash in &cert.root_relation_hashes {
        chunks.push(hash.0.to_vec());
    }
    chunks.push(Vec::new());
    for (var, value) in &cert.rational_consistency_witness {
        chunks.push(var.0.to_be_bytes().to_vec());
        chunks.push(rational_to_bytes(value));
    }
    hash_sequence("target-elimination-zero-certificate", &chunks)
}

fn hash_nonfinite_certificate(cert: &NonFiniteCertificate) -> Hash {
    let mut chunks = vec![
        cert.target.0.to_be_bytes().to_vec(),
        cert.composed_hash.0.to_vec(),
        format!("{:?}", cert.proof_kind).into_bytes(),
        cert.zero_target_elimination.certificate_hash.0.to_vec(),
    ];
    if let Some(real) = &cert.real_certificate {
        chunks.push(real.certificate_hash.0.to_vec());
    }
    hash_sequence("nonfinite-target-certificate", &chunks)
}

fn hash_real_nonfinite_certificate(cert: &RealNonFiniteTargetCertificate) -> Hash {
    let mut chunks = vec![
        cert.target.0.to_be_bytes().to_vec(),
        cert.composed_hash.0.to_vec(),
    ];
    for (var, value) in &cert.rational_real_witness {
        chunks.push(var.0.to_be_bytes().to_vec());
        chunks.push(rational_to_bytes(value));
    }
    hash_sequence("real-nonfinite-target-certificate", &chunks)
}

fn validate_composed_hash(composed: &ComposedProjection) -> Result<(), SolverError> {
    if hash_composed_projection(composed) != composed.composed_hash {
        return Err(implementation_bug(
            composed.target,
            "composed projection hash mismatch",
        ));
    }
    Ok(())
}

fn algorithmic_hard_case(target: VariableId, hash: Hash, reason: &str) -> SolverError {
    SolverError {
        target: Some(target),
        kind: SolverErrorKind::Failure(FailureKind::AlgorithmicHardCase {
            stage: StageId("P10FinalSupport".to_owned()),
            reason: AlgebraicReason(reason.to_owned()),
            minimal_block_hash: hash,
        }),
    }
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
    use crate::compose::compose::{hash_composed_projection, ComposedProjection};
    use crate::compose::message::ProjectionMessage;
    use crate::problem::context::new_context;
    use crate::result::cost_trace::{CompositionCostTrace, GlobalCostTrace};
    use crate::result::status::SolverStatus;
    use crate::solver::options::{RootIsolationMethod, SolverOptions};
    use crate::types::hash::hash_sequence;
    use crate::types::ids::BlockId;
    use crate::types::ids::VariableId;
    use crate::types::rational::int_q;
    use crate::types::univariate::{normalize_univariate, UniPolynomialQ};

    use super::{
        certify_nonfinite_target_image, finalize_candidate_cover_result,
        hash_nonfinite_certificate, verify_nonfinite_certificate, NonFiniteProofKind,
    };

    #[test]
    fn p12g_candidate_cover_with_no_real_roots_is_empty_cover_not_hard_case() {
        let t = VariableId(0);
        let support = normalize_univariate(UniPolynomialQ {
            variable: t,
            coeffs_low_to_high: vec![int_q(1), int_q(0), int_q(1)],
            hash: hash_sequence("univariate", &[]),
        });

        let result = finalize_candidate_cover_result(
            t,
            support.clone(),
            Vec::<ProjectionMessage>::new(),
            GlobalCostTrace::default(),
            RootIsolationMethod::Sturm,
        )
        .unwrap();

        assert_eq!(result.status, SolverStatus::CertifiedCandidateCover);
        assert_eq!(result.support_polynomial, Some(support.clone()));
        assert!(result.squarefree_support_polynomial.is_some());
        assert!(result.root_isolation.is_empty());
        assert!(result.decoded_candidates.is_empty());
        assert!(result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.name == "EmptyRealCandidateCover"));
    }

    #[test]
    fn p12g_nonfinite_certificate_kind_is_positive_and_hash_bound() {
        let t = VariableId(0);
        let mut composed = ComposedProjection {
            target: t,
            root_block_id: BlockId(0),
            message_relations: Vec::new(),
            root_relations: Vec::new(),
            source_message_hashes: Vec::new(),
            separator_elimination_hashes: Vec::new(),
            composition_cost: CompositionCostTrace::default(),
            composed_hash: hash_sequence("composed-projection", &[]),
        };
        composed.composed_hash = hash_composed_projection(&composed);
        let mut ctx = new_context(SolverOptions::default());
        let mut cert = certify_nonfinite_target_image(&composed, &mut ctx).unwrap();

        assert_eq!(
            cert.proof_kind,
            NonFiniteProofKind::ZeroTargetEliminationOverQ
        );
        verify_nonfinite_certificate(&cert, &composed).unwrap();
        cert.proof_kind = NonFiniteProofKind::ZeroTargetEliminationWithRealWitness;
        cert.certificate_hash = hash_nonfinite_certificate(&cert);
        let err = verify_nonfinite_certificate(&cert, &composed).unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::ImplementationBug);
    }
}
