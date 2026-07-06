use std::collections::{BTreeMap, BTreeSet};

use std::panic::{catch_unwind, AssertUnwindSafe};

use serde::{Deserialize, Serialize};

use crate::graph::projection_dag::ProjectionBlock;
use crate::kernels::action_krylov::plan_target_action_krylov;
use crate::kernels::linear_affine::{find_triangular_affine_order, plan_linear_affine};
use crate::kernels::norm_trace_projection::plan_norm_trace_projection;
use crate::kernels::regular_chain_projection::plan_regular_chain_projection;
use crate::kernels::sparse_resultant::plan_sparse_resultant;
use crate::kernels::specialization_interpolation::plan_specialization_interpolation;
use crate::kernels::traits::KernelKind;
use crate::kernels::universal_elimination::plan_universal_elimination;
use crate::planner::algebraic_cost::RouteBudget;
use crate::planner::cost_model::{estimate_kernel_cost, RouteCostClass};
use crate::planner::kernel_plan::{
    hash_kernel_execution_plan, planned_failure_behavior, rank_plan, resource_bounds_hash,
    support_plan_hash, template_plan, CertificateRoute, KernelExecutionPlan, KernelSupportPlan,
    LocalNonfinitePolicy, ResourceBounds,
};
use crate::planner::probes::ProbeResults;
use crate::planner::relation_schedule::{
    build_dense_relation_search_schedule, build_sparse_relation_search_schedule,
    dense_relation_search_decline_reason, estimate_dense_relation_search_schedule,
    estimate_sparse_relation_search_schedule, sparse_relation_search_decline_reason,
    DenseRelationSearchSchedule, RelationSearchStage, SparseRelationSearchSchedule,
};
use crate::preprocess::compression::CompressedSystemQ;
use crate::problem::context::SolverContext;
use crate::result::status::SolverStatus;
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::{KernelPlanId, RelationId, VariableId};
use crate::types::polynomial::poly_variables;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KernelAdmission {
    pub kind: KernelKind,
    pub block_id: crate::types::ids::BlockId,
    pub status: KernelAdmissionStatus,
    pub exported_variables: Vec<VariableId>,
    pub eliminated_variables: Vec<VariableId>,
    pub execution_plan: Option<KernelExecutionPlan>,
    pub admission_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KernelAdmissionStatus {
    Admitted,
    Declined {
        reason: String,
    },
    CostProhibited {
        reason: String,
        estimate_hash: Hash,
    },
    PlanProbeFailed {
        reason: String,
        constructed_object_hash: Hash,
    },
}

impl KernelAdmission {
    pub fn is_admitted(&self) -> bool {
        matches!(self.status, KernelAdmissionStatus::Admitted)
    }
}

pub fn all_planner_kernel_kinds() -> Vec<KernelKind> {
    vec![
        KernelKind::TargetUnivariate,
        KernelKind::LinearAffine,
        KernelKind::TargetRelationSearch,
        KernelKind::SparseResultantProjection,
        KernelKind::TargetActionKrylov,
        KernelKind::NormTraceProjection,
        KernelKind::RegularChainProjection,
        KernelKind::SpecializationInterpolation,
        KernelKind::UniversalTargetElimination,
    ]
}

pub fn collect_kernel_admissions(
    block: &ProjectionBlock,
    system: &CompressedSystemQ,
    probes: &ProbeResults,
    ctx: &SolverContext,
) -> Vec<KernelAdmission> {
    let relation_map = system
        .relations
        .iter()
        .map(|relation| (relation.id, relation))
        .collect::<BTreeMap<_, _>>();
    all_planner_kernel_kinds()
        .into_iter()
        .enumerate()
        .map(|(index, kind)| {
            let plan_id = KernelPlanId(index as u32);
            let admission = match catch_unwind(AssertUnwindSafe(|| {
                build_kernel_admission(plan_id, kind, block, system, &relation_map, probes, ctx)
            })) {
                Ok(admission) => admission,
                Err(payload) => route_panic_admission(kind, block, panic_message(payload)),
            };
            bind_admission_algebraic_cost(admission, block, system, probes)
        })
        .collect()
}

fn build_kernel_admission(
    plan_id: KernelPlanId,
    kind: KernelKind,
    block: &ProjectionBlock,
    system: &CompressedSystemQ,
    relations_by_id: &BTreeMap<RelationId, &crate::problem::canonicalize::CanonicalRelationQ>,
    probes: &ProbeResults,
    ctx: &SolverContext,
) -> KernelAdmission {
    let exported_variables = sorted_set(&block.exported_variables);
    let eliminated_variables = block
        .local_variables
        .difference(&block.exported_variables)
        .copied()
        .collect::<Vec<_>>();
    let local_relations = block_relations(block, relations_by_id);
    let relation_polys = local_relations
        .iter()
        .map(|relation| relation.polynomial.clone())
        .collect::<Vec<_>>();
    let relation_hashes = local_relations
        .iter()
        .map(|relation| relation.hash)
        .collect::<Vec<_>>();
    let source_relation_ids = local_relations
        .iter()
        .map(|relation| relation.id)
        .collect::<Vec<_>>();
    let status_and_plan = match kind {
        KernelKind::TargetUnivariate => {
            let target_set = [system.target].into_iter().collect::<BTreeSet<_>>();
            let target_relations = local_relations
                .iter()
                .filter(|relation| poly_variables(&relation.polynomial).is_subset(&target_set))
                .collect::<Vec<_>>();
            if !target_relations.is_empty() {
                let target_relation_ids = target_relations
                    .iter()
                    .map(|relation| relation.id)
                    .collect::<Vec<_>>();
                let target_relation_hashes = target_relations
                    .iter()
                    .map(|relation| relation.hash)
                    .collect::<Vec<_>>();
                let target_degree_bound = target_relations
                    .iter()
                    .map(|relation| {
                        crate::types::polynomial::poly_total_degree(&relation.polynomial) as usize
                    })
                    .max()
                    .unwrap_or(1);
                admitted_with_plan(
                    plan_id,
                    kind,
                    block,
                    target_relation_ids,
                    target_relation_hashes,
                    vec![system.target],
                    eliminated_variables.clone(),
                    basic_support_plan(None, None, probes, target_degree_bound),
                    resource_bounds(ctx, Some(target_degree_bound), probes),
                    CertificateRoute::SourceMembershipCertificate,
                    LocalNonfinitePolicy::NotApplicable,
                )
            } else {
                declined("no authorized target-only relation")
            }
        }
        KernelKind::TargetRelationSearch => {
            if relation_polys.is_empty() {
                declined("no authorized local relations for dense relation search")
            } else {
                let dense_preflight = estimate_dense_relation_search_schedule(
                    &relation_polys,
                    &eliminated_variables,
                    &exported_variables,
                    &ctx.options,
                );
                if dense_preflight.materialization_allowed {
                    let schedule = build_dense_relation_search_schedule(
                        &relation_polys,
                        &eliminated_variables,
                        &exported_variables,
                        &ctx.options,
                    );
                    let degree_bound = schedule.e_cap;
                    admitted_with_plan(
                        plan_id,
                        kind,
                        block,
                        source_relation_ids,
                        relation_hashes,
                        exported_variables.clone(),
                        eliminated_variables.clone(),
                        basic_support_plan(Some(schedule), None, probes, degree_bound),
                        resource_bounds(ctx, Some(degree_bound), probes),
                        CertificateRoute::DenseRelationSearchMembership,
                        LocalNonfinitePolicy::NotApplicable,
                    )
                } else {
                    let sparse_preflight = estimate_sparse_relation_search_schedule(
                        &relation_polys,
                        &eliminated_variables,
                        &exported_variables,
                        &ctx.options,
                    );
                    if !sparse_preflight.feasible {
                        let reason = format!(
                            "{}; {}",
                            dense_relation_search_decline_reason(&dense_preflight),
                            sparse_relation_search_decline_reason(&sparse_preflight),
                        );
                        cost_prohibited(
                            &reason,
                            hash_sequence(
                                "target-relation-search-dense-and-sparse-preflight",
                                &[
                                    dense_preflight.preflight_hash.0.to_vec(),
                                    sparse_preflight.preflight_hash.0.to_vec(),
                                ],
                            ),
                        )
                    } else {
                        let schedule = build_sparse_relation_search_schedule(
                            &relation_polys,
                            &eliminated_variables,
                            &exported_variables,
                            &ctx.options,
                        );
                        let degree_bound = schedule.stage.export_degree;
                        let stage = schedule.stage.clone();
                        admitted_with_plan(
                            plan_id,
                            kind,
                            block,
                            source_relation_ids,
                            relation_hashes,
                            exported_variables.clone(),
                            eliminated_variables.clone(),
                            basic_support_plan(None, Some(schedule), probes, degree_bound),
                            relation_search_resource_bounds(ctx, &stage, degree_bound),
                            CertificateRoute::DenseRelationSearchMembership,
                            LocalNonfinitePolicy::NotApplicable,
                        )
                    }
                }
            }
        }
        KernelKind::UniversalTargetElimination => {
            match plan_universal_elimination(block, system, ctx, plan_id) {
                Ok(plan) => (KernelAdmissionStatus::Admitted, Some(plan)),
                Err(err) => route_plan_error("universal projection planning", block, &err),
            }
        }
        KernelKind::LinearAffine => {
            if let Some(order) = find_triangular_affine_order(block, system) {
                match plan_linear_affine(block, system, &order, ctx) {
                    Ok(plan) => (KernelAdmissionStatus::Admitted, Some(plan)),
                    Err(err) => route_plan_error("linear affine planning", block, &err),
                }
            } else {
                declined("no complete safe triangular affine elimination order")
            }
        }
        KernelKind::SparseResultantProjection => {
            match plan_sparse_resultant(block, system, ctx, plan_id) {
                Ok(plan) => (KernelAdmissionStatus::Admitted, Some(plan)),
                Err(err) => route_plan_error("sparse resultant planning", block, &err),
            }
        }
        KernelKind::TargetActionKrylov => {
            match plan_target_action_krylov(block, system, ctx, plan_id) {
                Ok(plan) => (KernelAdmissionStatus::Admitted, Some(plan)),
                Err(err) => route_plan_error("target action Krylov planning", block, &err),
            }
        }
        KernelKind::NormTraceProjection => {
            match plan_norm_trace_projection(block, system, ctx, plan_id) {
                Ok(plan) => (KernelAdmissionStatus::Admitted, Some(plan)),
                Err(err) => route_plan_error("norm/trace projection planning", block, &err),
            }
        }
        KernelKind::RegularChainProjection => {
            match plan_regular_chain_projection(block, system, ctx, plan_id) {
                Ok(plan) => (KernelAdmissionStatus::Admitted, Some(plan)),
                Err(err) => route_plan_error("regular-chain projection planning", block, &err),
            }
        }
        KernelKind::SpecializationInterpolation => {
            match plan_specialization_interpolation(block, system, ctx, plan_id) {
                Ok(plan) => (KernelAdmissionStatus::Admitted, Some(plan)),
                Err(err) => route_plan_error("specialization-interpolation planning", block, &err),
            }
        }
    };
    finish_admission(
        kind,
        block.block_id,
        exported_variables,
        eliminated_variables,
        status_and_plan.0,
        status_and_plan.1,
    )
}

fn admitted_with_plan(
    plan_id: KernelPlanId,
    kind: KernelKind,
    block: &ProjectionBlock,
    source_relation_ids: Vec<RelationId>,
    source_relation_hashes: Vec<Hash>,
    exported_variables: Vec<VariableId>,
    eliminated_variables: Vec<VariableId>,
    support_plan: KernelSupportPlan,
    resource_bounds: ResourceBounds,
    certificate_route: CertificateRoute,
    local_nonfinite_policy: LocalNonfinitePolicy,
) -> (KernelAdmissionStatus, Option<KernelExecutionPlan>) {
    let failure_behavior = planned_failure_behavior(
        vec![
            SolverStatus::AlgorithmicHardCase,
            SolverStatus::FiniteResourceFailure,
            SolverStatus::CertificateDesignGap,
        ],
        local_nonfinite_policy,
    );
    let execution_plan = KernelExecutionPlan::new(
        plan_id,
        block.block_id,
        kind,
        block.authorization_hash,
        source_relation_ids,
        source_relation_hashes,
        block.child_block_ids.clone(),
        Vec::new(),
        exported_variables,
        eliminated_variables,
        support_plan,
        resource_bounds,
        certificate_route,
        failure_behavior,
    );
    (KernelAdmissionStatus::Admitted, Some(execution_plan))
}

fn declined(reason: &str) -> (KernelAdmissionStatus, Option<KernelExecutionPlan>) {
    (
        KernelAdmissionStatus::Declined {
            reason: reason.to_owned(),
        },
        None,
    )
}

fn cost_prohibited(
    reason: &str,
    estimate_hash: Hash,
) -> (KernelAdmissionStatus, Option<KernelExecutionPlan>) {
    (
        KernelAdmissionStatus::CostProhibited {
            reason: reason.to_owned(),
            estimate_hash,
        },
        None,
    )
}

fn plan_probe_failed(
    reason: &str,
    constructed_object_hash: Hash,
) -> (KernelAdmissionStatus, Option<KernelExecutionPlan>) {
    (
        KernelAdmissionStatus::PlanProbeFailed {
            reason: reason.to_owned(),
            constructed_object_hash,
        },
        None,
    )
}

fn route_plan_error(
    label: &str,
    block: &ProjectionBlock,
    err: &crate::result::status::SolverError,
) -> (KernelAdmissionStatus, Option<KernelExecutionPlan>) {
    let class = if err.public_status() == SolverStatus::ImplementationBug {
        "route-local invariant failure"
    } else {
        "route-local planning failure"
    };
    plan_probe_failed(
        &format!(
            "{label} {class}; status={:?}; error={:?}",
            err.public_status(),
            err.kind
        ),
        hash_sequence(
            "route-plan-probe-failed",
            &[
                label.as_bytes().to_vec(),
                block.block_hash.0.to_vec(),
                format!("{:?}", err.public_status()).into_bytes(),
                format!("{:?}", err.kind).into_bytes(),
            ],
        ),
    )
}

fn route_panic_admission(
    kind: KernelKind,
    block: &ProjectionBlock,
    panic_message: String,
) -> KernelAdmission {
    let exported_variables = sorted_set(&block.exported_variables);
    let eliminated_variables = block
        .local_variables
        .difference(&block.exported_variables)
        .copied()
        .collect::<Vec<_>>();
    let constructed_object_hash = hash_sequence(
        "route-admission-panic",
        &[
            format!("{kind:?}").into_bytes(),
            block.block_hash.0.to_vec(),
            panic_message.as_bytes().to_vec(),
        ],
    );
    finish_admission(
        kind,
        block.block_id,
        exported_variables,
        eliminated_variables,
        KernelAdmissionStatus::PlanProbeFailed {
            reason: format!("route-local panic during admission: {panic_message}"),
            constructed_object_hash,
        },
        None,
    )
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string panic payload".to_owned()
    }
}

