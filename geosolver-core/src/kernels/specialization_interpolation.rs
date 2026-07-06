use std::collections::BTreeSet;

use crate::algebra::elimination::{
    eliminate_to_keep_variables, validate_local_elimination_result, EliminationStrategy,
    LocalEliminationResult,
};
use crate::algebra::groebner::GroebnerOptions;
use crate::algebra::interpolation::{
    build_interpolation_certificate, build_multiseparator_coefficient_support,
    choose_multiseparator_specialization_points, interpolate_sparse_coefficients_with_support,
    specialize_polynomials, verify_interpolated_relation, InterpolationCertificate,
    SpecializationPoint, SpecializedRelation,
};
use crate::compose::message::{MessageRepresentation, ProjectionMessage, ProjectionStrength};
use crate::graph::projection_dag::authorize_block_relations;
use crate::graph::projection_dag::ProjectionBlock;
use crate::kernels::target_relation_search::{
    admit_target_relation_search, execute_target_relation_search,
};
use crate::kernels::traits::{KernelContext, KernelKind, ReplayResult, TargetProjectionKernel};
use crate::planner::admission::{KernelAdmission, KernelAdmissionStatus};
use crate::planner::kernel_plan::{
    hash_kernel_execution_plan, planned_failure_behavior, rank_plan, resource_bounds_hash,
    support_plan_hash, template_plan, CertificateRoute, KernelExecutionPlan, KernelSupportPlan,
    LocalNonfinitePolicy, ResourceBounds,
};
use crate::preprocess::compression::CompressedSystemQ;
use crate::problem::canonicalize::{canonicalize_relation, CanonicalRelationQ};
use crate::problem::context::SolverContext;
use crate::result::cost_trace::ProjectionCostTrace;
use crate::result::status::{
    AlgebraicReason, FailureKind, SolverError, SolverErrorKind, SolverStatus, StageId,
};
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::{BlockId, PackageId, RelationId, VariableId};
use crate::types::matrix::{matrix_density, SparseMatrixQ};
use crate::types::monomial::monomial_to_bytes;
use crate::types::polynomial::{
    clear_denominators_primitive, max_poly_coefficient_height_bits, poly_coefficient_height_bits,
    poly_monomial_count, poly_total_degree, poly_variables, SparsePolynomialQ,
};
use crate::types::rational::rational_to_bytes;
use crate::verify::certificates::{
    KernelCertificate, KernelCertificatePayload, SpecializationInterpolationProjectionCertificate,
};

pub struct SpecializationInterpolationKernel;

impl TargetProjectionKernel for SpecializationInterpolationKernel {
    fn kind(&self) -> KernelKind {
        KernelKind::SpecializationInterpolation
    }

    fn admit(&self, block: &ProjectionBlock, ctx: &KernelContext) -> KernelAdmission {
        admit_specialization_interpolation(block, ctx)
    }

    fn plan(
        &self,
        admission: &KernelAdmission,
        ctx: &KernelContext,
        solver_ctx: &SolverContext,
    ) -> Result<KernelExecutionPlan, SolverError> {
        plan_specialization_interpolation_from_admission(admission, ctx, solver_ctx)
    }

    fn execute(
        &self,
        plan: &KernelExecutionPlan,
        ctx: &mut KernelContext,
        solver_ctx: &mut SolverContext,
    ) -> Result<ProjectionMessage, SolverError> {
        execute_specialization_interpolation(plan, ctx, solver_ctx)
    }

    fn replay(&self, message: &ProjectionMessage, ctx: &KernelContext) -> ReplayResult {
        crate::kernels::traits::exact_replay_result(
            self.kind(),
            "specialization-interpolation-replay",
            message,
            ctx,
        )
    }
}

#[derive(Debug, Clone)]
struct SpecializationInterpolationTrace {
    relation: SparsePolynomialQ,
    elimination_result: LocalEliminationResult,
    samples: Vec<SpecializedRelation>,
    interpolation_certificate: InterpolationCertificate,
    matrix_rows: usize,
    matrix_cols: usize,
    degree_bound: usize,
    sample_hash: Hash,
    support_hash: Hash,
    trace_hash: Hash,
}

#[derive(Debug, Clone)]
struct SpecializationInterpolationPlanProbe {
    matrix_rows: usize,
    matrix_cols: usize,
    degree_bound: usize,
    sample_hash: Hash,
    support_hash: Hash,
}

#[derive(Debug, Clone)]
struct InterpolationRelationInput {
    polynomial: SparsePolynomialQ,
    source_relation_ids: Vec<RelationId>,
    source_hash: Hash,
    child_message_hash: Option<Hash>,
}

pub fn admit_specialization_interpolation(
    block: &ProjectionBlock,
    ctx: &KernelContext,
) -> KernelAdmission {
    let exported = sorted_set(&block.exported_variables);
    let eliminated = block
        .local_variables
        .difference(&block.exported_variables)
        .copied()
        .collect::<Vec<_>>();
    let inputs = collect_relation_inputs(block, &ctx.system, &ctx.child_messages);
    let relation_count = inputs.len();
    let has_separator = exported.iter().any(|var| *var != ctx.system.target);
    let has_target_relation = inputs
        .iter()
        .any(|input| poly_variables(&input.polynomial).contains(&ctx.system.target));
    let status = if relation_count >= 2
        && has_separator
        && has_target_relation
        && !eliminated.is_empty()
    {
        KernelAdmissionStatus::Admitted
    } else {
        KernelAdmissionStatus::Declined {
            reason: "specialization-interpolation requires local target-bearing relations and a non-target exported separator".to_owned(),
        }
    };
    finish_admission(block, status, exported, eliminated, None)
}

