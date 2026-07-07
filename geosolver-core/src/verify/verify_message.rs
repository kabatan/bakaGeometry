use std::collections::{BTreeMap, BTreeSet};

use crate::algebra::elimination::validate_local_elimination_result;
use crate::algebra::interpolation::verify_interpolated_relation;
use crate::algebra::krylov::{
    block_krylov_sequence, certify_krylov_coverage, recover_recurrence, verify_annihilator,
    KrylovPlan,
};
use crate::algebra::linear_solve::{solve_homogeneous_modular, MatrixBuilder, ModularSolvePlan};
use crate::algebra::norm_trace::verify_norm_tower_plan_relation;
use crate::algebra::normal_form::verify_membership_by_certificate;
use crate::algebra::quotient::{
    build_production_target_relevant_quotient_handle, unit_vector, TargetQuotientHandle,
};
use crate::algebra::regular_chain::{
    combine_chain_projections, local_regular_chain_decomposition, project_chain_to_variables,
    verify_regular_chain_dag_evidence, RegularChainInput,
};
use crate::algebra::resultant::{
    build_sparse_resultant_template, compute_resultant_relation, verify_resultant_certificate,
    ModularOptions,
};
use crate::compose::message::{hash_projection_message, ProjectionMessage};
use crate::compose::message::{MessageRepresentation, ProjectionStrength};
use crate::kernels::target_relation_search::build_membership_matrix;
use crate::kernels::target_univariate::target_only_support_from_polynomials;
use crate::kernels::traits::{KernelContext, KernelKind};
use crate::planner::cost_model::RouteCostClass;
use crate::planner::kernel_plan::{CertificateRoute, UniversalStrategy};
use crate::preprocess::compression::{affine_parts_in_variable, substitute_rational_and_clear};
use crate::result::status::{FailureKind, SolverError, SolverErrorKind};
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::VariableId;
use crate::types::matrix::{hash_matrix, VectorQ};
use crate::types::monomial::Monomial;
use crate::types::polynomial::{
    clear_denominators_primitive, constant_poly, normalize_poly, poly_add, poly_mul, poly_scale,
    poly_variables, substitute_poly, zero_poly, SparsePolynomialQ, SubstitutionMap, TermQ,
};
use crate::types::rational::{div_q, int_q, is_zero_q, neg_q, RationalQ};
use crate::types::univariate::UniPolynomialQ;
use crate::verify::certificates::{
    kernel_certificate_binding_hash, target_relation_exact_identity_hash,
    target_relation_hash_list, target_relation_monomial_support_hash,
    target_relation_multipliers_hash, target_relation_variable_hash, KernelCertificate,
    KernelCertificatePayload, MembershipProjectionCertificate, TargetRelationSearchCertificate,
};

pub fn verify_projection_message(
    message: &ProjectionMessage,
    ctx: &KernelContext,
) -> Result<(), SolverError> {
    if hash_projection_message(message) != message.package_hash {
        return Err(implementation_bug(
            "projection message package hash mismatch",
        ));
    }
    if message.block_id != ctx.block.block_id {
        return Err(implementation_bug(
            "projection message block does not match replay context",
        ));
    }
    let allowed_exports = ctx.block.exported_variables.clone();
    let exported = message
        .exported_variables
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    if !exported.is_subset(&allowed_exports) {
        return Err(implementation_bug(
            "projection message exports variables outside the block authorization",
        ));
    }
    for relation in &message.relation_generators {
        if !poly_variables(relation).is_subset(&exported) {
            return Err(implementation_bug(
                "projection message relation contains a non-exported variable",
            ));
        }
    }
    verify_kernel_certificate(message, ctx)?;
    Ok(())
}

fn verify_kernel_certificate(
    message: &ProjectionMessage,
    ctx: &KernelContext,
) -> Result<(), SolverError> {
    let cert = &message.certificate;
    if cert.binding_hash != kernel_certificate_binding_hash(cert) {
        return Err(implementation_bug(
            "kernel certificate binding hash mismatch",
        ));
    }
    if cert.plan_hash == hash_sequence("synthetic-kernel-plan", &[]) {
        return Err(certificate_gap(
            message.package_hash,
            "synthetic test certificate cannot verify a projection message",
        ));
    }
    if cert.certificate_route != expected_route(message.kernel_kind) {
        return Err(implementation_bug(
            "kernel certificate route does not match projection kernel kind",
        ));
    }
    if cert.exported_variables != message.exported_variables {
        return Err(implementation_bug(
            "kernel certificate exported variables do not match message exports",
        ));
    }
    let output_hashes = message
        .relation_generators
        .iter()
        .map(|relation| relation.hash)
        .collect::<Vec<_>>();
    if cert.output_relation_hashes != output_hashes || output_hashes.is_empty() {
        return Err(implementation_bug(
            "kernel certificate output relation hashes do not match message relations",
        ));
    }
    if !source_hashes_are_authorized(cert, message, ctx)? {
        return Err(implementation_bug(
            "kernel certificate source relation hashes do not match authorized source relations",
        ));
    }
    if !payload_sources_are_authorized(&cert.payload, ctx, Some(message.package_hash)) {
        return Err(implementation_bug(
            "certificate payload source polynomials are not authorized by replay context",
        ));
    }
    verify_variant_shape(cert, message)?;
    verify_payload_exact(cert, message, ctx)
}

fn verify_payload_exact(
    cert: &KernelCertificate,
    message: &ProjectionMessage,
    ctx: &KernelContext,
) -> Result<(), SolverError> {
    verify_payload_for_outputs(
        cert,
        &cert.payload,
        cert.certificate_route,
        &message.relation_generators,
        ctx,
        Some(message),
    )
}

