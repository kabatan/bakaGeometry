use std::collections::BTreeSet;

use crate::algebra::regular_chain::{
    combine_chain_projections, local_regular_chain_decomposition, project_chain_to_variables,
    ProjectionGenerators, RegularChainDAG, RegularChainInput, UnionSemantics,
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
use crate::preprocess::compression::{CompressedSystemQ, GuardRecord};
use crate::problem::canonicalize::CanonicalRelationQ;
use crate::problem::context::SolverContext;
use crate::result::cost_trace::ProjectionCostTrace;
use crate::result::status::{
    AlgebraicReason, FailureKind, SolverError, SolverErrorKind, SolverStatus, StageId,
};
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::{KernelPlanId, PackageId, RelationId, VariableId};
use crate::types::polynomial::{
    clear_denominators_primitive, max_poly_coefficient_height_bits, poly_monomial_count,
    poly_total_degree, poly_variables, SparsePolynomialQ,
};
use crate::verify::certificates::{
    KernelCertificate, KernelCertificatePayload, RegularChainProjectionCertificate,
};

pub struct RegularChainProjectionKernel;

impl TargetProjectionKernel for RegularChainProjectionKernel {
    fn kind(&self) -> KernelKind {
        KernelKind::RegularChainProjection
    }

    fn admit(&self, block: &ProjectionBlock, ctx: &KernelContext) -> KernelAdmission {
        admit_regular_chain_projection(block, ctx)
    }

    fn plan(
        &self,
        admission: &KernelAdmission,
        ctx: &KernelContext,
        solver_ctx: &SolverContext,
    ) -> Result<KernelExecutionPlan, SolverError> {
        plan_regular_chain_projection_from_admission(admission, ctx, solver_ctx)
    }

    fn execute(
        &self,
        plan: &KernelExecutionPlan,
        ctx: &mut KernelContext,
        solver_ctx: &mut SolverContext,
    ) -> Result<ProjectionMessage, SolverError> {
        execute_regular_chain_projection(plan, ctx, solver_ctx)
    }

    fn replay(&self, message: &ProjectionMessage, ctx: &KernelContext) -> ReplayResult {
        crate::kernels::traits::exact_replay_result(
            self.kind(),
            "regular-chain-replay",
            message,
            ctx,
        )
    }
}

#[derive(Debug, Clone)]
struct RegularChainRelationInput {
    polynomial: SparsePolynomialQ,
    source_relation_ids: Vec<RelationId>,
    source_hash: Hash,
}

#[derive(Debug, Clone)]
struct RegularChainProjectionTrace {
    dag: RegularChainDAG,
    projections: Vec<ProjectionGenerators>,
    generators: Vec<SparsePolynomialQ>,
    max_degree: usize,
    component_hash: Hash,
    trace_hash: Hash,
}

#[derive(Debug, Clone)]
struct RegularChainPlanProbe {
    row_count: usize,
    column_count: usize,
    max_degree: usize,
    source_shape_hash: Hash,
    output_support_hash: Hash,
}

pub fn admit_regular_chain_projection(
    block: &ProjectionBlock,
    ctx: &KernelContext,
) -> KernelAdmission {
    match plan_regular_chain_projection(
        block,
        &ctx.system,
        &SolverContext::new(Default::default()),
        KernelPlanId(KernelKind::RegularChainProjection as u32),
    ) {
        Ok(plan) => finish_admission(block, KernelAdmissionStatus::Admitted, Some(plan)),
        Err(_) => finish_admission(
            block,
            KernelAdmissionStatus::Declined {
                reason: "no triangular regular-chain projection plan is applicable".to_owned(),
            },
            None,
        ),
    }
}

pub fn plan_regular_chain_projection(
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
    let variables = sorted_set(&block.local_variables);
    let probe = probe_regular_chain_plan(&relations, &system.guards, &variables, &exported)?;
    let mut support_plan = KernelSupportPlan {
        dense_relation_search_schedule: None,
        affine_elimination_order: None,
        template_plan: Some(template_plan(
            probe.row_count,
            probe.column_count,
            probe.source_shape_hash,
            probe.output_support_hash,
        )),
        rank_plan: Some(rank_plan(probe.column_count)),
        universal_strategy_sequence: Vec::new(),
        degree_bound: probe.max_degree,
        support_hash: hash_sequence("kernel-support-plan", &[]),
    };
    support_plan.support_hash = support_plan_hash(&support_plan);
    let mut bounds = ResourceBounds {
        max_matrix_rows: solver_ctx.options.max_matrix_rows.or(Some(probe.row_count)),
        max_matrix_cols: solver_ctx
            .options
            .max_matrix_cols
            .or(Some(probe.column_count)),
        max_export_degree: Some(probe.max_degree),
        max_multiplier_total_degree: None,
        max_local_elimination_steps: Some(relations.len().saturating_mul(variables.len()).max(1)),
        max_memory_bytes: solver_ctx.options.max_memory_bytes,
        bounds_hash: hash_sequence("planner-resource-bounds", &[]),
    };
    bounds.bounds_hash = resource_bounds_hash(&bounds);
    Ok(KernelExecutionPlan::new(
        plan_id,
        block.block_id,
        KernelKind::RegularChainProjection,
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
        CertificateRoute::RegularChainGuardedProjection,
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

pub fn plan_regular_chain_projection_from_admission(
    admission: &KernelAdmission,
    ctx: &KernelContext,
    solver_ctx: &SolverContext,
) -> Result<KernelExecutionPlan, SolverError> {
    if admission.kind != KernelKind::RegularChainProjection
        || !matches!(admission.status, KernelAdmissionStatus::Admitted)
    {
        return Err(implementation_bug(
            "regular-chain plan requested for non-admitted kernel",
        ));
    }
    if let Some(plan) = &admission.execution_plan {
        return Ok(plan.clone());
    }
    plan_regular_chain_projection(
        &ctx.block,
        &ctx.system,
        solver_ctx,
        KernelPlanId(KernelKind::RegularChainProjection as u32),
    )
}

pub fn execute_regular_chain_projection(
    plan: &KernelExecutionPlan,
    ctx: &mut KernelContext,
    solver_ctx: &mut SolverContext,
) -> Result<ProjectionMessage, SolverError> {
    crate::problem::context::check_resource(
        solver_ctx,
        StageId("RegularChainProjection::execute_start".to_owned()),
    )?;
    validate_regular_chain_plan_binding(plan, ctx)?;
    let inputs = planned_relation_inputs(plan, ctx)?;
    let relations = inputs
        .iter()
        .map(|input| input.polynomial.clone())
        .collect::<Vec<_>>();
    crate::problem::context::check_resource_work(
        solver_ctx,
        StageId("RegularChainProjection::inputs_materialized".to_owned()),
        relations
            .iter()
            .map(poly_monomial_count)
            .sum::<usize>()
            .max(1) as u128,
    )?;
    let trace = build_regular_chain_trace(
        &relations,
        &ctx.system.guards,
        &sorted_set(&ctx.block.local_variables),
        &plan.exported_variables,
    )?;
    crate::problem::context::check_resource_work(
        solver_ctx,
        StageId("RegularChainProjection::trace_built".to_owned()),
        trace
            .dag
            .chains
            .len()
            .max(1)
            .saturating_mul(trace.generators.len().max(1)) as u128,
    )?;
    let Some(template) = &plan.support_plan.template_plan else {
        return Err(implementation_bug("regular-chain plan lacks template plan"));
    };
    let planned_source_hash = planned_regular_chain_source_shape_hash(
        &relations,
        &ctx.system.guards,
        &sorted_set(&ctx.block.local_variables),
    );
    if template.row_monomial_hash != planned_source_hash
        || template.column_support_hash
            != planned_regular_chain_output_support_hash(
                &plan.exported_variables,
                plan.support_plan.degree_bound,
            )
    {
        return Err(implementation_bug(
            "regular-chain projection trace does not match plan support",
        ));
    }
    if trace.max_degree > plan.support_plan.degree_bound {
        return Err(implementation_bug(
            "regular-chain execution exceeded planned export degree bound",
        ));
    }
    validate_exported_generators(&trace.generators, &plan.exported_variables)?;
    let certificate_hash = regular_chain_certificate_hash(plan, &trace);
    let cost_trace = ProjectionCostTrace {
        block_id: plan.block_id,
        kernel_kind: KernelKind::RegularChainProjection,
        local_variable_count: ctx.block.local_variables.len(),
        exported_variable_count: plan.exported_variables.len(),
        local_relation_count: relations.len(),
        local_monomial_count: relations.iter().map(poly_monomial_count).sum(),
        estimated_quotient_rank: Some(trace.dag.chains.len()),
        matrix_rows: Some(trace.dag.chains.len().max(1)),
        matrix_cols: Some(trace.generators.len().max(1)),
        matrix_density: None,
        coefficient_height_before_bits: max_poly_coefficient_height_bits(&relations),
        coefficient_height_after_bits: max_poly_coefficient_height_bits(&trace.generators),
        route_cost: Some(ProjectionCostTrace::route_cost_from_plan(plan)),
    };
    let certificate = KernelCertificate::from_execution_plan_with_payload(
        plan,
        &trace.generators,
        certificate_hash,
        KernelCertificatePayload::RegularChain(RegularChainProjectionCertificate {
            source_relations: relations.clone(),
            variables: sorted_set(&ctx.block.local_variables),
            exported_variables: plan.exported_variables.clone(),
            guards: ctx
                .system
                .guards
                .iter()
                .map(|guard| guard.factor.clone())
                .collect(),
            dag: trace.dag.clone(),
            projections: trace.projections.clone(),
            output_relations: trace.generators.clone(),
        }),
    );
    let mut message = ProjectionMessage {
        package_id: PackageId(plan.plan_id.0),
        block_id: plan.block_id,
        kernel_kind: KernelKind::RegularChainProjection,
        source_relation_ids: plan.source_relation_ids.clone(),
        eliminated_variables: plan.eliminated_variables.clone(),
        exported_variables: plan.exported_variables.clone(),
        relation_generators: trace.generators,
        representation: MessageRepresentation::TriangularChain,
        projection_strength: ProjectionStrength::CandidateCoverStrong,
        certificate,
        compression_trace: ctx.system.compression_trace.clone(),
        cost_trace,
        package_hash: hash_sequence("projection-message-initial", &[]),
    };
    message.package_hash = projection_message_hash(&message);
    Ok(message)
}

fn probe_regular_chain_plan(
    relations: &[SparsePolynomialQ],
    guards: &[GuardRecord],
    variables: &[VariableId],
    exported: &[VariableId],
) -> Result<RegularChainPlanProbe, SolverError> {
    if relations.is_empty() || exported.is_empty() {
        return Err(algorithmic_hard_case(
            "regular-chain projection plan requires local relations and exported variables",
        ));
    }
    let max_degree = relations.iter().map(poly_total_degree).max().unwrap_or(1) as usize;
    Ok(RegularChainPlanProbe {
        row_count: relations.len().max(1),
        column_count: exported.len().max(1),
        max_degree,
        source_shape_hash: planned_regular_chain_source_shape_hash(relations, guards, variables),
        output_support_hash: planned_regular_chain_output_support_hash(exported, max_degree),
    })
}

fn build_regular_chain_trace(
    relations: &[SparsePolynomialQ],
    guards: &[GuardRecord],
    variables: &[VariableId],
    exported: &[VariableId],
) -> Result<RegularChainProjectionTrace, SolverError> {
    let dag = local_regular_chain_decomposition(RegularChainInput {
        relations: relations.to_vec(),
        variables: variables.to_vec(),
        guards: guards.iter().map(|guard| guard.factor.clone()).collect(),
        semantics: UnionSemantics::ComponentUnion,
    })?;
    let projections = dag
        .chains
        .iter()
        .map(|chain| project_chain_to_variables(chain, exported))
        .collect::<Result<Vec<_>, _>>()?;
    let generators = combine_chain_projections(&projections, dag.semantics)?
        .into_iter()
        .filter(|generator| !generator.terms.is_empty())
        .map(|generator| clear_denominators_primitive(&generator))
        .collect::<Vec<_>>();
    if generators.is_empty() {
        return Err(algorithmic_hard_case(
            "regular-chain projection produced no exported generator",
        ));
    }
    validate_exported_generators(&generators, exported)?;
    let max_degree = generators.iter().map(poly_total_degree).max().unwrap_or(1) as usize;
    let component_hash = hash_component_projections(&projections, &generators, dag.semantics);
    let trace_hash = hash_sequence(
        "regular-chain-projection-trace",
        &[
            dag.dag_hash.0.to_vec(),
            component_hash.0.to_vec(),
            hash_polys(&generators).0.to_vec(),
        ],
    );
    Ok(RegularChainProjectionTrace {
        dag,
        projections,
        generators,
        max_degree,
        component_hash,
        trace_hash,
    })
}

fn planned_regular_chain_source_shape_hash(
    relations: &[SparsePolynomialQ],
    guards: &[GuardRecord],
    variables: &[VariableId],
) -> Hash {
    let mut chunks = Vec::new();
    for relation in relations {
        chunks.push(relation.hash.0.to_vec());
    }
    chunks.push(Vec::new());
    for guard in guards {
        chunks.push(guard.guard_hash.0.to_vec());
    }
    chunks.push(Vec::new());
    for variable in variables {
        chunks.push(variable.0.to_be_bytes().to_vec());
    }
    hash_sequence("regular-chain-planned-source-shape", &chunks)
}

fn planned_regular_chain_output_support_hash(exported: &[VariableId], max_degree: usize) -> Hash {
    let mut chunks = Vec::new();
    for variable in exported {
        chunks.push(variable.0.to_be_bytes().to_vec());
    }
    chunks.push(max_degree.to_be_bytes().to_vec());
    hash_sequence("regular-chain-planned-output-support", &chunks)
}

fn validate_regular_chain_plan_binding(
    plan: &KernelExecutionPlan,
    ctx: &KernelContext,
) -> Result<(), SolverError> {
    if plan.kernel_kind != KernelKind::RegularChainProjection {
        return Err(implementation_bug(
            "regular-chain execution received wrong plan kind",
        ));
    }
    if hash_kernel_execution_plan(plan) != plan.plan_hash {
        return Err(implementation_bug(
            "regular-chain execution plan hash mismatch",
        ));
    }
    if plan.block_id != ctx.block.block_id {
        return Err(implementation_bug("regular-chain block id mismatch"));
    }
    if plan.input_block_authorization_hash != ctx.block.authorization_hash {
        return Err(implementation_bug(
            "regular-chain block authorization hash mismatch",
        ));
    }
    if plan.certificate_route != CertificateRoute::RegularChainGuardedProjection {
        return Err(implementation_bug(
            "regular-chain certificate route mismatch",
        ));
    }
    if support_plan_hash(&plan.support_plan) != plan.support_plan.support_hash {
        return Err(implementation_bug("regular-chain support hash mismatch"));
    }
    Ok(())
}

fn planned_relation_inputs(
    plan: &KernelExecutionPlan,
    ctx: &KernelContext,
) -> Result<Vec<RegularChainRelationInput>, SolverError> {
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
        .map(|relation| RegularChainRelationInput {
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
) -> Vec<RegularChainRelationInput> {
    block_relations(block, system)
        .into_iter()
        .map(|relation| RegularChainRelationInput {
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
    inputs: &[RegularChainRelationInput],
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
            "regular-chain source relation hash mismatch",
        ));
    }
    Ok(())
}

fn validate_exported_generators(
    generators: &[SparsePolynomialQ],
    exported: &[VariableId],
) -> Result<(), SolverError> {
    let exported = exported.iter().copied().collect::<BTreeSet<_>>();
    for generator in generators {
        if !poly_variables(generator).is_subset(&exported) {
            return Err(implementation_bug(
                "regular-chain projection exported a non-keep variable",
            ));
        }
    }
    Ok(())
}

fn regular_chain_certificate_hash(
    plan: &KernelExecutionPlan,
    trace: &RegularChainProjectionTrace,
) -> Hash {
    let mut chunks = vec![
        plan.plan_hash.0.to_vec(),
        trace.dag.dag_hash.0.to_vec(),
        trace.component_hash.0.to_vec(),
        trace.trace_hash.0.to_vec(),
    ];
    for projection in &trace.projections {
        chunks.push(projection.projection_hash.0.to_vec());
        for guard in &projection.guards {
            chunks.push(guard.hash.0.to_vec());
        }
    }
    for generator in &trace.generators {
        chunks.push(generator.hash.0.to_vec());
    }
    hash_sequence("regular-chain-kernel-certificate", &chunks)
}

fn hash_component_projections(
    projections: &[ProjectionGenerators],
    generators: &[SparsePolynomialQ],
    semantics: UnionSemantics,
) -> Hash {
    let mut chunks = vec![vec![semantics as u8]];
    for projection in projections {
        chunks.push(projection.projection_hash.0.to_vec());
    }
    for generator in generators {
        chunks.push(generator.hash.0.to_vec());
    }
    hash_sequence("regular-chain-component-projections", &chunks)
}

fn hash_polys(polys: &[SparsePolynomialQ]) -> Hash {
    hash_sequence(
        "regular-chain-output-generators",
        &polys
            .iter()
            .map(|poly| poly.hash.0.to_vec())
            .collect::<Vec<_>>(),
    )
}

fn finish_admission(
    block: &ProjectionBlock,
    status: KernelAdmissionStatus,
    execution_plan: Option<KernelExecutionPlan>,
) -> KernelAdmission {
    let mut chunks = vec![
        b"RegularChainProjection".to_vec(),
        block.block_id.0.to_be_bytes().to_vec(),
        format!("{status:?}").into_bytes(),
    ];
    if let Some(plan) = &execution_plan {
        chunks.push(plan.plan_hash.0.to_vec());
    }
    KernelAdmission {
        kind: KernelKind::RegularChainProjection,
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
    crate::compose::message::hash_projection_message(message)
}

fn sorted_set(vars: &BTreeSet<VariableId>) -> Vec<VariableId> {
    vars.iter().copied().collect()
}

fn algorithmic_hard_case(reason: &str) -> SolverError {
    SolverError {
        target: None,
        kind: SolverErrorKind::Failure(FailureKind::AlgorithmicHardCase {
            stage: StageId("RegularChainProjectionKernel".to_owned()),
            reason: AlgebraicReason(reason.to_owned()),
            minimal_block_hash: hash_sequence(
                "regular-chain-kernel-hard-case",
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
    use crate::kernels::traits::TargetProjectionKernel;
    use crate::preprocess::compression::{CompressionState, GuardKind, GuardRecord};
    use crate::problem::canonicalize::canonicalize_system;
    use crate::problem::context::new_context;
    use crate::problem::input::make_problem;
    use crate::problem::validate::validate_input;
    use crate::solver::options::SolverOptions;
    use crate::types::hash::hash_sequence;
    use crate::types::ids::{BlockId, RelationId};
    use crate::types::polynomial::{constant_poly, poly_add, poly_mul, poly_sub, variable_poly};
    use crate::types::rational::int_q;

    use super::*;

    #[test]
    fn p9_regular_chain_kernel_projects_triangular_component() {
        let t = VariableId(0);
        let y = VariableId(1);
        let compressed = compressed_system(
            vec![t, y],
            t,
            vec![
                poly_sub(&variable_poly(y), &variable_poly(t)),
                poly_sub(
                    &poly_mul(&variable_poly(t), &variable_poly(t)),
                    &constant_poly(int_q(2)),
                ),
            ],
        );
        let block = test_block(&compressed, [t, y], [t]);
        let mut solver_ctx = new_context(SolverOptions::default());
        let mut kctx = KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };
        let kernel = RegularChainProjectionKernel;
        let admission = kernel.admit(&kctx.block, &kctx);
        assert!(admission.is_admitted());
        let plan = kernel.plan(&admission, &kctx, &solver_ctx).unwrap();
        let message = kernel.execute(&plan, &mut kctx, &mut solver_ctx).unwrap();

        assert_eq!(message.kernel_kind, KernelKind::RegularChainProjection);
        assert!(matches!(
            message.representation,
            MessageRepresentation::TriangularChain
        ));
        let exported = [t].into_iter().collect();
        assert!(message
            .relation_generators
            .iter()
            .all(|relation| poly_variables(relation).is_subset(&exported)));
        assert!(message
            .relation_generators
            .iter()
            .any(|relation| same_up_to_sign(
                relation,
                &poly_sub(
                    &poly_mul(&variable_poly(t), &variable_poly(t)),
                    &constant_poly(int_q(2)),
                ),
            )));
        assert!(kernel.replay(&message, &kctx).accepted);
    }

    #[test]
    fn p9_regular_chain_rejects_auth_source_and_plan_tamper() {
        let t = VariableId(0);
        let y = VariableId(1);
        let compressed = compressed_system(
            vec![t, y],
            t,
            vec![
                poly_sub(&variable_poly(y), &variable_poly(t)),
                poly_sub(
                    &poly_mul(&variable_poly(t), &variable_poly(t)),
                    &constant_poly(int_q(2)),
                ),
            ],
        );
        let block = test_block(&compressed, [t, y], [t]);
        let mut solver_ctx = new_context(SolverOptions::default());
        let kctx = KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };
        let kernel = RegularChainProjectionKernel;
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

        let mut stale = kctx.clone();
        stale.system.relations.pop();
        assert_eq!(
            kernel
                .execute(&plan, &mut stale, &mut solver_ctx)
                .unwrap_err()
                .public_status(),
            SolverStatus::ImplementationBug
        );
    }

    #[test]
    fn p9_regular_chain_guard_binding_is_operational() {
        let t = VariableId(0);
        let y = VariableId(1);
        let mut compressed = compressed_system(
            vec![t, y],
            t,
            vec![
                poly_sub(&variable_poly(y), &variable_poly(t)),
                poly_sub(
                    &poly_mul(&variable_poly(t), &variable_poly(t)),
                    &constant_poly(int_q(2)),
                ),
            ],
        );
        compressed.guards.push(test_guard(
            poly_add(&variable_poly(t), &constant_poly(int_q(1))),
            vec![RelationId(0)],
        ));
        let block = test_block(&compressed, [t, y], [t]);
        let mut solver_ctx = new_context(SolverOptions::default());
        let mut kctx = KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };
        let kernel = RegularChainProjectionKernel;
        let plan = kernel
            .plan(&kernel.admit(&kctx.block, &kctx), &kctx, &solver_ctx)
            .unwrap();
        let guarded = kernel
            .execute(&plan, &mut kctx.clone(), &mut solver_ctx)
            .unwrap();

        let mut unguarded = kctx.clone();
        unguarded.system.guards.clear();
        assert_eq!(
            kernel
                .execute(&plan, &mut unguarded, &mut solver_ctx)
                .unwrap_err()
                .public_status(),
            SolverStatus::ImplementationBug
        );

        kctx.system.guards[0] = test_guard(
            poly_add(&variable_poly(t), &constant_poly(int_q(2))),
            vec![RelationId(0)],
        );
        let amended_plan = kernel
            .plan(&kernel.admit(&kctx.block, &kctx), &kctx, &solver_ctx)
            .unwrap();
        let amended = kernel
            .execute(&amended_plan, &mut kctx, &mut solver_ctx)
            .unwrap();
        assert_ne!(
            guarded.certificate.certificate_hash,
            amended.certificate.certificate_hash
        );
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

    fn test_guard(factor: SparsePolynomialQ, source_relation_ids: Vec<RelationId>) -> GuardRecord {
        let guard_kind = GuardKind::ExplicitNonZeroWitness;
        let mut chunks = vec![vec![guard_kind as u8], factor.hash.0.to_vec()];
        for id in &source_relation_ids {
            chunks.push(id.0.to_be_bytes().to_vec());
        }
        GuardRecord {
            factor,
            source_relation_ids,
            guard_kind,
            guard_hash: hash_sequence("compression-guard", &chunks),
        }
    }
}
