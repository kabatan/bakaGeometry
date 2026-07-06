use std::collections::{BTreeMap, BTreeSet};

use crate::algebra::krylov::{
    block_krylov_sequence, certify_krylov_coverage, recover_recurrence, verify_annihilator,
    AnnihilatorCertificate, CoverageCertificate, KrylovPlan,
};
use crate::algebra::normal_form::{MembershipCertificate, MembershipTerm};
use crate::algebra::quotient::{
    build_production_target_relevant_quotient_handle,
    build_production_target_relevant_quotient_input_from_relations, hash_authorized_relations,
    make_action_column_certificate, monomial_basis_polynomials, normal_form_basis_certificate,
    unit_vector, BasisHandleId, BasisScope, ProductionQuotientHandleInput, TargetQuotientHandle,
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
use crate::result::status::{FailureKind, SolverError, SolverErrorKind, SolverStatus};
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::{KernelPlanId, PackageId, RelationId, VariableId};
use crate::types::matrix::{SparseMatrixQ, VectorQ};
use crate::types::monomial::normalize_monomial;
use crate::types::polynomial::{
    constant_poly, max_poly_coefficient_height_bits, normalize_poly, poly_coefficient_height_bits,
    poly_monomial_count, poly_mul, poly_scale, poly_total_degree, poly_variables, variable_poly,
    SparsePolynomialQ, TermQ,
};
use crate::types::rational::{add_q, div_q, is_zero_q, mul_q, neg_q, one_q, zero_q, RationalQ};
use crate::types::univariate::{normalize_univariate, UniPolynomialQ};
use crate::verify::certificates::{
    KernelCertificate, KernelCertificatePayload, TargetActionProjectionCertificate,
};

pub struct TargetActionKrylovKernel;

impl TargetProjectionKernel for TargetActionKrylovKernel {
    fn kind(&self) -> KernelKind {
        KernelKind::TargetActionKrylov
    }

    fn admit(&self, block: &ProjectionBlock, ctx: &KernelContext) -> KernelAdmission {
        admit_target_action_krylov(block, ctx)
    }

    fn plan(
        &self,
        admission: &KernelAdmission,
        ctx: &KernelContext,
        solver_ctx: &SolverContext,
    ) -> Result<KernelExecutionPlan, SolverError> {
        plan_target_action_krylov_from_admission(admission, ctx, solver_ctx)
    }

    fn execute(
        &self,
        plan: &KernelExecutionPlan,
        ctx: &mut KernelContext,
        solver_ctx: &mut SolverContext,
    ) -> Result<ProjectionMessage, SolverError> {
        execute_target_action_krylov(plan, ctx, solver_ctx)
    }

    fn replay(&self, message: &ProjectionMessage, ctx: &KernelContext) -> ReplayResult {
        crate::kernels::traits::exact_replay_result(
            self.kind(),
            "target-action-krylov-replay",
            message,
            ctx,
        )
    }
}

#[derive(Debug, Clone)]
struct TargetActionKrylovTrace {
    relation: SparsePolynomialQ,
    coverage: CoverageCertificate,
    annihilator: AnnihilatorCertificate,
    quotient_input: ProductionQuotientHandleInput,
    quotient_authorization_hash: Hash,
    matrix_rows: usize,
    matrix_cols: usize,
    degree_bound: usize,
    trace_hash: Hash,
}

#[derive(Debug, Clone)]
struct TargetActionKrylovPlanProbe {
    matrix_rows: usize,
    matrix_cols: usize,
    degree_bound: usize,
    action_shape_hash: Hash,
    characteristic_support_hash: Hash,
}

#[derive(Debug, Clone)]
struct ActionRelationInput {
    polynomial: SparsePolynomialQ,
    source_relation_ids: Vec<RelationId>,
    source_hash: Hash,
    child_message_hash: Option<Hash>,
}

#[derive(Debug, Clone)]
enum TargetActionSelection {
    TargetOnly(ActionRelationInput),
    AliasUnivariate {
        local_relation: ActionRelationInput,
        alias_relation: ActionRelationInput,
        local_variable: VariableId,
        target_coeff: RationalQ,
        local_coeff: RationalQ,
    },
    GenericQuotient {
        inputs: Vec<ActionRelationInput>,
        variables: Vec<VariableId>,
    },
}

impl TargetActionSelection {
    fn source_relation_ids(&self) -> Vec<RelationId> {
        let mut ids = Vec::new();
        match self {
            TargetActionSelection::TargetOnly(input) => {
                ids.extend(input.source_relation_ids.iter().copied());
            }
            TargetActionSelection::AliasUnivariate {
                local_relation,
                alias_relation,
                ..
            } => {
                ids.extend(local_relation.source_relation_ids.iter().copied());
                ids.extend(alias_relation.source_relation_ids.iter().copied());
            }
            TargetActionSelection::GenericQuotient { inputs, .. } => {
                for input in inputs {
                    ids.extend(input.source_relation_ids.iter().copied());
                }
            }
        }
        ids.sort();
        ids.dedup();
        ids
    }

    fn source_hashes(&self) -> Vec<Hash> {
        let mut hashes = match self {
            TargetActionSelection::TargetOnly(input) => vec![input.source_hash],
            TargetActionSelection::AliasUnivariate {
                local_relation,
                alias_relation,
                ..
            } => vec![local_relation.source_hash, alias_relation.source_hash],
            TargetActionSelection::GenericQuotient { inputs, .. } => {
                inputs.iter().map(|input| input.source_hash).collect()
            }
        };
        hashes.sort();
        hashes.dedup();
        hashes
    }

    fn child_message_hashes(&self) -> Vec<Hash> {
        let mut hashes = Vec::new();
        match self {
            TargetActionSelection::TargetOnly(input) => {
                hashes.extend(input.child_message_hash);
            }
            TargetActionSelection::AliasUnivariate {
                local_relation,
                alias_relation,
                ..
            } => {
                hashes.extend(local_relation.child_message_hash);
                hashes.extend(alias_relation.child_message_hash);
            }
            TargetActionSelection::GenericQuotient { inputs, .. } => {
                for input in inputs {
                    hashes.extend(input.child_message_hash);
                }
            }
        }
        hashes.sort();
        hashes.dedup();
        hashes
    }

    fn source_polynomials(&self) -> Vec<&SparsePolynomialQ> {
        match self {
            TargetActionSelection::TargetOnly(input) => vec![&input.polynomial],
            TargetActionSelection::AliasUnivariate {
                local_relation,
                alias_relation,
                ..
            } => vec![&local_relation.polynomial, &alias_relation.polynomial],
            TargetActionSelection::GenericQuotient { inputs, .. } => {
                inputs.iter().map(|input| &input.polynomial).collect()
            }
        }
    }

    fn planned_rank_estimate(&self, block: &ProjectionBlock) -> usize {
        match self {
            TargetActionSelection::GenericQuotient { variables, .. } => {
                let degree_factor = self
                    .source_polynomials()
                    .iter()
                    .map(|poly| poly_total_degree(poly) as usize + 1)
                    .max()
                    .unwrap_or(2)
                    .max(2);
                variables
                    .len()
                    .max(block.local_variables.len())
                    .max(1)
                    .min(8)
                    .checked_pow(degree_factor.min(4) as u32)
                    .unwrap_or(256)
                    .clamp(1, 256)
            }
            _ => block
                .local_variables
                .len()
                .max(self.source_hashes().len())
                .max(
                    self.source_polynomials()
                        .iter()
                        .map(|poly| poly_total_degree(poly) as usize)
                        .max()
                        .unwrap_or(1),
                )
                .max(1),
        }
    }
}

pub fn admit_target_action_krylov(block: &ProjectionBlock, ctx: &KernelContext) -> KernelAdmission {
    let solver_ctx = SolverContext::new(Default::default());
    match plan_target_action_krylov_with_messages(
        block,
        &ctx.system,
        &ctx.child_messages,
        &solver_ctx,
        KernelPlanId(KernelKind::TargetActionKrylov as u32),
    ) {
        Ok(plan) => finish_admission(block, KernelAdmissionStatus::Admitted, Some(plan)),
        Err(_) => finish_admission(
            block,
            KernelAdmissionStatus::Declined {
                reason: "no certifiable finite target action quotient with characteristic coverage"
                    .to_owned(),
            },
            None,
        ),
    }
}

pub fn plan_target_action_krylov(
    block: &ProjectionBlock,
    system: &CompressedSystemQ,
    solver_ctx: &SolverContext,
    plan_id: KernelPlanId,
) -> Result<KernelExecutionPlan, SolverError> {
    plan_target_action_krylov_with_messages(block, system, &[], solver_ctx, plan_id)
}

pub fn plan_target_action_krylov_with_messages(
    block: &ProjectionBlock,
    system: &CompressedSystemQ,
    child_messages: &[ProjectionMessage],
    solver_ctx: &SolverContext,
    plan_id: KernelPlanId,
) -> Result<KernelExecutionPlan, SolverError> {
    let inputs = collect_relation_inputs(block, system, child_messages);
    let selected = select_target_action_selection(&inputs, system.target)
        .ok_or_else(|| certificate_design_gap("no certifiable target-action quotient relation"))?;
    let probe = probe_target_action_krylov_plan(&selected, system.target, block);
    let template = template_plan(
        probe.matrix_rows,
        probe.matrix_cols,
        probe.action_shape_hash,
        probe.characteristic_support_hash,
    );
    let mut support_plan = KernelSupportPlan {
        dense_relation_search_schedule: None,
        affine_elimination_order: None,
        template_plan: Some(template),
        rank_plan: Some(rank_plan(probe.matrix_rows.min(probe.matrix_cols))),
        universal_strategy_sequence: Vec::new(),
        degree_bound: probe.degree_bound,
        support_hash: hash_sequence("kernel-support-plan", &[]),
    };
    support_plan.support_hash = support_plan_hash(&support_plan);
    let mut bounds = ResourceBounds {
        max_matrix_rows: Some(probe.matrix_rows),
        max_matrix_cols: Some(probe.matrix_cols),
        max_export_degree: Some(probe.degree_bound),
        max_multiplier_total_degree: None,
        max_local_elimination_steps: Some(0),
        max_memory_bytes: solver_ctx.options.max_memory_bytes,
        bounds_hash: hash_sequence("planner-resource-bounds", &[]),
    };
    bounds.bounds_hash = resource_bounds_hash(&bounds);
    let source_relation_ids = selected.source_relation_ids();
    let source_hashes = selected.source_hashes();
    let child_message_hashes = selected.child_message_hashes();
    Ok(KernelExecutionPlan::new(
        plan_id,
        block.block_id,
        KernelKind::TargetActionKrylov,
        block.authorization_hash,
        source_relation_ids,
        source_hashes.clone(),
        block.child_block_ids.clone(),
        child_message_hashes,
        vec![system.target],
        block
            .local_variables
            .iter()
            .copied()
            .filter(|var| *var != system.target)
            .collect(),
        support_plan,
        bounds,
        CertificateRoute::VerifiedCharacteristicSupportCoverage,
        planned_failure_behavior(
            vec![
                SolverStatus::FiniteResourceFailure,
                SolverStatus::CertificateDesignGap,
            ],
            LocalNonfinitePolicy::NotApplicable,
        ),
    ))
}

pub fn plan_target_action_krylov_from_admission(
    admission: &KernelAdmission,
    ctx: &KernelContext,
    solver_ctx: &SolverContext,
) -> Result<KernelExecutionPlan, SolverError> {
    if admission.kind != KernelKind::TargetActionKrylov
        || !matches!(admission.status, KernelAdmissionStatus::Admitted)
    {
        return Err(implementation_bug(
            "target-action-krylov plan requested for non-admitted kernel",
        ));
    }
    if let Some(plan) = &admission.execution_plan {
        return Ok(plan.clone());
    }
    plan_target_action_krylov_with_messages(
        &ctx.block,
        &ctx.system,
        &ctx.child_messages,
        solver_ctx,
        KernelPlanId(KernelKind::TargetActionKrylov as u32),
    )
}

pub fn execute_target_action_krylov(
    plan: &KernelExecutionPlan,
    ctx: &mut KernelContext,
    _solver_ctx: &mut SolverContext,
) -> Result<ProjectionMessage, SolverError> {
    validate_target_action_plan_binding(plan, ctx)?;
    let inputs = planned_relation_inputs(plan, ctx)?;
    let selected = select_target_action_selection(&inputs, ctx.system.target)
        .ok_or_else(|| implementation_bug("target-action-krylov planned source disappeared"))?;
    validate_selection_hash_coverage(plan, &selected)?;
    let trace = build_target_action_krylov_trace(
        &selected,
        ctx.system.target,
        BasisHandleId(plan.plan_id.0 as u64),
    )?;
    let Some(template) = &plan.support_plan.template_plan else {
        return Err(implementation_bug(
            "target-action-krylov plan lacks template plan",
        ));
    };
    let probe = probe_target_action_krylov_plan(&selected, ctx.system.target, &ctx.block);
    if template.matrix_rows != probe.matrix_rows
        || template.matrix_cols != probe.matrix_cols
        || template.row_monomial_hash != probe.action_shape_hash
        || template.column_support_hash != probe.characteristic_support_hash
        || support_plan_hash(&plan.support_plan) != plan.support_plan.support_hash
    {
        return Err(implementation_bug(
            "target-action-krylov characteristic coverage plan is not reproducible",
        ));
    }
    if trace.degree_bound > plan.support_plan.degree_bound {
        return Err(implementation_bug(
            "target-action-krylov execution exceeded planned characteristic degree bound",
        ));
    }
    if !trace.coverage.no_coordinate_roots_exported
        || !trace.coverage.no_full_coordinate_rur_exported
    {
        return Err(implementation_bug(
            "target-action-krylov quotient exported coordinate roots or RUR",
        ));
    }
    let exported = plan
        .exported_variables
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    if !poly_variables(&trace.relation).is_subset(&exported) {
        return Err(implementation_bug(
            "target-action-krylov characteristic relation is outside exported variables",
        ));
    }
    let certificate_hash = target_action_krylov_certificate_hash(plan, &trace);
    let cost_trace = ProjectionCostTrace {
        block_id: plan.block_id,
        kernel_kind: KernelKind::TargetActionKrylov,
        local_variable_count: ctx.block.local_variables.len(),
        exported_variable_count: plan.exported_variables.len(),
        local_relation_count: inputs.len(),
        local_monomial_count: inputs
            .iter()
            .map(|input| poly_monomial_count(&input.polynomial))
            .sum(),
        estimated_quotient_rank: Some(trace.coverage.quotient_rank),
        matrix_rows: Some(trace.matrix_rows),
        matrix_cols: Some(trace.matrix_cols),
        matrix_density: Some(crate::types::matrix::matrix_density(&SparseMatrixQ {
            rows: trace.matrix_rows.max(1),
            cols: trace.matrix_cols.max(1),
            entries: Vec::new(),
        })),
        coefficient_height_before_bits: max_poly_coefficient_height_bits(
            &inputs
                .iter()
                .map(|input| input.polynomial.clone())
                .collect::<Vec<_>>(),
        ),
        coefficient_height_after_bits: poly_coefficient_height_bits(&trace.relation),
        route_cost: Some(ProjectionCostTrace::route_cost_from_plan(plan)),
    };
    let certificate = KernelCertificate::from_execution_plan_with_payload(
        plan,
        std::slice::from_ref(&trace.relation),
        certificate_hash,
        KernelCertificatePayload::TargetAction(TargetActionProjectionCertificate {
            target: ctx.system.target,
            quotient_input: trace.quotient_input.clone(),
            output_relation: trace.relation.clone(),
            coverage: trace.coverage.clone(),
            annihilator: trace.annihilator.clone(),
        }),
    );
    let mut message = ProjectionMessage {
        package_id: PackageId(plan.plan_id.0),
        block_id: plan.block_id,
        kernel_kind: KernelKind::TargetActionKrylov,
        source_relation_ids: plan.source_relation_ids.clone(),
        eliminated_variables: plan.eliminated_variables.clone(),
        exported_variables: plan.exported_variables.clone(),
        relation_generators: vec![trace.relation],
        representation: MessageRepresentation::QuotientAction,
        projection_strength: ProjectionStrength::CandidateCoverStrong,
        certificate,
        compression_trace: ctx.system.compression_trace.clone(),
        cost_trace,
        package_hash: hash_sequence("projection-message-initial", &[]),
    };
    message.package_hash = projection_message_hash(&message);
    Ok(message)
}

fn probe_target_action_krylov_plan(
    selection: &TargetActionSelection,
    target: VariableId,
    block: &ProjectionBlock,
) -> TargetActionKrylovPlanProbe {
    let rank_estimate = selection.planned_rank_estimate(block);
    let degree_bound = rank_estimate.max(1);
    let mut chunks = vec![target.0.to_be_bytes().to_vec()];
    for hash in selection.source_hashes() {
        chunks.push(hash.0.to_vec());
    }
    for variable in &block.local_variables {
        chunks.push(variable.0.to_be_bytes().to_vec());
    }
    let action_shape_hash = hash_sequence("target-action-krylov-planned-action-shape", &chunks);
    let characteristic_support_hash = hash_sequence(
        "target-action-krylov-planned-characteristic-support",
        &[
            action_shape_hash.0.to_vec(),
            degree_bound.to_be_bytes().to_vec(),
        ],
    );
    TargetActionKrylovPlanProbe {
        matrix_rows: rank_estimate,
        matrix_cols: rank_estimate,
        degree_bound,
        action_shape_hash,
        characteristic_support_hash,
    }
}

fn build_target_action_krylov_trace(
    selection: &TargetActionSelection,
    target: VariableId,
    basis_id: BasisHandleId,
) -> Result<TargetActionKrylovTrace, SolverError> {
    let quotient_input =
        build_quotient_handle_input_from_selection(selection, target, basis_id, true, true)?;
    let handle = build_production_target_relevant_quotient_handle(quotient_input.clone())?;
    let start_vectors = (0..handle.basis_size())
        .map(|idx| unit_vector(handle.basis_size(), idx))
        .collect::<Vec<_>>();
    let seq = block_krylov_sequence(
        &handle,
        target,
        KrylovPlan {
            start_vectors,
            max_steps: handle.basis_size().saturating_add(1),
        },
    )?;
    let recurrence = recover_recurrence(&seq)?;
    let coverage = certify_krylov_coverage(&seq, &recurrence, &handle)?;
    let annihilator = verify_annihilator(&handle, &coverage.characteristic_polynomial)?;
    let relation = univariate_to_polynomial(&coverage.characteristic_polynomial);
    let trace_hash = hash_sequence(
        "target-action-krylov-trace",
        &[
            coverage.quotient_handle_hash.0.to_vec(),
            coverage.target_action_matrix_hash.0.to_vec(),
            coverage.characteristic_polynomial_hash.0.to_vec(),
            coverage.cayley_hamilton_verification_hash.0.to_vec(),
            relation.hash.0.to_vec(),
        ],
    );
    Ok(TargetActionKrylovTrace {
        relation,
        quotient_authorization_hash: handle.authorized_relation_hash(),
        quotient_input,
        matrix_rows: coverage.quotient_rank,
        matrix_cols: coverage.quotient_rank,
        degree_bound: coverage
            .characteristic_polynomial
            .coeffs_low_to_high
            .len()
            .saturating_sub(1),
        coverage,
        annihilator,
        trace_hash,
    })
}

#[cfg(test)]
fn build_quotient_handle_from_target_relation(
    input: &ActionRelationInput,
    target: VariableId,
    basis_id: BasisHandleId,
    no_coordinate_roots_exported: bool,
    no_full_coordinate_rur_exported: bool,
) -> Result<crate::algebra::quotient::ProductionProvenancedTargetQuotientHandle, SolverError> {
    let quotient_input = build_quotient_handle_input_from_target_relation(
        input,
        target,
        basis_id,
        no_coordinate_roots_exported,
        no_full_coordinate_rur_exported,
    )?;
    build_production_target_relevant_quotient_handle(quotient_input)
}

fn build_quotient_handle_input_from_selection(
    selection: &TargetActionSelection,
    target: VariableId,
    basis_id: BasisHandleId,
    no_coordinate_roots_exported: bool,
    no_full_coordinate_rur_exported: bool,
) -> Result<ProductionQuotientHandleInput, SolverError> {
    match selection {
        TargetActionSelection::TargetOnly(input) => {
            build_quotient_handle_input_from_target_relation(
                input,
                target,
                basis_id,
                no_coordinate_roots_exported,
                no_full_coordinate_rur_exported,
            )
        }
        TargetActionSelection::AliasUnivariate {
            local_relation,
            alias_relation,
            local_variable,
            target_coeff,
            local_coeff,
        } => build_quotient_handle_input_from_alias_univariate(
            local_relation,
            alias_relation,
            target,
            *local_variable,
            target_coeff,
            local_coeff,
            basis_id,
            no_coordinate_roots_exported,
            no_full_coordinate_rur_exported,
        ),
        TargetActionSelection::GenericQuotient { inputs, variables } => {
            let authorized_relations = inputs
                .iter()
                .map(|input| input.polynomial.clone())
                .collect::<Vec<_>>();
            let mut input = build_production_target_relevant_quotient_input_from_relations(
                target,
                variables,
                authorized_relations,
                basis_id,
            )?;
            input.no_coordinate_roots_exported = no_coordinate_roots_exported;
            input.no_full_coordinate_rur_exported = no_full_coordinate_rur_exported;
            Ok(input)
        }
    }
}

fn build_quotient_handle_input_from_target_relation(
    input: &ActionRelationInput,
    target: VariableId,
    basis_id: BasisHandleId,
    no_coordinate_roots_exported: bool,
    no_full_coordinate_rur_exported: bool,
) -> Result<ProductionQuotientHandleInput, SolverError> {
    let poly = polynomial_to_univariate(&input.polynomial, target)
        .ok_or_else(|| certificate_design_gap("target action relation is not univariate"))?;
    let degree = univariate_degree(&poly)
        .ok_or_else(|| certificate_design_gap("target action quotient needs positive degree"))?;
    if degree == 0 {
        return Err(certificate_design_gap(
            "target action quotient needs positive degree",
        ));
    }
    let leading = poly.coeffs_low_to_high[degree].clone();
    if is_zero_q(&leading) {
        return Err(certificate_design_gap(
            "target action quotient leading coefficient is zero",
        ));
    }
    let basis = monomial_basis_polynomials(target, degree);
    let authorized_relations = vec![input.polynomial.clone()];
    let quotient_auth_hash = hash_authorized_relations(&authorized_relations);
    let inv_lc = div_q(&one_q(), &leading).map_err(|_| {
        certificate_design_gap("target action quotient leading coefficient is zero")
    })?;
    let mut columns = Vec::with_capacity(degree);
    for basis_index in 0..degree {
        let normal_form_vector = if basis_index + 1 < degree {
            unit_vector(degree, basis_index + 1)
        } else {
            VectorQ {
                entries: (0..degree)
                    .map(|idx| {
                        neg_q(
                            &div_q(&poly.coeffs_low_to_high[idx], &leading)
                                .expect("leading coefficient is nonzero"),
                        )
                    })
                    .collect(),
            }
        };
        let membership_certificate = if basis_index + 1 < degree {
            MembershipCertificate {
                combination_terms: Vec::new(),
            }
        } else {
            MembershipCertificate {
                combination_terms: vec![MembershipTerm {
                    relation_id: 0,
                    multiplier: constant_poly(inv_lc.clone()),
                }],
            }
        };
        columns.push(make_action_column_certificate(
            target,
            basis_index,
            &basis,
            &authorized_relations,
            quotient_auth_hash,
            normal_form_vector,
            membership_certificate,
        )?);
    }
    Ok(ProductionQuotientHandleInput {
        basis_id,
        basis_scope: BasisScope::TargetRelevant {
            variables: vec![target],
        },
        authorized_relation_hash: quotient_auth_hash,
        authorized_relations,
        basis_polynomials: basis.clone(),
        normal_form_basis_certificate: normal_form_basis_certificate(&basis, quotient_auth_hash),
        action_columns: BTreeMap::from([(target, columns)]),
        no_coordinate_roots_exported,
        no_full_coordinate_rur_exported,
    })
}

#[allow(clippy::too_many_arguments)]
fn build_quotient_handle_input_from_alias_univariate(
    local_relation: &ActionRelationInput,
    alias_relation: &ActionRelationInput,
    target: VariableId,
    local_variable: VariableId,
    target_coeff: &RationalQ,
    local_coeff: &RationalQ,
    basis_id: BasisHandleId,
    no_coordinate_roots_exported: bool,
    no_full_coordinate_rur_exported: bool,
) -> Result<ProductionQuotientHandleInput, SolverError> {
    let poly = polynomial_to_univariate(&local_relation.polynomial, local_variable)
        .ok_or_else(|| certificate_design_gap("local quotient relation is not univariate"))?;
    let degree = univariate_degree(&poly)
        .ok_or_else(|| certificate_design_gap("local quotient needs positive degree"))?;
    if degree == 0 {
        return Err(certificate_design_gap(
            "local quotient needs positive degree",
        ));
    }
    let leading = poly.coeffs_low_to_high[degree].clone();
    if is_zero_q(&leading) || is_zero_q(target_coeff) {
        return Err(certificate_design_gap(
            "target-action alias or local leading coefficient is zero",
        ));
    }
    let target_coeff_inv = div_q(&one_q(), target_coeff)
        .map_err(|_| certificate_design_gap("target-action alias target coefficient is zero"))?;
    let alpha =
        neg_q(&div_q(local_coeff, target_coeff).map_err(|_| {
            certificate_design_gap("target-action alias target coefficient is zero")
        })?);
    let inv_lc = div_q(&one_q(), &leading)
        .map_err(|_| certificate_design_gap("local quotient leading coefficient is zero"))?;
    let alpha_over_lc = mul_q(&alpha, &inv_lc);
    let basis = monomial_basis_polynomials(local_variable, degree);
    let authorized_relations = vec![
        local_relation.polynomial.clone(),
        alias_relation.polynomial.clone(),
    ];
    let quotient_auth_hash = hash_authorized_relations(&authorized_relations);
    let mut columns = Vec::with_capacity(degree);
    for basis_index in 0..degree {
        let normal_form_vector = if basis_index + 1 < degree {
            let mut vector = unit_vector(degree, basis_index + 1);
            vector.entries[basis_index + 1] = alpha.clone();
            vector
        } else {
            VectorQ {
                entries: (0..degree)
                    .map(|idx| {
                        neg_q(&mul_q(
                            &alpha,
                            &div_q(&poly.coeffs_low_to_high[idx], &leading)
                                .expect("leading coefficient is nonzero"),
                        ))
                    })
                    .collect(),
            }
        };
        let alias_multiplier = poly_scale(
            &monomial_power(local_variable, basis_index),
            &target_coeff_inv,
        );
        let mut combination_terms = vec![MembershipTerm {
            relation_id: 1,
            multiplier: alias_multiplier,
        }];
        if basis_index + 1 == degree {
            combination_terms.push(MembershipTerm {
                relation_id: 0,
                multiplier: constant_poly(alpha_over_lc.clone()),
            });
        }
        columns.push(make_action_column_certificate(
            target,
            basis_index,
            &basis,
            &authorized_relations,
            quotient_auth_hash,
            normal_form_vector,
            MembershipCertificate { combination_terms },
        )?);
    }
    let mut scope_variables = vec![target, local_variable];
    scope_variables.sort();
    scope_variables.dedup();
    Ok(ProductionQuotientHandleInput {
        basis_id,
        basis_scope: BasisScope::TargetRelevant {
            variables: scope_variables,
        },
        authorized_relation_hash: quotient_auth_hash,
        authorized_relations,
        basis_polynomials: basis.clone(),
        normal_form_basis_certificate: normal_form_basis_certificate(&basis, quotient_auth_hash),
        action_columns: BTreeMap::from([(target, columns)]),
        no_coordinate_roots_exported,
        no_full_coordinate_rur_exported,
    })
}

fn polynomial_to_univariate(
    poly: &SparsePolynomialQ,
    target: VariableId,
) -> Option<UniPolynomialQ> {
    let target_set = [target].into_iter().collect::<BTreeSet<_>>();
    if !poly_variables(poly).is_subset(&target_set) {
        return None;
    }
    let degree = poly
        .terms
        .iter()
        .map(|term| target_exponent(term, target))
        .max()
        .unwrap_or(0);
    let mut coeffs = vec![zero_q(); degree as usize + 1];
    for term in &poly.terms {
        let exp = target_exponent(term, target) as usize;
        coeffs[exp] = add_q(&coeffs[exp], &term.coeff);
    }
    Some(normalize_univariate(UniPolynomialQ {
        variable: target,
        coeffs_low_to_high: coeffs,
        hash: hash_sequence("univariate", &[]),
    }))
}

fn target_exponent(term: &TermQ, target: VariableId) -> u32 {
    term.monomial
        .exponents
        .iter()
        .find(|(var, _)| *var == target)
        .map_or(0, |(_, exp)| *exp)
}

fn monomial_power(var: VariableId, exp: usize) -> SparsePolynomialQ {
    let mut out = constant_poly(one_q());
    for _ in 0..exp {
        out = poly_mul(&out, &variable_poly(var));
    }
    out
}

fn univariate_degree(poly: &UniPolynomialQ) -> Option<usize> {
    poly.coeffs_low_to_high
        .iter()
        .rposition(|coeff| !is_zero_q(coeff))
}

fn univariate_to_polynomial(poly: &UniPolynomialQ) -> SparsePolynomialQ {
    normalize_poly(SparsePolynomialQ {
        terms: poly
            .coeffs_low_to_high
            .iter()
            .enumerate()
            .filter_map(|(exp, coeff)| {
                (!is_zero_q(coeff)).then(|| TermQ {
                    coeff: coeff.clone(),
                    monomial: normalize_monomial(if exp == 0 {
                        Vec::new()
                    } else {
                        vec![(poly.variable, exp as u32)]
                    }),
                })
            })
            .collect(),
        hash: hash_sequence("poly", &[]),
    })
}

fn select_target_action_selection(
    inputs: &[ActionRelationInput],
    target: VariableId,
) -> Option<TargetActionSelection> {
    let mut candidates = inputs
        .iter()
        .filter_map(|input| {
            let poly = polynomial_to_univariate(&input.polynomial, target)?;
            let degree = univariate_degree(&poly)?;
            (degree > 0).then(|| {
                (
                    degree,
                    poly_monomial_count(&input.polynomial),
                    input.source_hash,
                    TargetActionSelection::TargetOnly(input.clone()),
                )
            })
        })
        .collect::<Vec<_>>();
    for alias_relation in inputs {
        let Some((local_variable, target_coeff, local_coeff)) =
            parse_target_alias_relation(&alias_relation.polynomial, target)
        else {
            continue;
        };
        for local_relation in inputs {
            if local_relation.source_hash == alias_relation.source_hash {
                continue;
            }
            let Some(poly) = polynomial_to_univariate(&local_relation.polynomial, local_variable)
            else {
                continue;
            };
            let Some(degree) = univariate_degree(&poly) else {
                continue;
            };
            if degree == 0 {
                continue;
            }
            candidates.push((
                degree,
                poly_monomial_count(&local_relation.polynomial)
                    + poly_monomial_count(&alias_relation.polynomial),
                local_relation.source_hash,
                TargetActionSelection::AliasUnivariate {
                    local_relation: local_relation.clone(),
                    alias_relation: alias_relation.clone(),
                    local_variable,
                    target_coeff: target_coeff.clone(),
                    local_coeff: local_coeff.clone(),
                },
            ));
        }
    }
    if let Some(generic) = generic_target_action_selection(inputs, target) {
        let source_hash = hash_sequence(
            "target-action-generic-selection",
            &generic
                .source_hashes()
                .iter()
                .map(|hash| hash.0.to_vec())
                .collect::<Vec<_>>(),
        );
        candidates.push((
            generic.planned_rank_estimate_for_selection_only(),
            generic
                .source_polynomials()
                .iter()
                .map(|p| poly_monomial_count(p))
                .sum(),
            source_hash,
            generic,
        ));
    }
    candidates.sort_by_key(|(degree, terms, hash, _)| (*degree, *terms, *hash));
    candidates.into_iter().next().map(|(_, _, _, input)| input)
}

impl TargetActionSelection {
    fn planned_rank_estimate_for_selection_only(&self) -> usize {
        match self {
            TargetActionSelection::GenericQuotient { variables, .. } => {
                variables.len().max(1).saturating_mul(
                    self.source_polynomials()
                        .iter()
                        .map(|poly| poly_total_degree(poly) as usize + 1)
                        .max()
                        .unwrap_or(2)
                        .max(2),
                )
            }
            _ => self
                .source_polynomials()
                .iter()
                .map(|poly| poly_total_degree(poly) as usize)
                .max()
                .unwrap_or(1),
        }
    }
}

fn generic_target_action_selection(
    inputs: &[ActionRelationInput],
    target: VariableId,
) -> Option<TargetActionSelection> {
    if inputs.len() < 2 {
        return None;
    }
    let mut variables = inputs
        .iter()
        .flat_map(|input| poly_variables(&input.polynomial))
        .collect::<Vec<_>>();
    variables.sort();
    variables.dedup();
    if !variables.contains(&target) {
        return None;
    }
    Some(TargetActionSelection::GenericQuotient {
        inputs: inputs.to_vec(),
        variables,
    })
}

fn parse_target_alias_relation(
    poly: &SparsePolynomialQ,
    target: VariableId,
) -> Option<(VariableId, RationalQ, RationalQ)> {
    let mut target_coeff = zero_q();
    let mut local_terms = BTreeMap::<VariableId, RationalQ>::new();
    for term in &poly.terms {
        if term.monomial.exponents.len() != 1 {
            return None;
        }
        let (var, exp) = term.monomial.exponents[0];
        if exp != 1 {
            return None;
        }
        if var == target {
            target_coeff = add_q(&target_coeff, &term.coeff);
        } else {
            let next = local_terms
                .remove(&var)
                .map_or(term.coeff.clone(), |old| add_q(&old, &term.coeff));
            if !is_zero_q(&next) {
                local_terms.insert(var, next);
            }
        }
    }
    if is_zero_q(&target_coeff) || local_terms.len() != 1 {
        return None;
    }
    let (local_variable, local_coeff) = local_terms.into_iter().next()?;
    (!is_zero_q(&local_coeff)).then_some((local_variable, target_coeff, local_coeff))
}

fn collect_relation_inputs(
    block: &ProjectionBlock,
    system: &CompressedSystemQ,
    child_messages: &[ProjectionMessage],
) -> Vec<ActionRelationInput> {
    let mut inputs = block_relations(block, system)
        .into_iter()
        .map(|relation| ActionRelationInput {
            polynomial: relation.polynomial.clone(),
            source_relation_ids: vec![relation.id],
            source_hash: relation.hash,
            child_message_hash: None,
        })
        .collect::<Vec<_>>();
    for message in child_messages {
        for relation in &message.relation_generators {
            inputs.push(ActionRelationInput {
                polynomial: relation.clone(),
                source_relation_ids: message.source_relation_ids.clone(),
                source_hash: relation.hash,
                child_message_hash: Some(message.package_hash),
            });
        }
    }
    inputs
}

fn planned_relation_inputs(
    plan: &KernelExecutionPlan,
    ctx: &KernelContext,
) -> Result<Vec<ActionRelationInput>, SolverError> {
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
    let mut inputs = ctx
        .system
        .relations
        .iter()
        .filter(|relation| {
            relation_ids.contains(&relation.id) && source_hashes.contains(&relation.hash)
        })
        .map(|relation| ActionRelationInput {
            polynomial: relation.polynomial.clone(),
            source_relation_ids: vec![relation.id],
            source_hash: relation.hash,
            child_message_hash: None,
        })
        .collect::<Vec<_>>();
    for message in &ctx.child_messages {
        if !child_hashes.contains(&message.package_hash) {
            continue;
        }
        for relation in &message.relation_generators {
            if source_hashes.contains(&relation.hash) {
                inputs.push(ActionRelationInput {
                    polynomial: relation.clone(),
                    source_relation_ids: message.source_relation_ids.clone(),
                    source_hash: relation.hash,
                    child_message_hash: Some(message.package_hash),
                });
            }
        }
    }
    validate_source_hash_coverage(plan, &inputs)?;
    Ok(inputs)
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

fn validate_target_action_plan_binding(
    plan: &KernelExecutionPlan,
    ctx: &KernelContext,
) -> Result<(), SolverError> {
    if plan.kernel_kind != KernelKind::TargetActionKrylov {
        return Err(implementation_bug("target-action-krylov kind mismatch"));
    }
    if hash_kernel_execution_plan(plan) != plan.plan_hash {
        return Err(implementation_bug(
            "target-action-krylov execution plan hash mismatch",
        ));
    }
    if plan.block_id != ctx.block.block_id {
        return Err(implementation_bug("target-action-krylov block id mismatch"));
    }
    if plan.input_block_authorization_hash != ctx.block.authorization_hash {
        return Err(implementation_bug(
            "target-action-krylov block authorization hash mismatch",
        ));
    }
    if plan.certificate_route != CertificateRoute::VerifiedCharacteristicSupportCoverage {
        return Err(implementation_bug(
            "target-action-krylov certificate route mismatch",
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
            "target-action-krylov child message hash binding mismatch",
        ));
    }
    Ok(())
}

fn validate_source_hash_coverage(
    plan: &KernelExecutionPlan,
    inputs: &[ActionRelationInput],
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
            "target-action-krylov source relation hash coverage mismatch",
        ));
    }
    Ok(())
}