fn payload_sources_are_authorized(
    payload: &KernelCertificatePayload,
    ctx: &KernelContext,
    excluded_child_hash: Option<Hash>,
) -> bool {
    let Some(payload_hashes) = payload_source_hashes(payload) else {
        return true;
    };
    let relations_by_id = ctx
        .system
        .relations
        .iter()
        .map(|relation| (relation.id, relation))
        .collect::<BTreeMap<_, _>>();
    let mut authorized = BTreeSet::new();
    for relation_id in &ctx.block.relation_ids {
        if let Some(relation) = relations_by_id.get(relation_id) {
            authorized.insert(relation.hash);
            authorized.insert(relation.polynomial.hash);
        }
    }
    for child in &ctx.child_messages {
        if Some(child.package_hash) == excluded_child_hash {
            continue;
        }
        for relation in &child.relation_generators {
            authorized.insert(relation.hash);
        }
    }
    payload_hashes.iter().all(|hash| authorized.contains(hash))
}

pub(crate) fn payload_source_hashes(payload: &KernelCertificatePayload) -> Option<Vec<Hash>> {
    match payload {
        KernelCertificatePayload::TargetOnlySupport(proof) => Some(
            proof
                .source_relations
                .iter()
                .map(|relation| relation.hash)
                .collect(),
        ),
        KernelCertificatePayload::Membership(proof) => Some(
            proof
                .source_relations
                .iter()
                .map(|relation| relation.hash)
                .collect(),
        ),
        KernelCertificatePayload::GuardedAffine(proof) => Some(
            proof
                .source_relations
                .iter()
                .map(|relation| relation.hash)
                .collect(),
        ),
        KernelCertificatePayload::SparseResultant(proof) => Some(
            proof
                .source_relations
                .iter()
                .map(|relation| relation.hash)
                .collect(),
        ),
        KernelCertificatePayload::TargetAction(proof) => Some(
            proof
                .quotient_input
                .authorized_relations
                .iter()
                .map(|relation| relation.hash)
                .collect(),
        ),
        KernelCertificatePayload::RegularChain(proof) => Some(
            proof
                .source_relations
                .iter()
                .map(|relation| relation.hash)
                .collect(),
        ),
        KernelCertificatePayload::NormTrace(proof) => {
            Some(proof.tower.source_relation_hashes.clone())
        }
        KernelCertificatePayload::SpecializationInterpolation(proof) => Some(
            proof
                .source_relations
                .iter()
                .map(|relation| relation.hash)
                .collect(),
        ),
        KernelCertificatePayload::Universal(proof) => {
            let mut hashes = proof
                .source_relations
                .iter()
                .map(|relation| relation.hash)
                .collect::<Vec<_>>();
            if let Some(inner) = &proof.inner_payload {
                if let Some(inner_hashes) = payload_source_hashes(inner) {
                    hashes.extend(inner_hashes);
                }
            }
            Some(hashes)
        }
        _ => None,
    }
}

fn verify_payload_for_outputs(
    cert: &KernelCertificate,
    payload: &KernelCertificatePayload,
    route: CertificateRoute,
    output_relations: &[SparsePolynomialQ],
    ctx: &KernelContext,
    message: Option<&ProjectionMessage>,
) -> Result<(), SolverError> {
    match (route, payload) {
        (
            CertificateRoute::SourceMembershipCertificate,
            KernelCertificatePayload::TargetOnlySupport(proof),
        ) => {
            if output_relations != std::slice::from_ref(&proof.support_relation) {
                return Err(implementation_bug("target-only support output mismatch"));
            }
            let expected = target_support_from_relations(&proof.source_relations, proof.target)
                .ok_or_else(|| {
                    certificate_gap(
                        proof.support_relation.hash,
                        "target-only support proof has no target-only source relation",
                    )
                })?;
            if expected != proof.support_relation {
                return Err(implementation_bug(
                    "target-only support proof does not recompute output support",
                ));
            }
            Ok(())
        }
        (
            CertificateRoute::DenseRelationSearchMembership,
            KernelCertificatePayload::Membership(proof),
        ) => verify_membership_projection_outputs(proof, output_relations, ctx, cert, message),
        (
            CertificateRoute::GuardedAffineProjectionCertificate,
            KernelCertificatePayload::GuardedAffine(proof),
        ) => {
            if output_relations != proof.output_relations {
                return Err(implementation_bug("guarded affine output mismatch"));
            }
            let recomputed = replay_guarded_affine_outputs(proof, ctx)?;
            if recomputed != proof.output_relations {
                return Err(implementation_bug(
                    "guarded affine payload does not replay to message outputs",
                ));
            }
            Ok(())
        }
        (
            CertificateRoute::SparseResultantExactVerification,
            KernelCertificatePayload::SparseResultant(proof),
        ) => {
            if output_relations != proof.output_relations {
                return Err(implementation_bug("sparse resultant output mismatch"));
            }
            if proof.resultant_certificates.is_empty() || !verify_sparse_resultant_payload(proof) {
                return Err(implementation_bug(
                    "sparse resultant certificate failed exact replay",
                ));
            }
            Ok(())
        }
        (
            CertificateRoute::VerifiedCharacteristicSupportCoverage,
            KernelCertificatePayload::TargetAction(proof),
        ) => verify_target_action_payload(proof, output_relations),
        (
            CertificateRoute::RegularChainGuardedProjection,
            KernelCertificatePayload::RegularChain(proof),
        ) => verify_regular_chain_payload(proof, output_relations, ctx),
        (
            CertificateRoute::NormTraceExactVerification,
            KernelCertificatePayload::NormTrace(proof),
        ) => {
            if output_relations != std::slice::from_ref(&proof.output_relation)
                || !verify_norm_tower_plan_relation(&proof.tower, &proof.output_relation)
            {
                return Err(implementation_bug(
                    "norm-trace payload failed exact tower replay",
                ));
            }
            Ok(())
        }
        (
            CertificateRoute::SpecializationInterpolationExactVerification,
            KernelCertificatePayload::SpecializationInterpolation(proof),
        ) => {
            if output_relations != std::slice::from_ref(&proof.output_relation)
                || !verify_interpolated_relation(
                    &proof.output_relation,
                    &proof.interpolation_certificate,
                )
            {
                return Err(implementation_bug(
                    "specialization-interpolation sample certificate failed replay",
                ));
            }
            validate_local_elimination_result(
                &proof.elimination_result,
                &proof
                    .output_relation
                    .terms
                    .iter()
                    .flat_map(|term| term.monomial.exponents.iter().map(|(var, _)| *var))
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>(),
                &proof.source_relations,
            )
            .map_err(|_| implementation_bug("specialization elimination payload invalid"))?;
            let primitive = clear_denominators_primitive(&proof.output_relation);
            if !proof
                .elimination_result
                .generators
                .iter()
                .any(|generator| clear_denominators_primitive(&generator.generator) == primitive)
            {
                return Err(implementation_bug(
                    "specialization interpolation output absent from exact elimination result",
                ));
            }
            Ok(())
        }
        (
            CertificateRoute::UniversalFixedLocalElimination,
            KernelCertificatePayload::Universal(proof),
        ) => {
            if output_relations != proof.output_relations {
                return Err(implementation_bug("universal output mismatch"));
            }
            verify_universal_strategy_trace(proof, cert, ctx)?;
            if let Some(inner) = &proof.inner_payload {
                verify_payload_for_outputs(
                    cert,
                    inner,
                    route_for_payload(inner).ok_or_else(|| {
                        implementation_bug("universal inner payload route is unknown")
                    })?,
                    output_relations,
                    ctx,
                    message,
                )?;
            }
            if !proof.output_memberships.is_empty() {
                verify_membership_outputs(
                    &proof.source_relations,
                    output_relations,
                    &proof.output_memberships,
                )?;
            }
            if proof.inner_payload.is_none() && proof.output_memberships.is_empty() {
                return Err(certificate_gap(
                    proof.stage_certificate_hash,
                    "universal payload has neither wrapped proof nor membership proof",
                ));
            }
            Ok(())
        }
        (
            _,
            KernelCertificatePayload::BindingOnly | KernelCertificatePayload::SyntheticForTests,
        ) => Err(certificate_gap(
            hash_sequence("p11-missing-payload", &[]),
            "kernel certificate lacks variant-specific proof payload",
        )),
        _ => Err(implementation_bug(
            "kernel certificate payload does not match certificate route",
        )),
    }
}

