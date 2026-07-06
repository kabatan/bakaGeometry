use std::collections::BTreeSet;

use crate::algebra::resultant::{
    build_sparse_resultant_template, compute_resultant_relation, verify_resultant_certificate,
    ModularOptions, ResultantInput, SparseResultantCertificate,
};
use crate::compose::message::{MessageRepresentation, ProjectionMessage, ProjectionStrength};
use crate::graph::projection_dag::ProjectionBlock;
use crate::kernels::traits::{KernelContext, KernelKind, ReplayResult, TargetProjectionKernel};
use crate::planner::admission::{KernelAdmission, KernelAdmissionStatus};
use crate::planner::algebraic_cost::{AlgebraicWorkEstimate, RouteBudget, SaturatingCount};
use crate::planner::kernel_plan::{
    hash_kernel_execution_plan, planned_failure_behavior, rank_plan, resource_bounds_hash,
    support_plan_hash, template_plan, CertificateRoute, KernelExecutionPlan, KernelSupportPlan,
    LocalNonfinitePolicy, PlanWorkClassification, ResourceBounds,
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
use crate::types::monomial::normalize_monomial;
use crate::types::polynomial::{
    clear_denominators_primitive, max_poly_coefficient_height_bits, normalize_poly,
    poly_coefficient_height_bits, poly_monomial_count, poly_total_degree, poly_variables,
    SparsePolynomialQ, TermQ,
};
use crate::types::rational::int_q;
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
    swell_preflight: SparseResultantSwellPreflight,
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
    swell_preflight: SparseResultantSwellPreflight,
    template_trace_hash: Hash,
    output_support_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SparseResultantSwellPreflight {
    pair_preflights: Vec<SparseResultantPairPreflight>,
    input_term_count_per_pair: Vec<usize>,
    max_pair_input_terms: usize,
    max_term_count_after_coefficient_multiplication: SaturatingCount,
    estimated_resultant_template_support: SaturatingCount,
    keep_variable_count: usize,
    coefficient_height_growth_bits: SaturatingCount,
    predicted_intermediate_terms: SaturatingCount,
    predicted_output_terms: SaturatingCount,
    route_work_units: SaturatingCount,
    preflight_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SparseResultantPairPreflight {
    eliminate: VariableId,
    left_term_count: usize,
    right_term_count: usize,
    left_degree: usize,
    right_degree: usize,
    keep_variable_count: usize,
    matrix_rows: usize,
    matrix_cols: usize,
    determinant_entry_term_product: SaturatingCount,
    output_term_upper_bound: SaturatingCount,
    coefficient_height_growth_bits: SaturatingCount,
    predicted_intermediate_terms: SaturatingCount,
    route_work_units: SaturatingCount,
    pair_hash: Hash,
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
    let algebraic_work_estimate = AlgebraicWorkEstimate::new(
        block.local_variables.len(),
        inputs.len(),
        exported.len(),
        relation_polys
            .iter()
            .map(poly_monomial_count)
            .sum::<usize>(),
        relation_polys
            .iter()
            .map(poly_monomial_count)
            .max()
            .unwrap_or(0)
            .max(1),
        probe.max_degree.max(1),
        probe
            .swell_preflight
            .keep_variable_count
            .max(exported.len())
            .max(1),
        Some(probe.matrix_rows),
        Some(probe.matrix_cols),
        Some(probe.matrix_rows.min(probe.matrix_cols)),
        Some(probe.swell_preflight.predicted_output_terms),
        Some(probe.swell_preflight.predicted_intermediate_terms),
        Some(probe.swell_preflight.coefficient_height_growth_bits),
        probe
            .swell_preflight
            .route_work_units
            .max(SaturatingCount::from_usize(
                probe
                    .matrix_rows
                    .max(1)
                    .saturating_mul(probe.matrix_cols.max(1)),
            )),
    );
    let route_budget = RouteBudget::from_estimate(&algebraic_work_estimate);
    Ok(KernelExecutionPlan::new_with_algebraic_cost(
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
        algebraic_work_estimate,
        route_budget,
        CertificateRoute::SparseResultantExactVerification,
        planned_failure_behavior(
            vec![
                SolverStatus::AlgorithmicHardCase,
                SolverStatus::FiniteResourceFailure,
                SolverStatus::CertificateDesignGap,
            ],
            LocalNonfinitePolicy::NotApplicable,
        ),
        PlanWorkClassification::PurePlan,
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
        plan,
    )?;
    let probe = probe_sparse_resultant_plan(
        &relation_polys,
        &plan.eliminated_variables,
        &plan.exported_variables,
        max_dim,
    )?;
    if probe.swell_preflight.preflight_hash
        != sparse_resultant_swell_preflight_hash(&probe.swell_preflight)
    {
        return Err(implementation_bug(
            "sparse resultant swell preflight hash is stale",
        ));
    }
    if trace.swell_preflight.preflight_hash != probe.swell_preflight.preflight_hash {
        return Err(implementation_bug(
            "sparse resultant execution route trace diverged from planned swell preflight",
        ));
    }
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
    let mut current = relations.to_vec();
    let mut template_hashes = Vec::new();
    let mut pair_costs = Vec::new();
    let mut matrix_rows = 0usize;
    let mut matrix_cols = 0usize;
    let mut max_degree = relations.iter().map(poly_total_degree).max().unwrap_or(1) as usize;
    for eliminate in eliminated {
        let Some(pair) = selectable_resultant_pair(&current, *eliminate, max_dim) else {
            continue;
        };
        let template = build_sparse_resultant_template(pair.input.clone())?;
        matrix_rows = matrix_rows.saturating_add(template.matrix_rows);
        matrix_cols = matrix_cols.saturating_add(template.matrix_cols);
        template_hashes.push(template.template_hash);
        let surrogate = simulated_resultant_relation(&pair.input, &pair.footprint);
        let mut next = current
            .into_iter()
            .enumerate()
            .filter_map(|(idx, poly)| {
                if idx == pair.left || idx == pair.right {
                    None
                } else {
                    Some(poly)
                }
            })
            .collect::<Vec<_>>();
        next.push(surrogate);
        current = next;
        pair_costs.push(pair.footprint);
        max_degree =
            max_degree.saturating_add(template.matrix_rows.max(template.matrix_cols).max(1));
    }
    if template_hashes.is_empty() {
        return Err(algorithmic_hard_case(
            "sparse resultant template chain was not applicable",
        ));
    }
    let swell_preflight = combine_swell_preflight(&pair_costs);
    let template_trace_hash = hash_sequence(
        "sparse-resultant-template-trace",
        &template_hashes
            .iter()
            .map(|hash| hash.0.to_vec())
            .chain(std::iter::once(swell_preflight.preflight_hash.0.to_vec()))
            .collect::<Vec<_>>(),
    );
    Ok(SparseResultantPlanProbe {
        matrix_rows,
        matrix_cols,
        max_degree,
        swell_preflight,
        template_trace_hash,
        output_support_hash: planned_resultant_output_support_hash(exported, max_degree),
    })
}

fn build_sparse_resultant_trace(
    relations: &[SparsePolynomialQ],
    eliminated: &[VariableId],
    exported: &[VariableId],
    max_dim: usize,
    plan: &KernelExecutionPlan,
) -> Result<SparseResultantTrace, SolverError> {
    let exported_set = exported.iter().copied().collect::<BTreeSet<_>>();
    let mut current = relations.to_vec();
    let mut certificates = Vec::new();
    let mut template_hashes = Vec::new();
    let mut pair_costs = Vec::new();
    let mut accumulated_work = SaturatingCount::ZERO;
    let mut matrix_rows = 0usize;
    let mut matrix_cols = 0usize;
    for (step, eliminate) in eliminated.iter().enumerate() {
        let Some(pair) = selectable_resultant_pair(&current, *eliminate, max_dim) else {
            continue;
        };
        guard_sparse_resultant_pair(plan, step, &pair.footprint)?;
        accumulated_work = accumulated_work.saturating_add(pair.footprint.route_work_units);
        if accumulated_work > plan.route_budget.max_work_units {
            return Err(sparse_resultant_guard_failure(
                plan,
                "SparseResultantRouteWorkBudget",
                pair.footprint.matrix_rows,
                pair.footprint.matrix_cols,
                pair.footprint
                    .coefficient_height_growth_bits
                    .as_usize_saturating(),
            ));
        }
        let template = build_sparse_resultant_template(pair.input)?;
        matrix_rows = matrix_rows.saturating_add(template.matrix_rows);
        matrix_cols = matrix_cols.saturating_add(template.matrix_cols);
        template_hashes.push(template.template_hash);
        pair_costs.push(pair.footprint.clone());
        let resultant = compute_resultant_relation(&template, ModularOptions::default())?;
        if !verify_resultant_certificate(&resultant.certificate) {
            return Err(implementation_bug(
                "sparse resultant certificate failed exact recomputation",
            ));
        }
        guard_sparse_resultant_output(plan, &resultant.relation, &pair.footprint)?;
        let mut next = current
            .into_iter()
            .enumerate()
            .filter_map(|(idx, poly)| {
                if idx == pair.left || idx == pair.right {
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
    let swell_preflight = combine_swell_preflight(&pair_costs);
    let template_trace_hash = hash_sequence(
        "sparse-resultant-template-trace",
        &template_hashes
            .iter()
            .map(|hash| hash.0.to_vec())
            .chain(std::iter::once(swell_preflight.preflight_hash.0.to_vec()))
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
            swell_preflight.preflight_hash.0.to_vec(),
            output_support_hash.0.to_vec(),
            relation.hash.0.to_vec(),
        ],
    );
    Ok(SparseResultantTrace {
        relation,
        certificates,
        swell_preflight,
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

fn simulated_resultant_relation(
    input: &ResultantInput,
    cost: &ResultantPairCost,
) -> SparsePolynomialQ {
    if input.keep_variables.is_empty() {
        return normalize_poly(SparsePolynomialQ {
            terms: vec![TermQ {
                coeff: int_q(1),
                monomial: normalize_monomial(Vec::new()),
            }],
            hash: hash_sequence("poly", &[]),
        });
    }
    let term_limit = cost
        .predicted_output_terms
        .as_usize_saturating()
        .max(1)
        .min(8_192);
    let degree_cap = cost.max_total_degree.max(1).min(u32::MAX as usize);
    let mut terms = Vec::with_capacity(term_limit);
    for idx in 0..term_limit {
        let primary = input.keep_variables[0];
        let mut exponents = vec![(primary, (idx + 1).min(u32::MAX as usize) as u32)];
        if input.keep_variables.len() > 1 {
            let secondary = input.keep_variables[idx % input.keep_variables.len()];
            let exp = ((idx % degree_cap) + 1).min(u32::MAX as usize) as u32;
            exponents.push((secondary, exp));
        }
        terms.push(TermQ {
            coeff: int_q(1),
            monomial: normalize_monomial(exponents),
        });
    }
    normalize_poly(SparsePolynomialQ {
        terms,
        hash: hash_sequence("poly", &[]),
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
) -> Option<SelectedResultantPair> {
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
                    footprint.route_work_units.as_usize_saturating(),
                    footprint
                        .coefficient_height_growth_bits
                        .as_usize_saturating(),
                    footprint
                        .max_term_count_after_coefficient_multiplication
                        .as_usize_saturating(),
                    footprint.input_term_count,
                    footprint.keep_variable_count,
                    template.matrix_rows.saturating_mul(template.matrix_cols),
                    relations[i].hash,
                    relations[j].hash,
                    i,
                    j,
                    input,
                    footprint,
                ));
            }
        }
    }
    candidates.sort_by_key(|entry| {
        (
            entry.0, entry.1, entry.2, entry.3, entry.4, entry.5, entry.6, entry.7, entry.8,
            entry.9, entry.10,
        )
    });
    candidates
        .into_iter()
        .next()
        .map(
            |(_, _, _, _, _, _, _, _, _, i, j, input, footprint)| SelectedResultantPair {
                left: i,
                right: j,
                input,
                footprint,
            },
        )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SelectedResultantPair {
    left: usize,
    right: usize,
    input: ResultantInput,
    footprint: ResultantPairCost,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResultantPairCost {
    input_term_count: usize,
    left_term_count: usize,
    right_term_count: usize,
    max_input_terms: usize,
    eliminate: VariableId,
    left_degree: usize,
    right_degree: usize,
    matrix_rows: usize,
    matrix_cols: usize,
    keep_variable_count: usize,
    max_total_degree: usize,
    max_term_count_after_coefficient_multiplication: SaturatingCount,
    estimated_resultant_template_support: SaturatingCount,
    coefficient_height_growth_bits: SaturatingCount,
    predicted_intermediate_terms: SaturatingCount,
    predicted_output_terms: SaturatingCount,
    route_work_units: SaturatingCount,
}

fn resultant_pair_cost(
    input: &ResultantInput,
    matrix_rows: usize,
    matrix_cols: usize,
) -> ResultantPairCost {
    let left_term_count = input
        .polynomials
        .first()
        .map(poly_monomial_count)
        .unwrap_or(0)
        .max(1);
    let right_term_count = input
        .polynomials
        .get(1)
        .map(poly_monomial_count)
        .unwrap_or(0)
        .max(1);
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
    let left_degree = input
        .polynomials
        .first()
        .map(|poly| degree_in_variable(poly, input.eliminate) as usize)
        .unwrap_or(0);
    let right_degree = input
        .polynomials
        .get(1)
        .map(|poly| degree_in_variable(poly, input.eliminate) as usize)
        .unwrap_or(0);
    let keep_variable_count = input.keep_variables.len();
    let max_total_degree = input
        .polynomials
        .iter()
        .map(poly_total_degree)
        .max()
        .unwrap_or(0) as usize
        + matrix_rows.max(matrix_cols).max(1);
    let matrix_area = matrix_rows.max(1).saturating_mul(matrix_cols.max(1));
    let max_term_count_after_coefficient_multiplication = input
        .polynomials
        .iter()
        .map(|poly| SaturatingCount::from_usize(poly_monomial_count(poly).max(1)))
        .fold(SaturatingCount::ONE, |acc, terms| acc.saturating_mul(terms));
    let estimated_resultant_template_support = max_term_count_after_coefficient_multiplication
        .saturating_mul(SaturatingCount::from_usize(
            matrix_area.saturating_mul(keep_variable_count.max(1)),
        ));
    let input_height_bits = max_poly_coefficient_height_bits(&input.polynomials).max(1);
    let coefficient_height_growth_bits = SaturatingCount::from_usize(input_height_bits)
        .saturating_mul(SaturatingCount::from_usize(
            matrix_rows.max(matrix_cols).max(1),
        ))
        .saturating_add(SaturatingCount::from_usize(input_term_count));
    let predicted_intermediate_terms = input
        .polynomials
        .iter()
        .map(|poly| SaturatingCount::from_usize(poly_monomial_count(poly).max(1)))
        .fold(SaturatingCount::ONE, |acc, terms| acc.saturating_mul(terms))
        .saturating_mul(SaturatingCount::from_usize(matrix_area))
        .saturating_mul(SaturatingCount::from_usize(keep_variable_count.max(1)));
    let predicted_output_terms = predicted_intermediate_terms
        .max(max_term_count_after_coefficient_multiplication)
        .max(SaturatingCount::from_usize(max_input_terms));
    let route_work_units = predicted_intermediate_terms
        .saturating_mul(SaturatingCount::from_usize(matrix_area))
        .saturating_add(coefficient_height_growth_bits);
    ResultantPairCost {
        input_term_count,
        left_term_count,
        right_term_count,
        max_input_terms,
        eliminate: input.eliminate,
        left_degree,
        right_degree,
        matrix_rows,
        matrix_cols,
        keep_variable_count,
        max_total_degree,
        max_term_count_after_coefficient_multiplication,
        estimated_resultant_template_support,
        coefficient_height_growth_bits,
        predicted_intermediate_terms,
        predicted_output_terms,
        route_work_units,
    }
}

fn resultant_pair_growth_is_prohibited(cost: &ResultantPairCost) -> bool {
    cost.max_input_terms > 8_192
        || cost.keep_variable_count > 24
        || cost
            .max_term_count_after_coefficient_multiplication
            .exceeds_usize(250_000)
        || cost
            .estimated_resultant_template_support
            .exceeds_usize(250_000)
        || cost.predicted_intermediate_terms.exceeds_usize(250_000)
        || cost.coefficient_height_growth_bits.exceeds_usize(16_384)
}

fn combine_swell_preflight(costs: &[ResultantPairCost]) -> SparseResultantSwellPreflight {
    let pair_preflights = costs
        .iter()
        .map(sparse_resultant_pair_preflight)
        .collect::<Vec<_>>();
    let input_term_count_per_pair = costs
        .iter()
        .map(|cost| cost.input_term_count)
        .collect::<Vec<_>>();
    let max_pair_input_terms = costs
        .iter()
        .map(|cost| cost.max_input_terms)
        .max()
        .unwrap_or(0);
    let max_term_count_after_coefficient_multiplication = costs
        .iter()
        .map(|cost| cost.max_term_count_after_coefficient_multiplication)
        .max()
        .unwrap_or_default();
    let estimated_resultant_template_support = costs
        .iter()
        .map(|cost| cost.estimated_resultant_template_support)
        .fold(SaturatingCount::ZERO, |acc, count| {
            acc.saturating_add(count)
        });
    let keep_variable_count = costs
        .iter()
        .map(|cost| cost.keep_variable_count)
        .max()
        .unwrap_or(0);
    let coefficient_height_growth_bits = costs
        .iter()
        .map(|cost| cost.coefficient_height_growth_bits)
        .max()
        .unwrap_or_default();
    let predicted_intermediate_terms = costs
        .iter()
        .map(|cost| cost.predicted_intermediate_terms)
        .fold(SaturatingCount::ZERO, |acc, count| {
            acc.saturating_add(count)
        });
    let predicted_output_terms = costs
        .iter()
        .map(|cost| cost.predicted_output_terms)
        .fold(SaturatingCount::ZERO, |acc, count| {
            acc.saturating_add(count)
        });
    let route_work_units = costs
        .iter()
        .map(|cost| cost.route_work_units)
        .fold(SaturatingCount::ZERO, |acc, count| {
            acc.saturating_add(count)
        });
    let mut preflight = SparseResultantSwellPreflight {
        pair_preflights,
        input_term_count_per_pair,
        max_pair_input_terms,
        max_term_count_after_coefficient_multiplication,
        estimated_resultant_template_support,
        keep_variable_count,
        coefficient_height_growth_bits,
        predicted_intermediate_terms,
        predicted_output_terms,
        route_work_units,
        preflight_hash: hash_sequence("sparse-resultant-swell-preflight", &[]),
    };
    preflight.preflight_hash = sparse_resultant_swell_preflight_hash(&preflight);
    preflight
}

fn sparse_resultant_pair_preflight(cost: &ResultantPairCost) -> SparseResultantPairPreflight {
    let mut preflight = SparseResultantPairPreflight {
        eliminate: cost.eliminate,
        left_term_count: cost.left_term_count,
        right_term_count: cost.right_term_count,
        left_degree: cost.left_degree,
        right_degree: cost.right_degree,
        keep_variable_count: cost.keep_variable_count,
        matrix_rows: cost.matrix_rows,
        matrix_cols: cost.matrix_cols,
        determinant_entry_term_product: cost.max_term_count_after_coefficient_multiplication,
        output_term_upper_bound: cost.predicted_output_terms,
        coefficient_height_growth_bits: cost.coefficient_height_growth_bits,
        predicted_intermediate_terms: cost.predicted_intermediate_terms,
        route_work_units: cost.route_work_units,
        pair_hash: hash_sequence("sparse-resultant-pair-preflight", &[]),
    };
    preflight.pair_hash = sparse_resultant_pair_preflight_hash(&preflight);
    preflight
}

fn sparse_resultant_pair_preflight_hash(preflight: &SparseResultantPairPreflight) -> Hash {
    hash_sequence(
        "sparse-resultant-pair-preflight",
        &[
            preflight.eliminate.0.to_be_bytes().to_vec(),
            preflight.left_term_count.to_be_bytes().to_vec(),
            preflight.right_term_count.to_be_bytes().to_vec(),
            preflight.left_degree.to_be_bytes().to_vec(),
            preflight.right_degree.to_be_bytes().to_vec(),
            preflight.keep_variable_count.to_be_bytes().to_vec(),
            preflight.matrix_rows.to_be_bytes().to_vec(),
            preflight.matrix_cols.to_be_bytes().to_vec(),
            preflight
                .determinant_entry_term_product
                .0
                .to_be_bytes()
                .to_vec(),
            preflight.output_term_upper_bound.0.to_be_bytes().to_vec(),
            preflight
                .coefficient_height_growth_bits
                .0
                .to_be_bytes()
                .to_vec(),
            preflight
                .predicted_intermediate_terms
                .0
                .to_be_bytes()
                .to_vec(),
            preflight.route_work_units.0.to_be_bytes().to_vec(),
        ],
    )
}

fn sparse_resultant_swell_preflight_hash(preflight: &SparseResultantSwellPreflight) -> Hash {
    let mut chunks = Vec::new();
    chunks.push(
        (preflight.pair_preflights.len() as u64)
            .to_be_bytes()
            .to_vec(),
    );
    for pair in &preflight.pair_preflights {
        chunks.push(pair.pair_hash.0.to_vec());
    }
    chunks.push(
        (preflight.input_term_count_per_pair.len() as u64)
            .to_be_bytes()
            .to_vec(),
    );
    for count in &preflight.input_term_count_per_pair {
        chunks.push(count.to_be_bytes().to_vec());
    }
    chunks.push(preflight.max_pair_input_terms.to_be_bytes().to_vec());
    chunks.push(
        preflight
            .max_term_count_after_coefficient_multiplication
            .0
            .to_be_bytes()
            .to_vec(),
    );
    chunks.push(
        preflight
            .estimated_resultant_template_support
            .0
            .to_be_bytes()
            .to_vec(),
    );
    chunks.push(preflight.keep_variable_count.to_be_bytes().to_vec());
    chunks.push(
        preflight
            .coefficient_height_growth_bits
            .0
            .to_be_bytes()
            .to_vec(),
    );
    chunks.push(
        preflight
            .predicted_intermediate_terms
            .0
            .to_be_bytes()
            .to_vec(),
    );
    chunks.push(preflight.predicted_output_terms.0.to_be_bytes().to_vec());
    chunks.push(preflight.route_work_units.0.to_be_bytes().to_vec());
    hash_sequence("sparse-resultant-swell-preflight", &chunks)
}

fn guard_sparse_resultant_pair(
    plan: &KernelExecutionPlan,
    step: usize,
    cost: &ResultantPairCost,
) -> Result<(), SolverError> {
    if step >= plan.route_budget.max_elapsed_steps {
        return Err(sparse_resultant_guard_failure(
            plan,
            "SparseResultantChainStepBudget",
            cost.matrix_rows,
            cost.matrix_cols,
            cost.coefficient_height_growth_bits.as_usize_saturating(),
        ));
    }
    if cost.max_input_terms > plan.route_budget.max_input_terms_per_pair {
        return Err(sparse_resultant_guard_failure(
            plan,
            "SparseResultantPairInputTermBudget",
            cost.matrix_rows,
            cost.matrix_cols,
            cost.coefficient_height_growth_bits.as_usize_saturating(),
        ));
    }
    if cost.keep_variable_count > plan.route_budget.max_keep_variables {
        return Err(sparse_resultant_guard_failure(
            plan,
            "SparseResultantKeepVariableBudget",
            cost.matrix_rows,
            cost.matrix_cols,
            cost.coefficient_height_growth_bits.as_usize_saturating(),
        ));
    }
    if cost
        .predicted_intermediate_terms
        .exceeds_usize(plan.route_budget.max_intermediate_terms)
    {
        return Err(sparse_resultant_guard_failure(
            plan,
            "SparseResultantIntermediateTermBudget",
            cost.matrix_rows,
            cost.matrix_cols,
            cost.coefficient_height_growth_bits.as_usize_saturating(),
        ));
    }
    if cost.max_total_degree > plan.route_budget.max_total_degree {
        return Err(sparse_resultant_guard_failure(
            plan,
            "SparseResultantTotalDegreeBudget",
            cost.matrix_rows,
            cost.matrix_cols,
            cost.coefficient_height_growth_bits.as_usize_saturating(),
        ));
    }
    if cost
        .coefficient_height_growth_bits
        .exceeds_usize(plan.route_budget.max_coefficient_height_bits)
    {
        return Err(sparse_resultant_guard_failure(
            plan,
            "SparseResultantCoefficientHeightBudget",
            cost.matrix_rows,
            cost.matrix_cols,
            cost.coefficient_height_growth_bits.as_usize_saturating(),
        ));
    }
    if cost.route_work_units > plan.route_budget.max_work_units {
        return Err(sparse_resultant_guard_failure(
            plan,
            "SparseResultantPairWorkBudget",
            cost.matrix_rows,
            cost.matrix_cols,
            cost.coefficient_height_growth_bits.as_usize_saturating(),
        ));
    }
    Ok(())
}

fn guard_sparse_resultant_output(
    plan: &KernelExecutionPlan,
    relation: &SparsePolynomialQ,
    cost: &ResultantPairCost,
) -> Result<(), SolverError> {
    if poly_monomial_count(relation) > plan.route_budget.max_output_terms {
        return Err(sparse_resultant_guard_failure(
            plan,
            "SparseResultantOutputTermBudget",
            cost.matrix_rows,
            cost.matrix_cols,
            poly_coefficient_height_bits(relation),
        ));
    }
    if poly_total_degree(relation) as usize > plan.route_budget.max_total_degree {
        return Err(sparse_resultant_guard_failure(
            plan,
            "SparseResultantOutputDegreeBudget",
            cost.matrix_rows,
            cost.matrix_cols,
            poly_coefficient_height_bits(relation),
        ));
    }
    if poly_coefficient_height_bits(relation) > plan.route_budget.max_coefficient_height_bits {
        return Err(sparse_resultant_guard_failure(
            plan,
            "SparseResultantOutputCoefficientHeightBudget",
            cost.matrix_rows,
            cost.matrix_cols,
            poly_coefficient_height_bits(relation),
        ));
    }
    Ok(())
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
        trace.swell_preflight.preflight_hash.0.to_vec(),
    ];
    for cert in &trace.certificates {
        chunks.push(resultant_certificate_hash(cert).0.to_vec());
    }
    hash_sequence("sparse-resultant-kernel-certificate", &chunks)
}

fn resultant_certificate_hash(cert: &SparseResultantCertificate) -> Hash {
    let mut chunks = vec![
        cert.template_hash.0.to_vec(),
        cert.relation_hash.0.to_vec(),
        format!("{:?}", cert.backend).into_bytes(),
        cert.exact_verification_hash.0.to_vec(),
    ];
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

fn sparse_resultant_guard_failure(
    plan: &KernelExecutionPlan,
    stage: &str,
    rows: usize,
    cols: usize,
    coefficient_height_bits: usize,
) -> SolverError {
    let route_trace_hash =
        sparse_resultant_route_failure_trace_hash(plan, stage, rows, cols, coefficient_height_bits);
    SolverError {
        target: None,
        kind: SolverErrorKind::Failure(FailureKind::FiniteResourceFailure {
            stage: StageId(format!(
                "{stage}|route_trace_hash={route_trace_hash:?}|plan_hash={:?}|budget_hash={:?}|estimate_hash={:?}",
                plan.plan_hash,
                plan.route_budget.budget_hash,
                plan.algebraic_work_estimate.estimate_hash
            )),
            block_id: Some(plan.block_id),
            matrix_rows: Some(rows),
            matrix_cols: Some(cols),
            matrix_density: None,
            quotient_rank_estimate: Some(rows.min(cols)),
            coefficient_height_bits: Some(coefficient_height_bits),
            memory_bytes: None,
        }),
    }
}

fn sparse_resultant_route_failure_trace_hash(
    plan: &KernelExecutionPlan,
    stage: &str,
    rows: usize,
    cols: usize,
    coefficient_height_bits: usize,
) -> Hash {
    hash_sequence(
        "sparse-resultant-route-failure-trace",
        &[
            stage.as_bytes().to_vec(),
            plan.block_id.0.to_be_bytes().to_vec(),
            plan.plan_hash.0.to_vec(),
            plan.route_budget.budget_hash.0.to_vec(),
            plan.algebraic_work_estimate.estimate_hash.0.to_vec(),
            rows.to_be_bytes().to_vec(),
            cols.to_be_bytes().to_vec(),
            coefficient_height_bits.to_be_bytes().to_vec(),
        ],
    )
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
    use crate::problem::context::{new_context, SolverContext};
    use crate::problem::input::make_problem;
    use crate::problem::validate::validate_input;
    use crate::result::cost_trace::ProjectionCostTrace;
    use crate::result::status::{FailureKind, SolverStatus};
    use crate::solver::options::SolverOptions;
    use crate::types::hash::hash_sequence;
    use crate::types::ids::{BlockId, PackageId, VariableId};
    use crate::types::monomial::normalize_monomial;
    use crate::types::polynomial::{
        constant_poly, normalize_poly, poly_scale, poly_sub, poly_variables, variable_poly,
        SparsePolynomialQ, TermQ,
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

    #[test]
    fn acr_p3_probe_records_swell_preflight_and_binds_it_to_plan_hash() {
        let t = VariableId(0);
        let y = VariableId(1);
        let base_relations = vec![
            poly_sub(&variable_poly(y), &variable_poly(t)),
            poly_sub(&variable_poly(y), &constant_poly(int_q(1))),
        ];
        let scaled_relations = vec![
            poly_sub(&variable_poly(y), &poly_scale(&variable_poly(t), &int_q(2))),
            poly_sub(&variable_poly(y), &constant_poly(int_q(1))),
        ];

        let base_probe = probe_sparse_resultant_plan(&base_relations, &[y], &[t], 4).unwrap();
        let scaled_probe = probe_sparse_resultant_plan(&scaled_relations, &[y], &[t], 4).unwrap();

        assert_eq!(base_probe.matrix_rows, scaled_probe.matrix_rows);
        assert_eq!(base_probe.matrix_cols, scaled_probe.matrix_cols);
        assert_eq!(
            base_probe.swell_preflight.input_term_count_per_pair,
            vec![4]
        );
        assert_eq!(base_probe.swell_preflight.keep_variable_count, 1);
        assert_ne!(
            base_probe.swell_preflight.preflight_hash,
            scaled_probe.swell_preflight.preflight_hash
        );
        assert_ne!(
            base_probe.template_trace_hash,
            scaled_probe.template_trace_hash
        );
    }

    #[test]
    fn acr_p3_pair_selection_ranks_huge_intermediate_behind_safe_pair() {
        let y = VariableId(20);
        let t = VariableId(0);
        let keep_variables = (0..12).map(VariableId).collect::<Vec<_>>();
        let relations = vec![
            dense_linear_polynomial(y, &keep_variables, 0),
            dense_linear_polynomial(y, &keep_variables, 511),
            poly_sub(&variable_poly(y), &variable_poly(t)),
            poly_sub(&variable_poly(y), &constant_poly(int_q(1))),
        ];

        let pair = selectable_resultant_pair(&relations, y, 4).unwrap();

        assert_eq!((pair.left, pair.right), (2, 3));
        assert!(
            pair.footprint.predicted_intermediate_terms
                < resultant_pair_cost(
                    &ResultantInput {
                        polynomials: vec![relations[0].clone(), relations[1].clone()],
                        eliminate: y,
                        keep_variables: selected_keep_variables(&relations[0], &relations[1], y),
                        max_matrix_dim: 4,
                    },
                    2,
                    2
                )
                .predicted_intermediate_terms
        );
    }

    #[test]
    fn acr_p3_larger_matrix_tiny_sparse_resultant_is_feasible_with_exact_verification() {
        let t = VariableId(0);
        let y = VariableId(1);
        let y_cubed = monomial_polynomial(&[(y, 3)]);
        let relations = vec![
            poly_sub(&y_cubed, &variable_poly(t)),
            poly_sub(&y_cubed, &constant_poly(int_q(1))),
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
        let plan = kernel.plan(&admission, &kctx, &solver_ctx).unwrap();

        let message = kernel.execute(&plan, &mut kctx, &mut solver_ctx).unwrap();

        assert_eq!(message.kernel_kind, KernelKind::SparseResultantProjection);
        assert!(kernel.replay(&message, &kctx).accepted);
    }

    #[test]
    fn acr_p4_runtime_output_guard_returns_route_local_finite_resource_failure() {
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
        plan.route_budget.max_output_terms = 1;
        plan.route_budget.budget_hash =
            crate::planner::algebraic_cost::route_budget_hash(&plan.route_budget);
        plan.plan_hash = hash_kernel_execution_plan(&plan);

        let err = kernel
            .execute(&plan, &mut kctx, &mut new_context(SolverOptions::default()))
            .unwrap_err();

        assert_eq!(err.public_status(), SolverStatus::FiniteResourceFailure);
        let SolverErrorKind::Failure(FailureKind::FiniteResourceFailure { stage, .. }) = err.kind
        else {
            panic!("expected finite resource failure");
        };
        assert!(stage.0.starts_with("SparseResultantOutputTermBudget"));
        assert!(stage.0.contains("route_trace_hash="));
        assert!(stage.0.contains("plan_hash="));
    }

    #[test]
    fn acr_p4_runtime_pair_guards_cover_declared_budget_classes() {
        let cases: Vec<(&str, Box<dyn Fn(&mut KernelExecutionPlan)>)> = vec![
            (
                "SparseResultantPairInputTermBudget",
                Box::new(|plan| plan.route_budget.max_input_terms_per_pair = 1),
            ),
            (
                "SparseResultantIntermediateTermBudget",
                Box::new(|plan| plan.route_budget.max_intermediate_terms = 1),
            ),
            (
                "SparseResultantKeepVariableBudget",
                Box::new(|plan| plan.route_budget.max_keep_variables = 0),
            ),
            (
                "SparseResultantTotalDegreeBudget",
                Box::new(|plan| plan.route_budget.max_total_degree = 1),
            ),
            (
                "SparseResultantCoefficientHeightBudget",
                Box::new(|plan| plan.route_budget.max_coefficient_height_bits = 1),
            ),
            (
                "SparseResultantChainStepBudget",
                Box::new(|plan| plan.route_budget.max_elapsed_steps = 0),
            ),
            (
                "SparseResultantPairWorkBudget",
                Box::new(|plan| plan.route_budget.max_work_units = SaturatingCount::ONE),
            ),
        ];

        for (expected_stage, mutate) in cases {
            let (mut kctx, solver_ctx, mut plan) = sparse_resultant_test_context();
            mutate(&mut plan);
            refresh_plan_budget_hashes(&mut plan);

            let err = SparseResultantProjectionKernel
                .execute(
                    &plan,
                    &mut kctx,
                    &mut new_context(solver_ctx.options.clone()),
                )
                .unwrap_err();

            assert_eq!(err.public_status(), SolverStatus::FiniteResourceFailure);
            let SolverErrorKind::Failure(FailureKind::FiniteResourceFailure { stage, .. }) =
                err.kind
            else {
                panic!("expected finite resource failure");
            };
            assert!(
                stage.0.starts_with(expected_stage),
                "expected {expected_stage}, got {}",
                stage.0
            );
            assert!(stage.0.contains("route_trace_hash="));
        }
    }

    #[test]
    fn acr_p4_huge_intermediate_guard_stops_before_next_resultant_step() {
        let t = VariableId(0);
        let y = VariableId(1);
        let z = VariableId(2);
        let relations = vec![
            poly_sub(
                &variable_poly(y),
                &poly_sub(&variable_poly(z), &variable_poly(t)),
            ),
            poly_sub(&variable_poly(y), &constant_poly(int_q(1))),
            poly_sub(&variable_poly(z), &variable_poly(t)),
        ];
        let compressed = compressed_system(vec![t, y, z], t, relations);
        let block = test_block(&compressed, [t, y, z], [t]);
        let mut kctx = KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };
        let kernel = SparseResultantProjectionKernel;
        let admission = kernel.admit(&kctx.block, &kctx);
        let mut plan = kernel
            .plan(&admission, &kctx, &new_context(SolverOptions::default()))
            .unwrap();
        assert!(plan.eliminated_variables.len() >= 2);
        plan.route_budget.max_output_terms = 1;
        refresh_plan_budget_hashes(&mut plan);

        let err = kernel
            .execute(&plan, &mut kctx, &mut new_context(SolverOptions::default()))
            .unwrap_err();

        assert_eq!(err.public_status(), SolverStatus::FiniteResourceFailure);
        let SolverErrorKind::Failure(FailureKind::FiniteResourceFailure { stage, .. }) = err.kind
        else {
            panic!("expected finite resource failure");
        };
        assert!(stage.0.starts_with("SparseResultantOutputTermBudget"));
        assert!(stage.0.contains("route_trace_hash="));
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

    fn sparse_resultant_test_context() -> (KernelContext, SolverContext, KernelExecutionPlan) {
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
        let admission = kernel.admit(&kctx.block, &kctx);
        let plan = kernel.plan(&admission, &kctx, &solver_ctx).unwrap();
        (kctx, solver_ctx, plan)
    }

    fn refresh_plan_budget_hashes(plan: &mut KernelExecutionPlan) {
        plan.route_budget.budget_hash =
            crate::planner::algebraic_cost::route_budget_hash(&plan.route_budget);
        plan.plan_hash = hash_kernel_execution_plan(plan);
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

    fn monomial_polynomial(exponents: &[(VariableId, u32)]) -> SparsePolynomialQ {
        normalize_poly(SparsePolynomialQ {
            terms: vec![TermQ {
                coeff: int_q(1),
                monomial: normalize_monomial(exponents.to_vec()),
            }],
            hash: hash_sequence("poly", &[]),
        })
    }
}
