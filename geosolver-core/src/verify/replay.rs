use std::collections::BTreeSet;

use crate::compose::compose::{hash_composed_projection, ComposedProjection};
use crate::graph::hypergraph::build_relation_variable_hypergraph;
use crate::graph::projection_dag::{authorize_block_relations, ProjectionBlock};
use crate::kernels::traits::ReplayResult;
use crate::preprocess::compression::CompressionState;
use crate::problem::canonicalize::canonicalize_system;
use crate::problem::input::RationalTargetProblem;
use crate::problem::validate::validate_input;
use crate::result::cost_trace::CompositionCostTrace;
use crate::result::output::TargetSolveResult;
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::BlockId;
use crate::types::polynomial::{poly_variables, SparsePolynomialQ};
use crate::verify::run_certificate::{
    derive_core_invariant_flags, hash_core_run_certificate, hash_decoded_candidates,
    hash_invariant_evidence, hash_projection_message_dag_binding_with_authorized_sources,
    hash_projection_messages, hash_root_isolation, projection_message_dependency_indices,
};
use crate::verify::verify_message::verify_projection_message;
use crate::verify::verify_support::verify_global_support;

pub fn replay_run_certificate(
    result: &TargetSolveResult,
    problem: &RationalTargetProblem,
) -> ReplayResult {
    let accepted = replay_checks(result, problem);
    ReplayResult {
        accepted,
        replay_hash: hash_sequence(
            "core-run-replay",
            &[
                problem.input_hash.0.to_vec(),
                result.target.0.to_be_bytes().to_vec(),
                result
                    .certificate
                    .as_ref()
                    .map(|cert| cert.run_hash.0.to_vec())
                    .unwrap_or_else(|| vec![0xff]),
                vec![accepted as u8],
            ],
        ),
    }
}

fn replay_checks(result: &TargetSolveResult, problem: &RationalTargetProblem) -> bool {
    let Some(cert) = &result.certificate else {
        return false;
    };
    if cert.run_hash != hash_core_run_certificate(cert) {
        return false;
    }
    if cert.input_hash != problem.input_hash || cert.input_hash != recompute_input_hash(problem) {
        return false;
    }
    let Ok(validated) = validate_input(problem.clone()) else {
        return false;
    };
    let Ok(canonical) = canonicalize_system(validated) else {
        return false;
    };
    if cert.canonical_system_hash != canonical.canonical_hash {
        return false;
    }
    let compressed = CompressionState::from_system(canonical.clone()).to_compressed_system();
    let base_source_hashes = base_source_hashes(&compressed);
    if cert.compression_hash != compressed.compressed_hash {
        return false;
    }
    if cert.hypergraph_hash != build_relation_variable_hypergraph(&compressed).hypergraph_hash {
        return false;
    }
    if cert.target_variable != result.target || result.target != problem.target {
        return false;
    }
    if cert.target_projection_dag_hash
        != hash_projection_message_dag_binding_with_authorized_sources(
            result.target,
            &result.projection_messages,
            &base_source_hashes,
        )
    {
        return false;
    }
    if cert.projection_message_hashes != hash_projection_messages(&result.projection_messages) {
        return false;
    }
    let message_plan_hashes = result
        .projection_messages
        .iter()
        .map(|message| message.certificate.plan_hash)
        .collect::<Vec<_>>();
    if cert.kernel_plan_hashes != message_plan_hashes {
        return false;
    }
    let support_hash = result
        .support_polynomial
        .as_ref()
        .map(|support| support.hash);
    if cert.global_support_hash != support_hash {
        return false;
    }
    let squarefree_hash = result
        .squarefree_support_polynomial
        .as_ref()
        .map(|support| support.hash);
    if cert.squarefree_support_hash != squarefree_hash {
        return false;
    }
    if cert.root_isolation_hash != Some(hash_root_isolation(&result.root_isolation)) {
        return false;
    }
    if cert.decoded_candidate_hash != Some(hash_decoded_candidates(&result.decoded_candidates)) {
        return false;
    }
    let Some(message_dependencies) =
        projection_message_dependency_indices(&result.projection_messages, &base_source_hashes)
    else {
        return false;
    };
    if cert.invariant_evidence_hash
        != hash_invariant_evidence(
            problem.input_hash,
            &result.projection_messages,
            result.support_polynomial.as_ref(),
            result.squarefree_support_polynomial.as_ref(),
            &result.root_isolation,
            &result.decoded_candidates,
        )
    {
        return false;
    }
    let projection_messages_verified =
        verify_projection_messages(result, &compressed, &message_dependencies);
    let support_verified =
        verify_support_from_messages(result, cert.global_support_certificate_hash);
    let roots_and_candidates_verified = verify_roots_and_candidates(result);
    let expected_invariants = derive_core_invariant_flags(
        &result.projection_messages,
        projection_messages_verified && support_verified,
        true,
    );
    if cert.invariants != expected_invariants || !expected_invariants.p11_replay_enforced() {
        return false;
    }
    if !projection_messages_verified || !support_verified {
        return false;
    }
    roots_and_candidates_verified
}

fn verify_projection_messages(
    result: &TargetSolveResult,
    compressed: &crate::preprocess::compression::CompressedSystemQ,
    message_dependencies: &[Vec<usize>],
) -> bool {
    if message_dependencies.len() != result.projection_messages.len() {
        return false;
    }
    for (message_idx, message) in result.projection_messages.iter().enumerate() {
        let mut local_variables = compressed
            .variables
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        local_variables.extend(message.exported_variables.iter().copied());
        local_variables.extend(message.eliminated_variables.iter().copied());
        let mut block = ProjectionBlock {
            block_id: message.block_id,
            local_variables,
            relation_ids: compressed.relation_order.clone(),
            exported_variables: message.exported_variables.iter().copied().collect(),
            child_block_ids: Vec::new(),
            parent_block_id: None,
            authorization_hash: hash_sequence("replay-block-auth", &[]),
            duplication_certificates: Vec::new(),
            block_hash: hash_sequence("replay-block", &[]),
        };
        block.authorization_hash = authorize_block_relations(&block, &compressed);
        let kctx = crate::kernels::traits::KernelContext {
            block,
            system: compressed.clone(),
            child_messages: message_dependencies[message_idx]
                .iter()
                .filter_map(|dep_idx| result.projection_messages.get(*dep_idx).cloned())
                .collect(),
        };
        if kctx.child_messages.len() != message_dependencies[message_idx].len() {
            return false;
        }
        if verify_projection_message(message, &kctx).is_err() {
            return false;
        }
    }
    true
}

#[allow(dead_code)]
pub(crate) fn verify_projection_messages_with_actual_blocks(
    result: &TargetSolveResult,
    compressed: &crate::preprocess::compression::CompressedSystemQ,
    blocks: &[ProjectionBlock],
    message_dependencies: &[Vec<usize>],
) -> bool {
    if message_dependencies.len() != result.projection_messages.len() {
        return false;
    }
    for (message_idx, message) in result.projection_messages.iter().enumerate() {
        let Some(block) = blocks
            .iter()
            .find(|block| block.block_id == message.block_id)
            .cloned()
        else {
            return false;
        };
        if authorize_block_relations(&block, compressed) != block.authorization_hash {
            return false;
        }
        let mut child_messages = Vec::new();
        for dep_idx in &message_dependencies[message_idx] {
            let Some(child_message) = result.projection_messages.get(*dep_idx).cloned() else {
                return false;
            };
            if !block.child_block_ids.contains(&child_message.block_id) {
                return false;
            }
            let Some(child_block) = blocks
                .iter()
                .find(|child| child.block_id == child_message.block_id)
            else {
                return false;
            };
            if child_block.parent_block_id != Some(block.block_id) {
                return false;
            }
            child_messages.push(child_message);
        }
        let kctx = crate::kernels::traits::KernelContext {
            block,
            system: compressed.clone(),
            child_messages,
        };
        if verify_projection_message(message, &kctx).is_err() {
            return false;
        }
    }
    true
}

fn base_source_hashes(compressed: &crate::preprocess::compression::CompressedSystemQ) -> Vec<Hash> {
    let mut hashes = BTreeSet::new();
    for relation in &compressed.relations {
        hashes.insert(relation.hash);
        hashes.insert(relation.polynomial.hash);
    }
    hashes.into_iter().collect()
}

fn verify_support_from_messages(
    result: &TargetSolveResult,
    expected_certificate_hash: Option<Hash>,
) -> bool {
    let Some(support) = &result.support_polynomial else {
        return result.squarefree_support_polynomial.is_none();
    };
    let Some(composed) = replay_composed_projection(result) else {
        return false;
    };
    let Ok(cert) = verify_global_support(support, &composed) else {
        return false;
    };
    expected_certificate_hash == Some(cert.certificate_hash)
}