pub fn plan_specialization_interpolation(
    block: &ProjectionBlock,
    system: &CompressedSystemQ,
    solver_ctx: &SolverContext,
    plan_id: crate::types::ids::KernelPlanId,
) -> Result<KernelExecutionPlan, SolverError> {
    plan_specialization_interpolation_with_messages(block, system, &[], solver_ctx, plan_id)
}

pub fn plan_specialization_interpolation_with_messages(
    block: &ProjectionBlock,
    system: &CompressedSystemQ,
    child_messages: &[ProjectionMessage],
    solver_ctx: &SolverContext,
    plan_id: crate::types::ids::KernelPlanId,
) -> Result<KernelExecutionPlan, SolverError> {
    let inputs = collect_relation_inputs(block, system, child_messages);
    let relation_polys = inputs
        .iter()
        .map(|input| input.polynomial.clone())
        .collect::<Vec<_>>();
    if !relation_polys
        .iter()
        .any(|relation| poly_variables(relation).contains(&system.target))
    {
        return Err(algorithmic_hard_case(
            "specialization-interpolation requires at least one target-bearing relation",
        ));
    }
    let exported = sorted_set(&block.exported_variables);
    let eliminated = block
        .local_variables
        .difference(&block.exported_variables)
        .copied()
        .collect::<Vec<_>>();
    let probe = probe_specialization_interpolation_plan(
        &relation_polys,
        &eliminated,
        &exported,
        system.target,
    )?;
    let template = template_plan(
        probe.matrix_rows,
        probe.matrix_cols,
        probe.sample_hash,
        probe.support_hash,
    );
    let mut support_plan = KernelSupportPlan {
        dense_relation_search_schedule: None,
        affine_elimination_order: None,
        template_plan: Some(template),
        rank_plan: Some(rank_plan(probe.matrix_cols)),
        universal_strategy_sequence: Vec::new(),
        degree_bound: probe.degree_bound,
        support_hash: hash_sequence("kernel-support-plan", &[]),
    };
    support_plan.support_hash = support_plan_hash(&support_plan);
    let mut bounds = ResourceBounds {
        max_matrix_rows: solver_ctx
            .options
            .max_matrix_rows
            .or(Some(probe.matrix_rows)),
        max_matrix_cols: solver_ctx
            .options
            .max_matrix_cols
            .or(Some(probe.matrix_cols)),
        max_export_degree: Some(probe.degree_bound),
        max_multiplier_total_degree: None,
        max_local_elimination_steps: Some(eliminated.len().max(1)),
        max_memory_bytes: solver_ctx.options.max_memory_bytes,
        bounds_hash: hash_sequence("planner-resource-bounds", &[]),
    };
    bounds.bounds_hash = resource_bounds_hash(&bounds);
    Ok(KernelExecutionPlan::new(
        plan_id,
        block.block_id,
        KernelKind::SpecializationInterpolation,
        block.authorization_hash,
        inputs
            .iter()
            .flat_map(|input| input.source_relation_ids.iter().copied())
            .collect(),
        inputs.iter().map(|input| input.source_hash).collect(),
        block.child_block_ids.clone(),
        dedup_hashes_in_order(
            inputs
                .iter()
                .filter_map(|input| input.child_message_hash)
                .collect(),
        ),
        exported,
        eliminated,
        support_plan,
        bounds,
        CertificateRoute::SpecializationInterpolationExactVerification,
        planned_failure_behavior(
            vec![
                SolverStatus::AlgorithmicHardCase,
                SolverStatus::FiniteResourceFailure,
                SolverStatus::CertificateDesignGap,
            ],
            LocalNonfinitePolicy::NotApplicable,
        ),
    ))
}

pub fn plan_specialization_interpolation_from_admission(
    admission: &KernelAdmission,
    ctx: &KernelContext,
    solver_ctx: &SolverContext,
) -> Result<KernelExecutionPlan, SolverError> {
    if admission.kind != KernelKind::SpecializationInterpolation
        || !matches!(admission.status, KernelAdmissionStatus::Admitted)
    {
        return Err(implementation_bug(
            "specialization-interpolation plan requested for non-admitted kernel",
        ));
    }
    if let Some(plan) = &admission.execution_plan {
        return Ok(plan.clone());
    }
    plan_specialization_interpolation_with_messages(
        &ctx.block,
        &ctx.system,
        &ctx.child_messages,
        solver_ctx,
        crate::types::ids::KernelPlanId(7),
    )
}

