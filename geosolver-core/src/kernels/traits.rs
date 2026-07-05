use serde::{Deserialize, Serialize};

use crate::compose::message::ProjectionMessage;
use crate::graph::projection_dag::ProjectionBlock;
use crate::planner::admission::{KernelAdmission, KernelAdmissionStatus};
use crate::planner::kernel_plan::KernelExecutionPlan;
use crate::problem::context::SolverContext;
use crate::result::status::{FailureKind, SolverError, SolverErrorKind};
use crate::types::hash::hash_sequence;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum KernelKind {
    TargetUnivariate,
    LinearAffine,
    TargetRelationSearch,
    SparseResultantProjection,
    TargetActionKrylov,
    UniversalTargetElimination,
    RegularChainProjection,
    NormTraceProjection,
    SpecializationInterpolation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KernelContext {
    pub block: ProjectionBlock,
    pub system: crate::preprocess::compression::CompressedSystemQ,
    pub child_messages: Vec<ProjectionMessage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayResult {
    pub accepted: bool,
    pub replay_hash: crate::types::hash::Hash,
}

pub trait TargetProjectionKernel {
    fn kind(&self) -> KernelKind;

    fn admit(&self, block: &ProjectionBlock, ctx: &KernelContext) -> KernelAdmission;

    fn plan(
        &self,
        admission: &KernelAdmission,
        ctx: &KernelContext,
        solver_ctx: &SolverContext,
    ) -> Result<KernelExecutionPlan, SolverError>;

    fn execute(
        &self,
        plan: &KernelExecutionPlan,
        ctx: &mut KernelContext,
        solver_ctx: &mut SolverContext,
    ) -> Result<ProjectionMessage, SolverError>;

    fn replay(&self, message: &ProjectionMessage, ctx: &KernelContext) -> ReplayResult;
}

pub fn declined_kernel_admission(
    kind: KernelKind,
    block: &ProjectionBlock,
    reason: &str,
) -> KernelAdmission {
    let status = KernelAdmissionStatus::Declined {
        reason: reason.to_owned(),
    };
    KernelAdmission {
        kind,
        block_id: block.block_id,
        status: status.clone(),
        exported_variables: block.exported_variables.iter().copied().collect(),
        eliminated_variables: block
            .local_variables
            .difference(&block.exported_variables)
            .copied()
            .collect(),
        execution_plan: None,
        admission_hash: hash_sequence(
            "kernel-admission",
            &[
                format!("{kind:?}").into_bytes(),
                block.block_id.0.to_be_bytes().to_vec(),
                format!("{status:?}").into_bytes(),
            ],
        ),
    }
}

pub fn kernel_not_ready_error(kind: KernelKind) -> SolverError {
    SolverError {
        target: None,
        kind: SolverErrorKind::Failure(FailureKind::CertificateDesignGap {
            constructed_object_hash: hash_sequence(
                "kernel-not-ready",
                &[format!("{kind:?}").into_bytes()],
            ),
            missing_certificate_kind: format!("{kind:?} execution is owned by a later phase"),
        }),
    }
}

pub fn exact_replay_result(
    kind: KernelKind,
    label: &str,
    message: &ProjectionMessage,
    ctx: &KernelContext,
) -> ReplayResult {
    let accepted = message.kernel_kind == kind
        && crate::verify::verify_message::verify_projection_message(message, ctx).is_ok();
    ReplayResult {
        accepted,
        replay_hash: hash_sequence(
            label,
            &[
                format!("{kind:?}").into_bytes(),
                message.package_hash.0.to_vec(),
                message.certificate.certificate_hash.0.to_vec(),
                ctx.block.authorization_hash.0.to_vec(),
                vec![accepted as u8],
            ],
        ),
    }
}