fn finish_admission(
    kind: KernelKind,
    block_id: crate::types::ids::BlockId,
    exported_variables: Vec<VariableId>,
    eliminated_variables: Vec<VariableId>,
    status: KernelAdmissionStatus,
    execution_plan: Option<KernelExecutionPlan>,
) -> KernelAdmission {
    let admission_hash = kernel_admission_hash(
        kind,
        block_id,
        &exported_variables,
        &eliminated_variables,
        &status,
        execution_plan.as_ref(),
    );
    KernelAdmission {
        kind,
        block_id,
        status,
        exported_variables,
        eliminated_variables,
        execution_plan,
        admission_hash,
    }
}

fn bind_admission_algebraic_cost(
    mut admission: KernelAdmission,
    block: &ProjectionBlock,
    system: &CompressedSystemQ,
    probes: &ProbeResults,
) -> KernelAdmission {
    let sparse_target_relation_plan = admission.kind == KernelKind::TargetRelationSearch
        && admission
            .execution_plan
            .as_ref()
            .is_some_and(|plan| plan.support_plan.sparse_relation_search_schedule.is_some());
    if sparse_target_relation_plan {
        if let Some(plan) = admission.execution_plan.as_mut() {
            plan.route_budget = RouteBudget::from_estimate(&plan.algebraic_work_estimate);
            plan.plan_hash = hash_kernel_execution_plan(plan);
        }
        admission.admission_hash = kernel_admission_hash(
            admission.kind,
            admission.block_id,
            &admission.exported_variables,
            &admission.eliminated_variables,
            &admission.status,
            admission.execution_plan.as_ref(),
        );
        return admission;
    }
    if admission.execution_plan.is_some() {
        let cost = estimate_kernel_cost(block, system, admission.kind, probes);
        if cost.cost_class == RouteCostClass::CostProhibited {
            admission.status = KernelAdmissionStatus::CostProhibited {
                reason: "dominant algebraic route budget prohibited before execution".to_owned(),
                estimate_hash: cost.estimate_hash,
            };
            admission.execution_plan = None;
            admission.admission_hash = kernel_admission_hash(
                admission.kind,
                admission.block_id,
                &admission.exported_variables,
                &admission.eliminated_variables,
                &admission.status,
                None,
            );
            return admission;
        }
    }
    if let Some(plan) = admission.execution_plan.as_mut() {
        let cost = estimate_kernel_cost(block, system, admission.kind, probes);
        plan.algebraic_work_estimate = cost.algebraic_work_estimate;
        plan.route_budget = RouteBudget::from_estimate(&plan.algebraic_work_estimate);
        plan.plan_hash = hash_kernel_execution_plan(plan);
        admission.admission_hash = kernel_admission_hash(
            admission.kind,
            admission.block_id,
            &admission.exported_variables,
            &admission.eliminated_variables,
            &admission.status,
            admission.execution_plan.as_ref(),
        );
    }
    admission
}