pub fn execute_specialization_interpolation(
    plan: &KernelExecutionPlan,
    ctx: &mut KernelContext,
    solver_ctx: &mut SolverContext,
) -> Result<ProjectionMessage, SolverError> {
    validate_specialization_plan_binding(plan, ctx)?;
    let inputs = planned_relation_inputs(plan, ctx)?;
    let relation_polys = inputs
        .iter()
        .map(|input| input.polynomial.clone())
        .collect::<Vec<_>>();
    let trace = build_specialization_interpolation_trace(
        &relation_polys,
        &plan.eliminated_variables,
        &plan.exported_variables,
        ctx.system.target,
        solver_ctx,
    )?;
    let Some(template) = &plan.support_plan.template_plan else {
        return Err(implementation_bug(
            "specialization-interpolation plan lacks template plan",
        ));
    };
    let probe = probe_specialization_interpolation_plan(
        &relation_polys,
        &plan.eliminated_variables,
        &plan.exported_variables,
        ctx.system.target,
    )?;
    if template.matrix_rows != trace.matrix_rows
        || template.matrix_cols != trace.matrix_cols
        || template.row_monomial_hash != probe.sample_hash
        || template.column_support_hash != probe.support_hash
        || trace.support_hash != probe.support_hash
        || support_plan_hash(&plan.support_plan) != plan.support_plan.support_hash
    {
        return Err(implementation_bug(
            "specialization-interpolation sample/support plan is not reproducible",
        ));
    }
    if trace.degree_bound > plan.support_plan.degree_bound {
        return Err(implementation_bug(
            "specialization-interpolation execution exceeded planned degree bound",
        ));
    }
    let exported = plan
        .exported_variables
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    if !poly_variables(&trace.relation).is_subset(&exported) {
        return Err(implementation_bug(
            "interpolated relation contains a non-exported variable",
        ));
    }
    if !verify_interpolated_relation_by_elimination(
        &trace.relation,
        &trace.elimination_result,
        &relation_polys,
        &plan.exported_variables,
    )? {
        return Err(implementation_bug(
            "interpolated relation failed exact Q elimination verification",
        ));
    }
    let certificate_hash = specialization_interpolation_certificate_hash(plan, &trace);
    let cost_trace = ProjectionCostTrace {
        block_id: plan.block_id,
        kernel_kind: KernelKind::SpecializationInterpolation,
        local_variable_count: ctx.block.local_variables.len(),
        exported_variable_count: plan.exported_variables.len(),
        local_relation_count: inputs.len(),
        local_monomial_count: relation_polys.iter().map(poly_monomial_count).sum(),
        estimated_quotient_rank: Some(trace.matrix_cols),
        matrix_rows: Some(trace.matrix_rows),
        matrix_cols: Some(trace.matrix_cols),
        matrix_density: Some(matrix_density(&SparseMatrixQ {
            rows: trace.matrix_rows.max(1),
            cols: trace.matrix_cols.max(1),
            entries: Vec::new(),
        })),
        coefficient_height_before_bits: max_poly_coefficient_height_bits(&relation_polys),
        coefficient_height_after_bits: poly_coefficient_height_bits(&trace.relation),
        route_cost: Some(ProjectionCostTrace::route_cost_from_plan(plan)),
    };
    let certificate = KernelCertificate::from_execution_plan_with_payload(
        plan,
        std::slice::from_ref(&trace.relation),
        certificate_hash,
        KernelCertificatePayload::SpecializationInterpolation(
            SpecializationInterpolationProjectionCertificate {
                source_relations: relation_polys.clone(),
                output_relation: trace.relation.clone(),
                interpolation_certificate: trace.interpolation_certificate.clone(),
                elimination_result: trace.elimination_result.clone(),
            },
        ),
    );
    let mut message = ProjectionMessage {
        package_id: PackageId(plan.plan_id.0),
        block_id: plan.block_id,
        kernel_kind: KernelKind::SpecializationInterpolation,
        source_relation_ids: plan.source_relation_ids.clone(),
        eliminated_variables: plan.eliminated_variables.clone(),
        exported_variables: plan.exported_variables.clone(),
        relation_generators: vec![trace.relation],
        representation: MessageRepresentation::SpecializationInterpolation,
        projection_strength: ProjectionStrength::CandidateCoverStrong,
        certificate,
        compression_trace: ctx.system.compression_trace.clone(),
        cost_trace,
        package_hash: hash_sequence("projection-message-initial", &[]),
    };
    message.package_hash = projection_message_hash(&message);
    Ok(message)
}

pub fn verify_interpolated_relation_by_elimination(
    relation: &SparsePolynomialQ,
    elimination_result: &LocalEliminationResult,
    source_relations: &[SparsePolynomialQ],
    exported_variables: &[VariableId],
) -> Result<bool, SolverError> {
    validate_local_elimination_result(elimination_result, exported_variables, source_relations)?;
    let primitive = clear_denominators_primitive(relation);
    Ok(elimination_result
        .generators
        .iter()
        .any(|generator| clear_denominators_primitive(&generator.generator) == primitive))
}

