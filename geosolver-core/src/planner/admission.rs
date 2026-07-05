use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::graph::projection_dag::ProjectionBlock;
#[cfg(test)]
use crate::kernels::action_krylov::plan_target_action_krylov;
use crate::kernels::linear_affine::{find_triangular_affine_order, plan_linear_affine};
#[cfg(test)]
use crate::kernels::norm_trace_projection::plan_norm_trace_projection;
#[cfg(test)]
use crate::kernels::regular_chain_projection::plan_regular_chain_projection;
#[cfg(test)]
use crate::kernels::sparse_resultant::plan_sparse_resultant;
#[cfg(test)]
use crate::kernels::specialization_interpolation::plan_specialization_interpolation;
use crate::kernels::traits::KernelKind;
#[cfg(test)]
use crate::kernels::universal_elimination::plan_universal_elimination;
use crate::planner::kernel_plan::{
    planned_failure_behavior, rank_plan, resource_bounds_hash, support_plan_hash, template_plan,
    CertificateRoute, KernelExecutionPlan, KernelSupportPlan, LocalNonfinitePolicy, ResourceBounds,
};
use crate::planner::probes::ProbeResults;
#[cfg(test)]
use crate::planner::relation_schedule::build_dense_relation_search_schedule;
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
    Declined { reason: String },
}

impl KernelAdmission {
    pub fn is_admitted(&self) -> bool {
        matches!(self.status, KernelAdmissionStatus::Admitted)
    }
}

pub fn all_planner_kernel_kinds() -> Vec<KernelKind> {
    #[cfg(not(test))]
    {
        return vec![KernelKind::TargetUnivariate, KernelKind::LinearAffine];
    }
    #[cfg(test)]
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
            build_kernel_admission(
                KernelPlanId(index as u32),
                kind,
                block,
                system,
                &relation_map,
                probes,
                ctx,
            )
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
    #[cfg(test)]
    let relation_polys = local_relations
        .iter()
        .map(|relation| relation.polynomial.clone())
        .collect::<Vec<_>>();
    #[cfg(test)]
    let relation_hashes = local_relations
        .iter()
        .map(|relation| relation.hash)
        .collect::<Vec<_>>();
    #[cfg(test)]
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
                    basic_support_plan(None, probes, target_degree_bound),
                    resource_bounds(ctx, Some(target_degree_bound), probes),
                    CertificateRoute::SourceMembershipCertificate,
                    LocalNonfinitePolicy::NotApplicable,
                )
            } else {
                declined("no authorized target-only relation")
            }
        }
        #[cfg(test)]
        KernelKind::TargetRelationSearch => {
            if relation_polys.is_empty() {
                declined("no authorized local relations for dense relation search")
            } else {
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
                    basic_support_plan(Some(schedule), probes, degree_bound),
                    resource_bounds(ctx, Some(degree_bound), probes),
                    CertificateRoute::DenseRelationSearchMembership,
                    LocalNonfinitePolicy::NotApplicable,
                )
            }
        }
        #[cfg(test)]
        KernelKind::UniversalTargetElimination => {
            match plan_universal_elimination(block, system, ctx, plan_id) {
                Ok(plan) => (KernelAdmissionStatus::Admitted, Some(plan)),
                Err(_) => {
                    declined("no authorized local relations for universal projection planning")
                }
            }
        }
        KernelKind::LinearAffine => {
            if let Some(order) = find_triangular_affine_order(block, system) {
                match plan_linear_affine(block, system, &order, ctx) {
                    Ok(plan) => (KernelAdmissionStatus::Admitted, Some(plan)),
                    Err(_) => declined("safe affine order exists but planning failed"),
                }
            } else {
                declined("no complete safe triangular affine elimination order")
            }
        }
        #[cfg(test)]
        KernelKind::SparseResultantProjection => {
            match plan_sparse_resultant(block, system, ctx, plan_id) {
                Ok(plan) => (KernelAdmissionStatus::Admitted, Some(plan)),
                Err(_) => declined("not sparse enough for a finite resultant template"),
            }
        }
        #[cfg(test)]
        KernelKind::TargetActionKrylov => {
            match plan_target_action_krylov(block, system, ctx, plan_id) {
                Ok(plan) => (KernelAdmissionStatus::Admitted, Some(plan)),
                Err(_) => declined(
                    "target-action-krylov verified characteristic coverage plan is not applicable",
                ),
            }
        }
        #[cfg(test)]
        KernelKind::NormTraceProjection => {
            match plan_norm_trace_projection(block, system, ctx, plan_id) {
                Ok(plan) => (KernelAdmissionStatus::Admitted, Some(plan)),
                Err(_) => declined("no explicit algebraic tower norm/trace projection plan"),
            }
        }
        #[cfg(test)]
        KernelKind::RegularChainProjection => {
            match plan_regular_chain_projection(block, system, ctx, plan_id) {
                Ok(plan) => (KernelAdmissionStatus::Admitted, Some(plan)),
                Err(_) => declined("no triangular regular-chain projection plan"),
            }
        }
        #[cfg(test)]
        KernelKind::SpecializationInterpolation => {
            match plan_specialization_interpolation(block, system, ctx, plan_id) {
                Ok(plan) => (KernelAdmissionStatus::Admitted, Some(plan)),
                Err(_) => declined(
                    "specialization-interpolation exact verification plan is not applicable",
                ),
            }
        }
        #[cfg(not(test))]
        KernelKind::TargetRelationSearch
        | KernelKind::SparseResultantProjection
        | KernelKind::TargetActionKrylov
        | KernelKind::NormTraceProjection
        | KernelKind::RegularChainProjection
        | KernelKind::SpecializationInterpolation
        | KernelKind::UniversalTargetElimination => {
            declined("FCR-P2 quarantined until generic production implementation is complete")
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

fn finish_admission(
    kind: KernelKind,
    block_id: crate::types::ids::BlockId,
    exported_variables: Vec<VariableId>,
    eliminated_variables: Vec<VariableId>,
    status: KernelAdmissionStatus,
    execution_plan: Option<KernelExecutionPlan>,
) -> KernelAdmission {
    let mut chunks = vec![
        format!("{kind:?}").into_bytes(),
        block_id.0.to_be_bytes().to_vec(),
        format!("{status:?}").into_bytes(),
    ];
    for variable in &exported_variables {
        chunks.push(variable.0.to_be_bytes().to_vec());
    }
    chunks.push(Vec::new());
    for variable in &eliminated_variables {
        chunks.push(variable.0.to_be_bytes().to_vec());
    }
    if let Some(plan) = &execution_plan {
        chunks.push(plan.plan_hash.0.to_vec());
    }
    let admission_hash = hash_sequence("kernel-admission", &chunks);
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

fn basic_support_plan(
    schedule: Option<crate::planner::relation_schedule::DenseRelationSearchSchedule>,
    probes: &ProbeResults,
    degree_bound: usize,
) -> KernelSupportPlan {
    let template = template_plan(
        probes.local_macaulay_size.template_estimate.row_count,
        probes.local_macaulay_size.template_estimate.column_count,
        probes.local_macaulay_size.template_estimate.estimate_hash,
        probes.mixed_support.probe_hash,
    );
    let rank = rank_plan(probes.modular_rank.rank_estimate.estimated_rank);
    let mut support_plan = KernelSupportPlan {
        dense_relation_search_schedule: schedule,
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

fn sorted_set(vars: &BTreeSet<VariableId>) -> Vec<VariableId> {
    vars.iter().copied().collect()
}