fn kernel_admission_hash(
    kind: KernelKind,
    block_id: crate::types::ids::BlockId,
    exported_variables: &[VariableId],
    eliminated_variables: &[VariableId],
    status: &KernelAdmissionStatus,
    execution_plan: Option<&KernelExecutionPlan>,
) -> Hash {
    let mut chunks = vec![
        format!("{kind:?}").into_bytes(),
        block_id.0.to_be_bytes().to_vec(),
        format!("{status:?}").into_bytes(),
    ];
    for variable in exported_variables {
        chunks.push(variable.0.to_be_bytes().to_vec());
    }
    chunks.push(Vec::new());
    for variable in eliminated_variables {
        chunks.push(variable.0.to_be_bytes().to_vec());
    }
    if let Some(plan) = &execution_plan {
        chunks.push(plan.plan_hash.0.to_vec());
    }
    hash_sequence("kernel-admission", &chunks)
}

fn basic_support_plan(
    schedule: Option<DenseRelationSearchSchedule>,
    sparse_schedule: Option<SparseRelationSearchSchedule>,
    probes: &ProbeResults,
    degree_bound: usize,
) -> KernelSupportPlan {
    let sparse_stage = sparse_schedule.as_ref().map(|schedule| &schedule.stage);
    let template = if let Some(stage) = sparse_stage {
        template_plan(
            stage.matrix_rows.max(1),
            stage.matrix_cols.max(1),
            stage.row_monomial_hash,
            stage.export_support_hash,
        )
    } else {
        template_plan(
            probes.local_macaulay_size.template_estimate.row_count,
            probes.local_macaulay_size.template_estimate.column_count,
            probes.local_macaulay_size.template_estimate.estimate_hash,
            probes.mixed_support.probe_hash,
        )
    };
    let rank = rank_plan(
        sparse_stage.map_or(probes.modular_rank.rank_estimate.estimated_rank, |stage| {
            stage.matrix_cols.max(1)
        }),
    );
    let mut support_plan = KernelSupportPlan {
        dense_relation_search_schedule: schedule,
        sparse_relation_search_schedule: sparse_schedule,
        affine_elimination_order: None,
        template_plan: Some(template),
        rank_plan: Some(rank),
        universal_strategy_sequence: Vec::new(),
        degree_bound,
        support_hash: hash_sequence("kernel-support-plan", &[]),
    };
    support_plan.support_hash = support_plan_hash(&support_plan);
    support_plan
}

