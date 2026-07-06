use std::collections::BTreeSet;

use crate::algebra::resultant::{
    build_sparse_resultant_template, compute_resultant_relation, verify_resultant_certificate,
    ModularOptions, ResultantInput, SparseResultantCertificate,
};
use crate::compose::message::{MessageRepresentation, ProjectionMessage, ProjectionStrength};
use crate::graph::projection_dag::ProjectionBlock;
use crate::kernels::traits::{KernelContext, KernelKind, ReplayResult, TargetProjectionKernel};
use crate::planner::admission::{KernelAdmission, KernelAdmissionStatus};
use crate::planner::algebraic_cost::SaturatingCount;
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
use crate::types::ids::{PackageId, RelationId, VariableId};
use crate::types::matrix::matrix_density;
use crate::types::polynomial::{
    clear_denominators_primitive, max_poly_coefficient_height_bits, poly_coefficient_height_bits,
    poly_monomial_count, poly_total_degree, poly_variables, SparsePolynomialQ,
};
use crate::verify::certificates::{
    KernelCertificate, KernelCertificatePayload, SparseResultantProjectionCertificate,
};

pub struct SparseResultantProjectionKernel;

impl TargetProjectionKernel for SparseResultantProjectionKernel {
    fn kind(&self) -> KernelKind {
        KernelKind::SparseResultantProjection
    }

    fn admit(&self, block: &ProjectionBlock, ctx: &KernelContext) -> KernelAdmission {
        admit_sparse_resultant(block, ctx)
    }

    fn plan(
        &self,
        admission: &KernelAdmission,
        ctx: &KernelContext,
        solver_ctx: &SolverContext,
    ) -> Result<KernelExecutionPlan, SolverError> {
        plan_sparse_resultant_from_admission(admission, ctx, solver_ctx)
    }

    fn execute(
        &self,
        plan: &KernelExecutionPlan,
        ctx: &mut KernelContext,
        solver_ctx: &mut SolverContext,
    ) -> Result<ProjectionMessage, SolverError> {
        execute_sparse_resultant(plan, ctx, solver_ctx)
    }

    fn replay(&self, message: &ProjectionMessage, ctx: &KernelContext) -> ReplayResult {
        crate::kernels::traits::exact_replay_result(
            self.kind(),
            "sparse-resultant-replay",
            message,
            ctx,
        )
    }
}

#[derive(Debug, Clone)]
struct SparseResultantTrace {
    relation: SparsePolynomialQ,
    certificates: Vec<SparseResultantCertificate>,
    matrix_rows: usize,
    matrix_cols: usize,
    max_degree: usize,
    output_support_hash: Hash,
    trace_hash: Hash,
}

#[derive(Debug, Clone)]
struct SparseResultantPlanProbe {
    matrix_rows: usize,
    matrix_cols: usize,
    max_degree: usize,
    template_trace_hash: Hash,
    output_support_hash: Hash,
}

#[derive(Debug, Clone)]
struct ResultantRelationInput {
    polynomial: SparsePolynomialQ,
    source_relation_ids: Vec<RelationId>,
    source_hash: Hash,
    child_message_hash: Option<Hash>,
}

pub fn admit_sparse_resultant(block: &ProjectionBlock, ctx: &KernelContext) -> KernelAdmission {
    let inputs = collect_relation_inputs(block, &ctx.system, &ctx.child_messages);
    let relation_polys = inputs
        .iter()
        .map(|input| input.polynomial.clone())
        .collect::<Vec<_>>();
    let exported = sorted_set(&block.exported_variables);
    let eliminated = block
        .local_variables
        .difference(&block.exported_variables)
        .copied()
        .collect::<Vec<_>>();
    let status = if relation_polys.len() >= 2
        && eliminated
            .iter()
            .any(|var| selectable_resultant_pair(&relation_polys, *var, 64).is_some())
    {
        KernelAdmissionStatus::Admitted
    } else {
        KernelAdmissionStatus::Declined {
            reason: "not sparse enough for a finite resultant template".to_owned(),
        }
    };
    finish_admission(block, status, exported, eliminated, None)
}

