use serde::{Deserialize, Serialize};

use crate::compose::message::ProjectionMessage;
use crate::result::cost_trace::GlobalCostTrace;
use crate::result::diagnostics::DiagnosticRecord;
use crate::result::status::{SolverError, SolverStatus};
use crate::roots::decode::TargetCandidate;
use crate::roots::isolate::RealRootRecord;
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
        TargetSolveResult {
            status: err.public_status(),
            target: err.target.unwrap_or(VariableId(0)),
            support_polynomial: None,
            squarefree_support_polynomial: None,
            root_isolation: Vec::new(),
            decoded_candidates: Vec::new(),
            projection_messages: Vec::new(),
            certificate: None,
            diagnostics: vec![DiagnosticRecord::from_solver_error(&err)],
            cost_trace: GlobalCostTrace::default(),
        }
    }
}

pub fn finalize_success_result(input: FinalizeSuccessInput) -> TargetSolveResult {
    input.result
}

pub fn finalize_failure_result(input: FinalizeFailureInput) -> TargetSolveResult {
    TargetSolveResult::from_solver_error(input.error)
}
