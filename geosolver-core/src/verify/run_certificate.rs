use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::compose::message::{hash_projection_message, ProjectionMessage};
use crate::roots::decode::TargetCandidate;
use crate::roots::isolate::RealRootRecord;
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::VariableId;
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