fn build_specialization_interpolation_trace(
    relations: &[SparsePolynomialQ],
    eliminated: &[VariableId],
    exported: &[VariableId],
    target: VariableId,
    solver_ctx: &mut SolverContext,
) -> Result<SpecializationInterpolationTrace, SolverError> {
    let separators = exported
        .iter()
        .copied()
        .filter(|var| *var != target)
        .collect::<Vec<_>>();
    if separators.is_empty() {
        return Err(algorithmic_hard_case(
            "specialization-interpolation requires a non-target exported separator",
        ));
    }
    let degree_bound = specialization_interpolation_degree_bound(relations, eliminated, exported);
    let coefficient_support = build_multiseparator_coefficient_support(&separators, degree_bound);
    let points =
        choose_multiseparator_specialization_points(&separators, &coefficient_support, 101);
    let mut samples = Vec::new();
    for point in &points {
        let relation =
            execute_inner_target_only_kernel(relations, eliminated, target, point, solver_ctx)?;
        samples.push(SpecializedRelation {
            point: point.clone(),
            relation,
        });
    }
    let relation = interpolate_sparse_coefficients_with_support(
        &samples,
        &separators,
        &coefficient_support,
        solver_ctx.options.max_coefficient_height_bits,
    )?;
    let interpolation_certificate =
        build_interpolation_certificate(&relation, samples.clone(), separators.clone());
    if !verify_interpolated_relation(&relation, &interpolation_certificate) {
        return Err(implementation_bug(
            "interpolation certificate failed exact sample replay",
        ));
    }
    let elimination_result = eliminate_to_keep_variables(
        relations,
        eliminated,
        exported,
        EliminationStrategy::LocalGroebner(GroebnerOptions::default()),
        solver_ctx,
    )?;
    if !verify_interpolated_relation_by_elimination(
        &relation,
        &elimination_result,
        relations,
        exported,
    )? {
        return Err(implementation_bug(
            "interpolation candidate was not verified by exact elimination",
        ));
    }
    let sample_hash = hash_specialization_samples(&samples);
    let support_hash = hash_sequence(
        "specialization-interpolation-coefficient-support",
        &coefficient_support
            .iter()
            .map(monomial_to_bytes)
            .collect::<Vec<_>>(),
    );
    let trace_hash = hash_sequence(
        "specialization-interpolation-trace",
        &[
            relation.hash.0.to_vec(),
            sample_hash.0.to_vec(),
            support_hash.0.to_vec(),
        ],
    );
    Ok(SpecializationInterpolationTrace {
        relation,
        elimination_result,
        samples,
        interpolation_certificate,
        matrix_rows: points.len(),
        matrix_cols: coefficient_support.len(),
        degree_bound,
        sample_hash,
        support_hash,
        trace_hash,
    })
}

fn probe_specialization_interpolation_plan(
    relations: &[SparsePolynomialQ],
    eliminated: &[VariableId],
    exported: &[VariableId],
    target: VariableId,
) -> Result<SpecializationInterpolationPlanProbe, SolverError> {
    let separators = exported
        .iter()
        .copied()
        .filter(|var| *var != target)
        .collect::<Vec<_>>();
    if separators.is_empty() {
        return Err(algorithmic_hard_case(
            "specialization-interpolation requires a non-target exported separator",
        ));
    }
    let degree_bound = specialization_interpolation_degree_bound(relations, eliminated, exported);
    let coefficient_support = build_multiseparator_coefficient_support(&separators, degree_bound);
    let points =
        choose_multiseparator_specialization_points(&separators, &coefficient_support, 101);
    let support_hash = hash_sequence(
        "specialization-interpolation-coefficient-support",
        &coefficient_support
            .iter()
            .map(monomial_to_bytes)
            .collect::<Vec<_>>(),
    );
    Ok(SpecializationInterpolationPlanProbe {
        matrix_rows: points.len(),
        matrix_cols: coefficient_support.len(),
        degree_bound,
        sample_hash: hash_specialization_points(&points),
        support_hash,
    })
}

fn specialization_interpolation_degree_bound(
    relations: &[SparsePolynomialQ],
    eliminated: &[VariableId],
    exported: &[VariableId],
) -> usize {
    relations
        .iter()
        .map(|relation| poly_total_degree(relation) as usize)
        .max()
        .unwrap_or(1)
        .saturating_mul(eliminated.len().max(1))
        .saturating_add(exported.len().saturating_sub(1))
        .max(1)
}

fn execute_inner_target_only_kernel(
    relations: &[SparsePolynomialQ],
    eliminated: &[VariableId],
    target: VariableId,
    point: &SpecializationPoint,
    solver_ctx: &mut SolverContext,
) -> Result<SparsePolynomialQ, SolverError> {
    let specialized = specialize_polynomials(relations, point);
    let mut variables = eliminated.to_vec();
    variables.push(target);
    variables.sort();
    variables.dedup();
    let system = compressed_system_from_polys(variables.clone(), target, specialized);
    let mut block = ProjectionBlock {
        block_id: BlockId(0),
        local_variables: variables.into_iter().collect(),
        relation_ids: system.relation_order.clone(),
        exported_variables: [target].into_iter().collect(),
        child_block_ids: Vec::new(),
        parent_block_id: None,
        authorization_hash: hash_sequence("inner-target-only-pending", &[]),
        duplication_certificates: Vec::new(),
        block_hash: hash_sequence("inner-target-only-block", &[]),
    };
    block.authorization_hash = authorize_block_relations(&block, &system);
    let mut kctx = KernelContext {
        block,
        system,
        child_messages: Vec::new(),
    };
    let admission = admit_target_relation_search(&kctx.block, &kctx, solver_ctx);
    if !admission.is_admitted() {
        return Err(algorithmic_hard_case(
            "inner target-only TargetRelationSearch was not admitted for a specialization sample",
        ));
    }
    let Some(plan) = admission.execution_plan.clone() else {
        return Err(implementation_bug(
            "inner target-only TargetRelationSearch admission lacks execution plan",
        ));
    };
    let message = execute_target_relation_search(&plan, &mut kctx, solver_ctx)?;
    let Some(relation) = message.relation_generators.first().cloned() else {
        return Err(implementation_bug(
            "inner target-only TargetRelationSearch returned no relation",
        ));
    };
    let target_set = [target].into_iter().collect();
    if !poly_variables(&relation).is_subset(&target_set) {
        return Err(implementation_bug(
            "inner target-only kernel returned a non-target relation",
        ));
    }
    Ok(relation)
}

