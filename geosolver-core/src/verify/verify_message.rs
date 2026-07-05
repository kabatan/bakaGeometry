use std::collections::{BTreeMap, BTreeSet};

use crate::algebra::elimination::validate_local_elimination_result;
use crate::algebra::interpolation::verify_interpolated_relation;
use crate::algebra::krylov::{
    block_krylov_sequence, certify_krylov_coverage, recover_recurrence, verify_annihilator,
    KrylovPlan,
};
use crate::algebra::norm_trace::verify_norm_tower_plan_relation;
use crate::algebra::normal_form::verify_membership_by_certificate;
use crate::algebra::quotient::{
    build_production_target_relevant_quotient_handle, unit_vector, TargetQuotientHandle,
};
use crate::algebra::regular_chain::{
    combine_chain_projections, local_regular_chain_decomposition, project_chain_to_variables,
    RegularChainInput,
};
use crate::algebra::resultant::{
    build_sparse_resultant_template, compute_resultant_relation, verify_resultant_certificate,
    ModularOptions,
};
use crate::compose::message::{hash_projection_message, ProjectionMessage};
use crate::compose::message::{MessageRepresentation, ProjectionStrength};
use crate::kernels::target_univariate::target_only_support_from_polynomials;
use crate::kernels::traits::{KernelContext, KernelKind};
use crate::planner::kernel_plan::CertificateRoute;
use crate::preprocess::compression::{affine_parts_in_variable, substitute_rational_and_clear};
use crate::result::status::{FailureKind, SolverError, SolverErrorKind};
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::VariableId;
use crate::types::polynomial::{
    clear_denominators_primitive, constant_poly, poly_add, poly_mul, poly_scale, poly_variables,
    substitute_poly, SparsePolynomialQ, SubstitutionMap,
};
use crate::types::rational::{div_q, int_q, is_zero_q, neg_q, RationalQ};
use crate::types::univariate::UniPolynomialQ;
use crate::verify::certificates::{
    kernel_certificate_binding_hash, KernelCertificate, KernelCertificatePayload,
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
        &cert.payload,
        cert.certificate_route,
        &message.relation_generators,
        ctx,
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
    let mut authorized = ctx
        .system
        .relations
        .iter()
        .map(|relation| relation.polynomial.hash)
        .collect::<BTreeSet<_>>();
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
    payload: &KernelCertificatePayload,
    route: CertificateRoute,
    output_relations: &[SparsePolynomialQ],
    ctx: &KernelContext,
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
        ) => verify_membership_outputs(
            &proof.source_relations,
            output_relations,
            &proof.output_memberships,
        ),
        (
            CertificateRoute::GuardedAffineProjectionCertificate,
            KernelCertificatePayload::GuardedAffine(proof),
        ) => {
            if output_relations != proof.output_relations {
                return Err(implementation_bug("guarded affine output mismatch"));
            }
            let recomputed = replay_guarded_affine_outputs(proof)?;
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
            if let Some(inner) = &proof.inner_payload {
                verify_payload_for_outputs(
                    inner,
                    route_for_payload(inner).ok_or_else(|| {
                        implementation_bug("universal inner payload route is unknown")
                    })?,
                    output_relations,
                    ctx,
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
    let mut authorized = BTreeSet::new();
    for id in &message.source_relation_ids {
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
    let recomputed = recomputed
        .into_iter()
        .filter(|relation| !relation.terms.is_empty())
        .map(|relation| clear_denominators_primitive(&relation))
        .collect::<Vec<_>>();
    if recomputed != proof.output_relations {
        return Err(implementation_bug(
            "regular chain payload does not replay to message outputs",
        ));
    }
    Ok(())
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
    clear_denominators_primitive(&out)
}

fn replay_guarded_affine_outputs(
    proof: &crate::verify::certificates::GuardedAffineProjectionCertificate,
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
            if step.denominator_guard_hash.is_none() {
                return Err(implementation_bug(
                    "guarded affine nonconstant pivot lacks guard hash",
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
        ProjectionStrength::CandidateCoverStrong
            | ProjectionStrength::ExactProjectionIdeal
            | ProjectionStrength::ExactRealFiberAware
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