fn verify_universal_strategy_trace(
    proof: &crate::verify::certificates::UniversalProjectionCertificate,
    cert: &KernelCertificate,
    ctx: &KernelContext,
) -> Result<(), SolverError> {
    let expected = [
        UniversalStrategy::EliminationGroebnerLocal,
        UniversalStrategy::F4EliminationLocal,
        UniversalStrategy::TargetRelationSearchEscalated,
        UniversalStrategy::ResultantIfSquareOrOverdetermined,
        UniversalStrategy::SpecializeProjectInterpolateVerify,
    ];
    if proof.attempted_strategies != expected {
        return Err(implementation_bug(
            "universal attempted strategy sequence is not the fixed generic sequence",
        ));
    }
    if proof.strategy_records.len() != proof.attempted_strategies.len()
        || proof
            .strategy_records
            .iter()
            .zip(&proof.attempted_strategies)
            .any(|(record, strategy)| record.strategy != *strategy)
    {
        return Err(implementation_bug(
            "universal strategy cost records do not match attempted sequence",
        ));
    }
    if !proof.attempted_strategies.contains(&proof.chosen_strategy) {
        return Err(implementation_bug(
            "universal chosen strategy is absent from attempted sequence",
        ));
    }
    let expected_chosen = if let Some(inner) = &proof.inner_payload {
        universal_strategy_for_inner_payload(inner).ok_or_else(|| {
            implementation_bug("universal inner payload has no declared strategy mapping")
        })?
    } else if !proof.output_memberships.is_empty()
        && matches!(
            proof.chosen_strategy,
            UniversalStrategy::EliminationGroebnerLocal | UniversalStrategy::F4EliminationLocal
        )
    {
        proof.chosen_strategy
    } else {
        return Err(certificate_gap(
            proof.stage_certificate_hash,
            "universal strategy trace has no chosen proof payload",
        ));
    };
    if proof.chosen_strategy != expected_chosen {
        return Err(implementation_bug(
            "universal chosen strategy does not match wrapped proof payload",
        ));
    }
    let chosen_index = proof
        .attempted_strategies
        .iter()
        .position(|strategy| *strategy == proof.chosen_strategy)
        .unwrap_or(proof.attempted_strategies.len());
    if proof.failed_strategy_hashes.len() > chosen_index {
        return Err(implementation_bug(
            "universal failed strategy trace exceeds chosen strategy position",
        ));
    }
    verify_universal_source_hash_binding(proof, cert, ctx)?;
    let expected_stage_hashes = expected_universal_stage_hashes(proof, cert)?;
    let skipped_cost_prohibited = proof
        .strategy_records
        .iter()
        .filter(|record| record.cost_class == RouteCostClass::CostProhibited)
        .map(|record| record.stage_hash)
        .collect::<Vec<_>>();
    if proof.skipped_cost_prohibited_strategy_hashes != skipped_cost_prohibited {
        return Err(implementation_bug(
            "universal cost-prohibited skip hashes do not match strategy records",
        ));
    }
    if proof.stage_hash != expected_stage_hashes[chosen_index] {
        return Err(implementation_bug(
            "universal chosen stage hash does not match replayed stage plan",
        ));
    }
    if proof.failed_strategy_hashes != expected_stage_hashes[..chosen_index] {
        return Err(implementation_bug(
            "universal failed strategy hashes do not match replayed attempted stage prefix",
        ));
    }
    let executed_failed = proof
        .strategy_records
        .iter()
        .take(chosen_index)
        .filter(|record| record.enabled && record.cost_class != RouteCostClass::CostProhibited)
        .map(|record| record.stage_hash)
        .collect::<Vec<_>>();
    if proof.executed_failed_strategy_hashes != executed_failed {
        return Err(implementation_bug(
            "universal executed failed strategy hashes do not match replayed enabled failure prefix",
        ));
    }
    Ok(())
}

