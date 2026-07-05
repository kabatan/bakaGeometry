use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::algebra::sign::SignDetermination;
use crate::fiber::hermite::{
    hermite_real_root_count_for_fiber, HermiteFiberCountMethod, HermiteFiberInput,
    RealFiberCountCertificate, RealFiberCountFactor,
};
use crate::fiber::slack_semantics::{
    apply_real_constraint_semantics, make_fiber_problem, verify_slack_encoding_consistency,
    FiberProblemWithSemantics,
};
use crate::fiber::thom::{
    hash_sign_classification_certificate, thom_sign_classify, SignClassificationCertificate,
    ThomSignInput,
};
use crate::preprocess::compression::{CompressedSystemQ, GuardKind};
use crate::problem::context::SolverContext;
use crate::problem::semantic::{RealConstraintEncoding, RealConstraintKind};
use crate::result::diagnostics::DiagnosticRecord;
use crate::result::status::{AlgebraicReason, FailureKind, SolverError, SolverErrorKind, StageId};
use crate::roots::decode::TargetCandidate;
use crate::roots::isolate::RealRootRecord;
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::{RelationId, VariableId};
use crate::types::interval::interval_disjoint;
use crate::types::monomial::normalize_monomial;
use crate::types::polynomial::{
    constant_poly, normalize_poly, poly_add, poly_variables, SparsePolynomialQ, TermQ,
};
use crate::types::rational::{
    add_q, int_q, is_zero_q, mul_q, neg_q, rational_to_bytes, zero_q, RationalQ,
};
use crate::types::univariate::{
    degree_uni, gcd_uni, normalize_univariate, squarefree_part_uni, UniPolynomialQ,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FiberCandidateDisposition {
    Realizable,
    RejectedByEquality,
    RejectedByGuard,
    RejectedBySemantic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticDecisionCertificate {
    pub semantic_hash: Hash,
    pub relation_id: Option<RelationId>,
    pub kind: RealConstraintKind,
    pub guard_source: SemanticGuardSource,
    pub guard_polynomial_hash: Hash,
    pub sign_certificate: SignClassificationCertificate,
    pub accepted: bool,
    pub certificate_hash: Hash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemanticGuardSource {
    DirectTargetPolynomial,
    SquareSlackEquation,
    NonZeroSlackProduct,
    CompressionGuard,
    Saturation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EqualityDecisionCertificate {
    pub relation_id: RelationId,
    pub relation_hash: Hash,
    pub sign_certificate: SignClassificationCertificate,
    pub satisfied: bool,
    pub certificate_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoordinateWitnessCertificate {
    pub relation_hashes: Vec<Hash>,
    pub assignment: Vec<(VariableId, RationalQ)>,
    pub certificate_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FiberClassificationRecord {
    pub target: VariableId,
    pub root: RealRootRecord,
    pub candidate: TargetCandidate,
    pub fiber_with_semantics: FiberProblemWithSemantics,
    pub equality_decisions: Vec<EqualityDecisionCertificate>,
    pub guard_decisions: Vec<SemanticDecisionCertificate>,
    pub semantic_decisions: Vec<SemanticDecisionCertificate>,
    pub coordinate_witness: Option<CoordinateWitnessCertificate>,
    pub hermite_certificate: Option<RealFiberCountCertificate>,
    pub disposition: FiberCandidateDisposition,
    pub rejection_reasons: Vec<String>,
    pub record_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FiberClassificationResult {
    pub target: VariableId,
    pub support_hash: Hash,
    pub records: Vec<FiberClassificationRecord>,
    pub exact_root_isolation: Vec<RealRootRecord>,
    pub exact_candidates: Vec<TargetCandidate>,
    pub rejected_candidates: Vec<TargetCandidate>,
    pub classification_hash: Hash,
}

pub fn classify_real_target_image(
    system: &CompressedSystemQ,
    support: &UniPolynomialQ,
    candidates: &[TargetCandidate],
    ctx: &mut SolverContext,
) -> Result<FiberClassificationResult, SolverError> {
    if support.variable != system.target {
        return Err(certificate_gap(
            system.target,
            support.hash,
            "ExactImageSupportTargetBinding",
        ));
    }

    let target_independent_witness = target_independent_witness(system)?;
    let mut records = Vec::new();
    let mut exact_root_isolation = Vec::new();
    let mut exact_candidates = Vec::new();
    let mut rejected_candidates = Vec::new();

    for candidate in candidates {
        if candidate.target != system.target || candidate.support_hash != support.hash {
            return Err(certificate_gap(
                system.target,
                candidate.candidate_hash,
                "ExactImageCandidateSupportBinding",
            ));
        }
        let root = RealRootRecord {
            support_hash: candidate.support_hash,
            root_index: candidate.root_index,
            isolating_interval: candidate.isolating_interval.clone(),
        };
        let record = classify_candidate(
            system,
            support,
            candidate,
            root,
            target_independent_witness.clone(),
        )?;
        if record.disposition == FiberCandidateDisposition::Realizable {
            exact_root_isolation.push(record.root.clone());
            exact_candidates.push(record.candidate.clone());
        } else {
            rejected_candidates.push(record.candidate.clone());
        }
        if !verify_slack_encoding_consistency(&record) {
            return Err(certificate_gap(
                system.target,
                record.record_hash,
                "SlackEncodingConsistency",
            ));
        }
        records.push(record);
    }

    let mut result = FiberClassificationResult {
        target: system.target,
        support_hash: support.hash,
        records,
        exact_root_isolation,
        exact_candidates,
        rejected_candidates,
        classification_hash: hash_sequence("fiber-classification-result", &[]),
    };
    result.classification_hash = hash_fiber_classification_result(&result);
    ctx.diagnostics.push(DiagnosticRecord::new(
        "ExactImageClassificationExecuted",
        format!(
            "classified {} candidates; exact={}; rejected={}; classification_hash={:?}",
            candidates.len(),
            result.exact_candidates.len(),
            result.rejected_candidates.len(),
            result.classification_hash
        ),
        Some(StageId("P13ExactImage".to_owned())),
    ));
    Ok(result)
}

pub fn hash_fiber_classification_result(result: &FiberClassificationResult) -> Hash {
    let mut chunks = vec![
        result.target.0.to_be_bytes().to_vec(),
        result.support_hash.0.to_vec(),
    ];
    chunks.extend(
        result
            .records
            .iter()
            .map(|record| record.record_hash.0.to_vec()),
    );
    chunks.extend(
        result
            .exact_candidates
            .iter()
            .map(|candidate| candidate.candidate_hash.0.to_vec()),
    );
    chunks.extend(
        result
            .rejected_candidates
            .iter()
            .map(|candidate| candidate.candidate_hash.0.to_vec()),
    );
    hash_sequence("fiber-classification-result", &chunks)
}

pub fn hash_fiber_classification_record(record: &FiberClassificationRecord) -> Hash {
    let mut chunks = vec![
        record.target.0.to_be_bytes().to_vec(),
        record.root.support_hash.0.to_vec(),
        record.root.root_index.to_be_bytes().to_vec(),
        rational_to_bytes(&record.root.isolating_interval.lo),
        rational_to_bytes(&record.root.isolating_interval.hi),
        record.candidate.candidate_hash.0.to_vec(),
        record.fiber_with_semantics.semantics_hash.0.to_vec(),
        format!("{:?}", record.disposition).into_bytes(),
    ];
    chunks.extend(
        record
            .equality_decisions
            .iter()
            .map(|cert| cert.certificate_hash.0.to_vec()),
    );
    chunks.extend(
        record
            .guard_decisions
            .iter()
            .map(|cert| cert.certificate_hash.0.to_vec()),
    );
    chunks.extend(
        record
            .semantic_decisions
            .iter()
            .map(|cert| cert.certificate_hash.0.to_vec()),
    );
    if let Some(witness) = &record.coordinate_witness {
        chunks.push(witness.certificate_hash.0.to_vec());
    } else {
        chunks.push(vec![0xff]);
    }
    if let Some(hermite) = &record.hermite_certificate {
        chunks.push(hermite.certificate_hash.0.to_vec());
    } else {
        chunks.push(vec![0xfe]);
    }
    chunks.extend(
        record
            .rejection_reasons
            .iter()
            .map(|reason| reason.as_bytes().to_vec()),
    );
    hash_sequence("fiber-classification-record", &chunks)
}

fn classify_candidate(
    system: &CompressedSystemQ,
    support: &UniPolynomialQ,
    candidate: &TargetCandidate,
    root: RealRootRecord,
    coordinate_witness: Option<CoordinateWitnessCertificate>,
) -> Result<FiberClassificationRecord, SolverError> {
    let relation_hashes = system
        .relations
        .iter()
        .map(|relation| relation.hash)
        .collect::<Vec<_>>();
    let guard_hashes = system
        .guards
        .iter()
        .map(|guard| guard.guard_hash)
        .collect::<Vec<_>>();
    let saturation_hashes = system
        .saturations
        .iter()
        .map(|saturation| saturation.saturation_hash)
        .collect::<Vec<_>>();
    let fiber = make_fiber_problem(
        system.target,
        support.hash,
        candidate.candidate_hash,
        candidate.root_index,
        relation_hashes.clone(),
        guard_hashes,
        saturation_hashes,
    );
    let fiber_with_semantics = apply_real_constraint_semantics(fiber, &system.semantic_encodings);

    let mut equality_decisions = Vec::new();
    let mut rejection_reasons = Vec::new();
    let mut mixed_relations = Vec::new();
    let semantic_relation_ids = system
        .semantic_encodings
        .iter()
        .flat_map(|encoding| encoding.encoded_relation_ids.iter().copied())
        .collect::<BTreeSet<_>>();
    for relation in &system.relations {
        if semantic_relation_ids.contains(&relation.id) {
            continue;
        }
        let vars = poly_variables(&relation.polynomial);
        if vars.is_subset(&BTreeSet::from([system.target])) {
            let Some(poly) = polynomial_to_univariate(&relation.polynomial, system.target) else {
                return Err(certificate_gap(
                    system.target,
                    relation.hash,
                    "TargetOnlyEqualityUnivariateCertificate",
                ));
            };
            let mut sign_cert = classify_sign(
                system.target,
                poly.clone(),
                &root,
                Some(relation.hash),
                None,
            )?;
            if sign_cert.sign == SignDetermination::RefinementRequired
                && same_zero_set_up_to_scalar(&squarefree_part_uni(&poly), support)
            {
                sign_cert.sign = SignDetermination::Zero;
                sign_cert.certificate_hash = hash_sign_classification_certificate(&sign_cert);
            }
            if sign_cert.sign == SignDetermination::RefinementRequired {
                return Err(certificate_gap(
                    system.target,
                    relation.hash,
                    "TargetOnlyEqualitySignCertificate",
                ));
            }
            let mut eq_cert = EqualityDecisionCertificate {
                relation_id: relation.id,
                relation_hash: relation.hash,
                satisfied: sign_cert.sign == SignDetermination::Zero,
                sign_certificate: sign_cert,
                certificate_hash: hash_sequence("fiber-equality-decision", &[]),
            };
            eq_cert.certificate_hash = hash_equality_decision_certificate(&eq_cert);
            if !eq_cert.satisfied {
                rejection_reasons.push(format!(
                    "relation {:?} not zero at target root",
                    relation.id
                ));
            }
            equality_decisions.push(eq_cert);
        } else if vars.contains(&system.target) {
            mixed_relations.push(relation.hash);
        }
    }
    if !mixed_relations.is_empty() {
        return Err(certificate_gap(
            system.target,
            hash_sequence(
                "mixed-fiber-relations",
                &mixed_relations
                    .iter()
                    .map(|hash| hash.0.to_vec())
                    .collect::<Vec<_>>(),
            ),
            "MixedCoordinateFiberRealRootCertificate",
        ));
    }
    if has_target_independent_relations(system) && coordinate_witness.is_none() {
        return Err(certificate_gap(
            system.target,
            system.compressed_hash,
            "TargetIndependentCoordinateWitness",
        ));
    }

    let mut guard_decisions = Vec::new();
    for guard in &system.guards {
        let Some(guard_poly) = target_guard_univariate(&guard.factor, system.target) else {
            return Err(certificate_gap(
                system.target,
                guard.guard_hash,
                "CompressionGuardSignCertificate",
            ));
        };
        let sign_cert = classify_sign(
            system.target,
            guard_poly,
            &root,
            Some(guard.guard_hash),
            None,
        )?;
        let accepted = match sign_cert.sign {
            SignDetermination::Negative | SignDetermination::Positive => true,
            SignDetermination::Zero => false,
            SignDetermination::RefinementRequired => {
                return Err(certificate_gap(
                    system.target,
                    guard.guard_hash,
                    "CompressionGuardSignCertificate",
                ));
            }
        };
        let mut decision = SemanticDecisionCertificate {
            semantic_hash: guard.guard_hash,
            relation_id: None,
            kind: match guard.guard_kind {
                GuardKind::ConstantNonZeroPivot
                | GuardKind::ExplicitNonZeroWitness
                | GuardKind::AffineDenominator => RealConstraintKind::NonZero,
            },
            guard_source: SemanticGuardSource::CompressionGuard,
            guard_polynomial_hash: sign_cert.polynomial_hash,
            sign_certificate: sign_cert,
            accepted,
            certificate_hash: hash_sequence("semantic-decision-certificate", &[]),
        };
        decision.certificate_hash = hash_semantic_decision_certificate(&decision);
        if !accepted {
            rejection_reasons.push("compression guard vanishes at target root".to_owned());
        }
        guard_decisions.push(decision);
    }

    let mut semantic_decisions = Vec::new();
    for encoding in &system.semantic_encodings {
        let mut decisions = classify_semantic_encoding(system, encoding, support, &root)?;
        for decision in &decisions {
            if !decision.accepted {
                rejection_reasons
                    .push(format!("{:?} semantic rejected target root", decision.kind));
            }
        }
        semantic_decisions.append(&mut decisions);
    }

    let disposition = if equality_decisions
        .iter()
        .any(|decision| !decision.satisfied)
    {
        FiberCandidateDisposition::RejectedByEquality
    } else if guard_decisions.iter().any(|decision| !decision.accepted) {
        FiberCandidateDisposition::RejectedByGuard
    } else if semantic_decisions.iter().any(|decision| !decision.accepted) {
        FiberCandidateDisposition::RejectedBySemantic
    } else {
        FiberCandidateDisposition::Realizable
    };

    let hermite_certificate = if disposition == FiberCandidateDisposition::Realizable {
        Some(hermite_real_root_count_for_fiber(HermiteFiberInput {
            target: system.target,
            support: support.clone(),
            root: root.clone(),
            candidate: candidate.clone(),
            equality_relation_hashes: relation_hashes,
            semantic_hashes: fiber_with_semantics.semantic_hashes.clone(),
            real_root_factors: hermite_count_factors(&semantic_decisions),
            method: if coordinate_witness.is_some() {
                HermiteFiberCountMethod::TargetOnlyRealWitness
            } else {
                HermiteFiberCountMethod::TargetAlgebraicCondition
            },
        })?)
    } else {
        None
    };
    if let Some(hermite) = &hermite_certificate {
        if hermite.real_root_count == 0 {
            return Err(certificate_gap(
                system.target,
                hermite.certificate_hash,
                "HermiteRealFiberNonemptyCertificate",
            ));
        }
    }

    let mut record = FiberClassificationRecord {
        target: system.target,
        root,
        candidate: candidate.clone(),
        fiber_with_semantics,
        equality_decisions,
        guard_decisions,
        semantic_decisions,
        coordinate_witness,
        hermite_certificate,
        disposition,
        rejection_reasons,
        record_hash: hash_sequence("fiber-classification-record", &[]),
    };
    record.record_hash = hash_fiber_classification_record(&record);
    Ok(record)
}

fn hermite_count_factors(
    semantic_decisions: &[SemanticDecisionCertificate],
) -> Vec<RealFiberCountFactor> {
    semantic_decisions
        .iter()
        .filter_map(|decision| {
            if !decision.accepted {
                return None;
            }
            let real_root_count = match decision.guard_source {
                SemanticGuardSource::SquareSlackEquation => match decision.sign_certificate.sign {
                    SignDetermination::Positive => 2,
                    SignDetermination::Zero => 1,
                    SignDetermination::Negative | SignDetermination::RefinementRequired => 0,
                },
                SemanticGuardSource::NonZeroSlackProduct => 1,
                SemanticGuardSource::DirectTargetPolynomial
                | SemanticGuardSource::CompressionGuard
                | SemanticGuardSource::Saturation => 1,
            };
            Some(RealFiberCountFactor {
                factor_hash: decision.certificate_hash,
                factor_kind: format!("{:?}", decision.guard_source),
                real_root_count,
            })
        })
        .collect()
}

fn classify_semantic_encoding(
    system: &CompressedSystemQ,
    encoding: &RealConstraintEncoding,
    support: &UniPolynomialQ,
    root: &RealRootRecord,
) -> Result<Vec<SemanticDecisionCertificate>, SolverError> {
    let mut decisions = Vec::new();
    for relation_id in &encoding.encoded_relation_ids {
        let Some(relation) = system
            .relations
            .iter()
            .find(|relation| relation.id == *relation_id)
        else {
            return Err(certificate_gap(
                system.target,
                encoding.semantic_hash,
                "SemanticRelationReferenceCertificate",
            ));
        };
        let Some((guard_poly, source)) =
            semantic_guard_polynomial(&relation.polynomial, encoding, system.target)?
        else {
            return Err(certificate_gap(
                system.target,
                relation.hash,
                "SlackGuardSemanticsCertificate",
            ));
        };
        let mut sign_cert = classify_sign(
            system.target,
            guard_poly.clone(),
            root,
            Some(relation.hash),
            Some(encoding.semantic_hash),
        )?;
        if sign_cert.sign == SignDetermination::RefinementRequired
            && zero_by_common_support_root(&guard_poly, support, root)?
        {
            sign_cert.sign = SignDetermination::Zero;
            sign_cert.certificate_hash = hash_sign_classification_certificate(&sign_cert);
        }
        if sign_cert.sign == SignDetermination::RefinementRequired {
            return Err(certificate_gap(
                system.target,
                relation.hash,
                "SemanticGuardSignCertificate",
            ));
        }
        let accepted = semantic_accepts(encoding.original_kind.clone(), sign_cert.sign)?;
        let mut decision = SemanticDecisionCertificate {
            semantic_hash: encoding.semantic_hash,
            relation_id: Some(*relation_id),
            kind: encoding.original_kind.clone(),
            guard_source: source,
            guard_polynomial_hash: sign_cert.polynomial_hash,
            sign_certificate: sign_cert,
            accepted,
            certificate_hash: hash_sequence("semantic-decision-certificate", &[]),
        };
        decision.certificate_hash = hash_semantic_decision_certificate(&decision);
        decisions.push(decision);
    }
    if decisions.is_empty() {
        return Err(certificate_gap(
            system.target,
            encoding.semantic_hash,
            "EmptySemanticEncodingCertificate",
        ));
    }
    Ok(decisions)
}

fn semantic_guard_polynomial(
    relation: &SparsePolynomialQ,
    encoding: &RealConstraintEncoding,
    target: VariableId,
) -> Result<Option<(UniPolynomialQ, SemanticGuardSource)>, SolverError> {
    if let Some(poly) = polynomial_to_univariate(relation, target) {
        return Ok(Some((poly, SemanticGuardSource::DirectTargetPolynomial)));
    }
    if encoding.slack_variables.len() == 1 {
        let slack = encoding.slack_variables[0];
        if matches!(
            encoding.original_kind,
            RealConstraintKind::NonNegative
                | RealConstraintKind::Positive
                | RealConstraintKind::BranchChoice
        ) {
            if let Some(poly) = square_slack_guard_polynomial(relation, target, slack) {
                return Ok(Some((poly, SemanticGuardSource::SquareSlackEquation)));
            }
        }
        if encoding.original_kind == RealConstraintKind::NonZero {
            if let Some(poly) = nonzero_slack_guard_polynomial(relation, target, slack) {
                return Ok(Some((poly, SemanticGuardSource::NonZeroSlackProduct)));
            }
        }
    }
    Ok(None)
}

fn semantic_accepts(
    kind: RealConstraintKind,
    sign: SignDetermination,
) -> Result<bool, SolverError> {
    Ok(match kind {
        RealConstraintKind::NonNegative => {
            matches!(sign, SignDetermination::Zero | SignDetermination::Positive)
        }
        RealConstraintKind::Positive | RealConstraintKind::BranchChoice => {
            sign == SignDetermination::Positive
        }
        RealConstraintKind::NonZero => {
            matches!(
                sign,
                SignDetermination::Negative | SignDetermination::Positive
            )
        }
        RealConstraintKind::Other => {
            return Err(SolverError {
                target: None,
                kind: SolverErrorKind::Failure(FailureKind::CertificateDesignGap {
                    constructed_object_hash: hash_sequence("other-real-constraint", &[]),
                    missing_certificate_kind: "OtherRealConstraintSemantics".to_owned(),
                }),
            });
        }
    })
}

fn classify_sign(
    target: VariableId,
    polynomial: UniPolynomialQ,
    root: &RealRootRecord,
    source_hash: Option<Hash>,
    semantic_hash: Option<Hash>,
) -> Result<SignClassificationCertificate, SolverError> {
    let cert = thom_sign_classify(ThomSignInput {
        target,
        polynomial,
        root: root.clone(),
        source_hash,
        semantic_hash,
    })?;
    if hash_sign_classification_certificate(&cert) != cert.certificate_hash {
        return Err(implementation_bug(target, "sign certificate hash mismatch"));
    }
    Ok(cert)
}

fn target_guard_univariate(poly: &SparsePolynomialQ, target: VariableId) -> Option<UniPolynomialQ> {
    if poly_variables(poly).is_subset(&BTreeSet::from([target])) {
        polynomial_to_univariate(poly, target)
    } else {
        None
    }
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

fn same_zero_set_up_to_scalar(a: &UniPolynomialQ, b: &UniPolynomialQ) -> bool {
    if a.variable != b.variable || a.coeffs_low_to_high.len() != b.coeffs_low_to_high.len() {
        return false;
    }
    let Some((a_pivot, b_pivot)) = a
        .coeffs_low_to_high
        .iter()
        .zip(&b.coeffs_low_to_high)
        .find(|(left, right)| !is_zero_q(left) || !is_zero_q(right))
    else {
        return false;
    };
    if is_zero_q(a_pivot) || is_zero_q(b_pivot) {
        return false;
    }
    a.coeffs_low_to_high
        .iter()
        .zip(&b.coeffs_low_to_high)
        .all(|(left, right)| mul_q(left, b_pivot) == mul_q(right, a_pivot))
}

fn zero_by_common_support_root(
    poly: &UniPolynomialQ,
    support: &UniPolynomialQ,
    root: &RealRootRecord,
) -> Result<bool, SolverError> {
    if poly.variable != support.variable {
        return Ok(false);
    }
    let gcd = gcd_uni(poly, support);
    if degree_uni(&gcd).is_none() {
        return Ok(false);
    }
    let roots = crate::algebra::real_root::isolate_real_roots_sturm(&gcd)?;
    Ok(roots
        .iter()
        .any(|gcd_root| !interval_disjoint(&gcd_root.isolating_interval, &root.isolating_interval)))
}

fn square_slack_guard_polynomial(
    relation: &SparsePolynomialQ,
    target: VariableId,
    slack: VariableId,
) -> Option<UniPolynomialQ> {
    let mut slack_coeff = None::<RationalQ>;
    let mut guard_terms = Vec::new();
    for term in &relation.terms {
        let is_square_slack = term.monomial.exponents == vec![(slack, 2)];
        if is_square_slack {
            if slack_coeff.is_some() {
                return None;
            }
            slack_coeff = Some(term.coeff.clone());
            continue;
        }
        if term.monomial.exponents.iter().any(|(var, _)| *var == slack) {
            return None;
        }
        guard_terms.push(term.clone());
    }
    let coeff = slack_coeff?;
    if coeff == int_q(-1) {
        polynomial_to_univariate(
            &normalize_poly(SparsePolynomialQ {
                terms: guard_terms,
                hash: hash_sequence("poly", &[]),
            }),
            target,
        )
    } else if coeff == int_q(1) {
        let guard = normalize_poly(SparsePolynomialQ {
            terms: guard_terms
                .into_iter()
                .map(|term| TermQ {
                    coeff: neg_q(&term.coeff),
                    monomial: term.monomial,
                })
                .collect(),
            hash: hash_sequence("poly", &[]),
        });
        polynomial_to_univariate(&guard, target)
    } else {
        None
    }
}

fn nonzero_slack_guard_polynomial(
    relation: &SparsePolynomialQ,
    target: VariableId,
    slack: VariableId,
) -> Option<UniPolynomialQ> {
    let mut slack_linear_terms = Vec::new();
    let mut other = constant_poly(zero_q());
    for term in &relation.terms {
        let slack_exp = term
            .monomial
            .exponents
            .iter()
            .find(|(var, _)| *var == slack)
            .map(|(_, exp)| *exp)
            .unwrap_or(0);
        match slack_exp {
            0 => {
                other = poly_add(
                    &other,
                    &normalize_poly(SparsePolynomialQ {
                        terms: vec![term.clone()],
                        hash: hash_sequence("poly", &[]),
                    }),
                );
            }
            1 => {
                let reduced = term
                    .monomial
                    .exponents
                    .iter()
                    .filter_map(|(var, exp)| {
                        if *var == slack {
                            None
                        } else {
                            Some((*var, *exp))
                        }
                    })
                    .collect::<Vec<_>>();
                slack_linear_terms.push(TermQ {
                    coeff: term.coeff.clone(),
                    monomial: normalize_monomial(reduced),
                });
            }
            _ => return None,
        }
    }
    if !poly_variables(&other).is_empty()
        || other.terms.len() != 1
        || other.terms[0].coeff != int_q(-1)
    {
        return None;
    }
    let guard = normalize_poly(SparsePolynomialQ {
        terms: slack_linear_terms,
        hash: hash_sequence("poly", &[]),
    });
    polynomial_to_univariate(&guard, target)
}

fn target_independent_witness(
    system: &CompressedSystemQ,
) -> Result<Option<CoordinateWitnessCertificate>, SolverError> {
    let target_independent = system
        .relations
        .iter()
        .filter(|relation| {
            let vars = poly_variables(&relation.polynomial);
            !vars.is_subset(&BTreeSet::from([system.target])) && !vars.contains(&system.target)
        })
        .collect::<Vec<_>>();
    if target_independent.is_empty() {
        return Ok(None);
    }
    let variables = target_independent
        .iter()
        .flat_map(|relation| poly_variables(&relation.polynomial))
        .filter(|var| *var != system.target)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let search_values = [int_q(-2), int_q(-1), int_q(0), int_q(1), int_q(2)];
    let mut assignment = BTreeMap::<VariableId, RationalQ>::new();
    if !search_witness_assignment(
        &target_independent
            .iter()
            .map(|relation| relation.polynomial.clone())
            .collect::<Vec<_>>(),
        &variables,
        &search_values,
        0,
        &mut assignment,
    ) {
        return Ok(None);
    }
    let assignment_vec = variables
        .into_iter()
        .map(|var| (var, assignment.get(&var).cloned().unwrap_or_else(zero_q)))
        .collect::<Vec<_>>();
    let mut cert = CoordinateWitnessCertificate {
        relation_hashes: target_independent
            .iter()
            .map(|relation| relation.hash)
            .collect(),
        assignment: assignment_vec,
        certificate_hash: hash_sequence("coordinate-witness-certificate", &[]),
    };
    cert.certificate_hash = hash_coordinate_witness_certificate(&cert);
    Ok(Some(cert))
}

fn has_target_independent_relations(system: &CompressedSystemQ) -> bool {
    system.relations.iter().any(|relation| {
        let vars = poly_variables(&relation.polynomial);
        !vars.is_subset(&BTreeSet::from([system.target])) && !vars.contains(&system.target)
    })
}

fn search_witness_assignment(
    relations: &[SparsePolynomialQ],
    variables: &[VariableId],
    search_values: &[RationalQ],
    index: usize,
    assignment: &mut BTreeMap<VariableId, RationalQ>,
) -> bool {
    if index == variables.len() {
        return relations
            .iter()
            .all(|relation| is_zero_q(&eval_poly_at_assignment(relation, assignment)));
    }
    let variable = variables[index];
    for value in search_values {
        assignment.insert(variable, value.clone());
        if search_witness_assignment(relations, variables, search_values, index + 1, assignment) {
            return true;
        }
    }
    assignment.remove(&variable);
    false
}

fn eval_poly_at_assignment(
    poly: &SparsePolynomialQ,
    assignment: &BTreeMap<VariableId, RationalQ>,
) -> RationalQ {
    let mut sum = zero_q();
    for term in &poly.terms {
        let mut product = term.coeff.clone();
        for (var, exp) in &term.monomial.exponents {
            let value = assignment.get(var).cloned().unwrap_or_else(zero_q);
            let mut power = int_q(1);
            for _ in 0..*exp {
                power = crate::types::rational::mul_q(&power, &value);
            }
            product = crate::types::rational::mul_q(&product, &power);
        }
        sum = add_q(&sum, &product);
    }
    sum
}

fn hash_equality_decision_certificate(cert: &EqualityDecisionCertificate) -> Hash {
    hash_sequence(
        "fiber-equality-decision",
        &[
            cert.relation_id.0.to_be_bytes().to_vec(),
            cert.relation_hash.0.to_vec(),
            cert.sign_certificate.certificate_hash.0.to_vec(),
            vec![cert.satisfied as u8],
        ],
    )
}

fn hash_semantic_decision_certificate(cert: &SemanticDecisionCertificate) -> Hash {
    hash_sequence(
        "semantic-decision-certificate",
        &[
            cert.semantic_hash.0.to_vec(),
            cert.relation_id
                .map(|id| id.0.to_be_bytes().to_vec())
                .unwrap_or_else(|| vec![0xff]),
            format!("{:?}", cert.kind).into_bytes(),
            format!("{:?}", cert.guard_source).into_bytes(),
            cert.guard_polynomial_hash.0.to_vec(),
            cert.sign_certificate.certificate_hash.0.to_vec(),
            vec![cert.accepted as u8],
        ],
    )
}

fn hash_coordinate_witness_certificate(cert: &CoordinateWitnessCertificate) -> Hash {
    let mut chunks = Vec::new();
    chunks.extend(cert.relation_hashes.iter().map(|hash| hash.0.to_vec()));
    for (var, value) in &cert.assignment {
        chunks.push(var.0.to_be_bytes().to_vec());
        chunks.push(rational_to_bytes(value));
    }
    hash_sequence("coordinate-witness-certificate", &chunks)
}

fn implementation_bug(target: VariableId, message: &str) -> SolverError {
    SolverError {
        target: Some(target),
        kind: SolverErrorKind::Failure(FailureKind::ImplementationBug {
            invariant_violated: message.to_owned(),
        }),
    }
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

#[allow(dead_code)]
fn algorithmic_hard_case(target: VariableId, object_hash: Hash, reason: &str) -> SolverError {
    SolverError {
        target: Some(target),
        kind: SolverErrorKind::Failure(FailureKind::AlgorithmicHardCase {
            stage: StageId("P13ExactImage".to_owned()),
            reason: AlgebraicReason(reason.to_owned()),
            minimal_block_hash: object_hash,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::problem::semantic::register_slack_encoding;
    use crate::types::polynomial::{poly_mul, poly_sub, variable_poly};

    #[test]
    fn square_slack_semantics_extracts_guard_polynomial() {
        let target = VariableId(0);
        let slack = VariableId(1);
        let relation = poly_sub(
            &variable_poly(target),
            &poly_mul(&variable_poly(slack), &variable_poly(slack)),
        );
        let encoding = register_slack_encoding(
            RealConstraintKind::NonNegative,
            vec![RelationId(1)],
            vec![slack],
        );
        let (guard, source) = semantic_guard_polynomial(&relation, &encoding, target)
            .unwrap()
            .unwrap();
        assert_eq!(source, SemanticGuardSource::SquareSlackEquation);
        assert_eq!(guard.coeffs_low_to_high, vec![int_q(0), int_q(1)]);
    }
}
