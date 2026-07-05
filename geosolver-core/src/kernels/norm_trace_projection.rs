use std::collections::BTreeSet;

use crate::algebra::norm_trace::{
    detect_explicit_tower_plan, norm_relation_for_tower_plan, verify_norm_tower_plan_relation,
    TowerPlanDescription,
};
use crate::compose::message::{MessageRepresentation, ProjectionMessage, ProjectionStrength};
use crate::graph::projection_dag::ProjectionBlock;
use crate::kernels::traits::{KernelContext, KernelKind, ReplayResult, TargetProjectionKernel};
use crate::planner::admission::{KernelAdmission, KernelAdmissionStatus};
use crate::planner::kernel_plan::{
    hash_kernel_execution_plan, planned_failure_behavior, rank_plan, resource_bounds_hash,
    support_plan_hash, template_plan, CertificateRoute, KernelExecutionPlan, KernelSupportPlan,
    LocalNonfinitePolicy, ResourceBounds,
};
use crate::preprocess::compression::CompressedSystemQ;
use crate::problem::canonicalize::CanonicalRelationQ;
use crate::problem::context::SolverContext;
use crate::result::cost_trace::ProjectionCostTrace;
use crate::result::status::{
    AlgebraicReason, FailureKind, SolverError, SolverErrorKind, SolverStatus, StageId,
};
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::{KernelPlanId, PackageId, RelationId, VariableId};
use crate::types::polynomial::{
    clear_denominators_primitive, poly_monomial_count, poly_total_degree, poly_variables,
    SparsePolynomialQ,
};
use crate::verify::certificates::{
    KernelCertificate, KernelCertificatePayload, NormTraceProjectionCertificate,
};

pub struct NormTraceProjectionKernel;

impl TargetProjectionKernel for NormTraceProjectionKernel {
    fn kind(&self) -> KernelKind {
        KernelKind::NormTraceProjection
    }

    fn admit(&self, block: &ProjectionBlock, ctx: &KernelContext) -> KernelAdmission {
        admit_norm_trace_projection(block, ctx)
    }

    fn plan(
        &self,
        admission: &KernelAdmission,
        ctx: &KernelContext,
        solver_ctx: &SolverContext,
    ) -> Result<KernelExecutionPlan, SolverError> {
        plan_norm_trace_projection_from_admission(admission, ctx, solver_ctx)
    }

    fn execute(
        &self,
        plan: &KernelExecutionPlan,
        ctx: &mut KernelContext,
        _solver_ctx: &mut SolverContext,
    ) -> Result<ProjectionMessage, SolverError> {
        execute_norm_trace_projection(plan, ctx)
    }

    fn replay(&self, message: &ProjectionMessage, ctx: &KernelContext) -> ReplayResult {
        crate::kernels::traits::exact_replay_result(self.kind(), "norm-trace-replay", message, ctx)
    }
}

#[derive(Debug, Clone)]
struct NormTraceRelationInput {
    polynomial: SparsePolynomialQ,
    source_relation_ids: Vec<RelationId>,
    source_hash: Hash,
}

#[derive(Debug, Clone)]
struct NormTraceProjectionTrace {
    tower: TowerPlanDescription,
    relation: SparsePolynomialQ,
    max_degree: usize,
    trace_hash: Hash,
}

pub fn admit_norm_trace_projection(
    block: &ProjectionBlock,
    ctx: &KernelContext,
) -> KernelAdmission {
    match plan_norm_trace_projection(
        block,
        &ctx.system,
        &SolverContext::new(Default::default()),
        KernelPlanId(KernelKind::NormTraceProjection as u32),
    ) {
        Ok(plan) => finish_admission(block, KernelAdmissionStatus::Admitted, Some(plan)),
        Err(_) => finish_admission(
            block,
            KernelAdmissionStatus::Declined {
                reason: "no explicit algebraic tower norm/trace projection plan is applicable"
                    .to_owned(),
            },
            None,
        ),
    }
}