fn verify_universal_source_hash_binding(
    proof: &crate::verify::certificates::UniversalProjectionCertificate,
    cert: &KernelCertificate,
    ctx: &KernelContext,
) -> Result<(), SolverError> {
    let mut proof_hashes = proof
        .source_relations
        .iter()
        .map(|relation| relation.hash)
        .collect::<Vec<_>>();
    let mut expected_hashes = expected_universal_payload_source_hashes(cert, ctx);
    proof_hashes.sort();
    expected_hashes.sort();
    if proof_hashes != expected_hashes {
        return Err(implementation_bug(
            "universal payload source relations do not exactly match plan-bound certificate sources",
        ));
    }
    Ok(())
}

fn expected_universal_payload_source_hashes(
    cert: &KernelCertificate,
    ctx: &KernelContext,
) -> Vec<Hash> {
    cert.source_relation_hashes
        .iter()
        .map(|source_hash| {
            ctx.system
                .relations
                .iter()
                .find(|relation| relation.hash == *source_hash)
                .map(|relation| relation.polynomial.hash)
                .or_else(|| {
                    ctx.child_messages
                        .iter()
                        .flat_map(|message| message.relation_generators.iter())
                        .find(|relation| relation.hash == *source_hash)
                        .map(|relation| relation.hash)
                })
                .unwrap_or(*source_hash)
        })
        .collect()
}

fn expected_universal_stage_hashes(
    proof: &crate::verify::certificates::UniversalProjectionCertificate,
    cert: &KernelCertificate,
) -> Result<Vec<Hash>, SolverError> {
    proof
        .strategy_records
        .iter()
        .enumerate()
        .map(|(index, record)| {
            let recomputed = hash_sequence(
                "universal-stage-plan",
                &[
                    cert.plan_hash.0.to_vec(),
                    format!("{:?}", record.strategy).into_bytes(),
                    index.to_be_bytes().to_vec(),
                    vec![record.enabled as u8],
                    record
                        .skip_reason
                        .as_deref()
                        .unwrap_or("")
                        .as_bytes()
                        .to_vec(),
                    format!("{:?}", record.cost_class).into_bytes(),
                    record.algebraic_work_estimate_hash.0.to_vec(),
                    record.algebraic_work_estimate_hash.0.to_vec(),
                    record.route_budget_hash.0.to_vec(),
                    record.route_budget_hash.0.to_vec(),
                    record.predicted_work_units.0.to_be_bytes().to_vec(),
                    record.route_budget_max_work_units.0.to_be_bytes().to_vec(),
                    record.route_budget_max_elapsed_steps.to_be_bytes().to_vec(),
                ],
            );
            if record.stage_hash != recomputed {
                return Err(implementation_bug(
                    "universal strategy record stage hash is not reproducible",
                ));
            }
            Ok(record.stage_hash)
        })
        .collect()
}

fn universal_strategy_for_inner_payload(
    payload: &KernelCertificatePayload,
) -> Option<UniversalStrategy> {
    match payload {
        KernelCertificatePayload::Membership(_) => {
            Some(UniversalStrategy::TargetRelationSearchEscalated)
        }
        KernelCertificatePayload::SparseResultant(_) => {
            Some(UniversalStrategy::ResultantIfSquareOrOverdetermined)
        }
        KernelCertificatePayload::SpecializationInterpolation(_) => {
            Some(UniversalStrategy::SpecializeProjectInterpolateVerify)
        }
        _ => None,
    }
}

fn verify_sparse_resultant_payload(
    proof: &crate::verify::certificates::SparseResultantProjectionCertificate,
) -> bool {
    let mut available_hashes = proof
        .source_relations
        .iter()
        .map(|relation| relation.hash)
        .collect::<BTreeSet<_>>();
    let output_hashes = proof
        .output_relations
        .iter()
        .map(|relation| relation.hash)
        .collect::<BTreeSet<_>>();
    let mut generated_hashes = BTreeSet::new();
    for cert in &proof.resultant_certificates {
        if !verify_resultant_certificate(cert)
            || !cert
                .input
                .polynomials
                .iter()
                .all(|poly| available_hashes.contains(&poly.hash))
        {
            return false;
        }
        let Ok(template) = build_sparse_resultant_template(cert.input.clone()) else {
            return false;
        };
        let Ok(resultant) = compute_resultant_relation(&template, ModularOptions::default()) else {
            return false;
        };
        let primitive = clear_denominators_primitive(&resultant.relation);
        available_hashes.insert(cert.relation_hash);
        available_hashes.insert(primitive.hash);
        generated_hashes.insert(cert.relation_hash);
        generated_hashes.insert(primitive.hash);
    }
    output_hashes
        .iter()
        .all(|hash| generated_hashes.contains(hash))
}