fn compressed_system_from_polys(
    variables: Vec<VariableId>,
    target: VariableId,
    relations: Vec<SparsePolynomialQ>,
) -> CompressedSystemQ {
    let canonical_relations = relations
        .into_iter()
        .enumerate()
        .map(|(idx, relation)| canonicalize_relation(RelationId(idx as u32), relation))
        .collect::<Vec<_>>();
    let relation_order = canonical_relations
        .iter()
        .map(|relation| relation.id)
        .collect::<Vec<_>>();
    let compressed_hash = hash_sequence(
        "specialization-inner-compressed-system",
        &canonical_relations
            .iter()
            .map(|relation| relation.hash.0.to_vec())
            .collect::<Vec<_>>(),
    );
    CompressedSystemQ {
        variables,
        target,
        relations: canonical_relations,
        relation_order,
        semantic_encodings: Vec::new(),
        substitutions: Vec::new(),
        guards: Vec::new(),
        rational_affine_transformations: Vec::new(),
        saturations: Vec::new(),
        feasibility_obligations: Vec::new(),
        diagnostics: Vec::new(),
        compression_trace: crate::preprocess::compression::CompressionTrace::default(),
        compressed_hash,
    }
}

fn validate_specialization_plan_binding(
    plan: &KernelExecutionPlan,
    ctx: &KernelContext,
) -> Result<(), SolverError> {
    if plan.kernel_kind != KernelKind::SpecializationInterpolation {
        return Err(implementation_bug(
            "specialization-interpolation kernel kind mismatch",
        ));
    }
    if hash_kernel_execution_plan(plan) != plan.plan_hash {
        return Err(implementation_bug(
            "specialization-interpolation execution plan hash mismatch",
        ));
    }
    if plan.block_id != ctx.block.block_id {
        return Err(implementation_bug(
            "specialization-interpolation block id mismatch",
        ));
    }
    if plan.input_block_authorization_hash != ctx.block.authorization_hash {
        return Err(implementation_bug(
            "specialization-interpolation block authorization hash mismatch",
        ));
    }
    if plan.certificate_route != CertificateRoute::SpecializationInterpolationExactVerification {
        return Err(implementation_bug(
            "specialization-interpolation certificate route mismatch",
        ));
    }
    let available_child_hashes = ctx
        .child_messages
        .iter()
        .filter(|message| plan.child_block_ids.contains(&message.block_id))
        .map(|message| message.package_hash)
        .collect::<BTreeSet<_>>();
    let planned_child_hashes = plan
        .child_message_hashes
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    if available_child_hashes != planned_child_hashes {
        return Err(implementation_bug(
            "specialization-interpolation child message hash binding mismatch",
        ));
    }
    Ok(())
}

fn planned_relation_inputs(
    plan: &KernelExecutionPlan,
    ctx: &KernelContext,
) -> Result<Vec<InterpolationRelationInput>, SolverError> {
    let source_hashes = plan
        .source_relation_hashes
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let relation_ids = plan
        .source_relation_ids
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let child_hashes = plan
        .child_message_hashes
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let mut out = ctx
        .system
        .relations
        .iter()
        .filter(|relation| relation_ids.contains(&relation.id))
        .filter(|relation| source_hashes.contains(&relation.hash))
        .map(|relation| InterpolationRelationInput {
            polynomial: relation.polynomial.clone(),
            source_relation_ids: vec![relation.id],
            source_hash: relation.hash,
            child_message_hash: None,
        })
        .collect::<Vec<_>>();
    out.extend(
        ctx.child_messages
            .iter()
            .filter(|message| child_hashes.contains(&message.package_hash))
            .flat_map(|message| {
                let source_hashes = source_hashes.clone();
                message
                    .relation_generators
                    .iter()
                    .filter_map(move |relation| {
                        source_hashes
                            .contains(&relation.hash)
                            .then(|| InterpolationRelationInput {
                                polynomial: relation.clone(),
                                source_relation_ids: message.source_relation_ids.clone(),
                                source_hash: relation.hash,
                                child_message_hash: Some(message.package_hash),
                            })
                    })
            }),
    );
    validate_source_hash_coverage(plan, &out)?;
    Ok(out)
}

fn block_relations(block: &ProjectionBlock, system: &CompressedSystemQ) -> Vec<CanonicalRelationQ> {
    let ids = block.relation_ids.iter().copied().collect::<BTreeSet<_>>();
    system
        .relations
        .iter()
        .filter(|relation| ids.contains(&relation.id))
        .cloned()
        .collect()
}

