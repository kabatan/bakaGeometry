use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::compose::message::{hash_projection_message, ProjectionMessage};
use crate::result::status::{FailureKind, SolverError, SolverErrorKind};
use crate::roots::decode::TargetCandidate;
use crate::roots::isolate::RealRootRecord;
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::{BlockId, VariableId};
use crate::types::rational::rational_to_bytes;
use crate::types::univariate::UniPolynomialQ;
use crate::verify::certificates::KernelCertificatePayload;
use crate::verify::verify_message::payload_source_hashes;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreRunCertificate {
    pub input_hash: Hash,
    pub canonical_system_hash: Hash,
    pub target_variable: VariableId,
    pub compression_hash: Hash,
    pub hypergraph_hash: Hash,
    pub target_projection_dag_hash: Hash,
    pub kernel_plan_hashes: Vec<Hash>,
    pub projection_message_hashes: Vec<Hash>,
    pub global_support_hash: Option<Hash>,
    pub squarefree_support_hash: Option<Hash>,
    pub root_isolation_hash: Option<Hash>,
    pub decoded_candidate_hash: Option<Hash>,
    pub global_support_certificate_hash: Option<Hash>,
    pub final_dag_replay_evidence_hash: Option<Hash>,
    pub invariants: CoreInvariantFlags,
    pub invariant_evidence_hash: Hash,
    pub run_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreInvariantFlags {
    pub no_geometry_dispatch: bool,
    pub no_problem_id_dispatch: bool,
    pub no_expected_answer_dispatch: bool,
    pub no_full_coordinate_solution_set: bool,
    pub no_full_coordinate_rur: bool,
    pub no_qe_cad: bool,
    pub exact_q_verification: bool,
    pub no_hidden_fallback: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FinalInvariantEvidence {
    pub no_geometry_dispatch_scan_hash: Option<Hash>,
    pub no_problem_id_dispatch_scan_hash: Option<Hash>,
    pub no_expected_answer_dispatch_scan_hash: Option<Hash>,
    pub no_qe_cad_scan_hash: Option<Hash>,
    pub exact_q_verification_hash: Option<Hash>,
    pub evidence_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FinalDagReplayEvidence {
    pub actual_projection_dag_hash: Option<Hash>,
    pub projection_message_hashes: Vec<Hash>,
    pub kernel_plan_hashes: Vec<Hash>,
    pub message_block_ids: Vec<BlockId>,
    pub per_message_source_relation_hashes: Vec<Vec<Hash>>,
    pub message_child_dependency_hashes: Vec<Vec<Hash>>,
    pub block_authorization_hashes: Vec<Hash>,
    pub edge_authorization_hashes: Vec<Hash>,
    pub actual_dag_replay_verified: bool,
    pub evidence_hash: Hash,
}

impl CoreInvariantFlags {
    pub fn p11_replay_enforced(&self) -> bool {
        self.no_full_coordinate_solution_set
            && self.no_full_coordinate_rur
            && self.exact_q_verification
            && self.no_hidden_fallback
    }
}

pub struct CoreRunCertificateInput<'a> {
    pub input_hash: Hash,
    pub canonical_hash: Hash,
    pub compression_hash: Hash,
    pub hypergraph_hash: Hash,
    pub dag_hash: Hash,
    pub kernel_plan_hashes: Vec<Hash>,
    pub projection_messages: &'a [ProjectionMessage],
    pub support: Option<&'a UniPolynomialQ>,
    pub squarefree_support: Option<&'a UniPolynomialQ>,
    pub root_isolation: &'a [RealRootRecord],
    pub decoded_candidates: &'a [TargetCandidate],
    pub global_support_certificate_hash: Option<Hash>,
}

pub fn build_core_run_certificate(input: CoreRunCertificateInput<'_>) -> CoreRunCertificate {
    let mut cert = CoreRunCertificate {
        input_hash: input.input_hash,
        canonical_system_hash: input.canonical_hash,
        target_variable: input
            .support
            .or(input.squarefree_support)
            .map_or(VariableId(0), |support| support.variable),
        compression_hash: input.compression_hash,
        hypergraph_hash: input.hypergraph_hash,
        target_projection_dag_hash: input.dag_hash,
        kernel_plan_hashes: input.kernel_plan_hashes,
        projection_message_hashes: hash_projection_messages(input.projection_messages),
        global_support_hash: input.support.map(|support| support.hash),
        squarefree_support_hash: input.squarefree_support.map(|support| support.hash),
        root_isolation_hash: Some(hash_root_isolation(input.root_isolation)),
        decoded_candidate_hash: Some(hash_decoded_candidates(input.decoded_candidates)),
        global_support_certificate_hash: input.global_support_certificate_hash,
        final_dag_replay_evidence_hash: None,
        invariants: derive_core_invariant_flags(
            input.projection_messages,
            messages_have_verifiable_payloads(input.projection_messages),
            projection_plans_are_bound(input.projection_messages),
        ),
        invariant_evidence_hash: hash_invariant_evidence(
            input.input_hash,
            input.projection_messages,
            input.support,
            input.squarefree_support,
            input.root_isolation,
            input.decoded_candidates,
        ),
        run_hash: hash_sequence("core-run-certificate", &[]),
    };
    cert.run_hash = hash_core_run_certificate(&cert);
    cert
}

pub fn final_invariant_evidence(
    no_geometry_dispatch_scan_hash: Option<Hash>,
    no_problem_id_dispatch_scan_hash: Option<Hash>,
    no_expected_answer_dispatch_scan_hash: Option<Hash>,
    no_qe_cad_scan_hash: Option<Hash>,
    exact_q_verification_hash: Option<Hash>,
) -> FinalInvariantEvidence {
    let mut evidence = FinalInvariantEvidence {
        no_geometry_dispatch_scan_hash,
        no_problem_id_dispatch_scan_hash,
        no_expected_answer_dispatch_scan_hash,
        no_qe_cad_scan_hash,
        exact_q_verification_hash,
        evidence_hash: hash_sequence("final-invariant-evidence", &[]),
    };
    evidence.evidence_hash = hash_final_invariant_evidence(&evidence);
    evidence
}

pub fn require_final_claim_invariant_evidence(
    flags: &CoreInvariantFlags,
    evidence: &FinalInvariantEvidence,
) -> Result<(), SolverError> {
    if evidence.evidence_hash != hash_final_invariant_evidence(evidence)
        || !flags.no_geometry_dispatch
        || !flags.no_problem_id_dispatch
        || !flags.no_expected_answer_dispatch
        || !flags.no_qe_cad
        || !flags.exact_q_verification
        || evidence.no_geometry_dispatch_scan_hash.is_none()
        || evidence.no_problem_id_dispatch_scan_hash.is_none()
        || evidence.no_expected_answer_dispatch_scan_hash.is_none()
        || evidence.no_qe_cad_scan_hash.is_none()
        || evidence.exact_q_verification_hash.is_none()
    {
        return Err(certificate_gap(
            "final invariant evidence is missing or does not justify final claim flags",
        ));
    }
    Ok(())
}

pub fn final_dag_replay_evidence(
    actual_projection_dag_hash: Option<Hash>,
    projection_message_hashes: Vec<Hash>,
    kernel_plan_hashes: Vec<Hash>,
    message_block_ids: Vec<BlockId>,
    per_message_source_relation_hashes: Vec<Vec<Hash>>,
    message_child_dependency_hashes: Vec<Vec<Hash>>,
    block_authorization_hashes: Vec<Hash>,
    edge_authorization_hashes: Vec<Hash>,
    actual_dag_replay_verified: bool,
) -> FinalDagReplayEvidence {
    let mut evidence = FinalDagReplayEvidence {
        actual_projection_dag_hash,
        projection_message_hashes,
        kernel_plan_hashes,
        message_block_ids,
        per_message_source_relation_hashes,
        message_child_dependency_hashes,
        block_authorization_hashes,
        edge_authorization_hashes,
        actual_dag_replay_verified,
        evidence_hash: hash_sequence("final-dag-replay-evidence", &[]),
    };
    evidence.evidence_hash = hash_final_dag_replay_evidence(&evidence);
    evidence
}

pub fn require_final_claim_dag_replay_evidence(
    cert: &CoreRunCertificate,
    evidence: &FinalDagReplayEvidence,
) -> Result<(), SolverError> {
    if !final_claim_dag_replay_structurally_bound_for_p12g(cert, evidence) {
        return Err(certificate_gap(
            "final claim requires hash-bound TargetProjectionDAG and block authorization evidence",
        ));
    }
    Err(certificate_gap(
        "P14 cannot close until actual DAG replay replaces synthetic all-relations replay for final claims",
    ))
}

pub fn final_claim_dag_replay_structurally_bound_for_p12g(
    cert: &CoreRunCertificate,
    evidence: &FinalDagReplayEvidence,
) -> bool {
    let message_count = cert.projection_message_hashes.len();
    evidence.evidence_hash == hash_final_dag_replay_evidence(evidence)
        && cert.final_dag_replay_evidence_hash == Some(evidence.evidence_hash)
        && evidence.actual_dag_replay_verified
        && evidence.actual_projection_dag_hash == Some(cert.target_projection_dag_hash)
        && evidence.projection_message_hashes == cert.projection_message_hashes
        && evidence.kernel_plan_hashes == cert.kernel_plan_hashes
        && evidence.message_block_ids.len() == message_count
        && evidence.per_message_source_relation_hashes.len() == message_count
        && evidence.message_child_dependency_hashes.len() == message_count
        && evidence.block_authorization_hashes.len() == message_count
        && (message_count <= 1 || !evidence.edge_authorization_hashes.is_empty())
}

pub fn hash_projection_messages(messages: &[ProjectionMessage]) -> Vec<Hash> {
    messages.iter().map(hash_projection_message).collect()
}

pub fn hash_root_isolation(roots: &[RealRootRecord]) -> Hash {
    let mut chunks = Vec::new();
    for root in roots {
        chunks.push(root.support_hash.0.to_vec());
        chunks.push(root.root_index.to_be_bytes().to_vec());
        chunks.push(rational_to_bytes(&root.isolating_interval.lo));
        chunks.push(rational_to_bytes(&root.isolating_interval.hi));
    }
    hash_sequence("root-isolation-records", &chunks)
}

pub fn hash_decoded_candidates(candidates: &[TargetCandidate]) -> Hash {
    let mut chunks = Vec::new();
    for candidate in candidates {
        chunks.push(candidate.target.0.to_be_bytes().to_vec());
        chunks.push(candidate.support_hash.0.to_vec());
        chunks.push(candidate.root_index.to_be_bytes().to_vec());
        chunks.push(rational_to_bytes(&candidate.isolating_interval.lo));
        chunks.push(rational_to_bytes(&candidate.isolating_interval.hi));
        chunks.push(candidate.candidate_hash.0.to_vec());
    }
    hash_sequence("decoded-target-candidates", &chunks)
}

pub fn hash_projection_message_dag_binding(
    target: VariableId,
    messages: &[ProjectionMessage],
) -> Hash {
    hash_projection_message_dag_binding_with_authorized_sources(target, messages, &[])
}

pub fn hash_projection_message_dag_binding_with_authorized_sources(
    target: VariableId,
    messages: &[ProjectionMessage],
    base_source_hashes: &[Hash],
) -> Hash {
    let mut chunks = vec![target.0.to_be_bytes().to_vec()];
    let dependencies = projection_message_dependency_indices(messages, base_source_hashes);
    for (idx, message) in messages.iter().enumerate() {
        chunks.push(message.block_id.0.to_be_bytes().to_vec());
        for variable in &message.exported_variables {
            chunks.push(variable.0.to_be_bytes().to_vec());
        }
        chunks.push(message.package_hash.0.to_vec());
        chunks.push(message.certificate.plan_hash.0.to_vec());
        chunks.push(Vec::new());
        if let Some(deps) = &dependencies {
            if let Some(message_deps) = deps.get(idx) {
                for dep_idx in message_deps {
                    chunks.push(messages[*dep_idx].package_hash.0.to_vec());
                }
            }
        } else {
            chunks.push(b"invalid-projection-message-dependency-graph".to_vec());
        }
    }
    hash_sequence("projection-message-dag-binding", &chunks)
}

pub fn projection_message_dependency_indices(
    messages: &[ProjectionMessage],
    base_source_hashes: &[Hash],
) -> Option<Vec<Vec<usize>>> {
    let base_sources = base_source_hashes.iter().copied().collect::<BTreeSet<_>>();
    let mut owners_by_relation = BTreeMap::<Hash, Vec<usize>>::new();
    for (idx, message) in messages.iter().enumerate() {
        for relation in &message.relation_generators {
            owners_by_relation
                .entry(relation.hash)
                .or_default()
                .push(idx);
        }
    }
    let mut dependencies = vec![Vec::new(); messages.len()];
    for (idx, message) in messages.iter().enumerate() {
        let mut source_hashes = BTreeSet::new();
        source_hashes.extend(message.certificate.source_relation_hashes.iter().copied());
        if let Some(payload_hashes) = payload_source_hashes(&message.certificate.payload) {
            source_hashes.extend(payload_hashes);
        }
        let mut deps = BTreeSet::new();
        for hash in source_hashes {
            if base_sources.contains(&hash) {
                continue;
            }
            if let Some(owners) = owners_by_relation.get(&hash) {
                for owner in owners {
                    if *owner != idx {
                        deps.insert(*owner);
                    }
                }
            }
        }
        dependencies[idx] = deps.into_iter().collect();
    }
    dependency_graph_is_acyclic(&dependencies).then_some(dependencies)
}

fn dependency_graph_is_acyclic(dependencies: &[Vec<usize>]) -> bool {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Mark {
        Visiting,
        Done,
    }

    fn visit(idx: usize, dependencies: &[Vec<usize>], marks: &mut [Option<Mark>]) -> bool {
        match marks[idx] {
            Some(Mark::Visiting) => return false,
            Some(Mark::Done) => return true,
            None => {}
        }
        marks[idx] = Some(Mark::Visiting);
        for dep in &dependencies[idx] {
            if *dep >= dependencies.len() || !visit(*dep, dependencies, marks) {
                return false;
            }
        }
        marks[idx] = Some(Mark::Done);
        true
    }

    let mut marks = vec![None; dependencies.len()];
    (0..dependencies.len()).all(|idx| visit(idx, dependencies, &mut marks))
}

pub fn hash_invariant_evidence(
    input_hash: Hash,
    messages: &[ProjectionMessage],
    support: Option<&UniPolynomialQ>,
    squarefree_support: Option<&UniPolynomialQ>,
    roots: &[RealRootRecord],
    candidates: &[TargetCandidate],
) -> Hash {
    let mut chunks = vec![input_hash.0.to_vec()];
    chunks.extend(
        hash_projection_messages(messages)
            .iter()
            .map(|hash| hash.0.to_vec()),
    );
    chunks.push(optional_hash_bytes(support.map(|support| support.hash)));
    chunks.push(optional_hash_bytes(
        squarefree_support.map(|support| support.hash),
    ));
    chunks.push(hash_root_isolation(roots).0.to_vec());
    chunks.push(hash_decoded_candidates(candidates).0.to_vec());
    hash_sequence("core-invariant-evidence", &chunks)
}

pub fn derive_core_invariant_flags(
    messages: &[ProjectionMessage],
    exact_q_verified: bool,
    no_hidden_fallback_verified: bool,
) -> CoreInvariantFlags {
    let no_coordinate_exports = messages.iter().all(message_forbids_coordinate_export);
    CoreInvariantFlags {
        no_geometry_dispatch: false,
        no_problem_id_dispatch: false,
        no_expected_answer_dispatch: false,
        no_full_coordinate_solution_set: no_coordinate_exports,
        no_full_coordinate_rur: no_coordinate_exports,
        no_qe_cad: false,
        exact_q_verification: exact_q_verified,
        no_hidden_fallback: no_hidden_fallback_verified && projection_plans_are_bound(messages),
    }
}

pub fn messages_have_verifiable_payloads(messages: &[ProjectionMessage]) -> bool {
    !messages.is_empty()
        && messages
            .iter()
            .all(|message| payload_is_verifiable(&message.certificate.payload))
}

pub fn projection_plans_are_bound(messages: &[ProjectionMessage]) -> bool {
    !messages.is_empty()
        && messages.iter().all(|message| {
            message.certificate.plan_hash != hash_sequence("synthetic-kernel-plan", &[])
        })
}

fn payload_is_verifiable(payload: &KernelCertificatePayload) -> bool {
    match payload {
        KernelCertificatePayload::BindingOnly | KernelCertificatePayload::SyntheticForTests => {
            false
        }
        KernelCertificatePayload::Universal(proof) => {
            proof
                .inner_payload
                .as_deref()
                .map_or(false, payload_is_verifiable)
                || !proof.output_memberships.is_empty()
        }
        _ => true,
    }
}

fn message_forbids_coordinate_export(message: &ProjectionMessage) -> bool {
    if !message.relation_generators.iter().all(|relation| {
        crate::types::polynomial::poly_variables(relation)
            .into_iter()
            .all(|var| message.exported_variables.contains(&var))
    }) {
        return false;
    }
    payload_forbids_coordinate_export(&message.certificate.payload)
}

fn payload_forbids_coordinate_export(payload: &KernelCertificatePayload) -> bool {
    match payload {
        KernelCertificatePayload::TargetAction(proof) => {
            proof.coverage.no_coordinate_roots_exported
                && proof.coverage.no_full_coordinate_rur_exported
                && proof.quotient_input.no_coordinate_roots_exported
                && proof.quotient_input.no_full_coordinate_rur_exported
        }
        KernelCertificatePayload::Universal(proof) => proof
            .inner_payload
            .as_deref()
            .map_or(true, payload_forbids_coordinate_export),
        KernelCertificatePayload::BindingOnly | KernelCertificatePayload::SyntheticForTests => {
            false
        }
        _ => true,
    }
}

pub fn hash_core_run_certificate(cert: &CoreRunCertificate) -> Hash {
    let mut chunks = vec![
        cert.input_hash.0.to_vec(),
        cert.canonical_system_hash.0.to_vec(),
        cert.target_variable.0.to_be_bytes().to_vec(),
        cert.compression_hash.0.to_vec(),
        cert.hypergraph_hash.0.to_vec(),
        cert.target_projection_dag_hash.0.to_vec(),
    ];
    for hash in &cert.kernel_plan_hashes {
        chunks.push(hash.0.to_vec());
    }
    chunks.push(Vec::new());
    for hash in &cert.projection_message_hashes {
        chunks.push(hash.0.to_vec());
    }
    chunks.push(Vec::new());
    chunks.push(optional_hash_bytes(cert.global_support_hash));
    chunks.push(optional_hash_bytes(cert.squarefree_support_hash));
    chunks.push(optional_hash_bytes(cert.root_isolation_hash));
    chunks.push(optional_hash_bytes(cert.decoded_candidate_hash));
    chunks.push(optional_hash_bytes(cert.global_support_certificate_hash));
    chunks.push(optional_hash_bytes(cert.final_dag_replay_evidence_hash));
    chunks.push(cert.invariant_evidence_hash.0.to_vec());
    chunks.push(vec![
        cert.invariants.no_geometry_dispatch as u8,
        cert.invariants.no_problem_id_dispatch as u8,
        cert.invariants.no_expected_answer_dispatch as u8,
        cert.invariants.no_full_coordinate_solution_set as u8,
        cert.invariants.no_full_coordinate_rur as u8,
        cert.invariants.no_qe_cad as u8,
        cert.invariants.exact_q_verification as u8,
        cert.invariants.no_hidden_fallback as u8,
    ]);
    hash_sequence("core-run-certificate", &chunks)
}

fn optional_hash_bytes(hash: Option<Hash>) -> Vec<u8> {
    hash.map(|value| value.0.to_vec())
        .unwrap_or_else(|| vec![0xff])
}

fn hash_final_invariant_evidence(evidence: &FinalInvariantEvidence) -> Hash {
    hash_sequence(
        "final-invariant-evidence",
        &[
            optional_hash_bytes(evidence.no_geometry_dispatch_scan_hash),
            optional_hash_bytes(evidence.no_problem_id_dispatch_scan_hash),
            optional_hash_bytes(evidence.no_expected_answer_dispatch_scan_hash),
            optional_hash_bytes(evidence.no_qe_cad_scan_hash),
            optional_hash_bytes(evidence.exact_q_verification_hash),
        ],
    )
}

fn hash_final_dag_replay_evidence(evidence: &FinalDagReplayEvidence) -> Hash {
    let mut chunks = vec![optional_hash_bytes(evidence.actual_projection_dag_hash)];
    for hash in &evidence.projection_message_hashes {
        chunks.push(hash.0.to_vec());
    }
    chunks.push(Vec::new());
    for hash in &evidence.kernel_plan_hashes {
        chunks.push(hash.0.to_vec());
    }
    chunks.push(Vec::new());
    for block_id in &evidence.message_block_ids {
        chunks.push(block_id.0.to_be_bytes().to_vec());
    }
    chunks.push(Vec::new());
    for hashes in &evidence.per_message_source_relation_hashes {
        for hash in hashes {
            chunks.push(hash.0.to_vec());
        }
        chunks.push(Vec::new());
    }
    chunks.push(Vec::new());
    for hashes in &evidence.message_child_dependency_hashes {
        for hash in hashes {
            chunks.push(hash.0.to_vec());
        }
        chunks.push(Vec::new());
    }
    chunks.push(Vec::new());
    chunks.extend(
        evidence
            .block_authorization_hashes
            .iter()
            .map(|hash| hash.0.to_vec()),
    );
    chunks.push(Vec::new());
    chunks.extend(
        evidence
            .edge_authorization_hashes
            .iter()
            .map(|hash| hash.0.to_vec()),
    );
    chunks.push(vec![evidence.actual_dag_replay_verified as u8]);
    hash_sequence("final-dag-replay-evidence", &chunks)
}

fn certificate_gap(missing_certificate_kind: &str) -> SolverError {
    SolverError {
        target: None,
        kind: SolverErrorKind::Failure(FailureKind::CertificateDesignGap {
            constructed_object_hash: hash_sequence(
                "certificate-gap",
                &[missing_certificate_kind.as_bytes().to_vec()],
            ),
            missing_certificate_kind: missing_certificate_kind.to_owned(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn p12g_final_invariant_claim_is_blocked_without_explicit_evidence() {
        let flags = CoreInvariantFlags {
            no_geometry_dispatch: false,
            no_problem_id_dispatch: false,
            no_expected_answer_dispatch: false,
            no_full_coordinate_solution_set: true,
            no_full_coordinate_rur: true,
            no_qe_cad: false,
            exact_q_verification: true,
            no_hidden_fallback: true,
        };
        let evidence = final_invariant_evidence(None, None, None, None, None);

        let err = require_final_claim_invariant_evidence(&flags, &evidence).unwrap_err();
        assert!(matches!(
            err.kind,
            SolverErrorKind::Failure(FailureKind::CertificateDesignGap { .. })
        ));
    }

    #[test]
    fn p12g_final_dag_claim_is_blocked_without_actual_dag_replay_evidence() {
        let cert = minimal_cert(
            hash_sequence("derived-message-dag", &[]),
            Vec::new(),
            Vec::new(),
            None,
        );
        let evidence = final_dag_replay_evidence(
            None,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            false,
        );

        let err = require_final_claim_dag_replay_evidence(&cert, &evidence).unwrap_err();
        assert!(matches!(
            err.kind,
            SolverErrorKind::Failure(FailureKind::CertificateDesignGap { .. })
        ));
    }

    #[test]
    fn p12g_replay_rejects_child_message_not_on_declared_dag_edge() {
        let dag_hash = hash_sequence("actual-dag", &[]);
        let message_hashes = vec![
            hash_sequence("message-parent", &[]),
            hash_sequence("message-child", &[]),
        ];
        let plan_hashes = vec![
            hash_sequence("plan-parent", &[]),
            hash_sequence("plan-child", &[]),
        ];
        let evidence = final_dag_replay_evidence(
            Some(dag_hash),
            message_hashes.clone(),
            plan_hashes.clone(),
            vec![BlockId(0), BlockId(1)],
            vec![
                vec![hash_sequence("source-parent", &[])],
                vec![hash_sequence("source-child", &[])],
            ],
            vec![Vec::new(), vec![message_hashes[0]]],
            vec![
                hash_sequence("block-parent-auth", &[]),
                hash_sequence("block-child-auth", &[]),
            ],
            Vec::new(),
            true,
        );
        let cert = minimal_cert(
            dag_hash,
            plan_hashes,
            message_hashes,
            Some(evidence.evidence_hash),
        );

        let err = require_final_claim_dag_replay_evidence(&cert, &evidence).unwrap_err();
        assert!(matches!(
            err.kind,
            SolverErrorKind::Failure(FailureKind::CertificateDesignGap { .. })
        ));
    }

    #[test]
    fn p12g_dag_authorization_hash_bound_into_run_certificate() {
        let dag_hash = hash_sequence("actual-dag", &[]);
        let message_hashes = vec![hash_sequence("message", &[])];
        let plan_hashes = vec![hash_sequence("plan", &[])];
        let evidence = final_dag_replay_evidence(
            Some(dag_hash),
            message_hashes.clone(),
            plan_hashes.clone(),
            vec![BlockId(0)],
            vec![vec![hash_sequence("source", &[])]],
            vec![Vec::new()],
            vec![hash_sequence("block-auth", &[])],
            Vec::new(),
            true,
        );
        let cert = minimal_cert(
            dag_hash,
            plan_hashes.clone(),
            message_hashes.clone(),
            Some(evidence.evidence_hash),
        );
        let unbound_cert = minimal_cert(dag_hash, plan_hashes, message_hashes, None);
        assert_ne!(
            hash_core_run_certificate(&unbound_cert),
            hash_core_run_certificate(&cert)
        );
        assert!(final_claim_dag_replay_structurally_bound_for_p12g(
            &cert, &evidence
        ));
        let err = require_final_claim_dag_replay_evidence(&cert, &evidence).unwrap_err();
        assert!(matches!(
            err.kind,
            SolverErrorKind::Failure(FailureKind::CertificateDesignGap { .. })
        ));

        let mut tampered = evidence.clone();
        tampered.block_authorization_hashes[0] = hash_sequence("different-block-auth", &[]);
        tampered.evidence_hash = hash_final_dag_replay_evidence(&tampered);
        assert!(!final_claim_dag_replay_structurally_bound_for_p12g(
            &cert, &tampered
        ));
        let err = require_final_claim_dag_replay_evidence(&cert, &tampered).unwrap_err();
        assert!(matches!(
            err.kind,
            SolverErrorKind::Failure(FailureKind::CertificateDesignGap { .. })
        ));
    }

    fn minimal_cert(
        dag_hash: Hash,
        kernel_plan_hashes: Vec<Hash>,
        projection_message_hashes: Vec<Hash>,
        final_dag_replay_evidence_hash: Option<Hash>,
    ) -> CoreRunCertificate {
        let mut cert = CoreRunCertificate {
            input_hash: hash_sequence("input", &[]),
            canonical_system_hash: hash_sequence("canonical", &[]),
            target_variable: VariableId(0),
            compression_hash: hash_sequence("compression", &[]),
            hypergraph_hash: hash_sequence("hypergraph", &[]),
            target_projection_dag_hash: dag_hash,
            kernel_plan_hashes,
            projection_message_hashes,
            global_support_hash: None,
            squarefree_support_hash: None,
            root_isolation_hash: None,
            decoded_candidate_hash: None,
            global_support_certificate_hash: None,
            final_dag_replay_evidence_hash,
            invariants: CoreInvariantFlags {
                no_geometry_dispatch: false,
                no_problem_id_dispatch: false,
                no_expected_answer_dispatch: false,
                no_full_coordinate_solution_set: true,
                no_full_coordinate_rur: true,
                no_qe_cad: false,
                exact_q_verification: true,
                no_hidden_fallback: true,
            },
            invariant_evidence_hash: hash_sequence("invariants", &[]),
            run_hash: hash_sequence("run", &[]),
        };
        cert.run_hash = hash_core_run_certificate(&cert);
        cert
    }
}