fn verify_variant_shape(
    cert: &KernelCertificate,
    message: &ProjectionMessage,
) -> Result<(), SolverError> {
    match cert.certificate_route {
        CertificateRoute::SourceMembershipCertificate => {
            require_representation(
                message,
                &[
                    MessageRepresentation::PrincipalSupport,
                    MessageRepresentation::GeneratorSet,
                ],
            )?;
            require_target_only_outputs(message)
        }
        CertificateRoute::DenseRelationSearchMembership => {
            require_representation(
                message,
                &[
                    MessageRepresentation::PrincipalSupport,
                    MessageRepresentation::GeneratorSet,
                ],
            )?;
            require_candidate_cover_strength(message)
        }
        CertificateRoute::GuardedAffineProjectionCertificate => {
            require_representation(message, &[MessageRepresentation::GeneratorSet])?;
            require_candidate_cover_strength(message)
        }
        CertificateRoute::SparseResultantExactVerification => {
            require_representation(
                message,
                &[
                    MessageRepresentation::SparseResultantMatrix,
                    MessageRepresentation::PrincipalSupport,
                    MessageRepresentation::GeneratorSet,
                ],
            )?;
            require_candidate_cover_strength(message)
        }
        CertificateRoute::VerifiedCharacteristicSupportCoverage => {
            require_representation(
                message,
                &[
                    MessageRepresentation::QuotientAction,
                    MessageRepresentation::PrincipalSupport,
                    MessageRepresentation::GeneratorSet,
                ],
            )?;
            require_candidate_cover_strength(message)
        }
        CertificateRoute::UniversalFixedLocalElimination => {
            require_representation(message, &[MessageRepresentation::GeneratorSet])?;
            require_candidate_cover_strength(message)
        }
        CertificateRoute::RegularChainGuardedProjection => {
            require_representation(
                message,
                &[
                    MessageRepresentation::TriangularChain,
                    MessageRepresentation::GeneratorSet,
                ],
            )?;
            require_candidate_cover_strength(message)
        }
        CertificateRoute::NormTraceExactVerification => {
            require_representation(
                message,
                &[
                    MessageRepresentation::NormTraceTower,
                    MessageRepresentation::GeneratorSet,
                    MessageRepresentation::PrincipalSupport,
                ],
            )?;
            require_candidate_cover_strength(message)
        }
        CertificateRoute::SpecializationInterpolationExactVerification => {
            require_representation(
                message,
                &[
                    MessageRepresentation::SpecializationInterpolation,
                    MessageRepresentation::GeneratorSet,
                    MessageRepresentation::PrincipalSupport,
                ],
            )?;
            require_candidate_cover_strength(message)
        }
    }
}

fn source_hashes_are_authorized(
    cert: &KernelCertificate,
    message: &ProjectionMessage,
    ctx: &KernelContext,
) -> Result<bool, SolverError> {
    let relations = ctx
        .system
        .relations
        .iter()
        .map(|relation| (relation.id, relation.hash))
        .collect::<BTreeMap<_, _>>();
    let authorized_relation_ids = ctx
        .block
        .relation_ids
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let mut authorized = BTreeSet::new();
    for id in &message.source_relation_ids {
        if !authorized_relation_ids.contains(id) {
            return Ok(false);
        }
        if let Some(hash) = relations.get(id).copied() {
            authorized.insert(hash);
        }
    }
    for child in &ctx.child_messages {
        if child.package_hash == message.package_hash {
            continue;
        }
        for relation in &child.relation_generators {
            authorized.insert(relation.hash);
        }
    }
    Ok(cert
        .source_relation_hashes
        .iter()
        .all(|hash| authorized.contains(hash)))
}

fn verify_membership_outputs(
    source_relations: &[SparsePolynomialQ],
    output_relations: &[SparsePolynomialQ],
    memberships: &[crate::algebra::normal_form::MembershipCertificate],
) -> Result<(), SolverError> {
    if output_relations.len() != memberships.len() {
        return Err(implementation_bug(
            "membership certificate count does not match output relation count",
        ));
    }
    for (relation, membership) in output_relations.iter().zip(memberships) {
        if !verify_membership_by_certificate(relation, membership, source_relations) {
            return Err(implementation_bug(
                "membership certificate failed exact Q identity replay",
            ));
        }
    }
    Ok(())
}

fn verify_membership_projection_outputs(
    proof: &MembershipProjectionCertificate,
    output_relations: &[SparsePolynomialQ],
    ctx: &KernelContext,
    cert: &KernelCertificate,
    message: Option<&ProjectionMessage>,
) -> Result<(), SolverError> {
    verify_membership_outputs(
        &proof.source_relations,
        output_relations,
        &proof.output_memberships,
    )?;
    if let Some(target_relation_search) = &proof.target_relation_search {
        verify_target_relation_search_certificate(
            proof,
            target_relation_search,
            output_relations,
            ctx,
            cert,
            message,
        )?;
    }
    Ok(())
}