fn collect_relation_inputs(
    block: &ProjectionBlock,
    system: &CompressedSystemQ,
    child_messages: &[ProjectionMessage],
) -> Vec<InterpolationRelationInput> {
    let mut inputs = block_relations(block, system)
        .into_iter()
        .map(|relation| InterpolationRelationInput {
            polynomial: relation.polynomial,
            source_relation_ids: vec![relation.id],
            source_hash: relation.hash,
            child_message_hash: None,
        })
        .collect::<Vec<_>>();
    for message in child_messages {
        for relation in &message.relation_generators {
            inputs.push(InterpolationRelationInput {
                polynomial: relation.clone(),
                source_relation_ids: message.source_relation_ids.clone(),
                source_hash: relation.hash,
                child_message_hash: Some(message.package_hash),
            });
        }
    }
    inputs
}

fn validate_source_hash_coverage(
    plan: &KernelExecutionPlan,
    inputs: &[InterpolationRelationInput],
) -> Result<(), SolverError> {
    let mut expected = plan.source_relation_hashes.clone();
    let mut actual = inputs
        .iter()
        .map(|input| input.source_hash)
        .collect::<Vec<_>>();
    expected.sort();
    actual.sort();
    if expected != actual {
        return Err(implementation_bug(
            "specialization-interpolation source relation hash mismatch",
        ));
    }
    Ok(())
}

fn specialization_interpolation_certificate_hash(
    plan: &KernelExecutionPlan,
    trace: &SpecializationInterpolationTrace,
) -> Hash {
    let mut chunks = vec![
        plan.plan_hash.0.to_vec(),
        trace.trace_hash.0.to_vec(),
        trace.interpolation_certificate.relation_hash.0.to_vec(),
        trace.degree_bound.to_be_bytes().to_vec(),
        trace.sample_hash.0.to_vec(),
        trace.support_hash.0.to_vec(),
    ];
    for sample in &trace.samples {
        chunks.push(sample.relation.hash.0.to_vec());
    }
    hash_sequence("specialization-interpolation-kernel-certificate", &chunks)
}

fn hash_specialization_samples(samples: &[SpecializedRelation]) -> Hash {
    let mut chunks = Vec::new();
    for sample in samples {
        for (var, value) in &sample.point.assignments {
            let mut bytes = var.0.to_be_bytes().to_vec();
            bytes.extend(crate::types::rational::rational_to_bytes(value));
            chunks.push(bytes);
        }
        chunks.push(sample.relation.hash.0.to_vec());
    }
    hash_sequence("specialization-interpolation-samples", &chunks)
}

fn hash_specialization_points(points: &[SpecializationPoint]) -> Hash {
    let mut chunks = Vec::new();
    for point in points {
        chunks.push(point.prime.to_be_bytes().to_vec());
        for (variable, value) in &point.assignments {
            chunks.push(variable.0.to_be_bytes().to_vec());
            chunks.push(rational_to_bytes(value));
        }
        chunks.push(Vec::new());
    }
    hash_sequence("specialization-interpolation-sample-points", &chunks)
}

fn finish_admission(
    block: &ProjectionBlock,
    status: KernelAdmissionStatus,
    exported_variables: Vec<VariableId>,
    eliminated_variables: Vec<VariableId>,
    execution_plan: Option<KernelExecutionPlan>,
) -> KernelAdmission {
    let mut chunks = vec![
        b"SpecializationInterpolation".to_vec(),
        block.block_id.0.to_be_bytes().to_vec(),
        format!("{status:?}").into_bytes(),
    ];
    if let Some(plan) = &execution_plan {
        chunks.push(plan.plan_hash.0.to_vec());
    }
    KernelAdmission {
        kind: KernelKind::SpecializationInterpolation,
        block_id: block.block_id,
        status,
        exported_variables,
        eliminated_variables,
        execution_plan,
        admission_hash: hash_sequence("kernel-admission", &chunks),
    }
}

fn projection_message_hash(message: &ProjectionMessage) -> Hash {
    crate::compose::message::hash_projection_message(message)
}

fn sorted_set(vars: &BTreeSet<VariableId>) -> Vec<VariableId> {
    vars.iter().copied().collect()
}

fn dedup_hashes_in_order(hashes: Vec<Hash>) -> Vec<Hash> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for hash in hashes {
        if seen.insert(hash) {
            out.push(hash);
        }
    }
    out
}