fn relation_search_resource_bounds(
    ctx: &SolverContext,
    stage: &RelationSearchStage,
    degree_bound: usize,
) -> ResourceBounds {
    let mut bounds = ResourceBounds {
        max_matrix_rows: Some(stage.matrix_rows.max(1)),
        max_matrix_cols: Some(stage.matrix_cols.max(1)),
        max_export_degree: Some(degree_bound),
        max_multiplier_total_degree: Some(stage.multiplier_total_degree),
        max_local_elimination_steps: Some(0),
        max_memory_bytes: ctx.options.max_memory_bytes,
        bounds_hash: hash_sequence("planner-resource-bounds", &[]),
    };
    bounds.bounds_hash = resource_bounds_hash(&bounds);
    bounds
}

fn resource_bounds(
    ctx: &SolverContext,
    max_export_degree: Option<usize>,
    probes: &ProbeResults,
) -> ResourceBounds {
    let mut bounds = ResourceBounds {
        max_matrix_rows: ctx
            .options
            .max_matrix_rows
            .or(Some(probes.local_macaulay_size.template_estimate.row_count)),
        max_matrix_cols: ctx.options.max_matrix_cols.or(Some(
            probes.local_macaulay_size.template_estimate.column_count,
        )),
        max_export_degree,
        max_multiplier_total_degree: max_export_degree
            .map(|degree| degree.saturating_add(probes.structural.max_total_degree)),
        max_local_elimination_steps: Some(
            probes
                .structural
                .relation_count
                .saturating_mul(probes.structural.variable_count.max(1)),
        ),
        max_memory_bytes: ctx.options.max_memory_bytes,
        bounds_hash: hash_sequence("planner-resource-bounds", &[]),
    };
    bounds.bounds_hash = resource_bounds_hash(&bounds);
    bounds
}

fn block_relations<'a>(
    block: &ProjectionBlock,
    relations_by_id: &'a BTreeMap<RelationId, &'a crate::problem::canonicalize::CanonicalRelationQ>,
) -> Vec<&'a crate::problem::canonicalize::CanonicalRelationQ> {
    block
        .relation_ids
        .iter()
        .filter_map(|id| relations_by_id.get(id).copied())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::all_planner_kernel_kinds;
    use crate::kernels::traits::KernelKind;

    #[test]
    fn fcr_p10_production_planner_kernel_list_has_full_candidate_cover_ladder() {
        let kinds = all_planner_kernel_kinds();
        assert_eq!(
            kinds,
            vec![
                KernelKind::TargetUnivariate,
                KernelKind::LinearAffine,
                KernelKind::TargetRelationSearch,
                KernelKind::SparseResultantProjection,
                KernelKind::TargetActionKrylov,
                KernelKind::NormTraceProjection,
                KernelKind::RegularChainProjection,
                KernelKind::SpecializationInterpolation,
                KernelKind::UniversalTargetElimination,
            ]
        );
    }
}

fn sorted_set(vars: &BTreeSet<VariableId>) -> Vec<VariableId> {
    vars.iter().copied().collect()
}