fn verify_target_relation_search_certificate(
    proof: &MembershipProjectionCertificate,
    target_relation_search: &TargetRelationSearchCertificate,
    output_relations: &[SparsePolynomialQ],
    ctx: &KernelContext,
    cert: &KernelCertificate,
    message: Option<&ProjectionMessage>,
) -> Result<(), SolverError> {
    if output_relations.len() != 1 || proof.output_memberships.len() != 1 {
        return Err(implementation_bug(
            "target relation search certificate must bind one output relation",
        ));
    }
    let relation = &output_relations[0];
    if target_relation_search.relation_hash != relation.hash {
        return Err(implementation_bug(
            "target relation search certificate relation hash mismatch",
        ));
    }
    if let Some(message) = message {
        if target_relation_search.source_relation_ids != message.source_relation_ids {
            return Err(implementation_bug(
                "target relation search source relation ids mismatch",
            ));
        }
    }
    let source_relation_hashes = proof
        .source_relations
        .iter()
        .map(|relation| relation.hash)
        .collect::<Vec<_>>();
    if target_relation_search.source_relation_hashes != source_relation_hashes {
        return Err(implementation_bug(
            "target relation search certificate source hashes mismatch",
        ));
    }
    if target_relation_search.exported_variables_hash
        != target_relation_variable_hash(
            "target-relation-search-exported-variables",
            &cert.exported_variables,
        )
    {
        return Err(implementation_bug(
            "target relation search exported variable hash mismatch",
        ));
    }
    let exported = cert
        .exported_variables
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let eliminated = ctx
        .block
        .local_variables
        .difference(&exported)
        .copied()
        .collect::<Vec<_>>();
    if target_relation_search.eliminated_variables_hash
        != target_relation_variable_hash("target-relation-search-eliminated-variables", &eliminated)
    {
        return Err(implementation_bug(
            "target relation search eliminated variable hash mismatch",
        ));
    }
    let support_hash = target_relation_hash_list(
        "target-relation-search-multiplier-support-hash-list",
        &target_relation_search.multiplier_support_hashes,
    );
    let export_support_hash = target_relation_monomial_support_hash(
        "rgq042-export-support",
        &target_relation_search.export_support,
    );
    if target_relation_search.export_support_hash != export_support_hash {
        return Err(implementation_bug(
            "target relation search export support hash mismatch",
        ));
    }
    let multiplier_support_hashes = target_relation_search
        .multiplier_supports
        .iter()
        .map(|support| target_relation_monomial_support_hash("rgq042-multiplier-support", support))
        .collect::<Vec<_>>();
    if target_relation_search.multiplier_support_hashes != multiplier_support_hashes {
        return Err(implementation_bug(
            "target relation search multiplier support hashes mismatch",
        ));
    }
    if target_relation_search.multiplier_support_hash != support_hash {
        return Err(implementation_bug(
            "target relation search multiplier support hash mismatch",
        ));
    }
    let matrix = build_membership_matrix(
        &proof.source_relations,
        &target_relation_search.export_support,
        &target_relation_search.multiplier_supports,
        &target_relation_search.row_monomials,
    );
    if target_relation_search.membership_matrix_hash != hash_matrix(&matrix) {
        return Err(implementation_bug(
            "target relation search membership matrix hash mismatch",
        ));
    }
    let recomputed_modular = solve_homogeneous_modular(
        MatrixBuilder {
            matrix: matrix.clone(),
        },
        ModularSolvePlan {
            seed: 101,
            max_primes: 4,
            stable_rank_after: 2,
            reconstruction_height_bound: None,
        },
    );
    let primes_used = recomputed_modular
        .traces
        .iter()
        .map(|trace| trace.prime)
        .collect::<Vec<_>>();
    if target_relation_search.primes_used != primes_used {
        return Err(implementation_bug(
            "target relation search modular prime trace mismatch",
        ));
    }
    let multipliers =
        membership_multipliers(proof.source_relations.len(), &proof.output_memberships[0])?;
    let (candidate_relation, candidate_multipliers) = relation_and_multipliers_from_candidate(
        &target_relation_search.accepted_candidate_vector,
        &target_relation_search.export_support,
        &target_relation_search.multiplier_supports,
    )?;
    if candidate_relation != *relation || candidate_multipliers != multipliers {
        return Err(implementation_bug(
            "target relation search accepted candidate vector mismatch",
        ));
    }
    let rational_reconstruction_hash = hash_sequence(
        "target-relation-search-rational-reconstruction",
        &[rational_vector_bytes(
            &target_relation_search.accepted_candidate_vector,
        )],
    );
    if target_relation_search.rational_reconstruction_hash != rational_reconstruction_hash {
        return Err(implementation_bug(
            "target relation search rational reconstruction hash mismatch",
        ));
    }
    if target_relation_search.multipliers_hash != target_relation_multipliers_hash(&multipliers) {
        return Err(implementation_bug(
            "target relation search multipliers hash mismatch",
        ));
    }
    if target_relation_search.exact_identity_hash
        != target_relation_exact_identity_hash(relation, &multipliers, &proof.source_relations)
    {
        return Err(implementation_bug(
            "target relation search exact identity hash mismatch",
        ));
    }
    Ok(())
}

fn relation_and_multipliers_from_candidate(
    vector: &VectorQ,
    export_support: &[Monomial],
    multiplier_supports: &[Vec<Monomial>],
) -> Result<(SparsePolynomialQ, Vec<SparsePolynomialQ>), SolverError> {
    let expected_len = export_support.len()
        + multiplier_supports
            .iter()
            .map(|support| support.len())
            .sum::<usize>();
    if vector.entries.len() != expected_len {
        return Err(implementation_bug(
            "target relation search accepted candidate vector has wrong width",
        ));
    }
    let relation = polynomial_from_support_coefficients(
        export_support,
        &vector.entries[..export_support.len()],
    );
    let mut offset = export_support.len();
    let mut multipliers = Vec::new();
    for support in multiplier_supports {
        let end = offset + support.len();
        multipliers.push(polynomial_from_support_coefficients(
            support,
            &vector.entries[offset..end],
        ));
        offset = end;
    }
    Ok((relation, multipliers))
}

fn polynomial_from_support_coefficients(
    support: &[Monomial],
    coeffs: &[RationalQ],
) -> SparsePolynomialQ {
    normalize_poly(SparsePolynomialQ {
        terms: support
            .iter()
            .zip(coeffs.iter())
            .filter(|(_, coeff)| !is_zero_q(coeff))
            .map(|(monomial, coeff)| TermQ {
                coeff: coeff.clone(),
                monomial: monomial.clone(),
            })
            .collect(),
        hash: hash_sequence("poly", &[]),
    })
}

fn rational_vector_bytes(vector: &VectorQ) -> Vec<u8> {
    vector
        .entries
        .iter()
        .flat_map(crate::types::rational::rational_to_bytes)
        .collect()
}

fn membership_multipliers(
    source_count: usize,
    membership: &crate::algebra::normal_form::MembershipCertificate,
) -> Result<Vec<SparsePolynomialQ>, SolverError> {
    let mut multipliers = vec![zero_poly(); source_count];
    for term in &membership.combination_terms {
        let Some(slot) = multipliers.get_mut(term.relation_id) else {
            return Err(implementation_bug(
                "target relation search membership references missing source relation",
            ));
        };
        *slot = poly_add(slot, &term.multiplier);
    }
    Ok(multipliers)
}