fn validate_selection_hash_coverage(
    plan: &KernelExecutionPlan,
    selection: &TargetActionSelection,
) -> Result<(), SolverError> {
    let mut expected = plan.source_relation_hashes.clone();
    let mut actual = selection.source_hashes();
    expected.sort();
    actual.sort();
    if expected != actual {
        return Err(implementation_bug(
            "target-action-krylov selected source relation hash mismatch",
        ));
    }
    Ok(())
}

fn target_action_krylov_certificate_hash(
    plan: &KernelExecutionPlan,
    trace: &TargetActionKrylovTrace,
) -> Hash {
    let mut chunks = vec![
        plan.plan_hash.0.to_vec(),
        trace.quotient_authorization_hash.0.to_vec(),
        trace.coverage.quotient_handle_hash.0.to_vec(),
        trace.coverage.basis_hash.0.to_vec(),
        trace.coverage.quotient_rank.to_be_bytes().to_vec(),
        trace.coverage.target_action_matrix_hash.0.to_vec(),
    ];
    chunks.extend(
        trace
            .coverage
            .column_normal_form_certificate_hashes
            .iter()
            .map(|hash| hash.0.to_vec()),
    );
    chunks.extend([
        trace.coverage.characteristic_polynomial_hash.0.to_vec(),
        trace.coverage.cayley_hamilton_verification_hash.0.to_vec(),
        vec![trace.coverage.no_coordinate_roots_exported as u8],
        vec![trace.coverage.no_full_coordinate_rur_exported as u8],
        trace.annihilator.polynomial_hash.0.to_vec(),
        trace
            .annihilator
            .cayley_hamilton_verification_hash
            .0
            .to_vec(),
        trace.relation.hash.0.to_vec(),
        trace.trace_hash.0.to_vec(),
    ]);
    hash_sequence("target-action-krylov-certificate", &chunks)
}

