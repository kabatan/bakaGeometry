use std::collections::{BTreeMap, BTreeSet};

use std::panic::{catch_unwind, AssertUnwindSafe};

use serde::{Deserialize, Serialize};

use crate::graph::projection_dag::ProjectionBlock;
use crate::kernels::all_kernels;
use crate::kernels::traits::KernelContext;
use crate::kernels::traits::KernelKind;
use crate::planner::algebraic_cost::RouteBudget;
use crate::planner::cost_model::{
    estimate_kernel_cost, estimate_kernel_cost_for_admission, RouteCostClass,
};
use crate::planner::kernel_plan::{
    hash_kernel_execution_plan, resource_bounds_hash, KernelExecutionPlan, ResourceBounds,
};
use crate::planner::probes::ProbeResults;
use crate::preprocess::compression::CompressedSystemQ;
use crate::problem::context::SolverContext;
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::{RelationId, VariableId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KernelAdmission {
    pub kind: KernelKind,
    pub block_id: crate::types::ids::BlockId,
    pub admission_evidence: KernelAdmissionEvidence,
    pub status: KernelAdmissionStatus,
    pub exported_variables: Vec<VariableId>,
    pub eliminated_variables: Vec<VariableId>,
    pub execution_plan: Option<KernelExecutionPlan>,
    pub admission_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KernelAdmissionEvidence {
    pub runtime_admission_hash: Option<Hash>,
    pub source_relation_ids: Vec<RelationId>,
    pub source_relation_hashes: Vec<Hash>,
    pub initial_resource_bounds: Option<ResourceBounds>,
    pub estimated_matrix_rows: Option<usize>,
    pub estimated_matrix_cols: Option<usize>,
    pub estimated_template_size: Option<usize>,
    pub evidence_hash: Hash,
}

impl KernelAdmissionEvidence {
    pub fn empty() -> Self {
        Self {
            runtime_admission_hash: None,
            source_relation_ids: Vec::new(),
            source_relation_hashes: Vec::new(),
            initial_resource_bounds: None,
            estimated_matrix_rows: None,
            estimated_matrix_cols: None,
            estimated_template_size: None,
            evidence_hash: hash_sequence("kernel-admission-evidence", &[]),
        }
    }
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
    let kernel_context = KernelContext {
        block: block.clone(),
        system: system.clone(),
        child_messages: Vec::new(),
    };
    all_kernels()
        .into_iter()
        .map(|kernel| {
            let kind = kernel.kind();
            let admission =
                match catch_unwind(AssertUnwindSafe(|| kernel.admit(block, &kernel_context))) {
                    Ok(mut admission) => {
                        if admission.kind != kind {
                            route_panic_admission(
                                kind,
                                block,
                                format!(
                                    "registered kernel returned mismatched admission kind {:?}",
                                    admission.kind
                                ),
                            )
                        } else if admission.is_admitted()
                            && (admission.execution_plan.is_none()
                                || kind == KernelKind::TargetRelationSearch)
                        {
                            match catch_unwind(AssertUnwindSafe(|| {
                                kernel.plan(&admission, &kernel_context, ctx)
                            })) {
                                Ok(Ok(plan)) => {
                                    admission.execution_plan = Some(plan);
                                    admission
                                }
                                Ok(Err(err)) => runtime_plan_error_admission(kind, block, &err),
                                Err(payload) => {
                                    route_panic_admission(kind, block, panic_message(payload))
                                }
                            }
                        } else {
                            admission
                        }
                    }
                    Err(payload) => route_panic_admission(kind, block, panic_message(payload)),
                };
            let runtime_admission_hash = Some(admission.admission_hash);
            bind_admission_algebraic_cost(
                admission,
                block,
                system,
                probes,
                ctx,
                runtime_admission_hash,
            )
        })
        .collect()
}

fn runtime_plan_error_admission(
    kind: KernelKind,
    block: &ProjectionBlock,
    err: &crate::result::status::SolverError,
) -> KernelAdmission {
    let status = KernelAdmissionStatus::PlanProbeFailed {
        reason: format!(
            "runtime kernel plan failed after admitted runtime admission; status={:?}; error={:?}",
            err.public_status(),
            err.kind
        ),
        constructed_object_hash: hash_sequence(
            "runtime-kernel-plan-failed",
            &[
                format!("{kind:?}").into_bytes(),
                block.block_hash.0.to_vec(),
                format!("{:?}", err.public_status()).into_bytes(),
                format!("{:?}", err.kind).into_bytes(),
            ],
        ),
    };
    finish_admission(
        kind,
        block.block_id,
        sorted_set(&block.exported_variables),
        block
            .local_variables
            .difference(&block.exported_variables)
            .copied()
            .collect(),
        status,
        None,
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
    let admission_evidence = KernelAdmissionEvidence::empty();
    let admission_hash = kernel_admission_hash(
        kind,
        block_id,
        &admission_evidence,
        &exported_variables,
        &eliminated_variables,
        &status,
        execution_plan.as_ref(),
    );
    KernelAdmission {
        kind,
        block_id,
        admission_evidence,
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
    ctx: &SolverContext,
    runtime_admission_hash: Option<Hash>,
) -> KernelAdmission {
    let mut evidence_cost = None;
    let scheduled_target_relation_plan = admission.kind == KernelKind::TargetRelationSearch
        && admission.execution_plan.as_ref().is_some_and(|plan| {
            plan.support_plan.dense_relation_search_schedule.is_some()
                || plan.support_plan.sparse_relation_search_schedule.is_some()
        });
    let plan_bound_sparse_resultant = admission.kind == KernelKind::SparseResultantProjection
        && admission
            .execution_plan
            .as_ref()
            .is_some_and(|plan| plan.support_plan.template_plan.is_some());
    let internally_budgeted_universal = admission.kind == KernelKind::UniversalTargetElimination
        && admission
            .execution_plan
            .as_ref()
            .is_some_and(|plan| !plan.support_plan.universal_strategy_sequence.is_empty());
    if scheduled_target_relation_plan
        || plan_bound_sparse_resultant
        || internally_budgeted_universal
    {
        let cost = estimate_kernel_cost_for_admission(
            block,
            system,
            admission.kind,
            probes,
            admission.execution_plan.as_ref(),
        );
        evidence_cost = Some(cost.clone());
        if plan_bound_sparse_resultant && cost.cost_class == RouteCostClass::CostProhibited {
            admission.status = KernelAdmissionStatus::CostProhibited {
                reason: "dominant sparse resultant expression swell prohibited before execution"
                    .to_owned(),
                estimate_hash: cost.estimate_hash,
            };
            admission.execution_plan = None;
        } else if let Some(plan) = admission.execution_plan.as_mut() {
            plan.algebraic_work_estimate = cost.algebraic_work_estimate;
            plan.route_budget = RouteBudget::from_estimate(&plan.algebraic_work_estimate);
            plan.plan_hash = hash_kernel_execution_plan(plan);
        }
        bind_admission_evidence_and_hash(
            &mut admission,
            block,
            system,
            probes,
            ctx,
            runtime_admission_hash,
            evidence_cost.as_ref(),
        );
        return admission;
    }
    if admission.execution_plan.is_some() {
        let cost = estimate_kernel_cost(block, system, admission.kind, probes);
        evidence_cost = Some(cost.clone());
        if cost.cost_class == RouteCostClass::CostProhibited {
            admission.status = KernelAdmissionStatus::CostProhibited {
                reason: "dominant algebraic route budget prohibited before execution".to_owned(),
                estimate_hash: cost.estimate_hash,
            };
            admission.execution_plan = None;
            bind_admission_evidence_and_hash(
                &mut admission,
                block,
                system,
                probes,
                ctx,
                runtime_admission_hash,
                evidence_cost.as_ref(),
            );
            return admission;
        }
    }
    if let Some(plan) = admission.execution_plan.as_mut() {
        let cost = evidence_cost
            .clone()
            .unwrap_or_else(|| estimate_kernel_cost(block, system, admission.kind, probes));
        evidence_cost = Some(cost.clone());
        plan.algebraic_work_estimate = cost.algebraic_work_estimate;
        plan.route_budget = RouteBudget::from_estimate(&plan.algebraic_work_estimate);
        plan.plan_hash = hash_kernel_execution_plan(plan);
    }
    bind_admission_evidence_and_hash(
        &mut admission,
        block,
        system,
        probes,
        ctx,
        runtime_admission_hash,
        evidence_cost.as_ref(),
    );
    admission
}

fn bind_admission_evidence_and_hash(
    admission: &mut KernelAdmission,
    block: &ProjectionBlock,
    system: &CompressedSystemQ,
    probes: &ProbeResults,
    ctx: &SolverContext,
    runtime_admission_hash: Option<Hash>,
    cost: Option<&crate::planner::cost_model::KernelCostEstimate>,
) {
    admission.admission_evidence = build_admission_evidence(
        admission,
        block,
        system,
        probes,
        ctx,
        runtime_admission_hash,
        cost,
    );
    admission.admission_hash = kernel_admission_hash(
        admission.kind,
        admission.block_id,
        &admission.admission_evidence,
        &admission.exported_variables,
        &admission.eliminated_variables,
        &admission.status,
        admission.execution_plan.as_ref(),
    );
}

fn build_admission_evidence(
    admission: &KernelAdmission,
    block: &ProjectionBlock,
    system: &CompressedSystemQ,
    probes: &ProbeResults,
    ctx: &SolverContext,
    runtime_admission_hash: Option<Hash>,
    cost: Option<&crate::planner::cost_model::KernelCostEstimate>,
) -> KernelAdmissionEvidence {
    let relations_by_id = system
        .relations
        .iter()
        .map(|relation| (relation.id, relation.hash))
        .collect::<BTreeMap<_, _>>();
    let source_relation_ids = admission
        .execution_plan
        .as_ref()
        .map(|plan| plan.source_relation_ids.clone())
        .unwrap_or_else(|| block.relation_ids.clone());
    let source_relation_hashes = admission
        .execution_plan
        .as_ref()
        .map(|plan| plan.source_relation_hashes.clone())
        .unwrap_or_else(|| {
            source_relation_ids
                .iter()
                .filter_map(|id| relations_by_id.get(id).copied())
                .collect::<Vec<_>>()
        });
    let initial_resource_bounds = admission
        .execution_plan
        .as_ref()
        .map(|plan| plan.resource_bounds.clone())
        .or_else(|| Some(resource_bounds(ctx, None, probes)));
    let estimated_matrix_rows = cost.map(|cost| cost.matrix_rows).or_else(|| {
        admission
            .execution_plan
            .as_ref()
            .and_then(|plan| plan.resource_bounds.max_matrix_rows)
    });
    let estimated_matrix_cols = cost.map(|cost| cost.matrix_cols).or_else(|| {
        admission
            .execution_plan
            .as_ref()
            .and_then(|plan| plan.resource_bounds.max_matrix_cols)
    });
    let estimated_template_size = admission
        .execution_plan
        .as_ref()
        .and_then(|plan| plan.support_plan.template_plan.as_ref())
        .map(|template| template.matrix_rows.saturating_mul(template.matrix_cols))
        .or_else(|| cost.map(|cost| cost.matrix_rows.saturating_mul(cost.matrix_cols)));
    let mut evidence = KernelAdmissionEvidence {
        runtime_admission_hash,
        source_relation_ids,
        source_relation_hashes,
        initial_resource_bounds,
        estimated_matrix_rows,
        estimated_matrix_cols,
        estimated_template_size,
        evidence_hash: hash_sequence("kernel-admission-evidence", &[]),
    };
    evidence.evidence_hash = kernel_admission_evidence_hash(&evidence);
    evidence
}

fn kernel_admission_hash(
    kind: KernelKind,
    block_id: crate::types::ids::BlockId,
    admission_evidence: &KernelAdmissionEvidence,
    exported_variables: &[VariableId],
    eliminated_variables: &[VariableId],
    status: &KernelAdmissionStatus,
    execution_plan: Option<&KernelExecutionPlan>,
) -> Hash {
    let mut chunks = vec![
        format!("{kind:?}").into_bytes(),
        block_id.0.to_be_bytes().to_vec(),
        format!("{status:?}").into_bytes(),
        admission_evidence.evidence_hash.0.to_vec(),
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

fn kernel_admission_evidence_hash(evidence: &KernelAdmissionEvidence) -> Hash {
    let mut chunks = Vec::new();
    if let Some(runtime_hash) = evidence.runtime_admission_hash {
        chunks.push(runtime_hash.0.to_vec());
    }
    chunks.push(Vec::new());
    for relation_id in &evidence.source_relation_ids {
        chunks.push(relation_id.0.to_be_bytes().to_vec());
    }
    chunks.push(Vec::new());
    for relation_hash in &evidence.source_relation_hashes {
        chunks.push(relation_hash.0.to_vec());
    }
    chunks.push(Vec::new());
    if let Some(bounds) = &evidence.initial_resource_bounds {
        chunks.push(bounds.bounds_hash.0.to_vec());
        chunks.push(bounds.max_matrix_rows.unwrap_or(0).to_be_bytes().to_vec());
        chunks.push(bounds.max_matrix_cols.unwrap_or(0).to_be_bytes().to_vec());
        chunks.push(bounds.max_export_degree.unwrap_or(0).to_be_bytes().to_vec());
        chunks.push(
            bounds
                .max_multiplier_total_degree
                .unwrap_or(0)
                .to_be_bytes()
                .to_vec(),
        );
        chunks.push(
            bounds
                .max_local_elimination_steps
                .unwrap_or(0)
                .to_be_bytes()
                .to_vec(),
        );
        chunks.push(bounds.max_memory_bytes.unwrap_or(0).to_be_bytes().to_vec());
    }
    chunks.push(Vec::new());
    chunks.push(
        evidence
            .estimated_matrix_rows
            .unwrap_or(0)
            .to_be_bytes()
            .to_vec(),
    );
    chunks.push(
        evidence
            .estimated_matrix_cols
            .unwrap_or(0)
            .to_be_bytes()
            .to_vec(),
    );
    chunks.push(
        evidence
            .estimated_template_size
            .unwrap_or(0)
            .to_be_bytes()
            .to_vec(),
    );
    hash_sequence("kernel-admission-evidence", &chunks)
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

#[cfg(test)]
mod tests {
    use super::all_planner_kernel_kinds;
    use crate::kernels::all_kernels;
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

    #[test]
    fn p7_planner_kernel_list_matches_runtime_registry_order() {
        let planner_kinds = all_planner_kernel_kinds();
        let runtime_kinds = all_kernels()
            .into_iter()
            .map(|kernel| kernel.kind())
            .collect::<Vec<_>>();

        assert_eq!(planner_kinds, runtime_kinds);
    }
}

fn sorted_set(vars: &BTreeSet<VariableId>) -> Vec<VariableId> {
    vars.iter().copied().collect()
}