fn verify_target_action_payload(
    proof: &crate::verify::certificates::TargetActionProjectionCertificate,
    output_relations: &[SparsePolynomialQ],
) -> Result<(), SolverError> {
    if output_relations != std::slice::from_ref(&proof.output_relation) {
        return Err(implementation_bug("target action output mismatch"));
    }
    let handle = build_production_target_relevant_quotient_handle(proof.quotient_input.clone())
        .map_err(|_| {
            implementation_bug("target action quotient certificate failed exact reconstruction")
        })?;
    if !handle.no_coordinate_solution_export()
        || proof.coverage.characteristic_polynomial.variable != proof.target
    {
        return Err(implementation_bug(
            "target action quotient metadata violates no-coordinate/export binding",
        ));
    }
    let start_vectors = (0..handle.basis_size())
        .map(|idx| unit_vector(handle.basis_size(), idx))
        .collect::<Vec<_>>();
    let sequence = block_krylov_sequence(
        &handle,
        proof.target,
        KrylovPlan {
            start_vectors,
            max_steps: handle.basis_size().saturating_add(1),
        },
    )
    .map_err(|_| implementation_bug("target action Krylov sequence replay failed"))?;
    let recurrence = recover_recurrence(&sequence)
        .map_err(|_| implementation_bug("target action recurrence replay failed"))?;
    let coverage = certify_krylov_coverage(&sequence, &recurrence, &handle)
        .map_err(|_| implementation_bug("target action coverage replay failed"))?;
    let annihilator = verify_annihilator(&handle, &coverage.characteristic_polynomial)
        .map_err(|_| implementation_bug("target action annihilator replay failed"))?;
    if coverage != proof.coverage
        || annihilator != proof.annihilator
        || proof.annihilator.polynomial_hash != coverage.characteristic_polynomial_hash
        || proof.output_relation != univariate_to_sparse(&coverage.characteristic_polynomial)
    {
        return Err(implementation_bug(
            "target action coverage certificate failed exact replay",
        ));
    }
    Ok(())
}

fn verify_regular_chain_payload(
    proof: &crate::verify::certificates::RegularChainProjectionCertificate,
    output_relations: &[SparsePolynomialQ],
    ctx: &KernelContext,
) -> Result<(), SolverError> {
    if output_relations != proof.output_relations {
        return Err(implementation_bug("regular chain output mismatch"));
    }
    let authorized_guards = ctx
        .system
        .guards
        .iter()
        .map(|guard| guard.factor.hash)
        .collect::<BTreeSet<_>>();
    if proof
        .guards
        .iter()
        .any(|guard| !authorized_guards.contains(&guard.hash))
    {
        return Err(implementation_bug(
            "regular chain guard payload is not authorized by replay context",
        ));
    }
    let dag = local_regular_chain_decomposition(RegularChainInput {
        relations: proof.source_relations.clone(),
        variables: proof.variables.clone(),
        guards: proof.guards.clone(),
        semantics: proof.dag.semantics,
    })
    .map_err(|_| implementation_bug("regular chain DAG replay failed"))?;
    if !verify_regular_chain_dag_evidence(&proof.dag) {
        return Err(implementation_bug(
            "regular chain regularity or guard evidence failed replay",
        ));
    }
    if dag != proof.dag {
        return Err(implementation_bug("regular chain DAG hash replay mismatch"));
    }
    let projections = dag
        .chains
        .iter()
        .map(|chain| project_chain_to_variables(chain, &proof.exported_variables))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| implementation_bug("regular chain projection replay failed"))?;
    if projections != proof.projections {
        return Err(implementation_bug(
            "regular chain projection certificates do not recompute from DAG",
        ));
    }
    let recomputed = combine_chain_projections(&projections, dag.semantics)
        .map_err(|_| implementation_bug("regular chain projection combination replay failed"))?;
    let recomputed = normalize_regular_chain_output_relations(recomputed);
    if recomputed != proof.output_relations {
        return Err(implementation_bug(
            "regular chain payload does not replay to message outputs",
        ));
    }
    Ok(())
}

fn normalize_regular_chain_output_relations(
    relations: Vec<SparsePolynomialQ>,
) -> Vec<SparsePolynomialQ> {
    let mut normalized = relations
        .into_iter()
        .filter(|relation| !relation.terms.is_empty())
        .map(|relation| clear_denominators_primitive(&relation))
        .collect::<Vec<_>>();
    if normalized.is_empty() {
        normalized.push(zero_poly());
    }
    normalized
}

fn target_support_from_relations(
    relations: &[SparsePolynomialQ],
    target: VariableId,
) -> Option<SparsePolynomialQ> {
    target_only_support_from_polynomials(relations, target)
}

fn univariate_to_sparse(poly: &UniPolynomialQ) -> SparsePolynomialQ {
    let mut out = crate::types::polynomial::zero_poly();
    let variable = crate::types::polynomial::variable_poly(poly.variable);
    let mut power = constant_poly(int_q(1));
    for coeff in &poly.coeffs_low_to_high {
        if !is_zero_q(coeff) {
            out = poly_add(&out, &poly_scale(&power, coeff));
        }
        power = poly_mul(&power, &variable);
    }
    out
}