pub fn plan_sparse_resultant(
    block: &ProjectionBlock,
    system: &CompressedSystemQ,
    solver_ctx: &SolverContext,
    plan_id: crate::types::ids::KernelPlanId,
) -> Result<KernelExecutionPlan, SolverError> {
    plan_sparse_resultant_with_messages(block, system, &[], solver_ctx, plan_id)
}

pub fn plan_sparse_resultant_with_messages(
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
    let exported = sorted_set(&block.exported_variables);
    let eliminated = block
        .local_variables
        .difference(&block.exported_variables)
        .copied()
        .collect::<Vec<_>>();
    let max_dim = sparse_resultant_matrix_cap(solver_ctx, &relation_polys);
    let probe = probe_sparse_resultant_plan(&relation_polys, &eliminated, &exported, max_dim)?;
    let template = template_plan(
        probe.matrix_rows,
        probe.matrix_cols,
        probe.template_trace_hash,
        probe.output_support_hash,
    );
    let mut support_plan = KernelSupportPlan {
        dense_relation_search_schedule: None,
        affine_elimination_order: None,
        template_plan: Some(template),
        rank_plan: Some(rank_plan(probe.matrix_rows.min(probe.matrix_cols))),
        universal_strategy_sequence: Vec::new(),
        degree_bound: probe.max_degree,
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
        max_export_degree: Some(probe.max_degree),
        max_multiplier_total_degree: None,
        max_local_elimination_steps: Some(eliminated.len().max(1)),
        max_memory_bytes: solver_ctx.options.max_memory_bytes,
        bounds_hash: hash_sequence("planner-resource-bounds", &[]),
    };
    bounds.bounds_hash = resource_bounds_hash(&bounds);
    Ok(KernelExecutionPlan::new(
        plan_id,
        block.block_id,
        KernelKind::SparseResultantProjection,
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
        CertificateRoute::SparseResultantExactVerification,
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

pub fn plan_sparse_resultant_from_admission(
    admission: &KernelAdmission,
    ctx: &KernelContext,
    solver_ctx: &SolverContext,
) -> Result<KernelExecutionPlan, SolverError> {
    if admission.kind != KernelKind::SparseResultantProjection
        || !matches!(admission.status, KernelAdmissionStatus::Admitted)
    {
        return Err(implementation_bug(
            "sparse resultant plan requested for non-admitted kernel",
        ));
    }
    if let Some(plan) = &admission.execution_plan {
        return Ok(plan.clone());
    }
    plan_sparse_resultant_with_messages(
        &ctx.block,
        &ctx.system,
        &ctx.child_messages,
        solver_ctx,
        crate::types::ids::KernelPlanId(3),
    )
}

pub fn execute_sparse_resultant(
    plan: &KernelExecutionPlan,
    ctx: &mut KernelContext,
    solver_ctx: &mut SolverContext,
) -> Result<ProjectionMessage, SolverError> {
    validate_sparse_resultant_plan_binding(plan, ctx)?;
    let inputs = planned_relation_inputs(plan, ctx)?;
    let relation_polys = inputs
        .iter()
        .map(|input| input.polynomial.clone())
        .collect::<Vec<_>>();
    let max_dim = plan
        .resource_bounds
        .max_matrix_rows
        .or(solver_ctx.options.max_matrix_rows)
        .unwrap_or_else(|| sparse_resultant_matrix_cap(solver_ctx, &relation_polys));
    let trace = build_sparse_resultant_trace(
        &relation_polys,
        &plan.eliminated_variables,
        &plan.exported_variables,
        max_dim,
    )?;
    let probe = probe_sparse_resultant_plan(
        &relation_polys,
        &plan.eliminated_variables,
        &plan.exported_variables,
        max_dim,
    )?;
    let Some(template) = &plan.support_plan.template_plan else {
        return Err(implementation_bug(
            "sparse resultant plan lacks template plan",
        ));
    };
    if template.matrix_rows != probe.matrix_rows
        || template.matrix_cols != probe.matrix_cols
        || template.row_monomial_hash != probe.template_trace_hash
        || template.column_support_hash != probe.output_support_hash
        || support_plan_hash(&plan.support_plan) != plan.support_plan.support_hash
    {
        return Err(implementation_bug(
            "sparse resultant template plan is not reproducible from authorized relations",
        ));
    }
    if trace.max_degree > plan.support_plan.degree_bound {
        return Err(implementation_bug(
            "sparse resultant execution exceeded planned export degree bound",
        ));
    }
    let exported = plan
        .exported_variables
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    if !poly_variables(&trace.relation).is_subset(&exported) {
        return Err(implementation_bug(
            "sparse resultant produced a relation outside exported variables",
        ));
    }
    let certificate_hash = sparse_resultant_certificate_hash(plan, &trace);
    let cost_trace = ProjectionCostTrace {
        block_id: plan.block_id,
        kernel_kind: KernelKind::SparseResultantProjection,
        local_variable_count: ctx.block.local_variables.len(),
        exported_variable_count: plan.exported_variables.len(),
        local_relation_count: inputs.len(),
        local_monomial_count: relation_polys.iter().map(poly_monomial_count).sum(),
        estimated_quotient_rank: Some(trace.matrix_rows.min(trace.matrix_cols)),
        matrix_rows: Some(trace.matrix_rows),
        matrix_cols: Some(trace.matrix_cols),
        matrix_density: Some(matrix_density(&crate::types::matrix::SparseMatrixQ {
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
        KernelCertificatePayload::SparseResultant(SparseResultantProjectionCertificate {
            source_relations: relation_polys.clone(),
            output_relations: vec![trace.relation.clone()],
            resultant_certificates: trace.certificates.clone(),
        }),
    );
    let mut message = ProjectionMessage {
        package_id: PackageId(plan.plan_id.0),
        block_id: plan.block_id,
        kernel_kind: KernelKind::SparseResultantProjection,
        source_relation_ids: plan.source_relation_ids.clone(),
        eliminated_variables: plan.eliminated_variables.clone(),
        exported_variables: plan.exported_variables.clone(),
        relation_generators: vec![trace.relation],
        representation: MessageRepresentation::SparseResultantMatrix,
        projection_strength: ProjectionStrength::CandidateCoverStrong,
        certificate,
        compression_trace: ctx.system.compression_trace.clone(),
        cost_trace,
        package_hash: hash_sequence("projection-message-initial", &[]),
    };
    message.package_hash = projection_message_hash(&message);
    Ok(message)
}

fn probe_sparse_resultant_plan(
    relations: &[SparsePolynomialQ],
    eliminated: &[VariableId],
    exported: &[VariableId],
    max_dim: usize,
) -> Result<SparseResultantPlanProbe, SolverError> {
    let mut template_hashes = Vec::new();
    let mut matrix_rows = 0usize;
    let mut matrix_cols = 0usize;
    let mut max_degree = relations.iter().map(poly_total_degree).max().unwrap_or(1) as usize;
    for eliminate in eliminated {
        let Some((_left, _right, input)) =
            selectable_resultant_pair(relations, *eliminate, max_dim)
        else {
            continue;
        };
        let template = build_sparse_resultant_template(input)?;
        matrix_rows = matrix_rows.saturating_add(template.matrix_rows);
        matrix_cols = matrix_cols.saturating_add(template.matrix_cols);
        template_hashes.push(template.template_hash);
        max_degree =
            max_degree.saturating_add(template.matrix_rows.max(template.matrix_cols).max(1));
    }
    if template_hashes.is_empty() {
        return Err(algorithmic_hard_case(
            "sparse resultant template chain was not applicable",
        ));
    }
    let template_trace_hash = hash_sequence(
        "sparse-resultant-template-trace",
        &template_hashes
            .iter()
            .map(|hash| hash.0.to_vec())
            .collect::<Vec<_>>(),
    );
    Ok(SparseResultantPlanProbe {
        matrix_rows,
        matrix_cols,
        max_degree,
        template_trace_hash,
        output_support_hash: planned_resultant_output_support_hash(exported, max_degree),
    })
}

fn build_sparse_resultant_trace(
    relations: &[SparsePolynomialQ],
    eliminated: &[VariableId],
    exported: &[VariableId],
    max_dim: usize,
) -> Result<SparseResultantTrace, SolverError> {
    let exported_set = exported.iter().copied().collect::<BTreeSet<_>>();
    let mut current = relations.to_vec();
    let mut certificates = Vec::new();
    let mut template_hashes = Vec::new();
    let mut matrix_rows = 0usize;
    let mut matrix_cols = 0usize;
    for eliminate in eliminated {
        let Some((left, right, input)) = selectable_resultant_pair(&current, *eliminate, max_dim)
        else {
            continue;
        };
        let template = build_sparse_resultant_template(input)?;
        matrix_rows = matrix_rows.saturating_add(template.matrix_rows);
        matrix_cols = matrix_cols.saturating_add(template.matrix_cols);
        template_hashes.push(template.template_hash);
        let resultant = compute_resultant_relation(&template, ModularOptions::default())?;
        if !verify_resultant_certificate(&resultant.certificate) {
            return Err(implementation_bug(
                "sparse resultant certificate failed exact recomputation",
            ));
        }
        let mut next = current
            .into_iter()
            .enumerate()
            .filter_map(|(idx, poly)| {
                if idx == left || idx == right {
                    None
                } else {
                    Some(poly)
                }
            })
            .collect::<Vec<_>>();
        next.push(clear_denominators_primitive(&resultant.relation));
        current = next;
        certificates.push(resultant.certificate);
    }
    let Some(relation) = current
        .iter()
        .filter(|poly| !poly.terms.is_empty() && poly_variables(poly).is_subset(&exported_set))
        .min_by_key(|poly| {
            (
                poly_total_degree(poly),
                poly_monomial_count(poly),
                poly.hash,
            )
        })
        .cloned()
    else {
        return Err(algorithmic_hard_case(
            "no exported sparse resultant relation within declared template chain",
        ));
    };
    if certificates.is_empty() {
        return Err(algorithmic_hard_case(
            "sparse resultant template chain was not applicable",
        ));
    }
    let template_trace_hash = hash_sequence(
        "sparse-resultant-template-trace",
        &template_hashes
            .iter()
            .map(|hash| hash.0.to_vec())
            .collect::<Vec<_>>(),
    );
    let output_support_hash = hash_sequence(
        "sparse-resultant-output-support",
        &relation
            .terms
            .iter()
            .map(|term| crate::types::monomial::monomial_to_bytes(&term.monomial))
            .collect::<Vec<_>>(),
    );
    let trace_hash = hash_sequence(
        "sparse-resultant-execution-trace",
        &[
            template_trace_hash.0.to_vec(),
            output_support_hash.0.to_vec(),
            relation.hash.0.to_vec(),
        ],
    );
    Ok(SparseResultantTrace {
        relation,
        certificates,
        matrix_rows,
        matrix_cols,
        max_degree: current
            .iter()
            .map(|poly| poly_total_degree(poly) as usize)
            .max()
            .unwrap_or(0),
        output_support_hash,
        trace_hash,
    })
}

fn planned_resultant_output_support_hash(exported: &[VariableId], max_degree: usize) -> Hash {
    let mut chunks = Vec::new();
    for variable in exported {
        chunks.push(variable.0.to_be_bytes().to_vec());
    }
    chunks.push(max_degree.to_be_bytes().to_vec());
    hash_sequence("sparse-resultant-planned-output-support", &chunks)
}

fn selectable_resultant_pair(
    relations: &[SparsePolynomialQ],
    eliminate: VariableId,
    max_dim: usize,
) -> Option<(usize, usize, ResultantInput)> {
    let mut candidates = Vec::new();
    for i in 0..relations.len() {
        for j in (i + 1)..relations.len() {
            if degree_in_variable(&relations[i], eliminate) == 0
                || degree_in_variable(&relations[j], eliminate) == 0
            {
                continue;
            }
            let keep = selected_keep_variables(&relations[i], &relations[j], eliminate);
            let input = ResultantInput {
                polynomials: vec![relations[i].clone(), relations[j].clone()],
                eliminate,
                keep_variables: keep,
                max_matrix_dim: max_dim,
            };
            if let Ok(template) = build_sparse_resultant_template(input.clone()) {
                let footprint =
                    resultant_pair_cost(&input, template.matrix_rows, template.matrix_cols);
                if resultant_pair_growth_is_prohibited(&footprint) {
                    continue;
                }
                candidates.push((
                    footprint.predicted_intermediate_terms.as_usize_saturating(),
                    footprint.input_term_count,
                    footprint.keep_variable_count,
                    template.matrix_rows.saturating_mul(template.matrix_cols),
                    relations[i].hash,
                    relations[j].hash,
                    i,
                    j,
                    input,
                ));
            }
        }
    }
    candidates.sort_by_key(|entry| {
        (
            entry.0, entry.1, entry.2, entry.3, entry.4, entry.5, entry.6, entry.7,
        )
    });
    candidates
        .into_iter()
        .next()
        .map(|(_, _, _, _, _, _, i, j, input)| (i, j, input))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResultantPairCost {
    input_term_count: usize,
    max_input_terms: usize,
    keep_variable_count: usize,
    predicted_intermediate_terms: SaturatingCount,
}

fn resultant_pair_cost(
    input: &ResultantInput,
    matrix_rows: usize,
    matrix_cols: usize,
) -> ResultantPairCost {
    let input_term_count = input
        .polynomials
        .iter()
        .map(poly_monomial_count)
        .sum::<usize>()
        .max(1);
    let max_input_terms = input
        .polynomials
        .iter()
        .map(poly_monomial_count)
        .max()
        .unwrap_or(0)
        .max(1);
    let keep_variable_count = input.keep_variables.len();
    let predicted_intermediate_terms = input
        .polynomials
        .iter()
        .map(|poly| SaturatingCount::from_usize(poly_monomial_count(poly).max(1)))
        .fold(SaturatingCount::ONE, |acc, terms| acc.saturating_mul(terms))
        .saturating_mul(SaturatingCount::from_usize(
            matrix_rows.max(1).saturating_mul(matrix_cols.max(1)),
        ))
        .saturating_mul(SaturatingCount::from_usize(keep_variable_count.max(1)));
    ResultantPairCost {
        input_term_count,
        max_input_terms,
        keep_variable_count,
        predicted_intermediate_terms,
    }
}

fn resultant_pair_growth_is_prohibited(cost: &ResultantPairCost) -> bool {
    cost.max_input_terms > 8_192
        || cost.keep_variable_count > 24
        || cost.predicted_intermediate_terms.exceeds_usize(250_000)
}

fn selected_keep_variables(
    left: &SparsePolynomialQ,
    right: &SparsePolynomialQ,
    eliminate: VariableId,
) -> Vec<VariableId> {
    let mut keep = poly_variables(left);
    keep.extend(poly_variables(right));
    keep.remove(&eliminate);
    keep.into_iter().collect()
}

fn degree_in_variable(poly: &SparsePolynomialQ, var: VariableId) -> u32 {
    poly.terms
        .iter()
        .map(|term| {
            term.monomial
                .exponents
                .iter()
                .find(|(candidate, _)| *candidate == var)
                .map_or(0, |(_, exp)| *exp)
        })
        .max()
        .unwrap_or(0)
}

fn sparse_resultant_matrix_cap(ctx: &SolverContext, relations: &[SparsePolynomialQ]) -> usize {
    ctx.options.max_matrix_rows.unwrap_or_else(|| {
        relations
            .iter()
            .map(|poly| poly_total_degree(poly) as usize)
            .sum::<usize>()
            .saturating_add(2)
            .max(4)
    })
}

fn validate_sparse_resultant_plan_binding(
    plan: &KernelExecutionPlan,
    ctx: &KernelContext,
) -> Result<(), SolverError> {
    if plan.kernel_kind != KernelKind::SparseResultantProjection {
        return Err(implementation_bug("sparse resultant kernel kind mismatch"));
    }
    if hash_kernel_execution_plan(plan) != plan.plan_hash {
        return Err(implementation_bug(
            "sparse resultant execution plan hash mismatch",
        ));
    }
    if plan.block_id != ctx.block.block_id {
        return Err(implementation_bug("sparse resultant block id mismatch"));
    }
    if plan.input_block_authorization_hash != ctx.block.authorization_hash {
        return Err(implementation_bug(
            "sparse resultant block authorization hash mismatch",
        ));
    }
    if plan.certificate_route != CertificateRoute::SparseResultantExactVerification {
        return Err(implementation_bug(
            "sparse resultant certificate route mismatch",
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
            "sparse resultant child message hash binding mismatch",
        ));
    }
    Ok(())
}

fn planned_relation_inputs(
    plan: &KernelExecutionPlan,
    ctx: &KernelContext,
) -> Result<Vec<ResultantRelationInput>, SolverError> {
    let source_hashes = plan
        .source_relation_hashes
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let child_hashes = plan
        .child_message_hashes
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let mut out = planned_local_relation_inputs(plan, ctx, &source_hashes);
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
                            .then(|| ResultantRelationInput {
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

fn planned_local_relation_inputs(
    plan: &KernelExecutionPlan,
    ctx: &KernelContext,
    source_hashes: &BTreeSet<Hash>,
) -> Vec<ResultantRelationInput> {
    let relation_ids = plan
        .source_relation_ids
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    ctx.system
        .relations
        .iter()
        .filter(|relation| relation_ids.contains(&relation.id))
        .filter(|relation| source_hashes.contains(&relation.hash))
        .map(|relation| ResultantRelationInput {
            polynomial: relation.polynomial.clone(),
            source_relation_ids: vec![relation.id],
            source_hash: relation.hash,
            child_message_hash: None,
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

fn collect_relation_inputs(
    block: &ProjectionBlock,
    system: &CompressedSystemQ,
    child_messages: &[ProjectionMessage],
) -> Vec<ResultantRelationInput> {
    let mut inputs = block_relations(block, system)
        .into_iter()
        .map(|relation| ResultantRelationInput {
            polynomial: relation.polynomial,
            source_relation_ids: vec![relation.id],
            source_hash: relation.hash,
            child_message_hash: None,
        })
        .collect::<Vec<_>>();
    for message in child_messages {
        for relation in &message.relation_generators {
            inputs.push(ResultantRelationInput {
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
    inputs: &[ResultantRelationInput],
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
            "sparse resultant source relation hash mismatch",
        ));
    }
    Ok(())
}

fn sparse_resultant_certificate_hash(
    plan: &KernelExecutionPlan,
    trace: &SparseResultantTrace,
) -> Hash {
    let mut chunks = vec![
        plan.plan_hash.0.to_vec(),
        trace.trace_hash.0.to_vec(),
        trace.output_support_hash.0.to_vec(),
    ];
    for cert in &trace.certificates {
        chunks.push(resultant_certificate_hash(cert).0.to_vec());
    }
    hash_sequence("sparse-resultant-kernel-certificate", &chunks)
}

fn resultant_certificate_hash(cert: &SparseResultantCertificate) -> Hash {
    let mut chunks = vec![cert.template_hash.0.to_vec(), cert.relation_hash.0.to_vec()];
    for trace in &cert.modular_traces {
        chunks.push(trace.prime.to_be_bytes().to_vec());
        chunks.push(trace.relation_mod_hash.0.to_vec());
    }
    hash_sequence("sparse-resultant-certificate", &chunks)
}

fn finish_admission(
    block: &ProjectionBlock,
    status: KernelAdmissionStatus,
    exported_variables: Vec<VariableId>,
    eliminated_variables: Vec<VariableId>,
    execution_plan: Option<KernelExecutionPlan>,
) -> KernelAdmission {
    let mut chunks = vec![
        b"SparseResultantProjection".to_vec(),
        block.block_id.0.to_be_bytes().to_vec(),
        format!("{status:?}").into_bytes(),
    ];
    if let Some(plan) = &execution_plan {
        chunks.push(plan.plan_hash.0.to_vec());
    }
    KernelAdmission {
        kind: KernelKind::SparseResultantProjection,
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
            stage: StageId("SparseResultantProjectionKernel".to_owned()),
            reason: AlgebraicReason(reason.to_owned()),
            minimal_block_hash: hash_sequence(
                "sparse-resultant-hard-case",
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
    use crate::planner::kernel_plan::CertificateRoute;
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
    use crate::types::monomial::normalize_monomial;
    use crate::types::polynomial::{
        constant_poly, normalize_poly, poly_sub, poly_variables, variable_poly, SparsePolynomialQ,
        TermQ,
    };
    use crate::types::rational::int_q;
    use crate::verify::certificates::KernelCertificate;

    use super::*;

    #[test]
    fn p8b_sparse_resultant_kernel_produces_exact_exported_relation() {
        let t = VariableId(0);
        let y = VariableId(1);
        let relations = vec![
            poly_sub(&variable_poly(y), &variable_poly(t)),
            poly_sub(&variable_poly(y), &constant_poly(int_q(1))),
        ];
        let compressed = compressed_system(vec![t, y], t, relations);
        let block = test_block(&compressed, [t, y], [t]);
        let mut solver_ctx = new_context(SolverOptions::default());
        let mut kctx = KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };
        let kernel = SparseResultantProjectionKernel;
        let admission = kernel.admit(&kctx.block, &kctx);

        assert!(admission.is_admitted());
        let plan = kernel.plan(&admission, &kctx, &solver_ctx).unwrap();
        let message = kernel.execute(&plan, &mut kctx, &mut solver_ctx).unwrap();

        assert_eq!(message.kernel_kind, KernelKind::SparseResultantProjection);
        assert_eq!(
            message.representation,
            MessageRepresentation::SparseResultantMatrix
        );
        assert_eq!(
            message.projection_strength,
            ProjectionStrength::CandidateCoverStrong
        );
        let exported = [t].into_iter().collect::<BTreeSet<_>>();
        assert!(message
            .relation_generators
            .iter()
            .all(|poly| poly_variables(poly).is_subset(&exported)));
        assert!(kernel.replay(&message, &kctx).accepted);
    }

    #[test]
    fn p8b_sparse_resultant_rejects_source_hash_tamper() {
        let t = VariableId(0);
        let y = VariableId(1);
        let relations = vec![
            poly_sub(&variable_poly(y), &variable_poly(t)),
            poly_sub(&variable_poly(y), &constant_poly(int_q(1))),
        ];
        let compressed = compressed_system(vec![t, y], t, relations);
        let block = test_block(&compressed, [t, y], [t]);
        let solver_ctx = new_context(SolverOptions::default());
        let mut kctx = KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };
        let kernel = SparseResultantProjectionKernel;
        let admission = kernel.admit(&kctx.block, &kctx);
        let mut plan = kernel.plan(&admission, &kctx, &solver_ctx).unwrap();

        plan.source_relation_hashes[0] = hash_sequence("tampered-source", &[]);
        let mut execute_ctx = new_context(SolverOptions::default());
        let err = kernel
            .execute(&plan, &mut kctx, &mut execute_ctx)
            .unwrap_err();

        assert_eq!(err.public_status(), SolverStatus::ImplementationBug);
    }

    #[test]
    fn p8b_sparse_resultant_consumes_child_message_relations_and_rejects_tamper() {
        let t = VariableId(0);
        let y = VariableId(1);
        let relations = vec![poly_sub(&variable_poly(y), &variable_poly(t))];
        let compressed = compressed_system(vec![t, y], t, relations);
        let mut block = test_block(&compressed, [t, y], [t]);
        block.child_block_ids = vec![BlockId(7)];
        block.authorization_hash =
            crate::graph::projection_dag::authorize_block_relations(&block, &compressed);
        let child_relation = poly_sub(&variable_poly(y), &constant_poly(int_q(1)));
        let child_message = child_projection_message(&compressed, child_relation);
        let mut solver_ctx = new_context(SolverOptions::default());
        let mut kctx = KernelContext {
            block,
            system: compressed,
            child_messages: vec![child_message],
        };
        let kernel = SparseResultantProjectionKernel;
        let admission = kernel.admit(&kctx.block, &kctx);
        assert!(admission.is_admitted());
        let plan = kernel.plan(&admission, &kctx, &solver_ctx).unwrap();
        assert_eq!(
            plan.child_message_hashes,
            vec![kctx.child_messages[0].package_hash]
        );
        let message = kernel.execute(&plan, &mut kctx, &mut solver_ctx).unwrap();
        assert_eq!(message.kernel_kind, KernelKind::SparseResultantProjection);

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
    fn p12g_sparse_resultant_template_plan_does_not_overclaim_binary_chain() {
        let t = VariableId(0);
        let y = VariableId(1);
        let relations = vec![
            poly_sub(&variable_poly(y), &variable_poly(t)),
            poly_sub(&variable_poly(y), &constant_poly(int_q(1))),
        ];
        let compressed = compressed_system(vec![t, y], t, relations);
        let block = test_block(&compressed, [t, y], [t]);
        let solver_ctx = new_context(SolverOptions::default());
        let kctx = KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };
        let kernel = SparseResultantProjectionKernel;
        let plan = kernel
            .plan(&kernel.admit(&kctx.block, &kctx), &kctx, &solver_ctx)
            .unwrap();

        assert_eq!(plan.kernel_kind, KernelKind::SparseResultantProjection);
        assert!(plan.support_plan.template_plan.is_some());
        assert!(plan.support_plan.rank_plan.is_some());
        assert_eq!(
            plan.certificate_route,
            CertificateRoute::SparseResultantExactVerification
        );
        assert_eq!(
            plan.plan_work_classification,
            crate::planner::kernel_plan::PlanWorkClassification::PurePlan
        );
        assert_eq!(plan.exported_variables, vec![t]);
        assert_eq!(plan.eliminated_variables, vec![y]);
        assert_eq!(plan.source_relation_hashes.len(), 2);
    }

    #[test]
    fn acr_p2_sparse_resultant_pair_scoring_rejects_large_entry_small_matrix() {
        let y = VariableId(20);
        let keep_variables = (0..12).map(VariableId).collect::<Vec<_>>();
        let relations = vec![
            dense_linear_polynomial(y, &keep_variables, 300),
            dense_linear_polynomial(y, &keep_variables, 700),
        ];

        assert!(selectable_resultant_pair(&relations, y, 4).is_none());
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
    ) -> ProjectionMessage {
        let certificate_hash = hash_sequence("test-child-certificate", &[relation.hash.0.to_vec()]);
        let mut message = ProjectionMessage {
            package_id: PackageId(701),
            block_id: BlockId(7),
            kernel_kind: KernelKind::TargetUnivariate,
            source_relation_ids: Vec::new(),
            eliminated_variables: Vec::new(),
            exported_variables: vec![compressed.target],
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

    fn dense_linear_polynomial(
        eliminate: VariableId,
        keep_variables: &[VariableId],
        offset: usize,
    ) -> SparsePolynomialQ {
        let terms = (0..300)
            .map(|idx| {
                let mut exponents = vec![(eliminate, 1)];
                let code = idx + offset;
                for (bit, variable) in keep_variables.iter().enumerate() {
                    if (code >> bit) & 1 == 1 {
                        exponents.push((*variable, 1));
                    }
                }
                TermQ {
                    coeff: int_q(1),
                    monomial: normalize_monomial(exponents),
                }
            })
            .collect::<Vec<_>>();
        normalize_poly(SparsePolynomialQ {
            terms,
            hash: hash_sequence("poly", &[]),
        })
    }
}