fn algorithmic_hard_case(reason: &str) -> SolverError {
    SolverError {
        target: None,
        kind: SolverErrorKind::Failure(FailureKind::AlgorithmicHardCase {
            stage: StageId("SpecializationInterpolationKernel".to_owned()),
            reason: AlgebraicReason(reason.to_owned()),
            minimal_block_hash: hash_sequence(
                "specialization-interpolation-hard-case",
                &[reason.as_bytes().to_vec()],
            ),
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::compose::message::{MessageRepresentation, ProjectionMessage, ProjectionStrength};
    use crate::graph::projection_dag::ProjectionBlock;
    use crate::kernels::traits::{KernelContext, KernelKind, TargetProjectionKernel};
    use crate::planner::admission::{collect_kernel_admissions, KernelAdmissionStatus};
    use crate::planner::kernel_plan::CertificateRoute;
    use crate::planner::probes::run_cost_probes;
    use crate::preprocess::compression::CompressionState;
    use crate::problem::canonicalize::canonicalize_system;
    use crate::problem::context::new_context;
    use crate::problem::input::make_problem;
    use crate::problem::validate::validate_input;
    use crate::result::cost_trace::ProjectionCostTrace;
    use crate::result::status::SolverStatus;
    use crate::solver::options::SolverOptions;
    use crate::types::hash::hash_sequence;
    use crate::types::ids::{BlockId, PackageId, VariableId};
    use crate::types::polynomial::{
        constant_poly, poly_add, poly_sub, poly_variables, variable_poly,
    };
    use crate::types::rational::int_q;
    use crate::verify::certificates::KernelCertificate;

    use super::*;

    #[test]
    fn p8b_specialization_interpolation_kernel_verifies_multiseparator_relation() {
        let t = VariableId(0);
        let u = VariableId(1);
        let v = VariableId(2);
        let x = VariableId(3);
        let rhs = poly_add(
            &poly_add(&variable_poly(t), &variable_poly(u)),
            &variable_poly(v),
        );
        let relations = vec![
            poly_sub(&variable_poly(x), &rhs),
            poly_sub(&variable_poly(x), &constant_poly(int_q(1))),
        ];
        let compressed = compressed_system(vec![t, u, v, x], t, relations);
        let block = test_block(&compressed, [t, u, v, x], [t, u, v]);
        let mut solver_ctx = new_context(SolverOptions::default());
        let mut kctx = KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };
        let kernel = SpecializationInterpolationKernel;
        let admission = kernel.admit(&kctx.block, &kctx);

        assert!(admission.is_admitted());
        let plan = kernel.plan(&admission, &kctx, &solver_ctx).unwrap();
        let message = kernel.execute(&plan, &mut kctx, &mut solver_ctx).unwrap();

        assert_eq!(message.kernel_kind, KernelKind::SpecializationInterpolation);
        assert_eq!(
            message.representation,
            MessageRepresentation::SpecializationInterpolation
        );
        assert_eq!(
            message.projection_strength,
            ProjectionStrength::CandidateCoverStrong
        );
        let exported = [t, u, v].into_iter().collect::<BTreeSet<_>>();
        assert!(message
            .relation_generators
            .iter()
            .all(|poly| poly_variables(poly).is_subset(&exported)));
        assert!(kernel.replay(&message, &kctx).accepted);
    }

    #[test]
    fn p8b_specialization_interpolation_rejects_authorization_tamper() {
        let t = VariableId(0);
        let u = VariableId(1);
        let x = VariableId(2);
        let rhs = poly_add(&variable_poly(t), &variable_poly(u));
        let relations = vec![
            poly_sub(&variable_poly(x), &rhs),
            poly_sub(&variable_poly(x), &constant_poly(int_q(1))),
        ];
        let compressed = compressed_system(vec![t, u, x], t, relations);
        let block = test_block(&compressed, [t, u, x], [t, u]);
        let solver_ctx = new_context(SolverOptions::default());
        let mut kctx = KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };
        let kernel = SpecializationInterpolationKernel;
        let admission = kernel.admit(&kctx.block, &kctx);
        let mut plan = kernel.plan(&admission, &kctx, &solver_ctx).unwrap();

        plan.input_block_authorization_hash = hash_sequence("tampered-auth", &[]);
        let mut execute_ctx = new_context(SolverOptions::default());
        let err = kernel
            .execute(&plan, &mut kctx, &mut execute_ctx)
            .unwrap_err();

        assert_eq!(err.public_status(), SolverStatus::ImplementationBug);
    }

    #[test]
    fn p8b_specialization_interpolation_consumes_child_messages_and_rejects_tamper() {
        let t = VariableId(0);
        let u = VariableId(1);
        let x = VariableId(2);
        let rhs = poly_add(&variable_poly(t), &variable_poly(u));
        let relations = vec![poly_sub(&variable_poly(x), &rhs)];
        let compressed = compressed_system(vec![t, u, x], t, relations);
        let mut block = test_block(&compressed, [t, u, x], [t, u]);
        block.child_block_ids = vec![BlockId(7)];
        block.authorization_hash =
            crate::graph::projection_dag::authorize_block_relations(&block, &compressed);
        let child_relation = poly_sub(&variable_poly(x), &constant_poly(int_q(1)));
        let child_message = child_projection_message(&compressed, child_relation, vec![t, u]);
        let mut solver_ctx = new_context(SolverOptions::default());
        let mut kctx = KernelContext {
            block,
            system: compressed,
            child_messages: vec![child_message],
        };
        let kernel = SpecializationInterpolationKernel;
        let admission = kernel.admit(&kctx.block, &kctx);
        assert!(admission.is_admitted());
        let plan = kernel.plan(&admission, &kctx, &solver_ctx).unwrap();
        assert_eq!(
            plan.child_message_hashes,
            vec![kctx.child_messages[0].package_hash]
        );

        let message = kernel.execute(&plan, &mut kctx, &mut solver_ctx).unwrap();
        assert_eq!(message.kernel_kind, KernelKind::SpecializationInterpolation);

        let mut tampered_ctx = kctx.clone();
        tampered_ctx.child_messages[0].package_hash = hash_sequence("tampered-child", &[]);
        let err = kernel
            .execute(
                &plan,
                &mut tampered_ctx,
                &mut new_context(SolverOptions::default()),
            )
            .unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::ImplementationBug);
    }

    #[test]
    fn p12g_specialization_interpolation_inner_schedule_is_declared() {
        let t = VariableId(0);
        let u = VariableId(1);
        let v = VariableId(2);
        let x = VariableId(3);
        let rhs = poly_add(
            &poly_add(&variable_poly(t), &variable_poly(u)),
            &variable_poly(v),
        );
        let relations = vec![
            poly_sub(&variable_poly(x), &rhs),
            poly_sub(&variable_poly(x), &constant_poly(int_q(1))),
        ];
        let compressed = compressed_system(vec![t, u, v, x], t, relations);
        let block = test_block(&compressed, [t, u, v, x], [t, u, v]);
        let solver_ctx = new_context(SolverOptions::default());
        let kctx = KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };
        let kernel = SpecializationInterpolationKernel;
        let plan = kernel
            .plan(&kernel.admit(&kctx.block, &kctx), &kctx, &solver_ctx)
            .unwrap();

        assert_eq!(plan.kernel_kind, KernelKind::SpecializationInterpolation);
        assert!(plan.support_plan.template_plan.is_some());
        assert!(plan.support_plan.rank_plan.is_some());
        assert!(plan.support_plan.degree_bound >= 1);
        assert!(plan.resource_bounds.max_local_elimination_steps.is_some());
        assert_eq!(
            plan.certificate_route,
            CertificateRoute::SpecializationInterpolationExactVerification
        );
        assert_eq!(
            plan.plan_work_classification,
            crate::planner::kernel_plan::PlanWorkClassification::PurePlan
        );
    }

    #[test]
    fn p8b_planner_admits_sparse_resultant_and_specialization_plans() {
        let t = VariableId(0);
        let u = VariableId(1);
        let v = VariableId(2);
        let x = VariableId(3);
        let rhs = poly_add(
            &poly_add(&variable_poly(t), &variable_poly(u)),
            &variable_poly(v),
        );
        let relations = vec![
            poly_sub(&variable_poly(x), &rhs),
            poly_sub(&variable_poly(x), &constant_poly(int_q(1))),
        ];
        let compressed = compressed_system(vec![t, u, v, x], t, relations);
        let block = test_block(&compressed, [t, u, v, x], [t, u, v]);
        let mut solver_ctx = new_context(SolverOptions::default());
        let probes = run_cost_probes(&block, &compressed, &mut solver_ctx);
        let admissions = collect_kernel_admissions(&block, &compressed, &probes, &solver_ctx);

        for kind in [
            KernelKind::SparseResultantProjection,
            KernelKind::SpecializationInterpolation,
        ] {
            let admission = admissions
                .iter()
                .find(|admission| admission.kind == kind)
                .unwrap();
            assert!(matches!(admission.status, KernelAdmissionStatus::Admitted));
            assert!(admission.execution_plan.is_some());
        }
    }

    fn compressed_system(
        variables: Vec<VariableId>,
        target: VariableId,
        relations: Vec<crate::types::polynomial::SparsePolynomialQ>,
    ) -> crate::preprocess::compression::CompressedSystemQ {
        let problem = make_problem(variables, target, relations, Vec::new());
        let validated = validate_input(problem).unwrap();
        let canonical = canonicalize_system(validated).unwrap();
        CompressionState::from_system(canonical).to_compressed_system()
    }

    fn test_block<const N: usize, const M: usize>(
        compressed: &crate::preprocess::compression::CompressedSystemQ,
        local_variables: [VariableId; N],
        exported_variables: [VariableId; M],
    ) -> ProjectionBlock {
        let mut block = ProjectionBlock {
            block_id: BlockId(0),
            local_variables: local_variables.into_iter().collect(),
            relation_ids: compressed.relation_order.clone(),
            exported_variables: exported_variables.into_iter().collect(),
            child_block_ids: Vec::new(),
            parent_block_id: None,
            authorization_hash: hash_sequence("pending", &[]),
            duplication_certificates: Vec::new(),
            block_hash: hash_sequence("test-block", &[]),
        };
        block.authorization_hash =
            crate::graph::projection_dag::authorize_block_relations(&block, compressed);
        block
    }

    fn child_projection_message(
        compressed: &crate::preprocess::compression::CompressedSystemQ,
        relation: crate::types::polynomial::SparsePolynomialQ,
        exported_variables: Vec<VariableId>,
    ) -> ProjectionMessage {
        let certificate_hash = hash_sequence("test-child-certificate", &[relation.hash.0.to_vec()]);
        let mut message = ProjectionMessage {
            package_id: PackageId(702),
            block_id: BlockId(7),
            kernel_kind: KernelKind::TargetUnivariate,
            source_relation_ids: Vec::new(),
            eliminated_variables: Vec::new(),
            exported_variables,
            relation_generators: vec![relation],
            representation: MessageRepresentation::GeneratorSet,
            projection_strength: ProjectionStrength::CandidateCoverStrong,
            certificate: KernelCertificate::synthetic_for_tests(certificate_hash),
            compression_trace: compressed.compression_trace.clone(),
            cost_trace: ProjectionCostTrace::default(),
            package_hash: hash_sequence("projection-message-initial", &[]),
        };
        message.package_hash = hash_sequence(
            "test-child-message",
            &[message.relation_generators[0].hash.0.to_vec()],
        );
        message
    }
}
