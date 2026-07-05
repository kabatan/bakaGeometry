use serde::{Deserialize, Serialize};

use crate::kernels::traits::KernelKind;
use crate::planner::admission::KernelAdmission;
use crate::planner::cost_model::KernelCostEstimate;
use crate::planner::relation_schedule::{
    hash_dense_relation_search_schedule, DenseRelationSearchSchedule,
};
use crate::result::status::{FailureKind, SolverError, SolverErrorKind, SolverStatus};
use crate::types::hash::{hash_sequence, Hash};
use crate::types::ids::{BlockId, KernelPlanId, RelationId, VariableId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KernelPlan {
    pub block_id: BlockId,
    pub declared_ladder: Vec<KernelExecutionPlan>,
    pub selected_first: KernelKind,
    pub admissions: Vec<KernelAdmission>,
    pub cost_estimates: Vec<KernelCostEstimate>,
    pub plan_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KernelExecutionPlan {
    pub plan_id: KernelPlanId,
    pub block_id: BlockId,
    pub kernel_kind: KernelKind,
    pub input_block_authorization_hash: Hash,
    pub source_relation_ids: Vec<RelationId>,
    pub source_relation_hashes: Vec<Hash>,
    pub child_block_ids: Vec<BlockId>,
    pub child_message_hashes: Vec<Hash>,
    pub exported_variables: Vec<VariableId>,
    pub eliminated_variables: Vec<VariableId>,
    pub support_plan: KernelSupportPlan,
    pub resource_bounds: ResourceBounds,
    pub certificate_route: CertificateRoute,
    pub failure_behavior: PlannedFailureBehavior,
    pub plan_work_classification: PlanWorkClassification,
    pub plan_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanWorkClassification {
    PurePlan,
    CertifiedProbePlan(CertifiedProbePlan),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertifiedProbePlan {
    pub authorization_hash: Hash,
    pub source_relation_hashes: Vec<Hash>,
    pub probe_output_hashes: Vec<Hash>,
    pub resource_trace_hash: Hash,
    pub cost_trace_hash: Hash,
    pub probe_certificate_hash: Hash,
    pub execute_replay_required: bool,
    pub probe_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KernelSupportPlan {
    pub dense_relation_search_schedule: Option<DenseRelationSearchSchedule>,
    pub affine_elimination_order: Option<AffineEliminationPlan>,
    pub template_plan: Option<TemplatePlan>,
    pub rank_plan: Option<RankPlan>,
    pub universal_strategy_sequence: Vec<UniversalStrategyPlanStep>,
    pub degree_bound: usize,
    pub support_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AffineEliminationPlan {
    pub steps: Vec<AffineEliminationStep>,
    pub order_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AffineEliminationStep {
    pub eliminated_variable: VariableId,
    pub source_relation_id: RelationId,
    pub pivot_hash: Hash,
    pub denominator_guard_hash: Option<Hash>,
    pub step_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplatePlan {
    pub matrix_rows: usize,
    pub matrix_cols: usize,
    pub row_monomial_hash: Hash,
    pub column_support_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RankPlan {
    pub estimated_rank: usize,
    pub rank_plan_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UniversalStrategyPlanStep {
    pub strategy: UniversalStrategy,
    pub enabled: bool,
    pub skip_reason: Option<String>,
    pub strategy_hash: Hash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UniversalStrategy {
    TargetRelationSearchEscalated,
    SparseResultantIfSquareOrOverdetermined,
    SpecializeProjectInterpolateVerify,
    LocalGroebnerEliminationToKeepZ,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceBounds {
    pub max_matrix_rows: Option<usize>,
    pub max_matrix_cols: Option<usize>,
    pub max_export_degree: Option<usize>,
    pub max_multiplier_total_degree: Option<usize>,
    pub max_local_elimination_steps: Option<usize>,
    pub max_memory_bytes: Option<u64>,
    pub bounds_hash: Hash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CertificateRoute {
    SourceMembershipCertificate,
    DenseRelationSearchMembership,
    GuardedAffineProjectionCertificate,
    SparseResultantExactVerification,
    VerifiedCharacteristicSupportCoverage,
    NormTraceExactVerification,
    RegularChainGuardedProjection,
    SpecializationInterpolationExactVerification,
    UniversalFixedLocalElimination,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedFailureBehavior {
    pub allowed_statuses: Vec<SolverStatus>,
    pub local_nonfinite_policy: LocalNonfinitePolicy,
    pub behavior_hash: Hash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocalNonfinitePolicy {
    NoLocalCertifiedNonFinite,
    NotApplicable,
}

impl KernelPlan {
    pub fn new(
        block_id: BlockId,
        declared_ladder: Vec<KernelExecutionPlan>,
        admissions: Vec<KernelAdmission>,
        cost_estimates: Vec<KernelCostEstimate>,
    ) -> Result<Self, SolverError> {
        let Some(selected_first) = declared_ladder.first().map(|plan| plan.kernel_kind) else {
            return Err(implementation_bug("declared ladder is empty"));
        };
        let mut plan = Self {
            block_id,
            declared_ladder,
            selected_first,
            admissions,
            cost_estimates,
            plan_hash: hash_sequence("kernel-plan", &[]),
        };
        plan.plan_hash = hash_kernel_plan(&plan);
        Ok(plan)
    }
}

impl KernelExecutionPlan {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        plan_id: KernelPlanId,
        block_id: BlockId,
        kernel_kind: KernelKind,
        input_block_authorization_hash: Hash,
        source_relation_ids: Vec<RelationId>,
        source_relation_hashes: Vec<Hash>,
        child_block_ids: Vec<BlockId>,
        child_message_hashes: Vec<Hash>,
        exported_variables: Vec<VariableId>,
        eliminated_variables: Vec<VariableId>,
        support_plan: KernelSupportPlan,
        resource_bounds: ResourceBounds,
        certificate_route: CertificateRoute,
        failure_behavior: PlannedFailureBehavior,
    ) -> Self {
        Self::new_with_work_classification(
            plan_id,
            block_id,
            kernel_kind,
            input_block_authorization_hash,
            source_relation_ids,
            source_relation_hashes,
            child_block_ids,
            child_message_hashes,
            exported_variables,
            eliminated_variables,
            support_plan,
            resource_bounds,
            certificate_route,
            failure_behavior,
            PlanWorkClassification::PurePlan,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_work_classification(
        plan_id: KernelPlanId,
        block_id: BlockId,
        kernel_kind: KernelKind,
        input_block_authorization_hash: Hash,
        source_relation_ids: Vec<RelationId>,
        source_relation_hashes: Vec<Hash>,
        child_block_ids: Vec<BlockId>,
        child_message_hashes: Vec<Hash>,
        exported_variables: Vec<VariableId>,
        eliminated_variables: Vec<VariableId>,
        support_plan: KernelSupportPlan,
        resource_bounds: ResourceBounds,
        certificate_route: CertificateRoute,
        failure_behavior: PlannedFailureBehavior,
        plan_work_classification: PlanWorkClassification,
    ) -> Self {
        let mut plan = Self {
            plan_id,
            block_id,
            kernel_kind,
            input_block_authorization_hash,
            source_relation_ids,
            source_relation_hashes,
            child_block_ids,
            child_message_hashes,
            exported_variables,
            eliminated_variables,
            support_plan,
            resource_bounds,
            certificate_route,
            failure_behavior,
            plan_work_classification,
            plan_hash: hash_sequence("kernel-execution-plan", &[]),
        };
        plan.plan_hash = hash_kernel_execution_plan(&plan);
        plan
    }
}

pub fn require_declared_kernel_plan(
    plan: &KernelPlan,
    kernel_kind: KernelKind,
    expected_plan_hash: Hash,
) -> Result<&KernelExecutionPlan, SolverError> {
    if plan.plan_hash != expected_plan_hash || hash_kernel_plan(plan) != expected_plan_hash {
        return Err(implementation_bug(
            "declared ladder hash mismatch before execution",
        ));
    }
    plan.declared_ladder
        .iter()
        .find(|entry| entry.kernel_kind == kernel_kind)
        .ok_or_else(|| implementation_bug("kernel absent from declared ladder"))
}

pub fn hash_kernel_plan(plan: &KernelPlan) -> Hash {
    let mut chunks = vec![plan.block_id.0.to_be_bytes().to_vec()];
    for ladder_plan in &plan.declared_ladder {
        chunks.push(hash_kernel_execution_plan(ladder_plan).0.to_vec());
    }
    chunks.push(Vec::new());
    for admission in &plan.admissions {
        chunks.push(admission.admission_hash.0.to_vec());
    }
    chunks.push(Vec::new());
    for cost in &plan.cost_estimates {
        chunks.push(cost.estimate_hash.0.to_vec());
    }
    hash_sequence("kernel-plan", &chunks)
}

pub fn hash_kernel_execution_plan(plan: &KernelExecutionPlan) -> Hash {
    let mut chunks = vec![
        plan.plan_id.0.to_be_bytes().to_vec(),
        plan.block_id.0.to_be_bytes().to_vec(),
        kernel_kind_bytes(plan.kernel_kind),
        plan.input_block_authorization_hash.0.to_vec(),
    ];
    for id in &plan.source_relation_ids {
        chunks.push(id.0.to_be_bytes().to_vec());
    }
    chunks.push(Vec::new());
    for hash in &plan.source_relation_hashes {
        chunks.push(hash.0.to_vec());
    }
    chunks.push(Vec::new());
    for child in &plan.child_block_ids {
        chunks.push(child.0.to_be_bytes().to_vec());
    }
    chunks.push(Vec::new());
    for hash in &plan.child_message_hashes {
        chunks.push(hash.0.to_vec());
    }
    chunks.push(Vec::new());
    for variable in &plan.exported_variables {
        chunks.push(variable.0.to_be_bytes().to_vec());
    }
    chunks.push(Vec::new());
    for variable in &plan.eliminated_variables {
        chunks.push(variable.0.to_be_bytes().to_vec());
    }
    chunks.push(plan.support_plan.support_hash.0.to_vec());
    chunks.push(support_plan_hash(&plan.support_plan).0.to_vec());
    chunks.push(plan.resource_bounds.bounds_hash.0.to_vec());
    chunks.push(resource_bounds_hash(&plan.resource_bounds).0.to_vec());
    chunks.push(format!("{:?}", plan.certificate_route).into_bytes());
    chunks.push(plan.failure_behavior.behavior_hash.0.to_vec());
    chunks.push(failure_behavior_hash(&plan.failure_behavior).0.to_vec());
    chunks.push(
        plan_work_classification_hash(&plan.plan_work_classification)
            .0
            .to_vec(),
    );
    hash_sequence("kernel-execution-plan", &chunks)
}

pub fn support_plan_hash(plan: &KernelSupportPlan) -> Hash {
    let mut chunks = vec![plan.degree_bound.to_be_bytes().to_vec()];
    if let Some(schedule) = &plan.dense_relation_search_schedule {
        chunks.push(schedule.schedule_hash.0.to_vec());
        chunks.push(hash_dense_relation_search_schedule(schedule).0.to_vec());
    }
    if let Some(order) = &plan.affine_elimination_order {
        chunks.push(order.order_hash.0.to_vec());
        chunks.push(hash_affine_elimination_plan(order).0.to_vec());
    }
    if let Some(template) = &plan.template_plan {
        chunks.push(template.row_monomial_hash.0.to_vec());
        chunks.push(template.column_support_hash.0.to_vec());
        chunks.push(template.matrix_rows.to_be_bytes().to_vec());
        chunks.push(template.matrix_cols.to_be_bytes().to_vec());
    }
    if let Some(rank) = &plan.rank_plan {
        chunks.push(rank.rank_plan_hash.0.to_vec());
        chunks.push(rank_plan_hash(rank.estimated_rank).0.to_vec());
    }
    for step in &plan.universal_strategy_sequence {
        chunks.push(step.strategy_hash.0.to_vec());
        chunks.push(universal_strategy_step_hash(step).0.to_vec());
    }
    hash_sequence("kernel-support-plan", &chunks)
}

pub fn resource_bounds_hash(bounds: &ResourceBounds) -> Hash {
    hash_sequence(
        "planner-resource-bounds",
        &[
            optional_usize_bytes(bounds.max_matrix_rows),
            optional_usize_bytes(bounds.max_matrix_cols),
            optional_usize_bytes(bounds.max_export_degree),
            optional_usize_bytes(bounds.max_multiplier_total_degree),
            optional_usize_bytes(bounds.max_local_elimination_steps),
            bounds
                .max_memory_bytes
                .map(|value| value.to_be_bytes().to_vec())
                .unwrap_or_else(|| vec![0xff]),
        ],
    )
}

pub fn affine_elimination_plan(steps: Vec<AffineEliminationStep>) -> AffineEliminationPlan {
    let mut plan = AffineEliminationPlan {
        steps,
        order_hash: hash_sequence("affine-elimination-plan", &[]),
    };
    plan.order_hash = hash_affine_elimination_plan(&plan);
    plan
}

pub fn affine_elimination_step(
    eliminated_variable: VariableId,
    source_relation_id: RelationId,
    pivot_hash: Hash,
    denominator_guard_hash: Option<Hash>,
) -> AffineEliminationStep {
    let mut step = AffineEliminationStep {
        eliminated_variable,
        source_relation_id,
        pivot_hash,
        denominator_guard_hash,
        step_hash: hash_sequence("affine-elimination-step", &[]),
    };
    step.step_hash = hash_affine_elimination_step(&step);
    step
}

pub fn planned_failure_behavior(
    allowed_statuses: Vec<SolverStatus>,
    local_nonfinite_policy: LocalNonfinitePolicy,
) -> PlannedFailureBehavior {
    let behavior_hash = failure_behavior_hash_from_parts(&allowed_statuses, local_nonfinite_policy);
    PlannedFailureBehavior {
        allowed_statuses,
        local_nonfinite_policy,
        behavior_hash,
    }
}

pub fn universal_strategy_step(
    strategy: UniversalStrategy,
    enabled: bool,
    skip_reason: Option<String>,
) -> UniversalStrategyPlanStep {
    let strategy_hash =
        universal_strategy_step_hash_from_parts(strategy, enabled, skip_reason.as_deref());
    UniversalStrategyPlanStep {
        strategy,
        enabled,
        skip_reason,
        strategy_hash,
    }
}

pub fn rank_plan(estimated_rank: usize) -> RankPlan {
    RankPlan {
        estimated_rank,
        rank_plan_hash: rank_plan_hash(estimated_rank),
    }
}

pub fn template_plan(
    matrix_rows: usize,
    matrix_cols: usize,
    row_monomial_hash: Hash,
    column_support_hash: Hash,
) -> TemplatePlan {
    TemplatePlan {
        matrix_rows,
        matrix_cols,
        row_monomial_hash,
        column_support_hash,
    }
}

pub fn certified_probe_plan(
    authorization_hash: Hash,
    source_relation_hashes: Vec<Hash>,
    probe_output_hashes: Vec<Hash>,
    resource_trace_hash: Hash,
    cost_trace_hash: Hash,
    probe_certificate_hash: Hash,
) -> PlanWorkClassification {
    let mut source_relation_hashes = source_relation_hashes;
    source_relation_hashes.sort();
    source_relation_hashes.dedup();
    let mut probe_output_hashes = probe_output_hashes;
    probe_output_hashes.sort();
    probe_output_hashes.dedup();
    let mut probe = CertifiedProbePlan {
        authorization_hash,
        source_relation_hashes,
        probe_output_hashes,
        resource_trace_hash,
        cost_trace_hash,
        probe_certificate_hash,
        execute_replay_required: true,
        probe_hash: hash_sequence("certified-probe-plan", &[]),
    };
    probe.probe_hash = hash_certified_probe_plan(&probe);
    PlanWorkClassification::CertifiedProbePlan(probe)
}

pub fn verify_certified_probe_replay(
    plan: &KernelExecutionPlan,
    source_relation_hashes: &[Hash],
    probe_output_hashes: &[Hash],
    probe_certificate_hash: Hash,
) -> Result<(), SolverError> {
    let expected = certified_probe_plan(
        plan.input_block_authorization_hash,
        source_relation_hashes.to_vec(),
        probe_output_hashes.to_vec(),
        plan.resource_bounds.bounds_hash,
        plan.support_plan.support_hash,
        probe_certificate_hash,
    );
    if plan.plan_work_classification != expected {
        return Err(implementation_bug(
            "certified probe plan was not replayed against its bound source/output hashes",
        ));
    }
    Ok(())
}

fn plan_work_classification_hash(classification: &PlanWorkClassification) -> Hash {
    match classification {
        PlanWorkClassification::PurePlan => hash_sequence("plan-work-classification", &[vec![0]]),
        PlanWorkClassification::CertifiedProbePlan(probe) => hash_sequence(
            "plan-work-classification",
            &[vec![1], hash_certified_probe_plan(probe).0.to_vec()],
        ),
    }
}

fn hash_certified_probe_plan(probe: &CertifiedProbePlan) -> Hash {
    hash_sequence(
        "certified-probe-plan",
        &std::iter::once(probe.authorization_hash.0.to_vec())
            .chain(std::iter::once(probe.resource_trace_hash.0.to_vec()))
            .chain(std::iter::once(probe.cost_trace_hash.0.to_vec()))
            .chain(std::iter::once(probe.probe_certificate_hash.0.to_vec()))
            .chain(
                probe
                    .source_relation_hashes
                    .iter()
                    .map(|hash| hash.0.to_vec()),
            )
            .chain(std::iter::once(Vec::new()))
            .chain(probe.probe_output_hashes.iter().map(|hash| hash.0.to_vec()))
            .chain(std::iter::once(vec![probe.execute_replay_required as u8]))
            .collect::<Vec<_>>(),
    )
}

fn kernel_kind_bytes(kind: KernelKind) -> Vec<u8> {
    format!("{kind:?}").into_bytes()
}

fn optional_usize_bytes(value: Option<usize>) -> Vec<u8> {
    value
        .map(|value| value.to_be_bytes().to_vec())
        .unwrap_or_else(|| vec![0xff])
}

fn failure_behavior_hash(behavior: &PlannedFailureBehavior) -> Hash {
    failure_behavior_hash_from_parts(&behavior.allowed_statuses, behavior.local_nonfinite_policy)
}

fn hash_affine_elimination_plan(plan: &AffineEliminationPlan) -> Hash {
    let mut chunks = Vec::new();
    for step in &plan.steps {
        chunks.push(step.step_hash.0.to_vec());
        chunks.push(hash_affine_elimination_step(step).0.to_vec());
    }
    hash_sequence("affine-elimination-plan", &chunks)
}

fn hash_affine_elimination_step(step: &AffineEliminationStep) -> Hash {
    let mut chunks = vec![
        step.eliminated_variable.0.to_be_bytes().to_vec(),
        step.source_relation_id.0.to_be_bytes().to_vec(),
        step.pivot_hash.0.to_vec(),
    ];
    if let Some(hash) = step.denominator_guard_hash {
        chunks.push(hash.0.to_vec());
    }
    hash_sequence("affine-elimination-step", &chunks)
}

fn failure_behavior_hash_from_parts(
    allowed_statuses: &[SolverStatus],
    local_nonfinite_policy: LocalNonfinitePolicy,
) -> Hash {
    let mut chunks = vec![format!("{:?}", local_nonfinite_policy).into_bytes()];
    for status in allowed_statuses {
        chunks.push(format!("{status:?}").into_bytes());
    }
    hash_sequence("planned-failure-behavior", &chunks)
}

fn universal_strategy_step_hash(step: &UniversalStrategyPlanStep) -> Hash {
    universal_strategy_step_hash_from_parts(
        step.strategy,
        step.enabled,
        step.skip_reason.as_deref(),
    )
}

fn universal_strategy_step_hash_from_parts(
    strategy: UniversalStrategy,
    enabled: bool,
    skip_reason: Option<&str>,
) -> Hash {
    hash_sequence(
        "universal-strategy-plan-step",
        &[
            format!("{strategy:?}").into_bytes(),
            vec![enabled as u8],
            skip_reason
                .map(|reason| reason.as_bytes().to_vec())
                .unwrap_or_default(),
        ],
    )
}

fn rank_plan_hash(estimated_rank: usize) -> Hash {
    hash_sequence("rank-plan", &[estimated_rank.to_be_bytes().to_vec()])
}

fn implementation_bug(message: &str) -> SolverError {
    SolverError {
        target: None,
        kind: SolverErrorKind::Failure(FailureKind::ImplementationBug {
            invariant_violated: message.to_owned(),
        }),
    }
}