fn verify_roots_and_candidates(result: &TargetSolveResult) -> bool {
    let support_hash = result
        .squarefree_support_polynomial
        .as_ref()
        .or(result.support_polynomial.as_ref())
        .map(|support| support.hash);
    if support_hash.is_none() {
        return result.root_isolation.is_empty() && result.decoded_candidates.is_empty();
    }
    let mut root_indices = BTreeSet::new();
    for root in &result.root_isolation {
        if Some(root.support_hash) != support_hash {
            return false;
        }
        if !root_indices.insert(root.root_index) {
            return false;
        }
    }
    if result.decoded_candidates.len() != result.root_isolation.len() {
        return false;
    }
    let mut candidate_indices = BTreeSet::new();
    for candidate in &result.decoded_candidates {
        if candidate.target != result.target || Some(candidate.support_hash) != support_hash {
            return false;
        }
        if !candidate_indices.insert(candidate.root_index) {
            return false;
        }
        let Some(root) = result
            .root_isolation
            .iter()
            .find(|root| root.root_index == candidate.root_index)
        else {
            return false;
        };
        if root.isolating_interval != candidate.isolating_interval {
            return false;
        }
        let Some(candidate_support_hash) = support_hash else {
            return false;
        };
        if candidate.candidate_hash
            != crate::roots::decode::hash_target_candidate(
                candidate.target,
                candidate_support_hash,
                candidate.root_index,
                &candidate.isolating_interval,
            )
        {
            return false;
        }
    }
    candidate_indices == root_indices
}

fn replay_composed_projection(result: &TargetSolveResult) -> Option<ComposedProjection> {
    let mut message_relations = Vec::<SparsePolynomialQ>::new();
    let mut root_relations = Vec::<SparsePolynomialQ>::new();
    let target_set = BTreeSet::from([result.target]);
    for message in &result.projection_messages {
        if crate::compose::message::hash_projection_message(message) != message.package_hash {
            return None;
        }
        for relation in &message.relation_generators {
            if poly_variables(relation).is_subset(&target_set) {
                root_relations.push(relation.clone());
            }
            message_relations.push(relation.clone());
        }
    }
    if root_relations.is_empty() {
        return None;
    }
    let mut composed = ComposedProjection {
        target: result.target,
        root_block_id: result
            .projection_messages
            .first()
            .map_or(BlockId(0), |message| message.block_id),
        message_relations,
        root_relations,
        source_message_hashes: result
            .projection_messages
            .iter()
            .map(|message| message.package_hash)
            .collect(),
        separator_elimination_hashes: Vec::new(),
        composition_cost: CompositionCostTrace {
            relation_count_before: 0,
            relation_count_after: 0,
        },
        composed_hash: hash_sequence("composed-projection", &[]),
    };
    composed.composition_cost.relation_count_before = composed.message_relations.len();
    composed.composition_cost.relation_count_after = composed.root_relations.len();
    composed.composed_hash = hash_composed_projection(&composed);
    Some(composed)
}