fn finish_admission(
    block: &ProjectionBlock,
    status: KernelAdmissionStatus,
    execution_plan: Option<KernelExecutionPlan>,
) -> KernelAdmission {
    let mut chunks = vec![
        format!("{:?}", KernelKind::TargetActionKrylov).into_bytes(),
        block.block_id.0.to_be_bytes().to_vec(),
        format!("{status:?}").into_bytes(),
    ];
    if let Some(plan) = &execution_plan {
        chunks.push(plan.plan_hash.0.to_vec());
    }
    KernelAdmission {
        kind: KernelKind::TargetActionKrylov,
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

fn certificate_design_gap(message: &str) -> SolverError {
    SolverError {
        target: None,
        kind: SolverErrorKind::Failure(FailureKind::CertificateDesignGap {
            constructed_object_hash: hash_sequence(
                "certificate-gap",
                &[message.as_bytes().to_vec()],
            ),
            missing_certificate_kind: message.to_owned(),
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
    use crate::compose::message::{MessageRepresentation, ProjectionStrength};
    use crate::kernels::traits::{KernelKind, TargetProjectionKernel};
    use crate::planner::admission::{collect_kernel_admissions, KernelAdmissionStatus};
    use crate::planner::kernel_plan::{
        hash_kernel_execution_plan, support_plan_hash, PlanWorkClassification,
    };
    use crate::planner::probes::run_cost_probes;
    use crate::preprocess::compression::CompressionState;
    use crate::problem::canonicalize::canonicalize_system;
    use crate::problem::context::new_context;
    use crate::problem::input::make_problem;
    use crate::problem::validate::validate_input;
    use crate::solver::options::SolverOptions;
    use crate::types::hash::hash_sequence;
    use crate::types::ids::{BlockId, VariableId};
    use crate::types::polynomial::{constant_poly, poly_add, poly_mul, poly_sub, variable_poly};
    use crate::types::rational::int_q;
    use crate::verify::certificates::KernelCertificatePayload;

    use super::*;

    #[test]
    fn p8c_action_krylov_kernel_produces_verified_characteristic_support() {
        let t = VariableId(0);
        let relation = poly_mul(
            &poly_sub(&variable_poly(t), &constant_poly(int_q(1))),
            &poly_sub(&variable_poly(t), &constant_poly(int_q(2))),
        );
        let compressed = compressed_system(t, vec![relation]);
        let block = test_block(&compressed, [t], [t]);
        let mut solver_ctx = new_context(SolverOptions::default());
        let mut kctx = KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };
        let kernel = TargetActionKrylovKernel;
        let admission = kernel.admit(&kctx.block, &kctx);
        assert!(matches!(admission.status, KernelAdmissionStatus::Admitted));
        let plan = kernel.plan(&admission, &kctx, &solver_ctx).unwrap();
        let message = kernel.execute(&plan, &mut kctx, &mut solver_ctx).unwrap();

        assert_eq!(message.kernel_kind, KernelKind::TargetActionKrylov);
        assert_eq!(
            message.representation,
            MessageRepresentation::QuotientAction
        );
        assert_eq!(
            message.projection_strength,
            ProjectionStrength::CandidateCoverStrong
        );
        assert_eq!(message.relation_generators.len(), 1);
        assert_eq!(
            message.relation_generators[0],
            poly_mul(
                &poly_sub(&variable_poly(t), &constant_poly(int_q(1))),
                &poly_sub(&variable_poly(t), &constant_poly(int_q(2))),
            )
        );
        assert!(kernel.replay(&message, &kctx).accepted);
    }

    #[test]
    fn p8c_planner_admits_target_action_krylov_for_verified_characteristic_path() {
        let t = VariableId(0);
        let relation = poly_mul(
            &poly_sub(&variable_poly(t), &constant_poly(int_q(1))),
            &poly_sub(&variable_poly(t), &constant_poly(int_q(2))),
        );
        let compressed = compressed_system(t, vec![relation]);
        let block = test_block(&compressed, [t], [t]);
        let mut solver_ctx = new_context(SolverOptions::default());
        let probes = run_cost_probes(&block, &compressed, &mut solver_ctx);
        let admissions = collect_kernel_admissions(&block, &compressed, &probes, &solver_ctx);
        let admission = admissions
            .iter()
            .find(|admission| admission.kind == KernelKind::TargetActionKrylov)
            .unwrap();
        assert!(matches!(admission.status, KernelAdmissionStatus::Admitted));
    }

    #[test]
    fn p8c_undercovered_single_vector_recurrence_cannot_escape_as_support() {
        let t = VariableId(0);
        let relation = poly_mul(
            &poly_sub(&variable_poly(t), &constant_poly(int_q(1))),
            &poly_sub(&variable_poly(t), &constant_poly(int_q(2))),
        );
        let compressed = compressed_system(t, vec![relation]);
        let input = action_input_from_compressed(&compressed);
        let handle =
            build_quotient_handle_from_target_relation(&input, t, BasisHandleId(88), true, true)
                .unwrap();
        let missed_eigenvalue_start = VectorQ {
            entries: vec![int_q(-2), int_q(1)],
        };
        let undercovered = block_krylov_sequence(
            &handle,
            t,
            KrylovPlan {
                start_vectors: vec![missed_eigenvalue_start],
                max_steps: 3,
            },
        )
        .unwrap();
        let recurrence = recover_recurrence(&undercovered).unwrap();
        assert_eq!(
            recurrence.polynomial.coeffs_low_to_high,
            vec![int_q(-1), int_q(1)]
        );
        let err = certify_krylov_coverage(&undercovered, &recurrence, &handle).unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::CertificateDesignGap);

        let block = test_block(&compressed, [t], [t]);
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
        assert_eq!(
            crate::types::polynomial::poly_total_degree(&message.relation_generators[0]),
            2
        );
    }

    #[test]
    fn p8c_quotient_handle_rejects_coordinate_root_or_rur_export() {
        let t = VariableId(0);
        let relation = poly_sub(
            &poly_mul(&variable_poly(t), &variable_poly(t)),
            &constant_poly(int_q(2)),
        );
        let compressed = compressed_system(t, vec![relation]);
        let input = action_input_from_compressed(&compressed);

        let err =
            build_quotient_handle_from_target_relation(&input, t, BasisHandleId(90), false, true)
                .unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::InvalidInput);

        let err =
            build_quotient_handle_from_target_relation(&input, t, BasisHandleId(91), true, false)
                .unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::InvalidInput);
    }

    #[test]
    fn p8c_action_krylov_rejects_auth_and_source_hash_tamper() {
        let t = VariableId(0);
        let relation = poly_mul(
            &poly_sub(&variable_poly(t), &constant_poly(int_q(1))),
            &poly_sub(&variable_poly(t), &constant_poly(int_q(2))),
        );
        let compressed = compressed_system(t, vec![relation]);
        let block = test_block(&compressed, [t], [t]);
        let mut solver_ctx = new_context(SolverOptions::default());
        let mut kctx = KernelContext {
            block: block.clone(),
            system: compressed,
            child_messages: Vec::new(),
        };
        let kernel = TargetActionKrylovKernel;
        let admission = kernel.admit(&kctx.block, &kctx);
        let mut plan = kernel.plan(&admission, &kctx, &solver_ctx).unwrap();

        let mut bad_auth_ctx = kctx.clone();
        bad_auth_ctx.block.authorization_hash = hash_sequence("tampered-auth", &[]);
        let err = kernel
            .execute(&plan, &mut bad_auth_ctx, &mut solver_ctx)
            .unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::ImplementationBug);

        plan.source_relation_hashes[0] = hash_sequence("tampered-source", &[]);
        plan.plan_hash = hash_kernel_execution_plan(&plan);
        let err = kernel
            .execute(&plan, &mut kctx, &mut solver_ctx)
            .unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::ImplementationBug);
    }

    #[test]
    fn p12g_action_krylov_builds_non_target_only_quotient_action() {
        let t = VariableId(0);
        let x = VariableId(1);
        let local_minpoly = poly_sub(
            &poly_mul(&variable_poly(x), &variable_poly(x)),
            &constant_poly(int_q(2)),
        );
        let target_alias = poly_sub(&variable_poly(t), &variable_poly(x));
        let compressed = compressed_system(t, vec![local_minpoly, target_alias]);
        let block = test_block(&compressed, [t, x], [t]);
        let mut solver_ctx = new_context(SolverOptions::default());
        let mut kctx = KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };
        let kernel = TargetActionKrylovKernel;
        let admission = kernel.admit(&kctx.block, &kctx);
        assert!(matches!(admission.status, KernelAdmissionStatus::Admitted));
        let plan = kernel.plan(&admission, &kctx, &solver_ctx).unwrap();
        let message = kernel.execute(&plan, &mut kctx, &mut solver_ctx).unwrap();

        assert_eq!(message.kernel_kind, KernelKind::TargetActionKrylov);
        assert_eq!(message.exported_variables, vec![t]);
        assert_eq!(message.eliminated_variables, vec![x]);
        assert_eq!(
            message.relation_generators[0],
            poly_sub(
                &poly_mul(&variable_poly(t), &variable_poly(t)),
                &constant_poly(int_q(2)),
            )
        );
        assert_eq!(message.source_relation_ids.len(), 2);
        let verification =
            crate::verify::verify_message::verify_projection_message(&message, &kctx);
        assert!(verification.is_ok(), "{verification:?}");
        assert!(kernel.replay(&message, &kctx).accepted);
    }

    #[test]
    fn fcr_action_multivariate_quotient_no_target_relation() {
        let t = VariableId(0);
        let x = VariableId(1);
        let y = VariableId(2);
        let x2_plus_y = poly_add(
            &poly_mul(&variable_poly(x), &variable_poly(x)),
            &variable_poly(y),
        );
        let y2 = poly_mul(&variable_poly(y), &variable_poly(y));
        let compressed = compressed_system(
            t,
            vec![
                poly_sub(&x2_plus_y, &constant_poly(int_q(1))),
                poly_sub(&y2, &variable_poly(x)),
                poly_sub(
                    &poly_sub(&variable_poly(t), &variable_poly(x)),
                    &variable_poly(y),
                ),
            ],
        );
        let block = test_block(&compressed, [t, x, y], [t]);
        let mut solver_ctx = new_context(SolverOptions::default());
        let mut kctx = KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };
        let kernel = TargetActionKrylovKernel;
        let admission = kernel.admit(&kctx.block, &kctx);
        assert!(matches!(admission.status, KernelAdmissionStatus::Admitted));
        let plan = kernel.plan(&admission, &kctx, &solver_ctx).unwrap();
        let message = kernel.execute(&plan, &mut kctx, &mut solver_ctx).unwrap();

        assert_eq!(message.kernel_kind, KernelKind::TargetActionKrylov);
        assert_eq!(message.source_relation_ids.len(), 3);
        assert_verified_action_message(&kernel, &message, &kctx);
        let proof = target_action_payload(&message);
        assert_eq!(proof.quotient_input.authorized_relations.len(), 3);
        assert!(proof.quotient_input.basis_polynomials.len() >= 4);
        assert!(message.relation_generators[0].terms.iter().all(|term| term
            .monomial
            .exponents
            .iter()
            .all(|(var, _)| *var == t)));
    }

    #[test]
    fn fcr_action_target_is_nonlinear_expression() {
        let t = VariableId(0);
        let x = VariableId(1);
        let y = VariableId(2);
        let xy = poly_mul(&variable_poly(x), &variable_poly(y));
        let compressed = compressed_system(
            t,
            vec![
                poly_sub(
                    &poly_mul(&variable_poly(x), &variable_poly(x)),
                    &constant_poly(int_q(2)),
                ),
                poly_sub(
                    &poly_mul(&variable_poly(y), &variable_poly(y)),
                    &constant_poly(int_q(3)),
                ),
                poly_sub(&poly_sub(&variable_poly(t), &xy), &variable_poly(x)),
            ],
        );
        let block = test_block(&compressed, [t, x, y], [t]);
        let mut solver_ctx = new_context(SolverOptions::default());
        let mut kctx = KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };
        let kernel = TargetActionKrylovKernel;
        let admission = kernel.admit(&kctx.block, &kctx);
        assert!(matches!(admission.status, KernelAdmissionStatus::Admitted));
        let plan = kernel.plan(&admission, &kctx, &solver_ctx).unwrap();
        let message = kernel.execute(&plan, &mut kctx, &mut solver_ctx).unwrap();

        assert_verified_action_message(&kernel, &message, &kctx);
        let proof = target_action_payload(&message);
        assert_eq!(proof.quotient_input.authorized_relations.len(), 3);
        assert_eq!(proof.quotient_input.basis_polynomials.len(), 4);
        assert!(proof
            .quotient_input
            .basis_polynomials
            .iter()
            .any(|basis| *basis == variable_poly(x)));
        assert!(proof
            .quotient_input
            .basis_polynomials
            .iter()
            .any(|basis| *basis == variable_poly(y)));
        assert!(proof
            .quotient_input
            .basis_polynomials
            .iter()
            .any(|basis| *basis == xy));
        assert_eq!(
            proof.quotient_input.action_columns.get(&t).unwrap().len(),
            proof.quotient_input.basis_polynomials.len()
        );
    }

    #[test]
    fn fcr_action_rejects_injected_basis_or_column() {
        let t = VariableId(0);
        let x = VariableId(1);
        let y = VariableId(2);
        let xy = poly_mul(&variable_poly(x), &variable_poly(y));
        let compressed = compressed_system(
            t,
            vec![
                poly_sub(
                    &poly_mul(&variable_poly(x), &variable_poly(x)),
                    &constant_poly(int_q(2)),
                ),
                poly_sub(
                    &poly_mul(&variable_poly(y), &variable_poly(y)),
                    &constant_poly(int_q(3)),
                ),
                poly_sub(&poly_sub(&variable_poly(t), &xy), &variable_poly(x)),
            ],
        );
        let block = test_block(&compressed, [t, x, y], [t]);
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
        assert_verified_action_message(&kernel, &message, &kctx);

        let mut bad_basis = message.clone();
        target_action_payload_mut(&mut bad_basis)
            .quotient_input
            .basis_polynomials[0] = variable_poly(x);
        assert_rejected_action_message(&kernel, &bad_basis, &kctx);

        let mut bad_column = message.clone();
        let column = &mut target_action_payload_mut(&mut bad_column)
            .quotient_input
            .action_columns
            .get_mut(&t)
            .unwrap()[0];
        column.normal_form_vector.entries[0] = int_q(99);
        assert_rejected_action_message(&kernel, &bad_column, &kctx);

        let mut bad_membership = message.clone();
        target_action_payload_mut(&mut bad_membership)
            .quotient_input
            .action_columns
            .get_mut(&t)
            .unwrap()[0]
            .normal_form_certificate
            .membership_certificate
            .combination_terms
            .clear();
        assert_rejected_action_message(&kernel, &bad_membership, &kctx);

        let mut bad_auth = message.clone();
        target_action_payload_mut(&mut bad_auth)
            .quotient_input
            .authorized_relation_hash = hash_sequence("tampered-action-auth", &[]);
        assert_rejected_action_message(&kernel, &bad_auth, &kctx);

        let mut bad_output = message.clone();
        bad_output.relation_generators[0] = poly_sub(&variable_poly(t), &constant_poly(int_q(7)));
        assert_rejected_action_message(&kernel, &bad_output, &kctx);
    }

    #[test]
    fn p12g_action_krylov_pure_plan_template_is_replayed_in_execute() {
        let t = VariableId(0);
        let x = VariableId(1);
        let compressed = compressed_system(
            t,
            vec![
                poly_sub(
                    &poly_mul(&variable_poly(x), &variable_poly(x)),
                    &constant_poly(int_q(2)),
                ),
                poly_sub(&variable_poly(t), &variable_poly(x)),
            ],
        );
        let block = test_block(&compressed, [t, x], [t]);
        let mut solver_ctx = new_context(SolverOptions::default());
        let mut kctx = KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };
        let kernel = TargetActionKrylovKernel;
        let admission = kernel.admit(&kctx.block, &kctx);
        let mut plan = kernel.plan(&admission, &kctx, &solver_ctx).unwrap();

        assert!(matches!(
            plan.plan_work_classification,
            PlanWorkClassification::PurePlan
        ));
        plan.support_plan
            .template_plan
            .as_mut()
            .unwrap()
            .row_monomial_hash = hash_sequence("tampered-target-action-template", &[]);
        plan.support_plan.support_hash = support_plan_hash(&plan.support_plan);
        plan.plan_hash = hash_kernel_execution_plan(&plan);
        let err = kernel
            .execute(&plan, &mut kctx, &mut solver_ctx)
            .unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::ImplementationBug);
    }

    #[test]
    fn p12g_plan_does_not_silently_execute_final_relation() {
        let t = VariableId(0);
        let x = VariableId(1);
        let compressed = compressed_system(
            t,
            vec![
                poly_sub(
                    &poly_mul(&variable_poly(x), &variable_poly(x)),
                    &constant_poly(int_q(2)),
                ),
                poly_sub(&variable_poly(t), &variable_poly(x)),
            ],
        );
        let block = test_block(&compressed, [t, x], [t]);
        let solver_ctx = new_context(SolverOptions::default());
        let kctx = KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };
        let kernel = TargetActionKrylovKernel;
        let plan = kernel
            .plan(&kernel.admit(&kctx.block, &kctx), &kctx, &solver_ctx)
            .unwrap();

        assert_eq!(
            plan.plan_work_classification,
            PlanWorkClassification::PurePlan
        );
        let template = plan.support_plan.template_plan.as_ref().unwrap();
        assert_eq!(template.matrix_rows, plan.support_plan.degree_bound);
        assert_eq!(template.matrix_cols, plan.support_plan.degree_bound);
        assert_eq!(plan.exported_variables, vec![t]);
        assert_eq!(plan.eliminated_variables, vec![x]);
    }

    #[test]
    fn p12g_target_action_support_plan_hash_tamper_fails() {
        let t = VariableId(0);
        let x = VariableId(1);
        let compressed = compressed_system(
            t,
            vec![
                poly_sub(
                    &poly_mul(&variable_poly(x), &variable_poly(x)),
                    &constant_poly(int_q(2)),
                ),
                poly_sub(&variable_poly(t), &variable_poly(x)),
            ],
        );
        let block = test_block(&compressed, [t, x], [t]);
        let mut solver_ctx = new_context(SolverOptions::default());
        let mut kctx = KernelContext {
            block,
            system: compressed,
            child_messages: Vec::new(),
        };
        let kernel = TargetActionKrylovKernel;
        let mut plan = kernel
            .plan(&kernel.admit(&kctx.block, &kctx), &kctx, &solver_ctx)
            .unwrap();
        plan.support_plan.degree_bound = plan.support_plan.degree_bound.saturating_add(1);
        plan.plan_hash = hash_kernel_execution_plan(&plan);

        let err = kernel
            .execute(&plan, &mut kctx, &mut solver_ctx)
            .unwrap_err();
        assert_eq!(err.public_status(), SolverStatus::ImplementationBug);
    }

    fn compressed_system(
        target: VariableId,
        relations: Vec<crate::types::polynomial::SparsePolynomialQ>,
    ) -> crate::preprocess::compression::CompressedSystemQ {
        let mut variables = vec![target];
        variables.extend(
            relations
                .iter()
                .flat_map(crate::types::polynomial::poly_variables)
                .filter(|var| *var != target),
        );
        variables.sort();
        variables.dedup();
        let canonical = canonicalize_system(
            validate_input(make_problem(variables, target, relations, Vec::new())).unwrap(),
        )
        .unwrap();
        CompressionState::from_system(canonical).to_compressed_system()
    }

    fn test_block<const N: usize, const M: usize>(
        compressed: &crate::preprocess::compression::CompressedSystemQ,
        local_variables: [VariableId; N],
        exported_variables: [VariableId; M],
    ) -> crate::graph::projection_dag::ProjectionBlock {
        let mut block = crate::graph::projection_dag::ProjectionBlock {
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

    fn action_input_from_compressed(
        compressed: &crate::preprocess::compression::CompressedSystemQ,
    ) -> ActionRelationInput {
        let relation = compressed.relations[0].clone();
        ActionRelationInput {
            polynomial: relation.polynomial,
            source_relation_ids: vec![relation.id],
            source_hash: relation.hash,
            child_message_hash: None,
        }
    }

    fn assert_verified_action_message(
        kernel: &TargetActionKrylovKernel,
        message: &ProjectionMessage,
        kctx: &KernelContext,
    ) {
        let verification = crate::verify::verify_message::verify_projection_message(message, kctx);
        assert!(verification.is_ok(), "{verification:?}");
        assert!(kernel.replay(message, kctx).accepted);
    }

    fn assert_rejected_action_message(
        kernel: &TargetActionKrylovKernel,
        message: &ProjectionMessage,
        kctx: &KernelContext,
    ) {
        assert!(crate::verify::verify_message::verify_projection_message(message, kctx).is_err());
        assert!(!kernel.replay(message, kctx).accepted);
    }

    fn target_action_payload(
        message: &ProjectionMessage,
    ) -> &crate::verify::certificates::TargetActionProjectionCertificate {
        match &message.certificate.payload {
            KernelCertificatePayload::TargetAction(proof) => proof,
            other => panic!("unexpected payload: {other:?}"),
        }
    }

    fn target_action_payload_mut(
        message: &mut ProjectionMessage,
    ) -> &mut crate::verify::certificates::TargetActionProjectionCertificate {
        match &mut message.certificate.payload {
            KernelCertificatePayload::TargetAction(proof) => proof,
            other => panic!("unexpected payload: {other:?}"),
        }
    }
}