fn replay_guarded_affine_outputs(
    proof: &crate::verify::certificates::GuardedAffineProjectionCertificate,
    ctx: &KernelContext,
) -> Result<Vec<SparsePolynomialQ>, SolverError> {
    if proof.source_relation_ids.len() != proof.source_relations.len() {
        return Err(implementation_bug(
            "guarded affine payload source id/relation count mismatch",
        ));
    }
    let mut relations = proof
        .source_relation_ids
        .iter()
        .copied()
        .zip(proof.source_relations.iter().cloned())
        .collect::<Vec<_>>();
    for step in &proof.steps {
        let Some((_, source)) = relations
            .iter()
            .find(|(id, _)| *id == step.source_relation_id)
            .cloned()
        else {
            return Err(implementation_bug(
                "guarded affine step source relation missing from payload",
            ));
        };
        let Some((pivot, rest)) = affine_parts_in_variable(&source, step.eliminated_variable)
        else {
            return Err(implementation_bug(
                "guarded affine step source is not affine in eliminated variable",
            ));
        };
        if pivot.hash != step.pivot_hash {
            return Err(implementation_bug("guarded affine pivot hash mismatch"));
        }
        let transformed = if let Some(constant) = constant_value(&pivot) {
            let scale = div_q(&neg_q(&int_q(1)), &constant)
                .map_err(|_| implementation_bug("zero affine pivot"))?;
            let expression = poly_scale(&rest, &scale);
            let mut subst = SubstitutionMap::new();
            subst.insert(step.eliminated_variable, expression);
            relations
                .into_iter()
                .filter(|(id, _)| *id != step.source_relation_id)
                .filter_map(|(id, relation)| {
                    let poly = clear_denominators_primitive(&substitute_poly(&relation, &subst));
                    (!poly.terms.is_empty()).then_some((id, poly))
                })
                .collect::<Vec<_>>()
        } else {
            let Some(denominator_guard_hash) = step.denominator_guard_hash else {
                return Err(implementation_bug(
                    "guarded affine nonconstant pivot lacks guard hash",
                ));
            };
            let guard_matches = ctx.system.guards.iter().any(|guard| {
                guard.guard_hash == denominator_guard_hash && guard.factor.hash == pivot.hash
            });
            if !guard_matches {
                return Err(implementation_bug(
                    "guarded affine denominator guard is not authorized for pivot",
                ));
            }
            let numerator = poly_scale(&rest, &int_q(-1));
            relations
                .into_iter()
                .filter(|(id, _)| *id != step.source_relation_id)
                .filter_map(|(id, relation)| {
                    let poly = clear_denominators_primitive(&substitute_rational_and_clear(
                        &relation,
                        step.eliminated_variable,
                        &numerator,
                        &pivot,
                    ));
                    (!poly.terms.is_empty()).then_some((id, poly))
                })
                .collect::<Vec<_>>()
        };
        relations = transformed;
    }
    Ok(relations
        .into_iter()
        .map(|(_, relation)| clear_denominators_primitive(&relation))
        .filter(|relation| !relation.terms.is_empty())
        .collect())
}

fn constant_value(poly: &SparsePolynomialQ) -> Option<RationalQ> {
    if poly.terms.len() != 1 || !poly.terms[0].monomial.exponents.is_empty() {
        return None;
    }
    Some(poly.terms[0].coeff.clone())
}

fn route_for_payload(payload: &KernelCertificatePayload) -> Option<CertificateRoute> {
    match payload {
        KernelCertificatePayload::TargetOnlySupport(_) => {
            Some(CertificateRoute::SourceMembershipCertificate)
        }
        KernelCertificatePayload::Membership(_) => {
            Some(CertificateRoute::DenseRelationSearchMembership)
        }
        KernelCertificatePayload::GuardedAffine(_) => {
            Some(CertificateRoute::GuardedAffineProjectionCertificate)
        }
        KernelCertificatePayload::SparseResultant(_) => {
            Some(CertificateRoute::SparseResultantExactVerification)
        }
        KernelCertificatePayload::TargetAction(_) => {
            Some(CertificateRoute::VerifiedCharacteristicSupportCoverage)
        }
        KernelCertificatePayload::RegularChain(_) => {
            Some(CertificateRoute::RegularChainGuardedProjection)
        }
        KernelCertificatePayload::NormTrace(_) => {
            Some(CertificateRoute::NormTraceExactVerification)
        }
        KernelCertificatePayload::SpecializationInterpolation(_) => {
            Some(CertificateRoute::SpecializationInterpolationExactVerification)
        }
        KernelCertificatePayload::Universal(_) => {
            Some(CertificateRoute::UniversalFixedLocalElimination)
        }
        KernelCertificatePayload::BindingOnly | KernelCertificatePayload::SyntheticForTests => None,
    }
}

fn expected_route(kind: KernelKind) -> CertificateRoute {
    match kind {
        KernelKind::TargetUnivariate => CertificateRoute::SourceMembershipCertificate,
        KernelKind::LinearAffine => CertificateRoute::GuardedAffineProjectionCertificate,
        KernelKind::TargetRelationSearch => CertificateRoute::DenseRelationSearchMembership,
        KernelKind::SparseResultantProjection => CertificateRoute::SparseResultantExactVerification,
        KernelKind::TargetActionKrylov => CertificateRoute::VerifiedCharacteristicSupportCoverage,
        KernelKind::UniversalTargetElimination => CertificateRoute::UniversalFixedLocalElimination,
        KernelKind::RegularChainProjection => CertificateRoute::RegularChainGuardedProjection,
        KernelKind::NormTraceProjection => CertificateRoute::NormTraceExactVerification,
        KernelKind::SpecializationInterpolation => {
            CertificateRoute::SpecializationInterpolationExactVerification
        }
    }
}

fn require_representation(
    message: &ProjectionMessage,
    allowed: &[MessageRepresentation],
) -> Result<(), SolverError> {
    if allowed.contains(&message.representation) {
        Ok(())
    } else {
        Err(implementation_bug(
            "projection message representation does not match certificate route",
        ))
    }
}

fn require_candidate_cover_strength(message: &ProjectionMessage) -> Result<(), SolverError> {
    if matches!(
        message.projection_strength,
        ProjectionStrength::CandidateCoverStrong | ProjectionStrength::ExactProjectionIdeal
    ) {
        Ok(())
    } else {
        Err(certificate_gap(
            message.package_hash,
            "weak projection message cannot verify as a candidate-cover certificate",
        ))
    }
}

fn require_target_only_outputs(message: &ProjectionMessage) -> Result<(), SolverError> {
    let exported = message
        .exported_variables
        .iter()
        .copied()
        .collect::<BTreeSet<VariableId>>();
    if exported.len() != 1 {
        return Err(certificate_gap(
            message.package_hash,
            "source membership support requires a single exported target variable",
        ));
    }
    Ok(())
}

fn certificate_gap(hash: Hash, missing: &str) -> SolverError {
    SolverError {
        target: None,
        kind: SolverErrorKind::Failure(FailureKind::CertificateDesignGap {
            constructed_object_hash: hash,
            missing_certificate_kind: missing.to_owned(),
        }),
    }
}

fn implementation_bug(message: &str) -> SolverError {
    SolverError {
        target: None,
        kind: SolverErrorKind::Failure(FailureKind::ImplementationBug {
            invariant_violated: message.to_owned(),
        }),
    }
}