fn recompute_input_hash(problem: &RationalTargetProblem) -> Hash {
    let mut chunks = problem
        .equations
        .iter()
        .map(|p| p.hash.0.to_vec())
        .collect::<Vec<_>>();
    chunks.extend(
        problem
            .semantic_encodings
            .iter()
            .map(|encoding| encoding.semantic_hash.0.to_vec()),
    );
    hash_sequence("problem-input", &chunks)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::compose::compose::ComposedProjection;
    use crate::compose::message::{
        hash_projection_message, MessageRepresentation, ProjectionMessage, ProjectionStrength,
    };
    use crate::graph::hypergraph::build_relation_variable_hypergraph;
    use crate::graph::projection_dag::{authorize_block_relations, ProjectionBlock};
    use crate::kernels::action_krylov::TargetActionKrylovKernel;
    use crate::kernels::norm_trace_projection::NormTraceProjectionKernel;
    use crate::kernels::regular_chain_projection::RegularChainProjectionKernel;
    use crate::kernels::sparse_resultant::SparseResultantProjectionKernel;
    use crate::kernels::target_univariate::{admit_target_univariate, execute_target_univariate};
    use crate::kernels::traits::KernelContext;
    use crate::kernels::traits::KernelKind;
    use crate::kernels::traits::TargetProjectionKernel;
    use crate::kernels::universal_elimination::UniversalTargetEliminationKernel;
    use crate::planner::kernel_plan::CertificateRoute;
    use crate::preprocess::compression::CompressionState;
    use crate::problem::canonicalize::canonicalize_system;
    use crate::problem::context::new_context;
    use crate::problem::input::make_problem;
    use crate::problem::validate::validate_input;
    use crate::result::cost_trace::{GlobalCostTrace, ProjectionCostTrace};
    use crate::result::output::TargetSolveResult;
    use crate::result::status::SolverStatus;
    use crate::roots::decode::TargetCandidate;
    use crate::roots::isolate::RealRootRecord;
    use crate::solver::options::SolverOptions;
    use crate::types::hash::{hash_sequence, Hash};
    use crate::types::ids::{BlockId, PackageId, RelationId, VariableId};
    use crate::types::interval::interval_new;
    use crate::types::monomial::monomial_to_bytes;
    use crate::types::polynomial::{
        constant_poly, poly_mul, poly_scale, poly_sub, variable_poly, SparsePolynomialQ,
    };
    use crate::types::rational::{int_q, rational_to_bytes};
    use crate::types::univariate::{normalize_univariate, UniPolynomialQ};
    use crate::verify::certificates::{
        kernel_certificate_binding_hash, KernelCertificate, KernelCertificatePayload,
        TargetOnlySupportCertificate, UniversalProjectionCertificate,
    };
    use crate::verify::run_certificate::{
        build_core_run_certificate, hash_core_run_certificate, hash_projection_message_dag_binding,
        hash_projection_message_dag_binding_with_authorized_sources, CoreRunCertificateInput,
    };
    use crate::verify::verify_message::verify_projection_message;
    use crate::verify::verify_support::verify_global_support;

    #[test]
    fn p12g_replay_rejects_message_using_relation_outside_original_block() {
        let t = VariableId(0);
        let problem = make_problem(
            vec![t],
            t,
            vec![
                poly_sub(&variable_poly(t), &constant_poly(int_q(1))),
                poly_sub(&variable_poly(t), &constant_poly(int_q(2))),
            ],
            Vec::new(),
        );
        let compressed = compressed_system(&problem);
        let authorized_relation = compressed.relations[0].clone();
        let outside_relation = compressed.relations[1].clone();
        let support = normalize_univariate(UniPolynomialQ {
            variable: t,
            coeffs_low_to_high: vec![int_q(-2), int_q(1)],
            hash: hash_sequence("univariate", &[]),
        });
        let message = input_authorized_target_support_message(
            PackageId(200),
            t,
            outside_relation.polynomial.clone(),
            hash_sequence("p12g-outside-plan", &[]),
            hash_sequence("p12g-outside-cert", &[]),
            BlockId(0),
            outside_relation.id,
            outside_relation.hash,
        );

        let mut synthetic_block = ProjectionBlock {
            block_id: BlockId(0),
            local_variables: BTreeSet::from([t]),
            relation_ids: compressed.relation_order.clone(),
            exported_variables: BTreeSet::from([t]),
            child_block_ids: Vec::new(),
            parent_block_id: None,
            authorization_hash: hash_sequence("tmp", &[]),
            duplication_certificates: Vec::new(),
            block_hash: hash_sequence("test-block", &[]),
        };
        synthetic_block.authorization_hash =
            authorize_block_relations(&synthetic_block, &compressed);
        let synthetic_ctx = KernelContext {
            block: synthetic_block,
            system: compressed.clone(),
            child_messages: Vec::new(),
        };
        let synthetic_verification = verify_projection_message(&message, &synthetic_ctx);
        assert!(synthetic_verification.is_ok(), "{synthetic_verification:?}");

        let mut actual_block = ProjectionBlock {
            block_id: BlockId(0),
            local_variables: BTreeSet::from([t]),
            relation_ids: vec![authorized_relation.id],
            exported_variables: BTreeSet::from([t]),
            child_block_ids: Vec::new(),
            parent_block_id: None,
            authorization_hash: hash_sequence("tmp", &[]),
            duplication_certificates: Vec::new(),
            block_hash: hash_sequence("test-block", &[]),
        };
        actual_block.authorization_hash = authorize_block_relations(&actual_block, &compressed);
        let messages = vec![message.clone()];
        let cert = build_core_run_certificate(CoreRunCertificateInput {
            input_hash: problem.input_hash,
            canonical_hash: canonical_hash(&problem),
            target_variable: problem.target,
            compression_hash: compression_hash(&problem),
            hypergraph_hash: hypergraph_hash(&problem),
            dag_hash: hash_projection_message_dag_binding(t, &messages),
            kernel_plan_hashes: vec![message.certificate.plan_hash],
            projection_messages: &messages,
            support: Some(&support),
            squarefree_support: Some(&support),
            root_isolation: &[],
            decoded_candidates: &[],
            global_support_certificate_hash: None,
            final_dag_replay_evidence_hash: None,
        });
        let result = result(t, support, messages, cert);

        assert!(!super::verify_projection_messages_with_actual_blocks(
            &result,
            &compressed,
            &[actual_block],
            &[Vec::new()],
        ));
    }

    #[test]
    fn p12g_replay_rejects_child_message_not_on_declared_dag_edge() {
        let t = VariableId(0);
        let problem = make_problem(
            vec![t],
            t,
            vec![poly_sub(&variable_poly(t), &constant_poly(int_q(1)))],
            Vec::new(),
        );
        let compressed = compressed_system(&problem);
        let base_relation = compressed.relations[0].clone();
        let support = support_poly(t);
        let child_message = input_authorized_target_support_message(
            PackageId(210),
            t,
            base_relation.polynomial.clone(),
            hash_sequence("p12g-child-plan", &[]),
            hash_sequence("p12g-child-cert", &[]),
            BlockId(1),
            base_relation.id,
            base_relation.hash,
        );
        let parent_message = forged_target_support_message(
            PackageId(211),
            t,
            child_message.relation_generators[0].clone(),
            hash_sequence("p12g-parent-plan", &[]),
            hash_sequence("p12g-parent-cert", &[]),
            BlockId(0),
        );

        let mut synthetic_block = ProjectionBlock {
            block_id: BlockId(0),
            local_variables: BTreeSet::from([t]),
            relation_ids: Vec::new(),
            exported_variables: BTreeSet::from([t]),
            child_block_ids: Vec::new(),
            parent_block_id: None,
            authorization_hash: hash_sequence("tmp", &[]),
            duplication_certificates: Vec::new(),
            block_hash: hash_sequence("test-block", &[]),
        };
        synthetic_block.authorization_hash =
            authorize_block_relations(&synthetic_block, &compressed);
        let synthetic_ctx = KernelContext {
            block: synthetic_block,
            system: compressed.clone(),
            child_messages: vec![child_message.clone()],
        };
        let synthetic_verification = verify_projection_message(&parent_message, &synthetic_ctx);
        assert!(synthetic_verification.is_ok(), "{synthetic_verification:?}");

        let mut parent_block = ProjectionBlock {
            block_id: BlockId(0),
            local_variables: BTreeSet::from([t]),
            relation_ids: Vec::new(),
            exported_variables: BTreeSet::from([t]),
            child_block_ids: Vec::new(),
            parent_block_id: None,
            authorization_hash: hash_sequence("tmp", &[]),
            duplication_certificates: Vec::new(),
            block_hash: hash_sequence("test-block", &[]),
        };
        parent_block.authorization_hash = authorize_block_relations(&parent_block, &compressed);
        let mut child_block = ProjectionBlock {
            block_id: BlockId(1),
            local_variables: BTreeSet::from([t]),
            relation_ids: vec![base_relation.id],
            exported_variables: BTreeSet::from([t]),
            child_block_ids: Vec::new(),
            parent_block_id: Some(BlockId(0)),
            authorization_hash: hash_sequence("tmp", &[]),
            duplication_certificates: Vec::new(),
            block_hash: hash_sequence("test-block", &[]),
        };
        child_block.authorization_hash = authorize_block_relations(&child_block, &compressed);
        let messages = vec![parent_message.clone(), child_message.clone()];
        let cert = build_core_run_certificate(CoreRunCertificateInput {
            input_hash: problem.input_hash,
            canonical_hash: canonical_hash(&problem),
            target_variable: problem.target,
            compression_hash: compression_hash(&problem),
            hypergraph_hash: hypergraph_hash(&problem),
            dag_hash: hash_projection_message_dag_binding(t, &messages),
            kernel_plan_hashes: vec![
                parent_message.certificate.plan_hash,
                child_message.certificate.plan_hash,
            ],
            projection_messages: &messages,
            support: Some(&support),
            squarefree_support: Some(&support),
            root_isolation: &[],
            decoded_candidates: &[],
            global_support_certificate_hash: None,
            final_dag_replay_evidence_hash: None,
        });
        let result = result(t, support, messages, cert);

        assert!(!super::verify_projection_messages_with_actual_blocks(
            &result,
            &compressed,
            &[parent_block, child_block],
            &[vec![1], Vec::new()],
        ));
    }

    #[test]
    fn p11_replay_fails_when_projection_message_is_deleted() {
        let t = VariableId(0);
        let support = support_poly(t);
        let (message, _) = verified_target_univariate_message(t);
        let messages = vec![message.clone()];
        let problem = make_problem(
            vec![t],
            t,
            vec![poly_sub(
                &variable_poly(t),
                &crate::types::polynomial::constant_poly(int_q(1)),
            )],
            Vec::new(),
        );
        let cert = build_core_run_certificate(CoreRunCertificateInput {
            input_hash: problem.input_hash,
            canonical_hash: canonical_hash(&problem),
            target_variable: problem.target,
            compression_hash: compression_hash(&problem),
            hypergraph_hash: hypergraph_hash(&problem),
            dag_hash: hash_projection_message_dag_binding(t, &messages),
            kernel_plan_hashes: vec![message.certificate.plan_hash],
            projection_messages: &messages,
            support: Some(&support),
            squarefree_support: Some(&support),
            root_isolation: &[],
            decoded_candidates: &[],
            global_support_certificate_hash: Some(global_support_certificate_hash(
                t, &messages, &support,
            )),
            final_dag_replay_evidence_hash: None,
        });
        let mut result = result(t, support, messages, cert);
        assert!(super::replay_run_certificate(&result, &problem).accepted);
        result.projection_messages.clear();
        assert!(!super::replay_run_certificate(&result, &problem).accepted);
    }

    #[test]
    fn p11_replay_fails_on_support_root_and_candidate_tamper() {
        let t = VariableId(0);
        let support = support_poly(t);
        let (message, _) = verified_target_univariate_message(t);
        let messages = vec![message.clone()];
        let root = RealRootRecord {
            support_hash: support.hash,
            root_index: 0,
            isolating_interval: interval_new(int_q(1), int_q(1)).unwrap(),
        };
        let candidate = TargetCandidate {
            target: t,
            support_hash: support.hash,
            root_index: 0,
            isolating_interval: root.isolating_interval.clone(),
            candidate_hash: crate::roots::decode::hash_target_candidate(
                t,
                support.hash,
                0,
                &root.isolating_interval,
            ),
        };
        let problem = make_problem(
            vec![t],
            t,
            vec![poly_sub(
                &variable_poly(t),
                &crate::types::polynomial::constant_poly(int_q(1)),
            )],
            Vec::new(),
        );
        let cert = build_core_run_certificate(CoreRunCertificateInput {
            input_hash: problem.input_hash,
            canonical_hash: canonical_hash(&problem),
            target_variable: problem.target,
            compression_hash: compression_hash(&problem),
            hypergraph_hash: hypergraph_hash(&problem),
            dag_hash: hash_projection_message_dag_binding(t, &messages),
            kernel_plan_hashes: vec![message.certificate.plan_hash],
            projection_messages: &messages,
            support: Some(&support),
            squarefree_support: Some(&support),
            root_isolation: std::slice::from_ref(&root),
            decoded_candidates: std::slice::from_ref(&candidate),
            global_support_certificate_hash: Some(global_support_certificate_hash(
                t, &messages, &support,
            )),
            final_dag_replay_evidence_hash: None,
        });
        let mut result = result(t, support.clone(), messages, cert);
        result.root_isolation = vec![root];
        result.decoded_candidates = vec![candidate];
        assert!(super::replay_run_certificate(&result, &problem).accepted);
        let mut support_tamper = result.clone();
        support_tamper.support_polynomial = Some(normalize_univariate(UniPolynomialQ {
            variable: t,
            coeffs_low_to_high: vec![int_q(-2), int_q(1)],
            hash: hash_sequence("univariate", &[]),
        }));
        assert!(!super::replay_run_certificate(&support_tamper, &problem).accepted);
        let mut root_tamper = result.clone();
        root_tamper.root_isolation[0].root_index = 1;
        assert!(!super::replay_run_certificate(&root_tamper, &problem).accepted);
        let mut candidate_tamper = result;
        candidate_tamper.decoded_candidates[0].root_index = 1;
        assert!(!super::replay_run_certificate(&candidate_tamper, &problem).accepted);
    }

    #[test]
    fn p12_replay_rejects_candidate_omission_and_duplicates_even_when_hashes_match() {
        let t = VariableId(0);
        let support = support_poly(t);
        let (message, _) = verified_target_univariate_message(t);
        let messages = vec![message.clone()];
        let root = RealRootRecord {
            support_hash: support.hash,
            root_index: 0,
            isolating_interval: interval_new(int_q(1), int_q(1)).unwrap(),
        };
        let candidate = TargetCandidate {
            target: t,
            support_hash: support.hash,
            root_index: 0,
            isolating_interval: root.isolating_interval.clone(),
            candidate_hash: crate::roots::decode::hash_target_candidate(
                t,
                support.hash,
                0,
                &root.isolating_interval,
            ),
        };
        let problem = make_problem(
            vec![t],
            t,
            vec![poly_sub(
                &variable_poly(t),
                &crate::types::polynomial::constant_poly(int_q(1)),
            )],
            Vec::new(),
        );

        let omitted_cert = build_core_run_certificate(CoreRunCertificateInput {
            input_hash: problem.input_hash,
            canonical_hash: canonical_hash(&problem),
            target_variable: problem.target,
            compression_hash: compression_hash(&problem),
            hypergraph_hash: hypergraph_hash(&problem),
            dag_hash: hash_projection_message_dag_binding(t, &messages),
            kernel_plan_hashes: vec![message.certificate.plan_hash],
            projection_messages: &messages,
            support: Some(&support),
            squarefree_support: Some(&support),
            root_isolation: std::slice::from_ref(&root),
            decoded_candidates: &[],
            global_support_certificate_hash: Some(global_support_certificate_hash(
                t, &messages, &support,
            )),
            final_dag_replay_evidence_hash: None,
        });
        let mut omitted_result = result(t, support.clone(), messages.clone(), omitted_cert);
        omitted_result.root_isolation = vec![root.clone()];
        omitted_result.decoded_candidates = Vec::new();
        assert!(!super::replay_run_certificate(&omitted_result, &problem).accepted);

        let duplicate_candidates = vec![candidate.clone(), candidate];
        let duplicate_cert = build_core_run_certificate(CoreRunCertificateInput {
            input_hash: problem.input_hash,
            canonical_hash: canonical_hash(&problem),
            target_variable: problem.target,
            compression_hash: compression_hash(&problem),
            hypergraph_hash: hypergraph_hash(&problem),
            dag_hash: hash_projection_message_dag_binding(t, &messages),
            kernel_plan_hashes: vec![message.certificate.plan_hash],
            projection_messages: &messages,
            support: Some(&support),
            squarefree_support: Some(&support),
            root_isolation: std::slice::from_ref(&root),
            decoded_candidates: &duplicate_candidates,
            global_support_certificate_hash: Some(global_support_certificate_hash(
                t, &messages, &support,
            )),
            final_dag_replay_evidence_hash: None,
        });
        let mut duplicate_result = result(t, support, messages, duplicate_cert);
        duplicate_result.root_isolation = vec![root];
        duplicate_result.decoded_candidates = duplicate_candidates;
        assert!(!super::replay_run_certificate(&duplicate_result, &problem).accepted);
    }

    #[test]
    fn p11_global_support_rejects_tampered_support_relation() {
        let t = VariableId(0);
        let relation = poly_sub(
            &variable_poly(t),
            &crate::types::polynomial::constant_poly(int_q(1)),
        );
        let composed = ComposedProjection::from_message_relations_for_test(
            t,
            vec![relation],
            vec![Hash([7; 32])],
        );
        let good = support_poly(t);
        assert!(verify_global_support(&good, &composed).is_ok());
        let bad = normalize_univariate(UniPolynomialQ {
            variable: t,
            coeffs_low_to_high: vec![int_q(-2), int_q(1)],
            hash: hash_sequence("univariate", &[]),
        });
        assert!(verify_global_support(&bad, &composed).is_err());
    }

    #[test]
    fn p11_verify_projection_message_rejects_synthetic_and_tampered_certificates() {
        let t = VariableId(0);
        let (message, kctx) = verified_target_univariate_message(t);
        let verification = verify_projection_message(&message, &kctx);
        assert!(verification.is_ok(), "{verification:?}");

        let mut synthetic = message.clone();
        synthetic.certificate = KernelCertificate::synthetic_for_tests(Hash([6; 32]));
        synthetic.package_hash = hash_projection_message(&synthetic);
        assert!(verify_projection_message(&synthetic, &kctx).is_err());

        let mut tampered = message;
        tampered.certificate.binding_hash = Hash([9; 32]);
        tampered.package_hash = hash_projection_message(&tampered);
        assert!(verify_projection_message(&tampered, &kctx).is_err());
    }

    #[test]
    fn p11_replay_rejects_synthetic_projection_certificate_even_when_hashes_match() {
        let t = VariableId(0);
        let support = support_poly(t);
        let message = message(t, support.clone());
        let messages = vec![message.clone()];
        let problem = make_problem(
            vec![t],
            t,
            vec![poly_sub(
                &variable_poly(t),
                &crate::types::polynomial::constant_poly(int_q(1)),
            )],
            Vec::new(),
        );
        let cert = build_core_run_certificate(CoreRunCertificateInput {
            input_hash: problem.input_hash,
            canonical_hash: canonical_hash(&problem),
            target_variable: problem.target,
            compression_hash: compression_hash(&problem),
            hypergraph_hash: hypergraph_hash(&problem),
            dag_hash: hash_projection_message_dag_binding(t, &messages),
            kernel_plan_hashes: vec![message.certificate.plan_hash],
            projection_messages: &messages,
            support: Some(&support),
            squarefree_support: Some(&support),
            root_isolation: &[],
            decoded_candidates: &[],
            global_support_certificate_hash: Some(global_support_certificate_hash(
                t, &messages, &support,
            )),
            final_dag_replay_evidence_hash: None,
        });
        let result = result(t, support, messages, cert);
        assert!(!super::replay_run_certificate(&result, &problem).accepted);
    }

    #[test]
    fn p11_replay_fails_on_input_canonical_dag_plan_and_squarefree_tamper() {
        let t = VariableId(0);
        let support = support_poly(t);
        let (message, _) = verified_target_univariate_message(t);
        let messages = vec![message.clone()];
        let problem = make_problem(
            vec![t],
            t,
            vec![poly_sub(
                &variable_poly(t),
                &crate::types::polynomial::constant_poly(int_q(1)),
            )],
            Vec::new(),
        );
        let cert = build_core_run_certificate(CoreRunCertificateInput {
            input_hash: problem.input_hash,
            canonical_hash: canonical_hash(&problem),
            target_variable: problem.target,
            compression_hash: compression_hash(&problem),
            hypergraph_hash: hypergraph_hash(&problem),
            dag_hash: hash_projection_message_dag_binding(t, &messages),
            kernel_plan_hashes: vec![message.certificate.plan_hash],
            projection_messages: &messages,
            support: Some(&support),
            squarefree_support: Some(&support),
            root_isolation: &[],
            decoded_candidates: &[],
            global_support_certificate_hash: Some(global_support_certificate_hash(
                t, &messages, &support,
            )),
            final_dag_replay_evidence_hash: None,
        });
        let result = result(t, support.clone(), messages, cert);
        assert!(super::replay_run_certificate(&result, &problem).accepted);

        let input_tamper = make_problem(
            vec![t],
            t,
            vec![poly_sub(
                &variable_poly(t),
                &crate::types::polynomial::constant_poly(int_q(2)),
            )],
            Vec::new(),
        );
        assert!(!super::replay_run_certificate(&result, &input_tamper).accepted);

        let mut canonical_tamper = result.clone();
        let cert = canonical_tamper.certificate.as_mut().unwrap();
        cert.canonical_system_hash = Hash([9; 32]);
        cert.run_hash = hash_core_run_certificate(cert);
        assert!(!super::replay_run_certificate(&canonical_tamper, &problem).accepted);

        let mut dag_tamper = result.clone();
        let cert = dag_tamper.certificate.as_mut().unwrap();
        cert.target_projection_dag_hash = Hash([8; 32]);
        cert.run_hash = hash_core_run_certificate(cert);
        assert!(!super::replay_run_certificate(&dag_tamper, &problem).accepted);

        let mut plan_tamper = result.clone();
        let cert = plan_tamper.certificate.as_mut().unwrap();
        cert.kernel_plan_hashes[0] = Hash([7; 32]);
        cert.run_hash = hash_core_run_certificate(cert);
        assert!(!super::replay_run_certificate(&plan_tamper, &problem).accepted);

        let mut support_cert_tamper = result.clone();
        let cert = support_cert_tamper.certificate.as_mut().unwrap();
        cert.global_support_certificate_hash = Some(Hash([5; 32]));
        cert.run_hash = hash_core_run_certificate(cert);
        assert!(!super::replay_run_certificate(&support_cert_tamper, &problem).accepted);

        let mut invariant_tamper = result.clone();
        let cert = invariant_tamper.certificate.as_mut().unwrap();
        cert.invariant_evidence_hash = Hash([4; 32]);
        cert.run_hash = hash_core_run_certificate(cert);
        assert!(!super::replay_run_certificate(&invariant_tamper, &problem).accepted);

        let mut compression_tamper = result.clone();
        let cert = compression_tamper.certificate.as_mut().unwrap();
        cert.compression_hash = Hash([3; 32]);
        cert.run_hash = hash_core_run_certificate(cert);
        assert!(!super::replay_run_certificate(&compression_tamper, &problem).accepted);

        let mut hypergraph_tamper = result.clone();
        let cert = hypergraph_tamper.certificate.as_mut().unwrap();
        cert.hypergraph_hash = Hash([2; 32]);
        cert.run_hash = hash_core_run_certificate(cert);
        assert!(!super::replay_run_certificate(&hypergraph_tamper, &problem).accepted);

        let mut squarefree_tamper = result;
        squarefree_tamper.squarefree_support_polynomial =
            Some(normalize_univariate(UniPolynomialQ {
                variable: t,
                coeffs_low_to_high: vec![int_q(-2), int_q(1)],
                hash: hash_sequence("univariate", &[]),
            }));
        assert!(!super::replay_run_certificate(&squarefree_tamper, &problem).accepted);
    }

    #[test]
    fn p11_rejects_forged_target_action_payload_in_message_and_kernel_replay() {
        let t = VariableId(0);
        let (mut message, kctx) = verified_target_action_message(t);
        assert!(verify_projection_message(&message, &kctx).is_ok());

        let forged_support = poly_sub(&variable_poly(t), &constant_poly(int_q(3)));
        let forged_uni = normalize_univariate(UniPolynomialQ {
            variable: t,
            coeffs_low_to_high: vec![int_q(-3), int_q(1)],
            hash: hash_sequence("univariate", &[]),
        });
        if let crate::verify::certificates::KernelCertificatePayload::TargetAction(proof) =
            &mut message.certificate.payload
        {
            proof.output_relation = forged_support.clone();
            proof.coverage.characteristic_polynomial = forged_uni.clone();
            proof.coverage.characteristic_polynomial_hash = forged_uni.hash;
            proof.annihilator.polynomial_hash = forged_uni.hash;
        } else {
            panic!("expected target action payload");
        }
        message.relation_generators = vec![forged_support.clone()];
        message.certificate.output_relation_hashes = vec![forged_support.hash];
        message.certificate.binding_hash = kernel_certificate_binding_hash(&message.certificate);
        message.package_hash = hash_projection_message(&message);

        assert!(verify_projection_message(&message, &kctx).is_err());
        let kernel = TargetActionKrylovKernel;
        assert!(!kernel.replay(&message, &kctx).accepted);
    }

    #[test]
    fn p11_rejects_forged_regular_chain_payload_in_message_and_kernel_replay() {
        let t = VariableId(0);
        let y = VariableId(1);
        let (mut message, kctx) = verified_regular_chain_message(t, y);
        assert!(verify_projection_message(&message, &kctx).is_ok());

        let forged_support = poly_sub(&variable_poly(t), &constant_poly(int_q(99)));
        if let crate::verify::certificates::KernelCertificatePayload::RegularChain(proof) =
            &mut message.certificate.payload
        {
            proof.output_relations = vec![forged_support.clone()];
            proof.projections[0].generators = vec![forged_support.clone()];
        } else {
            panic!("expected regular chain payload");
        }
        message.relation_generators = vec![forged_support.clone()];
        message.certificate.output_relation_hashes = vec![forged_support.hash];
        message.certificate.binding_hash = kernel_certificate_binding_hash(&message.certificate);
        message.package_hash = hash_projection_message(&message);

        assert!(verify_projection_message(&message, &kctx).is_err());
        let kernel = RegularChainProjectionKernel;
        assert!(!kernel.replay(&message, &kctx).accepted);
    }

    #[test]
    fn p11_rejects_forged_sparse_resultant_payload_in_message_and_kernel_replay() {
        let t = VariableId(0);
        let y = VariableId(1);
        let (mut message, kctx) = verified_sparse_resultant_message(t, y);
        let verification = verify_projection_message(&message, &kctx);
        assert!(verification.is_ok(), "{verification:?}");

        let forged_relation = poly_sub(&variable_poly(t), &constant_poly(int_q(3)));
        if let KernelCertificatePayload::SparseResultant(proof) = &mut message.certificate.payload {
            proof.output_relations = vec![forged_relation.clone()];
        } else {
            panic!("expected sparse resultant payload");
        }
        message.relation_generators = vec![forged_relation.clone()];
        message.certificate.output_relation_hashes = vec![forged_relation.hash];
        message.certificate.binding_hash = kernel_certificate_binding_hash(&message.certificate);
        message.package_hash = hash_projection_message(&message);

        assert!(verify_projection_message(&message, &kctx).is_err());
        let kernel = SparseResultantProjectionKernel;
        assert!(!kernel.replay(&message, &kctx).accepted);
    }

    #[test]
    fn p11_rejects_forged_norm_trace_payload_source_and_tower_hash() {
        let t = VariableId(0);
        let a = VariableId(1);
        let (message, kctx) = verified_norm_trace_message(t, a);
        assert!(verify_projection_message(&message, &kctx).is_ok());

        let mut unauthorized_source = message.clone();
        if let KernelCertificatePayload::NormTrace(proof) =
            &mut unauthorized_source.certificate.payload
        {
            proof.tower.source_relation_hashes = vec![Hash([42; 32])];
        } else {
            panic!("expected norm trace payload");
        }
        unauthorized_source.certificate.binding_hash =
            kernel_certificate_binding_hash(&unauthorized_source.certificate);
        unauthorized_source.package_hash = hash_projection_message(&unauthorized_source);
        assert!(verify_projection_message(&unauthorized_source, &kctx).is_err());
        let kernel = NormTraceProjectionKernel;
        assert!(!kernel.replay(&unauthorized_source, &kctx).accepted);

        let mut tampered_tower_hash = message.clone();
        if let KernelCertificatePayload::NormTrace(proof) =
            &mut tampered_tower_hash.certificate.payload
        {
            proof.tower.tower_hash = Hash([55; 32]);
        } else {
            panic!("expected norm trace payload");
        }
        tampered_tower_hash.certificate.binding_hash =
            kernel_certificate_binding_hash(&tampered_tower_hash.certificate);
        tampered_tower_hash.package_hash = hash_projection_message(&tampered_tower_hash);
        assert!(verify_projection_message(&tampered_tower_hash, &kctx).is_err());
        assert!(!kernel.replay(&tampered_tower_hash, &kctx).accepted);

        let mut rehashed_body_tamper = message;
        if let KernelCertificatePayload::NormTrace(proof) =
            &mut rehashed_body_tamper.certificate.payload
        {
            proof.tower.steps[0].minimal_polynomial =
                poly_scale(&proof.tower.steps[0].minimal_polynomial, &int_q(2));
            proof.tower.steps[0].step_hash = test_tower_step_hash(&proof.tower.steps[0]);
            proof.tower.tower_hash = test_tower_plan_hash(&proof.tower);
        } else {
            panic!("expected norm trace payload");
        }
        rehashed_body_tamper.certificate.binding_hash =
            kernel_certificate_binding_hash(&rehashed_body_tamper.certificate);
        rehashed_body_tamper.package_hash = hash_projection_message(&rehashed_body_tamper);
        assert!(verify_projection_message(&rehashed_body_tamper, &kctx).is_err());
        assert!(!kernel.replay(&rehashed_body_tamper, &kctx).accepted);
    }

    #[test]
    fn p11_rejects_universal_inner_payload_with_unauthorized_sources() {
        let t = VariableId(0);
        let authorized_source = poly_sub(&variable_poly(t), &constant_poly(int_q(1)));
        let forged_source = poly_sub(&variable_poly(t), &constant_poly(int_q(7)));
        let forged_support = normalize_univariate(UniPolynomialQ {
            variable: t,
            coeffs_low_to_high: vec![int_q(-7), int_q(1)],
            hash: hash_sequence("univariate", &[]),
        });
        let canonical = canonicalize_system(
            validate_input(make_problem(
                vec![t],
                t,
                vec![authorized_source.clone()],
                Vec::new(),
            ))
            .unwrap(),
        )
        .unwrap();
        let compressed = CompressionState::from_system(canonical).to_compressed_system();
        let mut block = ProjectionBlock {
            block_id: BlockId(0),
            local_variables: BTreeSet::from([t]),
            relation_ids: compressed.relation_order.clone(),
            exported_variables: BTreeSet::from([t]),
            child_block_ids: Vec::new(),
            parent_block_id: None,
            authorization_hash: hash_sequence("tmp", &[]),
            duplication_certificates: Vec::new(),
            block_hash: hash_sequence("test-block", &[]),
        };
        block.authorization_hash = authorize_block_relations(&block, &compressed);
        let kctx = KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };

        let payload = KernelCertificatePayload::Universal(UniversalProjectionCertificate {
            stage_hash: hash_sequence("forged-universal-stage", &[]),
            stage_certificate_hash: hash_sequence("forged-universal-stage-cert", &[]),
            output_relations: vec![forged_source.clone()],
            inner_payload: Some(Box::new(KernelCertificatePayload::TargetOnlySupport(
                TargetOnlySupportCertificate {
                    target: t,
                    source_relations: vec![forged_source.clone()],
                    support_relation: forged_source.clone(),
                },
            ))),
            output_memberships: Vec::new(),
            source_relations: vec![authorized_source.clone()],
        });
        let mut cert = KernelCertificate {
            certificate_hash: hash_sequence("forged-universal-cert", &[]),
            certificate_route: CertificateRoute::UniversalFixedLocalElimination,
            plan_hash: hash_sequence("forged-universal-plan", &[]),
            source_relation_hashes: vec![authorized_source.hash],
            output_relation_hashes: vec![forged_source.hash],
            exported_variables: vec![t],
            binding_hash: hash_sequence("kernel-certificate-binding", &[]),
            payload,
        };
        cert.binding_hash = kernel_certificate_binding_hash(&cert);
        let mut message = ProjectionMessage {
            package_id: PackageId(11),
            block_id: kctx.block.block_id,
            kernel_kind: KernelKind::UniversalTargetElimination,
            source_relation_ids: vec![RelationId(0)],
            eliminated_variables: Vec::new(),
            exported_variables: vec![t],
            relation_generators: vec![forged_source.clone()],
            representation: MessageRepresentation::GeneratorSet,
            projection_strength: ProjectionStrength::CandidateCoverStrong,
            certificate: cert,
            compression_trace: Default::default(),
            cost_trace: ProjectionCostTrace::default(),
            package_hash: hash_sequence("projection-message-initial", &[]),
        };
        message.package_hash = hash_projection_message(&message);

        assert!(verify_projection_message(&message, &kctx).is_err());
        let kernel = UniversalTargetEliminationKernel;
        assert!(!kernel.replay(&message, &kctx).accepted);

        let messages = vec![message.clone()];
        let problem = make_problem(vec![t], t, vec![authorized_source], Vec::new());
        let cert = build_core_run_certificate(CoreRunCertificateInput {
            input_hash: problem.input_hash,
            canonical_hash: canonical_hash(&problem),
            target_variable: problem.target,
            compression_hash: compression_hash(&problem),
            hypergraph_hash: hypergraph_hash(&problem),
            dag_hash: hash_projection_message_dag_binding(t, &messages),
            kernel_plan_hashes: vec![message.certificate.plan_hash],
            projection_messages: &messages,
            support: Some(&forged_support),
            squarefree_support: Some(&forged_support),
            root_isolation: &[],
            decoded_candidates: &[],
            global_support_certificate_hash: Some(global_support_certificate_hash(
                t,
                &messages,
                &forged_support,
            )),
            final_dag_replay_evidence_hash: None,
        });
        let result = result(t, forged_support, messages, cert);
        assert!(!super::replay_run_certificate(&result, &problem).accepted);
    }

    #[test]
    fn p11_replay_rejects_mutual_projection_message_source_cycle() {
        let t = VariableId(0);
        let authorized_source = poly_sub(&variable_poly(t), &constant_poly(int_q(1)));
        let forged_relation = poly_sub(&variable_poly(t), &constant_poly(int_q(2)));
        let forged_support = normalize_univariate(UniPolynomialQ {
            variable: t,
            coeffs_low_to_high: vec![int_q(-2), int_q(1)],
            hash: hash_sequence("univariate", &[]),
        });
        let canonical = canonicalize_system(
            validate_input(make_problem(
                vec![t],
                t,
                vec![authorized_source.clone()],
                Vec::new(),
            ))
            .unwrap(),
        )
        .unwrap();
        let compressed = CompressionState::from_system(canonical).to_compressed_system();
        let mut block = ProjectionBlock {
            block_id: BlockId(0),
            local_variables: BTreeSet::from([t]),
            relation_ids: compressed.relation_order.clone(),
            exported_variables: BTreeSet::from([t]),
            child_block_ids: Vec::new(),
            parent_block_id: None,
            authorization_hash: hash_sequence("tmp", &[]),
            duplication_certificates: Vec::new(),
            block_hash: hash_sequence("test-block", &[]),
        };
        block.authorization_hash = authorize_block_relations(&block, &compressed);
        let kctx = KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };

        let mut message_a = forged_target_support_message(
            PackageId(21),
            t,
            forged_relation.clone(),
            hash_sequence("mutual-plan-a", &[]),
            hash_sequence("mutual-cert-a", &[]),
            kctx.block.block_id,
        );
        let mut message_b = forged_target_support_message(
            PackageId(22),
            t,
            forged_relation.clone(),
            hash_sequence("mutual-plan-b", &[]),
            hash_sequence("mutual-cert-b", &[]),
            kctx.block.block_id,
        );
        message_a.package_hash = hash_projection_message(&message_a);
        message_b.package_hash = hash_projection_message(&message_b);

        let mut a_ctx = kctx.clone();
        a_ctx.child_messages = vec![message_b.clone()];
        assert!(verify_projection_message(&message_a, &a_ctx).is_ok());
        let mut b_ctx = kctx;
        b_ctx.child_messages = vec![message_a.clone()];
        assert!(verify_projection_message(&message_b, &b_ctx).is_ok());

        let messages = vec![message_a.clone(), message_b.clone()];
        let problem = make_problem(vec![t], t, vec![authorized_source], Vec::new());
        let cert = build_core_run_certificate(CoreRunCertificateInput {
            input_hash: problem.input_hash,
            canonical_hash: canonical_hash(&problem),
            target_variable: problem.target,
            compression_hash: compression_hash(&problem),
            hypergraph_hash: hypergraph_hash(&problem),
            dag_hash: hash_projection_message_dag_binding(t, &messages),
            kernel_plan_hashes: vec![
                message_a.certificate.plan_hash,
                message_b.certificate.plan_hash,
            ],
            projection_messages: &messages,
            support: Some(&forged_support),
            squarefree_support: Some(&forged_support),
            root_isolation: &[],
            decoded_candidates: &[],
            global_support_certificate_hash: Some(global_support_certificate_hash(
                t,
                &messages,
                &forged_support,
            )),
            final_dag_replay_evidence_hash: None,
        });
        let result = result(t, forged_support, messages, cert);
        assert!(!super::replay_run_certificate(&result, &problem).accepted);
    }

    #[test]
    fn p11_replay_accepts_duplicate_hash_when_source_is_input_authorized() {
        let t = VariableId(0);
        let seed_source = poly_sub(&variable_poly(t), &constant_poly(int_q(1)));
        let authorized_source =
            crate::kernels::target_univariate::target_only_support_from_polynomials(
                &[seed_source],
                t,
            )
            .unwrap();
        let support_relation =
            crate::kernels::target_univariate::target_only_support_from_polynomials(
                std::slice::from_ref(&authorized_source),
                t,
            )
            .unwrap();
        assert_eq!(support_relation, authorized_source);
        let support = support_poly(t);
        let canonical = canonicalize_system(
            validate_input(make_problem(
                vec![t],
                t,
                vec![authorized_source.clone()],
                Vec::new(),
            ))
            .unwrap(),
        )
        .unwrap();
        let compressed = CompressionState::from_system(canonical).to_compressed_system();
        let canonical_relation = compressed.relations[0].clone();
        let base_hashes = vec![canonical_relation.hash, canonical_relation.polynomial.hash];
        let mut block = ProjectionBlock {
            block_id: BlockId(0),
            local_variables: BTreeSet::from([t]),
            relation_ids: compressed.relation_order.clone(),
            exported_variables: BTreeSet::from([t]),
            child_block_ids: Vec::new(),
            parent_block_id: None,
            authorization_hash: hash_sequence("tmp", &[]),
            duplication_certificates: Vec::new(),
            block_hash: hash_sequence("test-block", &[]),
        };
        block.authorization_hash = authorize_block_relations(&block, &compressed);
        let kctx = KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };

        let message_a = input_authorized_target_support_message(
            PackageId(31),
            t,
            canonical_relation.polynomial.clone(),
            hash_sequence("input-dup-plan-a", &[]),
            hash_sequence("input-dup-cert-a", &[]),
            kctx.block.block_id,
            canonical_relation.id,
            canonical_relation.hash,
        );
        let message_b = input_authorized_target_support_message(
            PackageId(32),
            t,
            canonical_relation.polynomial.clone(),
            hash_sequence("input-dup-plan-b", &[]),
            hash_sequence("input-dup-cert-b", &[]),
            kctx.block.block_id,
            canonical_relation.id,
            canonical_relation.hash,
        );
        let a_verification = verify_projection_message(&message_a, &kctx);
        assert!(a_verification.is_ok(), "{a_verification:?}");
        let b_verification = verify_projection_message(&message_b, &kctx);
        assert!(b_verification.is_ok(), "{b_verification:?}");

        let messages = vec![message_a.clone(), message_b.clone()];
        let problem = make_problem(vec![t], t, vec![authorized_source], Vec::new());
        let cert = build_core_run_certificate(CoreRunCertificateInput {
            input_hash: problem.input_hash,
            canonical_hash: canonical_hash(&problem),
            target_variable: problem.target,
            compression_hash: compression_hash(&problem),
            hypergraph_hash: hypergraph_hash(&problem),
            dag_hash: hash_projection_message_dag_binding_with_authorized_sources(
                t,
                &messages,
                &base_hashes,
            ),
            kernel_plan_hashes: vec![
                message_a.certificate.plan_hash,
                message_b.certificate.plan_hash,
            ],
            projection_messages: &messages,
            support: Some(&support),
            squarefree_support: Some(&support),
            root_isolation: &[],
            decoded_candidates: &[],
            global_support_certificate_hash: Some(global_support_certificate_hash(
                t, &messages, &support,
            )),
            final_dag_replay_evidence_hash: None,
        });
        let result = result(t, support, messages, cert);
        assert!(super::replay_run_certificate(&result, &problem).accepted);
    }

    fn support_poly(t: VariableId) -> UniPolynomialQ {
        normalize_univariate(UniPolynomialQ {
            variable: t,
            coeffs_low_to_high: vec![int_q(-1), int_q(1)],
            hash: hash_sequence("univariate", &[]),
        })
    }

    fn canonical_hash(problem: &crate::problem::input::RationalTargetProblem) -> Hash {
        canonicalize_system(validate_input(problem.clone()).unwrap())
            .unwrap()
            .canonical_hash
    }

    fn compression_hash(problem: &crate::problem::input::RationalTargetProblem) -> Hash {
        compressed_system(problem).compressed_hash
    }

    fn hypergraph_hash(problem: &crate::problem::input::RationalTargetProblem) -> Hash {
        build_relation_variable_hypergraph(&compressed_system(problem)).hypergraph_hash
    }

    fn compressed_system(
        problem: &crate::problem::input::RationalTargetProblem,
    ) -> crate::preprocess::compression::CompressedSystemQ {
        let canonical = canonicalize_system(validate_input(problem.clone()).unwrap()).unwrap();
        CompressionState::from_system(canonical).to_compressed_system()
    }

    fn global_support_certificate_hash(
        t: VariableId,
        messages: &[ProjectionMessage],
        support: &UniPolynomialQ,
    ) -> Hash {
        let relations = messages
            .iter()
            .flat_map(|message| message.relation_generators.clone())
            .collect::<Vec<_>>();
        let composed = ComposedProjection::from_message_relations_for_test(
            t,
            relations,
            messages
                .iter()
                .map(|message| message.package_hash)
                .collect(),
        );
        verify_global_support(support, &composed)
            .unwrap()
            .certificate_hash
    }

    fn verified_target_univariate_message(t: VariableId) -> (ProjectionMessage, KernelContext) {
        let relation = poly_sub(
            &variable_poly(t),
            &crate::types::polynomial::constant_poly(int_q(1)),
        );
        let canonical = canonicalize_system(
            validate_input(make_problem(vec![t], t, vec![relation], Vec::new())).unwrap(),
        )
        .unwrap();
        let compressed = CompressionState::from_system(canonical).to_compressed_system();
        let mut block = ProjectionBlock {
            block_id: BlockId(0),
            local_variables: BTreeSet::from([t]),
            relation_ids: compressed.relation_order.clone(),
            exported_variables: BTreeSet::from([t]),
            child_block_ids: Vec::new(),
            parent_block_id: None,
            authorization_hash: hash_sequence("tmp", &[]),
            duplication_certificates: Vec::new(),
            block_hash: hash_sequence("test-block", &[]),
        };
        block.authorization_hash = authorize_block_relations(&block, &compressed);
        let mut ctx = new_context(SolverOptions::default());
        let admission = admit_target_univariate(&block, &compressed, &ctx);
        let plan = admission.execution_plan.clone().unwrap();
        let mut kctx = KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };
        let message = execute_target_univariate(&plan, &mut kctx, &mut ctx).unwrap();
        (message, kctx)
    }

    fn verified_target_action_message(t: VariableId) -> (ProjectionMessage, KernelContext) {
        let relation = poly_mul(
            &poly_sub(&variable_poly(t), &constant_poly(int_q(1))),
            &poly_sub(&variable_poly(t), &constant_poly(int_q(2))),
        );
        let canonical = canonicalize_system(
            validate_input(make_problem(vec![t], t, vec![relation], Vec::new())).unwrap(),
        )
        .unwrap();
        let compressed = CompressionState::from_system(canonical).to_compressed_system();
        let mut block = ProjectionBlock {
            block_id: BlockId(0),
            local_variables: BTreeSet::from([t]),
            relation_ids: compressed.relation_order.clone(),
            exported_variables: BTreeSet::from([t]),
            child_block_ids: Vec::new(),
            parent_block_id: None,
            authorization_hash: hash_sequence("tmp", &[]),
            duplication_certificates: Vec::new(),
            block_hash: hash_sequence("test-block", &[]),
        };
        block.authorization_hash = authorize_block_relations(&block, &compressed);
        let mut solver_ctx = new_context(SolverOptions::default());
        let mut kctx = KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };
        let kernel = TargetActionKrylovKernel;
        let admission = kernel.admit(&kctx.block, &kctx);
        let plan = kernel.plan(&admission, &kctx, &solver_ctx).unwrap();
        let message = kernel.execute(&plan, &mut kctx, &mut solver_ctx).unwrap();
        (message, kctx)
    }

    fn verified_regular_chain_message(
        t: VariableId,
        y: VariableId,
    ) -> (ProjectionMessage, KernelContext) {
        let canonical = canonicalize_system(
            validate_input(make_problem(
                vec![t, y],
                t,
                vec![
                    poly_sub(&variable_poly(y), &variable_poly(t)),
                    poly_sub(
                        &poly_mul(&variable_poly(t), &variable_poly(t)),
                        &constant_poly(int_q(2)),
                    ),
                ],
                Vec::new(),
            ))
            .unwrap(),
        )
        .unwrap();
        let compressed = CompressionState::from_system(canonical).to_compressed_system();
        let mut block = ProjectionBlock {
            block_id: BlockId(0),
            local_variables: BTreeSet::from([t, y]),
            relation_ids: compressed.relation_order.clone(),
            exported_variables: BTreeSet::from([t]),
            child_block_ids: Vec::new(),
            parent_block_id: None,
            authorization_hash: hash_sequence("tmp", &[]),
            duplication_certificates: Vec::new(),
            block_hash: hash_sequence("test-block", &[]),
        };
        block.authorization_hash = authorize_block_relations(&block, &compressed);
        let mut solver_ctx = new_context(SolverOptions::default());
        let mut kctx = KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };
        let kernel = RegularChainProjectionKernel;
        let admission = kernel.admit(&kctx.block, &kctx);
        let plan = kernel.plan(&admission, &kctx, &solver_ctx).unwrap();
        let message = kernel.execute(&plan, &mut kctx, &mut solver_ctx).unwrap();
        (message, kctx)
    }

    fn verified_sparse_resultant_message(
        t: VariableId,
        y: VariableId,
    ) -> (ProjectionMessage, KernelContext) {
        let canonical = canonicalize_system(
            validate_input(make_problem(
                vec![t, y],
                t,
                vec![
                    poly_sub(&variable_poly(y), &variable_poly(t)),
                    poly_sub(&variable_poly(y), &constant_poly(int_q(1))),
                ],
                Vec::new(),
            ))
            .unwrap(),
        )
        .unwrap();
        let compressed = CompressionState::from_system(canonical).to_compressed_system();
        let mut block = ProjectionBlock {
            block_id: BlockId(0),
            local_variables: BTreeSet::from([t, y]),
            relation_ids: compressed.relation_order.clone(),
            exported_variables: BTreeSet::from([t]),
            child_block_ids: Vec::new(),
            parent_block_id: None,
            authorization_hash: hash_sequence("tmp", &[]),
            duplication_certificates: Vec::new(),
            block_hash: hash_sequence("test-block", &[]),
        };
        block.authorization_hash = authorize_block_relations(&block, &compressed);
        let mut solver_ctx = new_context(SolverOptions::default());
        let mut kctx = KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };
        let kernel = SparseResultantProjectionKernel;
        let admission = kernel.admit(&kctx.block, &kctx);
        let plan = kernel.plan(&admission, &kctx, &solver_ctx).unwrap();
        let message = kernel.execute(&plan, &mut kctx, &mut solver_ctx).unwrap();
        (message, kctx)
    }

    fn verified_norm_trace_message(
        t: VariableId,
        a: VariableId,
    ) -> (ProjectionMessage, KernelContext) {
        let canonical = canonicalize_system(
            validate_input(make_problem(
                vec![t, a],
                t,
                vec![
                    poly_sub(
                        &poly_mul(&variable_poly(a), &variable_poly(a)),
                        &constant_poly(int_q(2)),
                    ),
                    poly_sub(&variable_poly(t), &variable_poly(a)),
                ],
                Vec::new(),
            ))
            .unwrap(),
        )
        .unwrap();
        let compressed = CompressionState::from_system(canonical).to_compressed_system();
        let mut block = ProjectionBlock {
            block_id: BlockId(0),
            local_variables: BTreeSet::from([t, a]),
            relation_ids: compressed.relation_order.clone(),
            exported_variables: BTreeSet::from([t]),
            child_block_ids: Vec::new(),
            parent_block_id: None,
            authorization_hash: hash_sequence("tmp", &[]),
            duplication_certificates: Vec::new(),
            block_hash: hash_sequence("test-block", &[]),
        };
        block.authorization_hash = authorize_block_relations(&block, &compressed);
        let mut solver_ctx = new_context(SolverOptions::default());
        let mut kctx = KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };
        let kernel = NormTraceProjectionKernel;
        let admission = kernel.admit(&kctx.block, &kctx);
        let plan = kernel.plan(&admission, &kctx, &solver_ctx).unwrap();
        let message = kernel.execute(&plan, &mut kctx, &mut solver_ctx).unwrap();
        (message, kctx)
    }

    fn message(t: VariableId, support: UniPolynomialQ) -> ProjectionMessage {
        let relation = poly_sub(
            &variable_poly(t),
            &crate::types::polynomial::constant_poly(int_q(1)),
        );
        let mut msg = ProjectionMessage {
            package_id: PackageId(1),
            block_id: BlockId(1),
            kernel_kind: KernelKind::TargetUnivariate,
            source_relation_ids: vec![RelationId(0)],
            eliminated_variables: Vec::new(),
            exported_variables: vec![t],
            relation_generators: vec![relation],
            representation: MessageRepresentation::PrincipalSupport,
            projection_strength: ProjectionStrength::CandidateCoverStrong,
            certificate: KernelCertificate::synthetic_for_tests(support.hash),
            compression_trace: Default::default(),
            cost_trace: ProjectionCostTrace::default(),
            package_hash: hash_sequence("projection-message-initial", &[]),
        };
        msg.package_hash = hash_projection_message(&msg);
        msg
    }

    fn forged_target_support_message(
        package_id: PackageId,
        target: VariableId,
        relation: crate::types::polynomial::SparsePolynomialQ,
        plan_hash: Hash,
        certificate_hash: Hash,
        block_id: BlockId,
    ) -> ProjectionMessage {
        let payload = KernelCertificatePayload::TargetOnlySupport(TargetOnlySupportCertificate {
            target,
            source_relations: vec![relation.clone()],
            support_relation: relation.clone(),
        });
        let mut cert = KernelCertificate {
            certificate_hash,
            certificate_route: CertificateRoute::SourceMembershipCertificate,
            plan_hash,
            source_relation_hashes: vec![relation.hash],
            output_relation_hashes: vec![relation.hash],
            exported_variables: vec![target],
            binding_hash: hash_sequence("kernel-certificate-binding", &[]),
            payload,
        };
        cert.binding_hash = kernel_certificate_binding_hash(&cert);
        let mut message = ProjectionMessage {
            package_id,
            block_id,
            kernel_kind: KernelKind::TargetUnivariate,
            source_relation_ids: Vec::new(),
            eliminated_variables: Vec::new(),
            exported_variables: vec![target],
            relation_generators: vec![relation],
            representation: MessageRepresentation::PrincipalSupport,
            projection_strength: ProjectionStrength::CandidateCoverStrong,
            certificate: cert,
            compression_trace: Default::default(),
            cost_trace: ProjectionCostTrace::default(),
            package_hash: hash_sequence("projection-message-initial", &[]),
        };
        message.package_hash = hash_projection_message(&message);
        message
    }

    fn input_authorized_target_support_message(
        package_id: PackageId,
        target: VariableId,
        relation: crate::types::polynomial::SparsePolynomialQ,
        plan_hash: Hash,
        certificate_hash: Hash,
        block_id: BlockId,
        source_relation_id: RelationId,
        source_relation_hash: Hash,
    ) -> ProjectionMessage {
        let support = crate::kernels::target_univariate::target_only_support_from_polynomials(
            std::slice::from_ref(&relation),
            target,
        )
        .unwrap();
        let payload = KernelCertificatePayload::TargetOnlySupport(TargetOnlySupportCertificate {
            target,
            source_relations: vec![relation.clone()],
            support_relation: support.clone(),
        });
        let mut cert = KernelCertificate {
            certificate_hash,
            certificate_route: CertificateRoute::SourceMembershipCertificate,
            plan_hash,
            source_relation_hashes: vec![source_relation_hash],
            output_relation_hashes: vec![support.hash],
            exported_variables: vec![target],
            binding_hash: hash_sequence("kernel-certificate-binding", &[]),
            payload,
        };
        cert.binding_hash = kernel_certificate_binding_hash(&cert);
        let mut message = ProjectionMessage {
            package_id,
            block_id,
            kernel_kind: KernelKind::TargetUnivariate,
            source_relation_ids: vec![source_relation_id],
            eliminated_variables: Vec::new(),
            exported_variables: vec![target],
            relation_generators: vec![support],
            representation: MessageRepresentation::PrincipalSupport,
            projection_strength: ProjectionStrength::CandidateCoverStrong,
            certificate: cert,
            compression_trace: Default::default(),
            cost_trace: ProjectionCostTrace::default(),
            package_hash: hash_sequence("projection-message-initial", &[]),
        };
        message.package_hash = hash_projection_message(&message);
        message
    }

    fn test_tower_step_hash(step: &crate::algebra::norm_trace::TowerStep) -> Hash {
        hash_sequence(
            "tower-step",
            &[
                step.algebraic_variable.0.to_be_bytes().to_vec(),
                step.minimal_polynomial.hash.0.to_vec(),
            ],
        )
    }

    fn test_tower_plan_hash(tower: &crate::algebra::norm_trace::TowerPlanDescription) -> Hash {
        let mut chunks = Vec::new();
        for step in &tower.steps {
            chunks.push(step.step_hash.0.to_vec());
        }
        chunks.push(Vec::new());
        for var in &tower.exported_variables {
            chunks.push(var.0.to_be_bytes().to_vec());
        }
        chunks.push(test_poly_bytes(&tower.target_minus_expression));
        for hash in &tower.source_relation_hashes {
            chunks.push(hash.0.to_vec());
        }
        hash_sequence("tower-plan-description", &chunks)
    }

    fn test_poly_bytes(poly: &SparsePolynomialQ) -> Vec<u8> {
        let mut chunks = Vec::new();
        for term in &poly.terms {
            chunks.extend_from_slice(&rational_to_bytes(&term.coeff));
            chunks.extend_from_slice(&monomial_to_bytes(&term.monomial));
        }
        chunks
    }

    fn result(
        t: VariableId,
        support: UniPolynomialQ,
        messages: Vec<ProjectionMessage>,
        certificate: crate::verify::run_certificate::CoreRunCertificate,
    ) -> TargetSolveResult {
        TargetSolveResult {
            status: SolverStatus::CertifiedCandidateCover,
            target: t,
            support_polynomial: Some(support.clone()),
            squarefree_support_polynomial: Some(support),
            root_isolation: Vec::new(),
            decoded_candidates: Vec::new(),
            projection_messages: messages,
            certificate: Some(certificate),
            diagnostics: Vec::new(),
            cost_trace: GlobalCostTrace::default(),
        }
    }
}