pub fn plan_norm_trace_projection(
    block: &ProjectionBlock,
    system: &CompressedSystemQ,
    solver_ctx: &SolverContext,
    plan_id: KernelPlanId,
) -> Result<KernelExecutionPlan, SolverError> {
    let inputs = collect_relation_inputs(block, system);
    let relations = inputs
        .iter()
        .map(|input| input.polynomial.clone())
        .collect::<Vec<_>>();
    let exported = sorted_set(&block.exported_variables);
    let trace = build_norm_trace_trace(&relations, &exported)?;
    let mut support_plan = KernelSupportPlan {
        dense_relation_search_schedule: None,
        affine_elimination_order: None,
        template_plan: Some(template_plan(
            1,
            trace.relation.terms.len().max(1),
            trace.tower.tower_hash,
            trace.relation.hash,
        )),
        rank_plan: Some(rank_plan(1)),
        universal_strategy_sequence: Vec::new(),
        degree_bound: trace.max_degree,
        support_hash: hash_sequence("kernel-support-plan", &[]),
    };
    support_plan.support_hash = support_plan_hash(&support_plan);
    let mut bounds = ResourceBounds {
        max_matrix_rows: solver_ctx.options.max_matrix_rows.or(Some(1)),
        max_matrix_cols: solver_ctx
            .options
            .max_matrix_cols
            .or(Some(trace.relation.terms.len().max(1))),
        max_export_degree: Some(trace.max_degree),
        max_multiplier_total_degree: None,
        max_local_elimination_steps: Some(relations.len().max(1)),
        max_memory_bytes: solver_ctx.options.max_memory_bytes,
        bounds_hash: hash_sequence("planner-resource-bounds", &[]),
    };
    bounds.bounds_hash = resource_bounds_hash(&bounds);
    Ok(KernelExecutionPlan::new(
        plan_id,
        block.block_id,
        KernelKind::NormTraceProjection,
        block.authorization_hash,
        inputs
            .iter()
            .flat_map(|input| input.source_relation_ids.iter().copied())
            .collect(),
        inputs.iter().map(|input| input.source_hash).collect(),
        block.child_block_ids.clone(),
        Vec::new(),
        exported,
        block
            .local_variables
            .difference(&block.exported_variables)
            .copied()
            .collect(),
        support_plan,
        bounds,
        CertificateRoute::NormTraceExactVerification,
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

pub fn plan_norm_trace_projection_from_admission(
    admission: &KernelAdmission,
    ctx: &KernelContext,
    solver_ctx: &SolverContext,
) -> Result<KernelExecutionPlan, SolverError> {
    if admission.kind != KernelKind::NormTraceProjection
        || !matches!(admission.status, KernelAdmissionStatus::Admitted)
    {
        return Err(implementation_bug(
            "norm-trace plan requested for non-admitted kernel",
        ));
    }
    if let Some(plan) = &admission.execution_plan {
        return Ok(plan.clone());
    }
    plan_norm_trace_projection(
        &ctx.block,
        &ctx.system,
        solver_ctx,
        KernelPlanId(KernelKind::NormTraceProjection as u32),
    )
}

pub fn execute_norm_trace_projection(
    plan: &KernelExecutionPlan,
    ctx: &mut KernelContext,
) -> Result<ProjectionMessage, SolverError> {
    validate_norm_trace_plan_binding(plan, ctx)?;
    let inputs = planned_relation_inputs(plan, ctx)?;
    let relations = inputs
        .iter()
        .map(|input| input.polynomial.clone())
        .collect::<Vec<_>>();
    let trace = build_norm_trace_trace(&relations, &plan.exported_variables)?;
    let Some(template) = &plan.support_plan.template_plan else {
        return Err(implementation_bug("norm-trace plan lacks template plan"));
    };
    if template.row_monomial_hash != trace.tower.tower_hash
        || template.column_support_hash != trace.relation.hash
    {
        return Err(implementation_bug(
            "norm-trace projection trace does not match plan support",
        ));
    }
    validate_exported_relation(&trace.relation, &plan.exported_variables)?;
    let certificate_hash = norm_trace_certificate_hash(plan, &trace);
    let cost_trace = ProjectionCostTrace {
        block_id: plan.block_id,
        kernel_kind: KernelKind::NormTraceProjection,
        local_variable_count: ctx.block.local_variables.len(),
        exported_variable_count: plan.exported_variables.len(),
        local_relation_count: relations.len(),
        local_monomial_count: relations.iter().map(poly_monomial_count).sum(),
        estimated_quotient_rank: Some(1),
        matrix_rows: Some(1),
        matrix_cols: Some(trace.relation.terms.len().max(1)),
        matrix_density: None,
        coefficient_height_before_bits: 0,
        coefficient_height_after_bits: poly_monomial_count(&trace.relation),
    };
    let certificate = KernelCertificate::from_execution_plan_with_payload(
        plan,
        std::slice::from_ref(&trace.relation),
        certificate_hash,
        KernelCertificatePayload::NormTrace(NormTraceProjectionCertificate {
            tower: trace.tower.clone(),
            output_relation: trace.relation.clone(),
        }),
    );
    let mut message = ProjectionMessage {
        package_id: PackageId(plan.plan_id.0),
        block_id: plan.block_id,
        kernel_kind: KernelKind::NormTraceProjection,
        source_relation_ids: plan.source_relation_ids.clone(),
        eliminated_variables: plan.eliminated_variables.clone(),
        exported_variables: plan.exported_variables.clone(),
        relation_generators: vec![trace.relation],
        representation: MessageRepresentation::NormTraceTower,
        projection_strength: ProjectionStrength::CandidateCoverStrong,
        certificate,
        compression_trace: ctx.system.compression_trace.clone(),
        cost_trace,
        package_hash: hash_sequence("projection-message-initial", &[]),
    };
    message.package_hash = projection_message_hash(&message);
    Ok(message)
}

fn build_norm_trace_trace(
    relations: &[SparsePolynomialQ],
    exported: &[VariableId],
) -> Result<NormTraceProjectionTrace, SolverError> {
    let Some(tower) = detect_explicit_tower_plan(relations, exported) else {
        return Err(algorithmic_hard_case(
            "no explicit algebraic tower detected by algebraic form",
        ));
    };
    let relation = norm_relation_for_tower_plan(&tower)?.into_multivariate();
    let relation = clear_denominators_primitive(&relation);
    if !verify_norm_tower_plan_relation(&tower, &relation) {
        return Err(certificate_design_gap(
            relation.hash,
            "NormTraceExactVerification",
        ));
    }
    validate_exported_relation(&relation, exported)?;
    let max_degree = poly_total_degree(&relation) as usize;
    let trace_hash = hash_sequence(
        "norm-trace-projection-trace",
        &[
            tower.tower_hash.0.to_vec(),
            relation.hash.0.to_vec(),
            hash_sequence(
                "norm-trace-source-relations",
                &tower
                    .source_relation_hashes
                    .iter()
                    .map(|hash| hash.0.to_vec())
                    .collect::<Vec<_>>(),
            )
            .0
            .to_vec(),
        ],
    );
    Ok(NormTraceProjectionTrace {
        tower,
        relation,
        max_degree,
        trace_hash,
    })
}

fn validate_norm_trace_plan_binding(
    plan: &KernelExecutionPlan,
    ctx: &KernelContext,
) -> Result<(), SolverError> {
    if plan.kernel_kind != KernelKind::NormTraceProjection {
        return Err(implementation_bug(
            "norm-trace execution received wrong plan kind",
        ));
    }
    if hash_kernel_execution_plan(plan) != plan.plan_hash {
        return Err(implementation_bug(
            "norm-trace execution plan hash mismatch",
        ));
    }
    if plan.block_id != ctx.block.block_id {
        return Err(implementation_bug("norm-trace block id mismatch"));
    }
    if plan.input_block_authorization_hash != ctx.block.authorization_hash {
        return Err(implementation_bug(
            "norm-trace block authorization hash mismatch",
        ));
    }
    if plan.certificate_route != CertificateRoute::NormTraceExactVerification {
        return Err(implementation_bug("norm-trace certificate route mismatch"));
    }
    if support_plan_hash(&plan.support_plan) != plan.support_plan.support_hash {
        return Err(implementation_bug("norm-trace support hash mismatch"));
    }
    Ok(())
}

fn planned_relation_inputs(
    plan: &KernelExecutionPlan,
    ctx: &KernelContext,
) -> Result<Vec<NormTraceRelationInput>, SolverError> {
    let relation_ids = plan
        .source_relation_ids
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let source_hashes = plan
        .source_relation_hashes
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let inputs = ctx
        .system
        .relations
        .iter()
        .filter(|relation| relation_ids.contains(&relation.id))
        .filter(|relation| source_hashes.contains(&relation.hash))
        .map(|relation| NormTraceRelationInput {
            polynomial: relation.polynomial.clone(),
            source_relation_ids: vec![relation.id],
            source_hash: relation.hash,
        })
        .collect::<Vec<_>>();
    validate_source_hash_coverage(plan, &inputs)?;
    Ok(inputs)
}

fn collect_relation_inputs(
    block: &ProjectionBlock,
    system: &CompressedSystemQ,
) -> Vec<NormTraceRelationInput> {
    block_relations(block, system)
        .into_iter()
        .map(|relation| NormTraceRelationInput {
            polynomial: relation.polynomial,
            source_relation_ids: vec![relation.id],
            source_hash: relation.hash,
        })
        .collect()
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

fn validate_source_hash_coverage(
    plan: &KernelExecutionPlan,
    inputs: &[NormTraceRelationInput],
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
            "norm-trace source relation hash mismatch",
        ));
    }
    Ok(())
}

fn validate_exported_relation(
    relation: &SparsePolynomialQ,
    exported: &[VariableId],
) -> Result<(), SolverError> {
    let exported = exported.iter().copied().collect::<BTreeSet<_>>();
    if !poly_variables(relation).is_subset(&exported) {
        return Err(implementation_bug(
            "norm-trace relation contains a non-exported variable",
        ));
    }
    Ok(())
}

fn norm_trace_certificate_hash(
    plan: &KernelExecutionPlan,
    trace: &NormTraceProjectionTrace,
) -> Hash {
    hash_sequence(
        "norm-trace-kernel-certificate",
        &[
            plan.plan_hash.0.to_vec(),
            trace.tower.tower_hash.0.to_vec(),
            trace.relation.hash.0.to_vec(),
            trace.trace_hash.0.to_vec(),
        ],
    )
}

fn finish_admission(
    block: &ProjectionBlock,
    status: KernelAdmissionStatus,
    execution_plan: Option<KernelExecutionPlan>,
) -> KernelAdmission {
    let mut chunks = vec![
        b"NormTraceProjection".to_vec(),
        block.block_id.0.to_be_bytes().to_vec(),
        format!("{status:?}").into_bytes(),
    ];
    if let Some(plan) = &execution_plan {
        chunks.push(plan.plan_hash.0.to_vec());
    }
    KernelAdmission {
        kind: KernelKind::NormTraceProjection,
        block_id: block.block_id,
        status,
        exported_variables: block.exported_variables.iter().copied().collect(),
        eliminated_variables: block
            .local_variables
            .difference(&block.exported_variables)
            .copied()
            .collect(),
        execution_plan,
        admission_hash: hash_sequence("kernel-admission", &chunks),
    }
}

fn projection_message_hash(message: &ProjectionMessage) -> Hash {
    let mut chunks = vec![
        message.package_id.0.to_be_bytes().to_vec(),
        message.block_id.0.to_be_bytes().to_vec(),
        format!("{:?}", message.kernel_kind).into_bytes(),
        message.certificate.certificate_hash.0.to_vec(),
    ];
    for relation in &message.relation_generators {
        chunks.push(relation.hash.0.to_vec());
    }
    hash_sequence("projection-message", &chunks)
}

fn sorted_set(vars: &BTreeSet<VariableId>) -> Vec<VariableId> {
    vars.iter().copied().collect()
}

fn algorithmic_hard_case(reason: &str) -> SolverError {
    SolverError {
        target: None,
        kind: SolverErrorKind::Failure(FailureKind::AlgorithmicHardCase {
            stage: StageId("NormTraceProjectionKernel".to_owned()),
            reason: AlgebraicReason(reason.to_owned()),
            minimal_block_hash: hash_sequence(
                "norm-trace-kernel-hard-case",
                &[reason.as_bytes().to_vec()],
            ),
        }),
    }
}

fn certificate_design_gap(object_hash: Hash, kind: &str) -> SolverError {
    SolverError {
        target: None,
        kind: SolverErrorKind::Failure(FailureKind::CertificateDesignGap {
            constructed_object_hash: object_hash,
            missing_certificate_kind: kind.to_owned(),
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
    use crate::kernels::traits::TargetProjectionKernel;
    use crate::preprocess::compression::CompressionState;
    use crate::problem::canonicalize::canonicalize_system;
    use crate::problem::context::new_context;
    use crate::problem::input::make_problem;
    use crate::problem::validate::validate_input;
    use crate::solver::options::SolverOptions;
    use crate::types::hash::hash_sequence;
    use crate::types::ids::BlockId;
    use crate::types::polynomial::{constant_poly, poly_add, poly_mul, poly_sub, variable_poly};
    use crate::types::rational::int_q;

    use super::*;

    #[test]
    fn p9_norm_trace_kernel_exports_verified_tower_support() {
        let t = VariableId(0);
        let a = VariableId(1);
        let compressed = compressed_system(
            vec![t, a],
            t,
            vec![
                poly_sub(
                    &poly_mul(&variable_poly(a), &variable_poly(a)),
                    &constant_poly(int_q(2)),
                ),
                poly_sub(&variable_poly(t), &variable_poly(a)),
            ],
        );
        let block = test_block(&compressed, [t, a], [t]);
        let mut solver_ctx = new_context(SolverOptions::default());
        let mut kctx = KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };
        let kernel = NormTraceProjectionKernel;
        let admission = kernel.admit(&kctx.block, &kctx);
        assert!(admission.is_admitted());
        let plan = kernel.plan(&admission, &kctx, &solver_ctx).unwrap();
        let message = kernel.execute(&plan, &mut kctx, &mut solver_ctx).unwrap();
        let expected = poly_sub(
            &poly_mul(&variable_poly(t), &variable_poly(t)),
            &constant_poly(int_q(2)),
        );

        assert_eq!(message.kernel_kind, KernelKind::NormTraceProjection);
        assert!(matches!(
            message.representation,
            MessageRepresentation::NormTraceTower
        ));
        assert_eq!(message.relation_generators.len(), 1);
        assert!(same_up_to_sign(&message.relation_generators[0], &expected));
        assert!(kernel.replay(&message, &kctx).accepted);
    }

    #[test]
    fn p9_norm_trace_detection_uses_algebraic_form_and_rejects_tamper() {
        let t = VariableId(0);
        let a = VariableId(1);
        let compressed = compressed_system(
            vec![t, a],
            t,
            vec![
                poly_sub(
                    &poly_mul(&variable_poly(a), &variable_poly(a)),
                    &constant_poly(int_q(2)),
                ),
                poly_sub(&variable_poly(t), &variable_poly(a)),
            ],
        );
        let block = test_block(&compressed, [t, a], [t]);
        let mut solver_ctx = new_context(SolverOptions::default());
        let kctx = KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };
        let kernel = NormTraceProjectionKernel;
        let plan = kernel
            .plan(&kernel.admit(&kctx.block, &kctx), &kctx, &solver_ctx)
            .unwrap();

        let mut bad_plan = plan.clone();
        bad_plan.plan_hash = hash_sequence("tampered-plan", &[]);
        assert_eq!(
            kernel
                .execute(&bad_plan, &mut kctx.clone(), &mut solver_ctx)
                .unwrap_err()
                .public_status(),
            SolverStatus::ImplementationBug
        );

        let mut bad_auth = kctx.clone();
        bad_auth.block.authorization_hash = hash_sequence("tampered-auth", &[]);
        assert_eq!(
            kernel
                .execute(&plan, &mut bad_auth, &mut solver_ctx)
                .unwrap_err()
                .public_status(),
            SolverStatus::ImplementationBug
        );

        let non_tower = compressed_system(
            vec![t, a],
            t,
            vec![
                poly_sub(&variable_poly(t), &variable_poly(a)),
                poly_add(&variable_poly(t), &constant_poly(int_q(1))),
            ],
        );
        let non_tower_block = test_block(&non_tower, [t, a], [t]);
        let non_tower_ctx = KernelContext {
            block: non_tower_block,
            system: non_tower,
            child_messages: Vec::new(),
        };
        assert!(!kernel
            .admit(&non_tower_ctx.block, &non_tower_ctx)
            .is_admitted());
    }

    #[test]
    fn p9_norm_trace_multistep_tower_exports_verified_support() {
        let t = VariableId(0);
        let a = VariableId(1);
        let b = VariableId(2);
        let compressed = compressed_system(
            vec![t, a, b],
            t,
            vec![
                poly_sub(
                    &poly_mul(&variable_poly(a), &variable_poly(a)),
                    &constant_poly(int_q(2)),
                ),
                poly_sub(
                    &poly_mul(&variable_poly(b), &variable_poly(b)),
                    &variable_poly(a),
                ),
                poly_sub(&variable_poly(t), &variable_poly(b)),
            ],
        );
        let block = test_block(&compressed, [t, a, b], [t]);
        let mut solver_ctx = new_context(SolverOptions::default());
        let mut kctx = KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };
        let kernel = NormTraceProjectionKernel;
        let plan = kernel
            .plan(&kernel.admit(&kctx.block, &kctx), &kctx, &solver_ctx)
            .unwrap();
        let message = kernel.execute(&plan, &mut kctx, &mut solver_ctx).unwrap();
        let t2 = poly_mul(&variable_poly(t), &variable_poly(t));
        let expected = poly_sub(&poly_mul(&t2, &t2), &constant_poly(int_q(2)));

        assert_eq!(message.relation_generators.len(), 1);
        assert!(same_up_to_sign(&message.relation_generators[0], &expected));
        assert!(matches!(
            message.representation,
            MessageRepresentation::NormTraceTower
        ));
    }

    #[test]
    fn p12g_g7_norm_trace_stress_covers_single_and_two_step_towers() {
        p9_norm_trace_kernel_exports_verified_tower_support();
        p9_norm_trace_multistep_tower_exports_verified_support();
    }

    fn compressed_system(
        variables: Vec<VariableId>,
        target: VariableId,
        relations: Vec<SparsePolynomialQ>,
    ) -> CompressedSystemQ {
        let canonical = canonicalize_system(
            validate_input(make_problem(variables, target, relations, Vec::new())).unwrap(),
        )
        .unwrap();
        CompressionState::from_system(canonical).to_compressed_system()
    }

    fn test_block<const N: usize, const M: usize>(
        compressed: &CompressedSystemQ,
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
            authorization_hash: hash_sequence("tmp", &[]),
            duplication_certificates: Vec::new(),
            block_hash: hash_sequence("test-block", &[]),
        };
        block.authorization_hash =
            crate::graph::projection_dag::authorize_block_relations(&block, compressed);
        block
    }

    fn same_up_to_sign(left: &SparsePolynomialQ, right: &SparsePolynomialQ) -> bool {
        left == right || poly_add(left, right).terms.is_empty()
    }
}
