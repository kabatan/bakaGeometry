use serde::{Deserialize, Serialize};

use crate::compose::message::ProjectionMessage;
use crate::kernels::traits::KernelKind;
use crate::result::cost_trace::{GlobalCostTrace, ProjectionCostTrace};
use crate::result::diagnostics::DiagnosticRecord;
use crate::result::status::{FailureKind, SolverError, SolverErrorKind, SolverStatus};
use crate::roots::decode::TargetCandidate;
use crate::roots::isolate::RealRootRecord;
use crate::types::ids::BlockId;
use crate::types::ids::VariableId;
use crate::types::univariate::UniPolynomialQ;
use crate::verify::run_certificate::CoreRunCertificate;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetSolveResult {
    pub status: SolverStatus,
    pub target: VariableId,
    pub support_polynomial: Option<UniPolynomialQ>,
    pub squarefree_support_polynomial: Option<UniPolynomialQ>,
    pub root_isolation: Vec<RealRootRecord>,
    pub decoded_candidates: Vec<TargetCandidate>,
    pub projection_messages: Vec<ProjectionMessage>,
    pub certificate: Option<CoreRunCertificate>,
    pub diagnostics: Vec<DiagnosticRecord>,
    pub cost_trace: GlobalCostTrace,
}

pub struct FinalizeSuccessInput {
    pub result: TargetSolveResult,
}

pub struct FinalizeFailureInput {
    pub error: SolverError,
}

impl TargetSolveResult {
    pub fn from_solver_error(err: SolverError) -> TargetSolveResult {
        Self::from_solver_error_with_target(err, None)
    }

    pub fn from_solver_error_for_target(
        err: SolverError,
        fallback_target: VariableId,
    ) -> TargetSolveResult {
        Self::from_solver_error_with_target(err, Some(fallback_target))
    }

    pub fn from_solver_error_for_target_with_cost_trace(
        err: SolverError,
        fallback_target: VariableId,
        cost_trace: GlobalCostTrace,
    ) -> TargetSolveResult {
        Self::from_solver_error_with_target_and_cost_trace(err, Some(fallback_target), cost_trace)
    }

    fn from_solver_error_with_target(
        err: SolverError,
        fallback_target: Option<VariableId>,
    ) -> TargetSolveResult {
        let cost_trace = cost_trace_from_solver_error(&err);
        Self::from_solver_error_with_target_and_cost_trace(err, fallback_target, cost_trace)
    }

    fn from_solver_error_with_target_and_cost_trace(
        err: SolverError,
        fallback_target: Option<VariableId>,
        cost_trace: GlobalCostTrace,
    ) -> TargetSolveResult {
        TargetSolveResult {
            status: err.public_status(),
            target: err.target.or(fallback_target).unwrap_or(VariableId(0)),
            support_polynomial: None,
            squarefree_support_polynomial: None,
            root_isolation: Vec::new(),
            decoded_candidates: Vec::new(),
            projection_messages: Vec::new(),
            certificate: None,
            diagnostics: vec![DiagnosticRecord::from_solver_error(&err)],
            cost_trace,
        }
    }
}

pub fn finalize_success_result(input: FinalizeSuccessInput) -> TargetSolveResult {
    input.result
}

pub fn finalize_failure_result(input: FinalizeFailureInput) -> TargetSolveResult {
    TargetSolveResult::from_solver_error(input.error)
}

fn cost_trace_from_solver_error(err: &SolverError) -> GlobalCostTrace {
    let mut trace = GlobalCostTrace::default();
    if let Some(block_trace) = projection_trace_from_solver_error(err) {
        trace.block_traces.push(block_trace);
    }
    trace
}

pub(crate) fn projection_trace_from_solver_error(err: &SolverError) -> Option<ProjectionCostTrace> {
    if let SolverErrorKind::Failure(FailureKind::FiniteResourceFailure {
        stage,
        block_id,
        matrix_rows,
        matrix_cols,
        matrix_density,
        quotient_rank_estimate,
        coefficient_height_bits,
        ..
    }) = &err.kind
    {
        return Some(ProjectionCostTrace {
            block_id: block_id.unwrap_or(BlockId(0)),
            kernel_kind: kernel_kind_from_failure_stage(stage),
            estimated_quotient_rank: *quotient_rank_estimate,
            matrix_rows: *matrix_rows,
            matrix_cols: *matrix_cols,
            matrix_density: matrix_density.clone(),
            coefficient_height_before_bits: coefficient_height_bits.unwrap_or(0),
            coefficient_height_after_bits: coefficient_height_bits.unwrap_or(0),
            ..ProjectionCostTrace::default()
        });
    }
    None
}

fn kernel_kind_from_failure_stage(stage: &crate::result::status::StageId) -> KernelKind {
    match stage.0.as_str() {
        "LinearAffineKernel" => KernelKind::LinearAffine,
        "TargetRelationSearchKernel" => KernelKind::TargetRelationSearch,
        "SparseResultantProjectionKernel" => KernelKind::SparseResultantProjection,
        "SparseResultantTemplateMatrixCap" => KernelKind::SparseResultantProjection,
        "UniversalTargetEliminationKernel" => KernelKind::UniversalTargetElimination,
        "GroebnerLocalPairLimit" => KernelKind::UniversalTargetElimination,
        "RegularChainProjectionKernel" => KernelKind::RegularChainProjection,
        "NormTraceProjectionKernel" => KernelKind::NormTraceProjection,
        "SpecializationInterpolationKernel" => KernelKind::SpecializationInterpolation,
        _ => KernelKind::TargetRelationSearch,
    }
}
